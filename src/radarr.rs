//! Radarr tool surface — `radarr.*`. Thin wrappers over the shared
//! `crate::clients` base library.
//!
//! Tool surface (one-tool-per-resource):
//!
//! - `radarr.{list, detail, create, update, delete}` — endpoint registry CRUD.
//! - `radarr.health` — **detection.** Health issues from `/api/v3/health`.
//! - `radarr.indexers` — configured indexers from `/api/v3/indexer`.
//! - `radarr.test_indexers` — **remediation.** Re-test every indexer.
//! - `radarr.system_status` — app name / version / instance.
//!
//! Radarr speaks `/api/v3` with a `/movie` media model; the [`Flavor`] is
//! fixed to `Radarr`.
//!
//! Imports flow through `plugin_toolkit::prelude::*` only.
#![allow(clippy::disallowed_types)]

use plugin_toolkit::prelude::*;

use crate::clients::{
    Client, Config, Flavor, HealthIssue, Indexer, IndexerTestResult, SystemStatus,
};

#[endpoint_resource(plugin = "radarr")]
pub struct RadarrEndpoint {
    pub name: String,
    #[secret]
    pub api_key: String,
}

/// Resolve a registered Radarr endpoint to a live [`Client`], picking the
/// first reachable address from the endpoint's fallback list.
async fn make_client(name: &str) -> Result<Client> {
    let row = {
        let conn = runtime::open_db()?;
        endpoint_db::get(&conn, name)?
            .with_context(|| format!("radarr endpoint '{name}' not registered"))?
    };
    if !row.enabled {
        bail!("radarr endpoint '{name}' is disabled");
    }
    let base_url = address::resolve_reachable(name, &row.addresses).await?;
    Ok(Client::new(Config::new(
        base_url,
        row.api_key,
        Flavor::Radarr,
    )))
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrHealthArgs {
    /// Registered endpoint name.
    pub endpoint: String,
}

#[derive(
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrHealthOutput {
    /// Total health issues raised.
    pub issue_count: usize,
    /// Health issues from `/health`.
    pub issues: Vec<HealthIssue>,
}

/// **Detection.** Health issues currently raised by a registered Radarr server.
#[orca_tool(domain = "radarr", verb = "health")]
async fn radarr_health(args: RadarrHealthArgs, _ctx: &ToolCtx) -> Result<RadarrHealthOutput> {
    let issues = crate::clients::ops::health(&make_client(&args.endpoint).await?).await?;
    Ok(RadarrHealthOutput {
        issue_count: issues.len(),
        issues,
    })
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrIndexersArgs {
    /// Registered endpoint name.
    pub endpoint: String,
}

#[derive(
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrIndexersOutput {
    /// Configured indexers from `/indexer`.
    pub indexers: Vec<Indexer>,
}

/// Configured indexers on a registered Radarr server.
#[orca_tool(domain = "radarr", verb = "indexers")]
async fn radarr_indexers(args: RadarrIndexersArgs, _ctx: &ToolCtx) -> Result<RadarrIndexersOutput> {
    let indexers = crate::clients::ops::indexers(&make_client(&args.endpoint).await?).await?;
    Ok(RadarrIndexersOutput { indexers })
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrTestIndexersArgs {
    /// Registered endpoint name.
    pub endpoint: String,
}

#[derive(
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrTestIndexersOutput {
    /// Indexers tested.
    pub tested_count: usize,
    /// Indexers that passed.
    pub valid_count: usize,
    /// Per-indexer result.
    pub results: Vec<IndexerTestResult>,
}

/// **Remediation.** Re-test every indexer on a registered Radarr server,
/// clearing stale backoff so recovered indexers are used again immediately.
#[orca_tool(domain = "radarr", verb = "test_indexers")]
async fn radarr_test_indexers(
    args: RadarrTestIndexersArgs,
    _ctx: &ToolCtx,
) -> Result<RadarrTestIndexersOutput> {
    let results = crate::clients::ops::test_indexers(&make_client(&args.endpoint).await?).await?;
    let valid_count = results
        .iter()
        .filter(|r| r.is_valid.unwrap_or(false))
        .count();
    Ok(RadarrTestIndexersOutput {
        tested_count: results.len(),
        valid_count,
        results,
    })
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrSystemStatusArgs {
    /// Registered endpoint name.
    pub endpoint: String,
}

/// App name, version, and instance name from `/system/status`.
#[orca_tool(domain = "radarr", verb = "system_status")]
async fn radarr_system_status(
    args: RadarrSystemStatusArgs,
    _ctx: &ToolCtx,
) -> Result<SystemStatus> {
    Ok(crate::clients::ops::system_status(&make_client(&args.endpoint).await?).await?)
}

// TODO: radarr.add_movie — Radarr-specific media tool over `/api/v3/movie`.
