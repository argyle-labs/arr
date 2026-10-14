//! Radarr tool surface — `radarr.*`. Thin wrappers over the shared
//! `arr_base` base library.
//!
//! Tool surface (one-tool-per-resource):
//!
//! - `radarr.{list, detail, create, update, delete}` — endpoint registry CRUD.
//! - `radarr.health` — **detection.** Health issues from `/api/v3/health`.
//! - `radarr.indexers` — configured indexers from `/api/v3/indexer`.
//! - `radarr.test_indexers` — **remediation.** Re-test every indexer.
//! - `radarr.category_collisions` — **detection.** Download-client categories
//!   shared by more than one registered Radarr endpoint.
//! - `radarr.usenet_preference` — **detection.** Delay profiles that prefer
//!   usenet without a torrent delay, so torrents are still grabbed first.
//! - `radarr.system_status` — app name / version / instance.
//!
//! Radarr speaks `/api/v3` with a `/movie` media model; the [`Flavor`] is
//! fixed to `Radarr`.
//!
//! Imports flow through `plugin_toolkit::prelude::*` only.
#![allow(clippy::disallowed_types)]

use plugin_toolkit::prelude::*;

use arr_base::{
    Client, Config, Flavor, HealthIssue, Indexer, IndexerTestResult, SystemStatus,
    UsenetPreferenceGap,
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
        endpoint_db::get(name)?
            .with_context(|| format!("radarr endpoint '{name}' not registered"))?
    };
    if !row.enabled {
        bail!("radarr endpoint '{name}' is disabled");
    }
    let base_url = route::resolve_reachable(name, &row.routes, false).await?;
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
    let issues = arr_base::ops::health(&make_client(&args.endpoint).await?).await?;
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
    let indexers = arr_base::ops::indexers(&make_client(&args.endpoint).await?).await?;
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
    let results = arr_base::ops::test_indexers(&make_client(&args.endpoint).await?).await?;
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
pub struct RadarrCategoryCollisionsArgs {}

/// **Detection.** Download-client categories shared by more than one enabled
/// Radarr endpoint (e.g. a 4K instance on the main instance's category), which
/// lets one instance import the other's grabs. Read-only. Covers this app's
/// endpoints only; categories used by other *arr apps are not compared.
#[orca_tool(domain = "radarr", verb = "category_collisions")]
async fn radarr_category_collisions(
    _args: RadarrCategoryCollisionsArgs,
    _ctx: &ToolCtx,
) -> Result<arr_base::CategoryReport> {
    let mut instances = Vec::new();
    for row in endpoint_db::list()?.into_iter().filter(|r| r.enabled) {
        let client = make_client(&row.name).await.map_err(|e| format!("{e:#}"));
        instances.push((row.name, client));
    }
    Ok(arr_base::ops::category_report(instances).await)
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct RadarrUsenetPreferenceArgs {
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
pub struct RadarrUsenetPreferenceOutput {
    /// Delay profiles whose usenet preference is ineffective.
    pub gaps: Vec<UsenetPreferenceGap>,
}

/// **Detection.** Delay profiles that prefer usenet but whose torrent delay
/// does not exceed the usenet delay, so a torrent is grabbed before a usenet
/// release has had time to appear. Read-only.
#[orca_tool(domain = "radarr", verb = "usenet_preference")]
async fn radarr_usenet_preference(
    args: RadarrUsenetPreferenceArgs,
    _ctx: &ToolCtx,
) -> Result<RadarrUsenetPreferenceOutput> {
    let gaps = arr_base::ops::usenet_preference(&make_client(&args.endpoint).await?).await?;
    Ok(RadarrUsenetPreferenceOutput { gaps })
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
    Ok(arr_base::ops::system_status(&make_client(&args.endpoint).await?).await?)
}

// TODO: radarr.add_movie — Radarr-specific media tool over `/api/v3/movie`.
