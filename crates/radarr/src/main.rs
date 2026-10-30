//! Dynamic (subprocess) entrypoint for the radarr plugin.
//!
//! A `[[bin]]` that owns no runtime and reaches orca only through the socket.
//! Built with the typed `Plugin` builder; `use radarr as _` force-links the
//! crate's `#[orca_tool]` inventory (nothing else in the bin references the
//! rlib, so without this the linker would dead-strip every tool registration).
plugin_toolkit::instrument::bootstrap!();
use plugin_toolkit::plugin::Plugin;
use radarr as _;
fn main() -> plugin_toolkit::anyhow::Result<()> {
    Plugin::named("radarr")
        .version(env!("CARGO_PKG_VERSION"))
        .tools(["radarr."])
        .service(arr_base::ArrServiceBackend::new("radarr", 7878))
        .serve()
}
