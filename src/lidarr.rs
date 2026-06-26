//! Lidarr tool surface — `lidarr.*`. Thin wrappers over the shared
//! `crate::clients` base library.
//!
//! Tool surface (one-tool-per-resource):
//!
//! - `lidarr.{list, detail, create, update, delete}` — endpoint registry CRUD.
//! - `lidarr.health` — **detection.** Health issues from `/api/v1/health`.
//! - `lidarr.indexers` — configured indexers from `/api/v1/indexer`.
//! - `lidarr.test_indexers` — **remediation.** Re-test every indexer.
//! - `lidarr.system_status` — app name / version / instance.
//!
//! Lidarr speaks `/api/v1` with a `/artist` media model; the [`Flavor`] is
//! fixed to `Lidarr`.
//!
//! Imports flow through `plugin_toolkit::prelude::*` only.
#![allow(clippy::disallowed_types)]

use plugin_toolkit::prelude::*;

use crate::clients::{Client, Config, Flavor, HealthIssue, Indexer, IndexerTestResult, SystemStatus};

#[endpoint_resource(plugin = "lidarr")]
pub struct LidarrEndpoint {
    pub name: String,
    #[secret]
    pub api_key: String,
}

/// Resolve a registered Lidarr endpoint to a live [`Client`], picking the
/// first reachable address from the endpoint's fallback list.
async fn make_client(name: &str) -> Result<Client> {
    let row = {
        let conn = runtime::open_db()?;
        endpoint_db::get(&conn, name)?
            .with_context(|| format!("lidarr endpoint '{name}' not registered"))?
    };
    if !row.enabled {
        bail!("lidarr endpoint '{name}' is disabled");
    }
    let base_url = address::resolve_reachable(name, &row.addresses).await?;
    Ok(Client::new(Config::new(base_url, row.api_key, Flavor::Lidarr)))
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct LidarrHealthArgs {
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
pub struct LidarrHealthOutput {
    /// Total health issues raised.
    pub issue_count: usize,
    /// Health issues from `/health`.
    pub issues: Vec<HealthIssue>,
}

/// **Detection.** Health issues currently raised by a registered Lidarr server.
#[orca_tool(domain = "lidarr", verb = "health")]
async fn lidarr_health(args: LidarrHealthArgs, _ctx: &ToolCtx) -> Result<LidarrHealthOutput> {
    let issues = crate::clients::ops::health(&make_client(&args.endpoint).await?).await?;
    Ok(LidarrHealthOutput {
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
pub struct LidarrIndexersArgs {
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
pub struct LidarrIndexersOutput {
    /// Configured indexers from `/indexer`.
    pub indexers: Vec<Indexer>,
}

/// Configured indexers on a registered Lidarr server.
#[orca_tool(domain = "lidarr", verb = "indexers")]
async fn lidarr_indexers(args: LidarrIndexersArgs, _ctx: &ToolCtx) -> Result<LidarrIndexersOutput> {
    let indexers = crate::clients::ops::indexers(&make_client(&args.endpoint).await?).await?;
    Ok(LidarrIndexersOutput { indexers })
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct LidarrTestIndexersArgs {
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
pub struct LidarrTestIndexersOutput {
    /// Indexers tested.
    pub tested_count: usize,
    /// Indexers that passed.
    pub valid_count: usize,
    /// Per-indexer result.
    pub results: Vec<IndexerTestResult>,
}

/// **Remediation.** Re-test every indexer on a registered Lidarr server,
/// clearing stale backoff so recovered indexers are used again immediately.
#[orca_tool(domain = "lidarr", verb = "test_indexers")]
async fn lidarr_test_indexers(
    args: LidarrTestIndexersArgs,
    _ctx: &ToolCtx,
) -> Result<LidarrTestIndexersOutput> {
    let results = crate::clients::ops::test_indexers(&make_client(&args.endpoint).await?).await?;
    let valid_count = results.iter().filter(|r| r.is_valid.unwrap_or(false)).count();
    Ok(LidarrTestIndexersOutput {
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
pub struct LidarrSystemStatusArgs {
    /// Registered endpoint name.
    pub endpoint: String,
}

/// App name, version, and instance name from `/system/status`.
#[orca_tool(domain = "lidarr", verb = "system_status")]
async fn lidarr_system_status(
    args: LidarrSystemStatusArgs,
    _ctx: &ToolCtx,
) -> Result<SystemStatus> {
    Ok(crate::clients::ops::system_status(&make_client(&args.endpoint).await?).await?)
}

// TODO: lidarr.add_artist — Lidarr-specific media tool over `/api/v1/artist`.
