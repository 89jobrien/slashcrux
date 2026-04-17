use serde::{Deserialize, Serialize};

/// Status of a step in a workflow or execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}

impl StepStatus {
    /// Check if the step is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StepStatus::Completed | StepStatus::Failed | StepStatus::Skipped | StepStatus::Cancelled
        )
    }

    /// Check if the step is active.
    pub fn is_active(&self) -> bool {
        matches!(self, StepStatus::Running)
    }

    /// Check if the step is waiting to start.
    pub fn is_waiting(&self) -> bool {
        matches!(self, StepStatus::Pending)
    }
}

impl Default for StepStatus {
    fn default() -> Self {
        StepStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_status_terminal() {
        assert!(StepStatus::Completed.is_terminal());
        assert!(StepStatus::Failed.is_terminal());
        assert!(StepStatus::Skipped.is_terminal());
        assert!(StepStatus::Cancelled.is_terminal());
        assert!(!StepStatus::Pending.is_terminal());
        assert!(!StepStatus::Running.is_terminal());
    }

    #[test]
    fn test_step_status_active() {
        assert!(StepStatus::Running.is_active());
        assert!(!StepStatus::Pending.is_active());
        assert!(!StepStatus::Completed.is_active());
    }

    #[test]
    fn test_step_status_waiting() {
        assert!(StepStatus::Pending.is_waiting());
        assert!(!StepStatus::Running.is_waiting());
        assert!(!StepStatus::Completed.is_waiting());
    }

    #[test]
    fn test_step_status_serde() {
        let status = StepStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"running\"");
        let parsed: StepStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, StepStatus::Running);
    }
}
