//! Workflow Engine Module
//!
//! Flow/Stage based workflow pipelines: YAML-defined flows executed by
//! `FlowEngine` with faceted prompting, rule-based routing, and NDJSON
//! event recording.

pub mod cycle;
pub mod facets;
pub mod flow;
pub mod interactive;
pub mod judge;
pub mod permissions;
pub mod pipeline;
pub mod repertoire;
pub mod sangha;
pub mod team_leader;

pub use cycle::{CycleAnalysis, LoopTracker, analyze_flow};
pub use facets::{
    ComposedPrompt, FacetRegistry, KnowledgeFacet, PersonaFacet, PolicyFacet, builtin_personas,
    builtin_policies,
};
pub use flow::{
    Flow, FlowEngine, FlowState, FlowStatus, MovementPermission, MovementRule, OutputContract,
    RuleCondition, Stage, builtin_flows,
};
pub use interactive::{InteractiveAction, InteractiveConfig, InteractiveMode, InteractiveSession};
pub use judge::{JudgeConfig, JudgeResult, MatchMethod, MovementJudge};
pub use permissions::PermissionEnforcer;
pub use pipeline::{
    PipelineConfig, PipelineConfigBuilder, PipelineExitCode, PipelineOutput, PipelineRunner,
    PipelineStatus,
};
