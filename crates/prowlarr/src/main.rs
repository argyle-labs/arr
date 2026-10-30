//! Dynamic (subprocess) entrypoint for the prowlarr plugin.
//!
//! A `[[bin]]` that owns no runtime and reaches orca only through the socket.
//! Built with the typed `Plugin` builder; `use prowlarr as _` force-links the
//! crate's `#[orca_tool]` inventory (nothing else in the bin references the
//! rlib, so without this the linker would dead-strip every tool registration).
plugin_toolkit::instrument::bootstrap!();
use plugin_toolkit::plugin::Plugin;
use prowlarr as _;
fn main() -> plugin_toolkit::anyhow::Result<()> {
    Plugin::named("prowlarr")
        .version(env!("CARGO_PKG_VERSION"))
        .tools(["prowlarr."])
        .service(arr_base::ArrServiceBackend::new("prowlarr", 9696))
        .serve()
}
