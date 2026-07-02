// The tool surface crosses this FFI boundary as opaque JSON — the designated
// JSON dispatch seam, identical to orca's `plugin-loader` and
// `dispatch::ErasedTool::run_json`. The payload type is aliased (`sj`) at this
// one seam, and the workspace disallowed-types lint is suppressed for this file.
#![allow(clippy::disallowed_types)]

//! ABI-stable cdylib export for the `prowlarr` plugin.
//!
//! Builds and exports the single [`PluginModRef`] root module orca's
//! `plugin-loader` `dlopen`s. This cdylib owns exactly one tool namespace —
//! `prowlarr.*` — over the shared `arr_base` transport. `manifest()`/`invoke()`
//! are filtered to that prefix so the statically-linked toolkit domain
//! inventory (containers / notifications / …) does not leak across the ABI.

use std::sync::Arc;
use std::sync::OnceLock;

// `#[export_root_module]` expands to bare `::abi_stable` paths at the cdylib FFI
// boundary the loader dlopens; pinned to the toolkit's version (layout-hash match).
use abi_stable::export_root_module;
use abi_stable::prefix_type::PrefixTypeTrait;
use abi_stable::std_types::{RErr, ROk, RResult, RStr, RString};
use plugin_toolkit::abi::{PluginMod, PluginModRef, ToolDef};
use plugin_toolkit::contract::ToolCtx;
use plugin_toolkit::contract::config::{Config, Model, Ports};
use plugin_toolkit::dispatch::{dispatch, tool_manifest_json};
// The JSON dispatch payload type, named once here at the designated opaque seam.
use plugin_toolkit::serde_json as sj;
use plugin_toolkit::tokio::runtime::{Builder, Runtime};

/// This plugin's single owned tool-name prefix.
const TOOL_PREFIX: &str = "prowlarr.";

extern "C" fn plugin_semver() -> RString {
    RString::from(env!("CARGO_PKG_VERSION"))
}

extern "C" fn target_software() -> RString {
    RString::from("prowlarr")
}

extern "C" fn target_compat() -> RString {
    RString::from("prowlarr=v1")
}

extern "C" fn orca_compat() -> RString {
    RString::from(">=0.0.8, <0.1.0")
}

fn owned(name: &str) -> bool {
    name.starts_with(TOOL_PREFIX)
}

/// This plugin's own tool surface: `tool_manifest_json()` filtered to the one
/// owned namespace. Shared by `manifest()` and `invoke()` so both agree on
/// exactly which tools cross the ABI.
fn own_tools() -> Vec<ToolDef> {
    let all: Vec<ToolDef> = sj::from_str(&tool_manifest_json()).unwrap_or_default();
    all.into_iter().filter(|d| owned(&d.name)).collect()
}

extern "C" fn manifest() -> RString {
    let defs = own_tools();
    RString::from(sj::to_string(&defs).unwrap_or_else(|_| "[]".to_string()))
}

/// Shared multi-thread runtime driving the async tool bodies behind the
/// synchronous FFI `invoke`. Built once and kept for the process lifetime.
fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build plugin tokio runtime")
    })
}

/// A minimal `ToolCtx` for in-cdylib dispatch. The *arr tool surface (HTTP-only
/// endpoint CRUD + diagnosis) needs no host-injected services, so an empty
/// service registry over a placeholder config suffices.
fn minimal_ctx() -> ToolCtx {
    let config = Config {
        anthropic_api_key: None,
        lmstudio_url: String::new(),
        ollama_url: String::new(),
        default_model: Model::LMStudio {
            id: String::new(),
            url: String::new(),
        },
        app_dir: std::env::temp_dir(),
        memory_root: std::env::temp_dir(),
        db_path: std::env::temp_dir().join("orca-plugin.db"),
        ports: Ports::default(),
    };
    ToolCtx::new(Arc::new(config))
}

extern "C" fn invoke(name: RStr<'_>, args_json: RStr<'_>) -> RResult<RString, RString> {
    if !owned(name.as_str()) {
        return RErr(RString::from(format!(
            "tool '{}' is not in this plugin's namespace ({})",
            name.as_str(),
            TOOL_PREFIX
        )));
    }
    let args: sj::Value = match sj::from_str(args_json.as_str()) {
        Ok(v) => v,
        Err(e) => return RErr(RString::from(format!("invalid args JSON: {e}"))),
    };
    let ctx = minimal_ctx();
    let result = runtime().block_on(dispatch(name.as_str(), args, &ctx));
    match result {
        Ok(value) => match sj::to_string(&value) {
            Ok(s) => ROk(RString::from(s)),
            Err(e) => RErr(RString::from(format!("failed to encode result: {e}"))),
        },
        Err(e) => RErr(RString::from(format!("{e:#}"))),
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    /// The cdylib statically links the toolkit's domain inventory, so this
    /// guards the namespace filter: `manifest()` must surface this plugin's
    /// tools and nothing else (no host-owned tool leaks across the ABI).
    #[test]
    fn manifest_surfaces_only_own_namespace() {
        let defs = own_tools();
        assert!(!defs.is_empty(), "plugin exposes at least one tool");
        assert!(defs.iter().all(|d| d.name.starts_with(TOOL_PREFIX)));
    }
}

/// Domain backends this plugin contributes: none (pure tool-surface plugin).
extern "C" fn backends() -> RString {
    RString::from("[]")
}

/// Declared SQL tables: none (this plugin owns no plugin-scoped tables).
extern "C" fn schemas() -> RString {
    RString::from(r#"{"namespace":"","tables":[]}"#)
}

#[export_root_module]
fn export() -> PluginModRef {
    PluginMod {
        plugin_semver,
        target_software,
        target_compat,
        orca_compat,
        manifest,
        invoke,
        backends,
        schemas,
    }
    .leak_into_prefix()
}
