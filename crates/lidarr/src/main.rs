//! Dynamic (subprocess) entrypoint for the lidarr plugin.
//!
//! A `[[bin]]` that owns no runtime and reaches orca only through the socket.
//! Built with the typed `Plugin` builder; `use lidarr as _` force-links the
//! crate's `#[orca_tool]` inventory (nothing else in the bin references the
//! rlib, so without this the linker would dead-strip every tool registration).
plugin_toolkit::instrument::bootstrap!();
use lidarr as _;
use plugin_toolkit::plugin::Plugin;
fn main() -> plugin_toolkit::anyhow::Result<()> {
    Plugin::named("lidarr")
        .version(env!("CARGO_PKG_VERSION"))
        .tools(["lidarr."])
        .service(arr_base::ArrServiceBackend::new("lidarr", 8686))
        .serve()
}
