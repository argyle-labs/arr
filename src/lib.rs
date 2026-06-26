//! `arr` — a single orca cdylib plugin for the whole *arr stack.
//!
//! ONE crate, ONE cdylib, FOUR tool namespaces: `sonarr.*`, `radarr.*`,
//! `prowlarr.*`, `lidarr.*`. All four sit on a shared, crate-INTERNAL base
//! library ([`clients`]) — the typed progenitor clients + hand-written `Client`
//! transport + common `ops`. The base library is NOT a separately published
//! crate; it lives here and is reached as `crate::clients`.
//!
//! ## Adding a flavor
//!
//! 1. Drop `specs/<flavor>.openapi.json` into the repo (`build.rs` codegens a
//!    typed client for every spec present).
//! 2. Add `flavor!(<flavor>);` in [`clients`].
//! 3. Add a sibling `src/<flavor>.rs` tool-surface module + `pub mod <flavor>;`
//!    below, and add `"<flavor>."` to `abi_export::TOOL_PREFIXES`.
//!
//! ## Single orca dependency
//!
//! Every orca primitive is reached through `plugin_toolkit::*` / its prelude.
//! The only genuinely-external dep named in source is `abi_stable` (the cdylib
//! FFI boundary the loader `dlopen`s, which can't route through the toolkit).
//! The progenitor-generated clients carry serde/reqwest/etc. paths that the
//! toolkit's build-time codegen rewrites to `::plugin_toolkit::*`.
#![allow(clippy::disallowed_types)]

pub mod abi_export;
pub mod clients;

pub mod lidarr;
pub mod prowlarr;
pub mod radarr;
pub mod sonarr;
