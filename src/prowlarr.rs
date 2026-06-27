//! Prowlarr tool surface — `prowlarr.*`. Thin wrappers over the shared
//! `crate::clients` base library.
//!
//! Prowlarr is the indexer-manager of the *arr stack: it speaks `/api/v1`
//! (not `/api/v3` like sonarr/radarr) and is the only flavor exposing
//! `/indexerstatus`. The [`Flavor`] is fixed to `Prowlarr`.
//!
//! Tool surface (one-tool-per-resource):
//!
//! - `prowlarr.{list, detail, create, update, delete}` — endpoint registry CRUD.
//! - `prowlarr.health` — **detection.** Health issues from `/api/v1/health`.
//! - `prowlarr.indexers` — configured indexers from `/api/v1/indexer`.
//! - `prowlarr.indexer_status` — per-indexer backoff from `/api/v1/indexerstatus`.
//! - `prowlarr.test_indexers` — **remediation.** Re-test every indexer.
//! - `prowlarr.system_status` — app name / version / instance.
//!
//! Imports flow through `plugin_toolkit::prelude::*` only.
#![allow(clippy::disallowed_types)]

use plugin_toolkit::prelude::*;

use crate::clients::{
    Client, Config, Flavor, HealthIssue, Indexer, IndexerStatus, IndexerTestResult, SystemStatus,
};

#[endpoint_resource(plugin = "prowlarr")]
pub struct ProwlarrEndpoint {
    pub name: String,
    #[secret]
    pub api_key: String,
}

/// Resolve a registered Prowlarr endpoint to a live [`Client`], picking the
/// first reachable address from the endpoint's fallback list.
async fn make_client(name: &str) -> Result<Client> {
    let row = {
        let conn = runtime::open_db()?;
        endpoint_db::get(&conn, name)?
            .with_context(|| format!("prowlarr endpoint '{name}' not registered"))?
    };
    if !row.enabled {
        bail!("prowlarr endpoint '{name}' is disabled");
    }
    let base_url = address::resolve_reachable(name, &row.addresses).await?;
    Ok(Client::new(Config::new(
        base_url,
        row.api_key,
        Flavor::Prowlarr,
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
pub struct ProwlarrHealthArgs {
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
pub struct ProwlarrHealthOutput {
    /// Total health issues raised.
    pub issue_count: usize,
    /// Health issues from `/health`.
    pub issues: Vec<HealthIssue>,
}

/// **Detection.** Health issues currently raised by a registered Prowlarr server.
#[orca_tool(domain = "prowlarr", verb = "health")]
async fn prowlarr_health(args: ProwlarrHealthArgs, _ctx: &ToolCtx) -> Result<ProwlarrHealthOutput> {
    let issues = crate::clients::ops::health(&make_client(&args.endpoint).await?).await?;
    Ok(ProwlarrHealthOutput {
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
pub struct ProwlarrIndexersArgs {
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
pub struct ProwlarrIndexersOutput {
    /// Configured indexers from `/indexer`.
    pub indexers: Vec<Indexer>,
}

/// Configured indexers on a registered Prowlarr server.
#[orca_tool(domain = "prowlarr", verb = "indexers")]
async fn prowlarr_indexers(
    args: ProwlarrIndexersArgs,
    _ctx: &ToolCtx,
) -> Result<ProwlarrIndexersOutput> {
    let indexers = crate::clients::ops::indexers(&make_client(&args.endpoint).await?).await?;
    Ok(ProwlarrIndexersOutput { indexers })
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct ProwlarrIndexerStatusArgs {
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
pub struct ProwlarrIndexerStatusOutput {
    /// Per-indexer backoff status from `/indexerstatus` — rows present only for
    /// indexers currently backed off.
    pub status: Vec<IndexerStatus>,
}

/// Per-indexer backoff status from Prowlarr's `/indexerstatus`. Rows appear
/// only for indexers Prowlarr has temporarily disabled after failures.
#[orca_tool(domain = "prowlarr", verb = "indexer_status")]
async fn prowlarr_indexer_status(
    args: ProwlarrIndexerStatusArgs,
    _ctx: &ToolCtx,
) -> Result<ProwlarrIndexerStatusOutput> {
    let status = crate::clients::ops::indexer_status(&make_client(&args.endpoint).await?).await?;
    Ok(ProwlarrIndexerStatusOutput { status })
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct ProwlarrTestIndexersArgs {
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
pub struct ProwlarrTestIndexersOutput {
    /// Indexers tested.
    pub tested_count: usize,
    /// Indexers that passed.
    pub valid_count: usize,
    /// Per-indexer result.
    pub results: Vec<IndexerTestResult>,
}

/// **Remediation.** Re-test every indexer on a registered Prowlarr server,
/// clearing stale backoff so recovered indexers are used again immediately.
#[orca_tool(domain = "prowlarr", verb = "test_indexers")]
async fn prowlarr_test_indexers(
    args: ProwlarrTestIndexersArgs,
    _ctx: &ToolCtx,
) -> Result<ProwlarrTestIndexersOutput> {
    let results = crate::clients::ops::test_indexers(&make_client(&args.endpoint).await?).await?;
    let valid_count = results
        .iter()
        .filter(|r| r.is_valid.unwrap_or(false))
        .count();
    Ok(ProwlarrTestIndexersOutput {
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
pub struct ProwlarrSystemStatusArgs {
    /// Registered endpoint name.
    pub endpoint: String,
}

/// App name, version, and instance name from `/system/status`.
#[orca_tool(domain = "prowlarr", verb = "system_status")]
async fn prowlarr_system_status(
    args: ProwlarrSystemStatusArgs,
    _ctx: &ToolCtx,
) -> Result<SystemStatus> {
    Ok(crate::clients::ops::system_status(&make_client(&args.endpoint).await?).await?)
}

// TODO: prowlarr.add_indexer — add an indexer over `/api/v1/indexer`.
// TODO: prowlarr.sync_apps — push indexers to linked sonarr/radarr/lidarr apps
//       over `/api/v1/applications`.
