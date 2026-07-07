//! Readarr tool surface — `readarr.*`. Thin wrappers over the shared
//! `arr_base` base library.
//!
//! Tool surface (one-tool-per-resource):
//!
//! - `readarr.{list, detail, create, update, delete}` — endpoint registry CRUD.
//! - `readarr.health` — **detection.** Health issues from `/api/v1/health`.
//! - `readarr.indexers` — configured indexers from `/api/v1/indexer`.
//! - `readarr.test_indexers` — **remediation.** Re-test every indexer.
//! - `readarr.system_status` — app name / version / instance.
//!
//! Readarr speaks `/api/v1` with a `/author` media model; the [`Flavor`] is
//! fixed to `Readarr`.
//!
//! Imports flow through `plugin_toolkit::prelude::*` only.
#![allow(clippy::disallowed_types)]

pub mod abi_export;

use plugin_toolkit::prelude::*;

use arr_base::{Client, Config, Flavor, HealthIssue, Indexer, IndexerTestResult, SystemStatus};

#[endpoint_resource(plugin = "readarr")]
pub struct ReadarrEndpoint {
    pub name: String,
    #[secret]
    pub api_key: String,
}

/// Resolve a registered Readarr endpoint to a live [`Client`], picking the
/// first reachable address from the endpoint's fallback list.
async fn make_client(name: &str) -> Result<Client> {
    let row = {
        endpoint_db::get(name)?
            .with_context(|| format!("readarr endpoint '{name}' not registered"))?
    };
    if !row.enabled {
        bail!("readarr endpoint '{name}' is disabled");
    }
    let base_url = address::resolve_reachable(name, &row.addresses, false).await?;
    Ok(Client::new(Config::new(
        base_url,
        row.api_key,
        Flavor::Readarr,
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
pub struct ReadarrHealthArgs {
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
pub struct ReadarrHealthOutput {
    /// Total health issues raised.
    pub issue_count: usize,
    /// Health issues from `/health`.
    pub issues: Vec<HealthIssue>,
}

/// **Detection.** Health issues currently raised by a registered Readarr server.
#[orca_tool(domain = "readarr", verb = "health")]
async fn readarr_health(args: ReadarrHealthArgs, _ctx: &ToolCtx) -> Result<ReadarrHealthOutput> {
    let issues = arr_base::ops::health(&make_client(&args.endpoint).await?).await?;
    Ok(ReadarrHealthOutput {
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
pub struct ReadarrIndexersArgs {
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
pub struct ReadarrIndexersOutput {
    /// Configured indexers from `/indexer`.
    pub indexers: Vec<Indexer>,
}

/// Configured indexers on a registered Readarr server.
#[orca_tool(domain = "readarr", verb = "indexers")]
async fn readarr_indexers(
    args: ReadarrIndexersArgs,
    _ctx: &ToolCtx,
) -> Result<ReadarrIndexersOutput> {
    let indexers = arr_base::ops::indexers(&make_client(&args.endpoint).await?).await?;
    Ok(ReadarrIndexersOutput { indexers })
}

#[derive(
    plugin_toolkit::clap::Args,
    plugin_toolkit::serde::Serialize,
    plugin_toolkit::serde::Deserialize,
    plugin_toolkit::schemars::JsonSchema,
)]
#[serde(crate = "plugin_toolkit::serde")]
#[schemars(crate = "plugin_toolkit::schemars")]
pub struct ReadarrTestIndexersArgs {
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
pub struct ReadarrTestIndexersOutput {
    /// Indexers tested.
    pub tested_count: usize,
    /// Indexers that passed.
    pub valid_count: usize,
    /// Per-indexer result.
    pub results: Vec<IndexerTestResult>,
}

/// **Remediation.** Re-test every indexer on a registered Readarr server,
/// clearing stale backoff so recovered indexers are used again immediately.
#[orca_tool(domain = "readarr", verb = "test_indexers")]
async fn readarr_test_indexers(
    args: ReadarrTestIndexersArgs,
    _ctx: &ToolCtx,
) -> Result<ReadarrTestIndexersOutput> {
    let results = arr_base::ops::test_indexers(&make_client(&args.endpoint).await?).await?;
    let valid_count = results
        .iter()
        .filter(|r| r.is_valid.unwrap_or(false))
        .count();
    Ok(ReadarrTestIndexersOutput {
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
pub struct ReadarrSystemStatusArgs {
    /// Registered endpoint name.
    pub endpoint: String,
}

/// App name, version, and instance name from `/system/status`.
#[orca_tool(domain = "readarr", verb = "system_status")]
async fn readarr_system_status(
    args: ReadarrSystemStatusArgs,
    _ctx: &ToolCtx,
) -> Result<SystemStatus> {
    Ok(arr_base::ops::system_status(&make_client(&args.endpoint).await?).await?)
}

// TODO: readarr.add_author — Readarr-specific media tool over `/api/v1/author`.
