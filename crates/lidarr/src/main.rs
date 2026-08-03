//! Dynamic (subprocess) entrypoint for the lidarr plugin.
//!
//! The toolkit's `serve_tool_plugin!` emits `fn main`, serving this plugin over the orca
//! socket. The plugin is a `[[bin]]`, owns no runtime, and reaches orca only
//! through the socket.
plugin_toolkit::serve_tool_plugin! {
    name: "lidarr",
    target_compat: "lidarr=v1",
    link: lidarr,
}
