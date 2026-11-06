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
