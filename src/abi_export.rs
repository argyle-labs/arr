// The tool surface crosses this FFI boundary as opaque JSON — the designated
// JSON dispatch seam, identical to orca's `plugin-loader` and
// `dispatch::ErasedTool::run_json`. The payload type is aliased (`sj`) at this
// one seam, exactly as the loader aliases it, and the workspace
// disallowed-types lint is suppressed for this file only.
#![allow(clippy::disallowed_types)]

//! ABI-stable cdylib export.
//!
//! Builds and exports the single [`PluginModRef`] root module orca's
//! `plugin-loader` `dlopen`s. The accessor fns carry the version header the
//! loader reads before invoking anything.
//!
//! This single cdylib bundles FOUR tool namespaces — `sonarr.*`, `radarr.*`,
//! `prowlarr.*`, `lidarr.*` — over one shared `crate::clients` base library.
//! `manifest()`/`invoke()` surface the union of those four namespaces, filtered
//! to exactly those prefixes so the statically-linked toolkit domain inventory
//! (containers / notifications / …) does not leak host-owned tools across the
//! ABI.
//!
//! Only the entrypoint + metadata cross as `StableAbi` types; the tool surface
//! itself crosses as JSON (manifest array + invoke args/result strings).

use std::sync::Arc;
use std::sync::OnceLock;

// `#[export_root_module]` expands to bare `::abi_stable` paths in this crate's
// root, so abi_stable is a direct dep — a genuinely-external (non-orca) crate.
// Pinned to the toolkit's version so the layout hash the loader checks matches.
use abi_stable::export_root_module;
use abi_stable::prefix_type::PrefixTypeTrait;
use abi_stable::std_types::{RErr, ROk, RResult, RStr, RString};
use plugin_toolkit::abi::{PluginMod, PluginModRef, ToolDef};
use plugin_toolkit::contract::config::{Config, Model, Ports};
use plugin_toolkit::contract::ToolCtx;
use plugin_toolkit::dispatch::{dispatch, tool_manifest_json};
// The JSON dispatch payload type, named once here at the designated opaque seam.
use plugin_toolkit::serde_json as sj;
use plugin_toolkit::tokio::runtime::{Builder, Runtime};

extern "C" fn plugin_semver() -> RString {
    RString::from(env!("CARGO_PKG_VERSION"))
}

extern "C" fn target_software() -> RString {
    // One cdylib serving the whole *arr stack — sonarr/radarr/prowlarr/lidarr.
    RString::from("arr")
}

extern "C" fn target_compat() -> RString {
    // sonarr/radarr expose `/api/v3` (server v4); prowlarr/lidarr expose
    // `/api/v1` (prowlarr v1, lidarr v1/v2). Stated as the *arr API versions.
    RString::from("sonarr=v3,radarr=v3,prowlarr=v1,lidarr=v1")
}

extern "C" fn orca_compat() -> RString {
    RString::from(">=0.0.8, <0.1.0")
}

/// Tool-name prefixes this plugin owns — one per bundled flavor. The cdylib
/// statically links the toolkit's domain crates (containers / notifications /
/// …), each carrying its own `#[orca_tool]` inventory entries, so the raw
/// `tool_manifest_json()` walk returns those host-owned tools alongside the
/// plugin's. The plugin exposes only its own four namespaces across the ABI.
const TOOL_PREFIXES: [&str; 4] = ["sonarr.", "radarr.", "prowlarr.", "lidarr."];

fn owned(name: &str) -> bool {
    TOOL_PREFIXES.iter().any(|p| name.starts_with(p))
}

/// The plugin's own tool surface: `tool_manifest_json()` filtered to the four
/// owned namespaces. Shared by `manifest()` (serialized back out) and
/// `invoke()` (admission check) so both agree on exactly which tools cross.
fn own_tools() -> Vec<ToolDef> {
    let all: Vec<ToolDef> = sj::from_str(&tool_manifest_json()).unwrap_or_default();
    all.into_iter().filter(|d| owned(&d.name)).collect()
}

extern "C" fn manifest() -> RString {
    let defs = own_tools();
    RString::from(sj::to_string(&defs).unwrap_or_else(|_| "[]".to_string()))
}

/// Shared multi-thread runtime driving the async tool bodies behind the
/// synchronous FFI `invoke`. Built once on first call and kept for the process
/// lifetime so repeated invocations don't spin a fresh runtime each time.
fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build plugin tokio runtime")
    })
}

/// A minimal `ToolCtx` for in-cdylib dispatch. The *arr tool surface
/// (HTTP-only endpoint CRUD + diagnosis) needs no host-injected services, so an
/// empty service registry over a placeholder config suffices; any tool reaching
/// for a service errors cleanly rather than panicking.
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
            "tool '{}' is not in this plugin's namespaces (sonarr./radarr./prowlarr./lidarr.)",
            name.as_str()
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
    /// guards the namespace filter: `manifest()` must surface every bundled
    /// flavor and nothing else (no host-owned tool leaks across the ABI).
    #[test]
    fn manifest_surfaces_all_four_namespaces_and_nothing_else() {
        let defs = own_tools();
        let count = |p: &str| defs.iter().filter(|d| d.name.starts_with(p)).count();
        let (s, r, pr, l) = (
            count("sonarr."),
            count("radarr."),
            count("prowlarr."),
            count("lidarr."),
        );
        // No host-owned (non-arr) tool leaks across the ABI.
        assert_eq!(s + r + pr + l, defs.len());
        // Every namespace present.
        assert!(s > 0 && r > 0 && pr > 0 && l > 0);
        // prowlarr carries the extra `indexer_status` tool.
        assert!(pr > s);
    }
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
    }
    .leak_into_prefix()
}
