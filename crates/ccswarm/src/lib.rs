//! ccswarm - AI-powered multi-agent orchestration system

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::type_complexity)]
#![allow(clippy::option_if_let_else)]

pub mod agent;
pub mod cli;
pub mod config;
pub mod error;
pub mod events;
pub mod git;
pub mod governance;
pub mod hooks;
pub mod identity;
pub(crate) mod providers;
pub mod resource;
pub mod session;
pub mod utils;
pub mod workflow;

/// Public surface used only by the BDD test harness in `tests/bdd/`.
///
/// Regular consumers should ignore this module — it re-exports internal helpers
/// that the cucumber scenarios exercise (flow suggestion heuristic, run-ID
/// validation, provider resolution). Marked `doc(hidden)` so it doesn't
/// pollute documentation, but `pub` because integration tests are external
/// crates and can't reach `pub(crate)` items.
#[doc(hidden)]
pub mod bdd_api {
    pub use crate::cli::handlers::run_utils::validate_run_id;
    pub use crate::cli::handlers::workflow::suggest_flow_for_task;

    /// Resolve which provider kind a flow stage should use, mirroring the
    /// precedence in `FlowEngine::execute_stage`:
    /// stage YAML `provider:` > `CCSWARM_PROVIDER` env > Claude default.
    pub fn resolve_provider_kind(
        stage_provider: Option<&str>,
        env_value: Option<&str>,
    ) -> &'static str {
        use crate::providers::ProviderKind;
        let kind = stage_provider
            .and_then(ProviderKind::parse)
            .or_else(|| env_value.and_then(ProviderKind::parse))
            .unwrap_or(ProviderKind::Claude);
        kind.as_str()
    }

    /// Alias parse helper, used by BDD scenarios that exercise alias handling.
    pub fn parse_provider_kind(input: &str) -> Option<&'static str> {
        crate::providers::ProviderKind::parse(input).map(|k| k.as_str())
    }
}
