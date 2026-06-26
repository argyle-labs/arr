//! Generate typed clients for every *arr spec under `specs/`. One spec →
//! one module included by `src/clients.rs` (via the `flavor!` macro). Add a
//! new spec *and* a `flavor!(name)` line to enable a flavor.
//!
//! All "make this upstream spec digestible by progenitor" work lives in the
//! toolkit's `openapi::normalize` — never hand-patch a spec here. The build-time
//! codegen also rewrites the generated serde/reqwest/progenitor crate paths to
//! `::plugin_toolkit::*`, so the plugin needs none of those as direct deps.
//!
//! Update a spec by overwriting its file, e.g.:
//!   curl http://sonarr.local/api/v3/openapi.json -o specs/sonarr.openapi.json

fn main() {
    let specs_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("specs");
    plugin_toolkit_build::openapi::generate_all(specs_dir, "arr").expect("arr openapi codegen");
}
