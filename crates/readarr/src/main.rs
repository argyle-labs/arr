//! Dynamic (subprocess) entrypoint for the readarr plugin.
//!
//! A `[[bin]]` that owns no runtime and reaches orca only through the socket.
//! Built with the typed `Plugin` builder; `use readarr as _` force-links the
//! crate's `#[orca_tool]` inventory (nothing else in the bin references the
//! rlib, so without this the linker would dead-strip every tool registration).
plugin_toolkit::instrument::bootstrap!();
use plugin_toolkit::plugin::Plugin;
use readarr as _;
fn main() -> plugin_toolkit::anyhow::Result<()> {
    Plugin::named("readarr")
        .version(env!("CARGO_PKG_VERSION"))
        .tools(["readarr."])
        .service(arr_base::ArrServiceBackend::new("readarr", 8787))
        .serve()
}
