/// Type-state pattern for Agent lifecycle management
///
/// This module implements compile-time state machine validation
/// using Rust's type system, ensuring agents can only perform
/// valid state transitions.

use std::marker::PhantomData;
use crate::error::Result;

/// Agent states as zero-sized types (zero runtime cost)
pub struct Idle;
pub struct Working;
pub struct Reviewing;
pub struct Completed;

/// Agent with compile-time state tracking
pub struct Agent<State> {
    id: String,
    name: String,
    _state: PhantomData<State>,
}

/// Methods available only in Idle state
impl Agent<Idle> {
    pub fn new(id: String, name: String) -> Self {
        Agent {
            id,
            name,
            _state: PhantomData,
        }
    }

    /// Transition from Idle to Working
    pub fn start_work(self, task: Task) -> Result<Agent<Working>> {
        tracing::info!("Agent {} starting work on task", self.id);

        Ok(Agent {
            id: self.id,
            name: self.name,
            _state: PhantomData,
        })
    }
}

/// Methods available only in Working state
impl Agent<Working> {
    /// Submit work for review
    pub fn submit_for_review(self) -> Agent<Reviewing> {
        tracing::info!("Agent {} submitting work for review", self.id);

        Agent {
            id: self.id,
            name: self.name,
            _state: PhantomData,
        }
    }

    /// Return to idle if work is cancelled
    pub fn cancel(self) -> Agent<Idle> {
        tracing::warn!("Agent {} work cancelled", self.id);

        Agent {
            id: self.id,
            name: self.name,
            _state: PhantomData,
        }
    }
}

/// Methods available only in Reviewing state
impl Agent<Reviewing> {
    /// Review passed, mark as completed
    pub fn approve(self) -> Agent<Completed> {
        tracing::info!("Agent {} work approved", self.id);

        Agent {
            id: self.id,
            name: self.name,
            _state: PhantomData,
        }
    }

    /// Review failed, return to working
    pub fn request_changes(self) -> Agent<Working> {
        tracing::info!("Agent {} needs to address review comments", self.id);

        Agent {
            id: self.id,
            name: self.name,
            _state: PhantomData,
        }
    }
}

/// Methods available only in Completed state
impl Agent<Completed> {
    /// Reset agent to idle for next task
    pub fn reset(self) -> Agent<Idle> {
        tracing::info!("Agent {} resetting to idle", self.id);

        Agent {
            id: self.id,
            name: self.name,
            _state: PhantomData,
        }
    }
}

/// Common methods available in all states
impl<State> Agent<State> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// Import the actual Task from the module
use super::Task;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_transitions() {
        let agent = Agent::<Idle>::new("agent-1".to_string(), "Frontend".to_string());

        // This compiles: valid transition from Idle to Working
        let task = Task {
            id: "task-1".to_string(),
            description: "Create UI".to_string(),
            priority: super::Priority::Medium,
            task_type: crate::agent::TaskType::Development,
            agent_id: None,
            metadata: None,
            tags: vec![],
            context: None,
        };
        let working_agent = agent.start_work(task).unwrap();

        // This compiles: valid transition from Working to Reviewing
        let reviewing_agent = working_agent.submit_for_review();

        // This compiles: valid transition from Reviewing to Completed
        let completed_agent = reviewing_agent.approve();

        // This compiles: valid transition from Completed to Idle
        let _idle_again = completed_agent.reset();

        // This would NOT compile (uncomment to test):
        // completed_agent.submit_for_review(); // Error: no such method
        // idle_agent.approve(); // Error: no such method
    }
}