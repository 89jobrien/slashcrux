//! Workflow step lifecycle states and state classification helpers.

use serde::{Deserialize, Serialize};

/// Lifecycle state of a step during workflow execution.
///
/// Represents where a step is in its execution lifecycle, from waiting to start
/// through to a terminal state. Distinct from step *outcome* (ok/err/rejected),
/// which is execution-engine-specific.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}

impl StepState {
    /// Check if the step is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StepState::Completed | StepState::Failed | StepState::Skipped | StepState::Cancelled
        )
    }

    /// Check if the step is active.
    pub fn is_active(&self) -> bool {
        matches!(self, StepState::Running)
    }

    /// Check if the step is waiting to start.
    pub fn is_waiting(&self) -> bool {
        matches!(self, StepState::Pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_state_terminal() {
        assert!(StepState::Completed.is_terminal());
        assert!(StepState::Failed.is_terminal());
        assert!(StepState::Skipped.is_terminal());
        assert!(StepState::Cancelled.is_terminal());
        assert!(!StepState::Pending.is_terminal());
        assert!(!StepState::Running.is_terminal());
    }

    #[test]
    fn test_step_state_active() {
        assert!(StepState::Running.is_active());
        assert!(!StepState::Pending.is_active());
        assert!(!StepState::Completed.is_active());
    }

    #[test]
    fn test_step_state_waiting() {
        assert!(StepState::Pending.is_waiting());
        assert!(!StepState::Running.is_waiting());
        assert!(!StepState::Completed.is_waiting());
    }

    #[test]
    fn test_step_state_serde() {
        let state = StepState::Running;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"running\"");
        let parsed: StepState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, StepState::Running);
    }
}
