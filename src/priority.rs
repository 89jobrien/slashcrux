//! Task priority levels and their relative sorting scores.

use serde::{Deserialize, Serialize};

/// Priority level for tasks and operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Deferred,
}

impl Priority {
    /// Numeric score for sorting (higher = higher priority).
    pub fn score(&self) -> i32 {
        match self {
            Priority::Critical => 5,
            Priority::High => 4,
            Priority::Medium => 3,
            Priority::Low => 2,
            Priority::Deferred => 1,
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_scores() {
        assert!(Priority::Critical.score() > Priority::High.score());
        assert!(Priority::High.score() > Priority::Medium.score());
        assert!(Priority::Medium.score() > Priority::Low.score());
        assert!(Priority::Low.score() > Priority::Deferred.score());
    }

    #[test]
    fn test_priority_serde() {
        let p = Priority::Critical;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "\"critical\"");
        let parsed: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Priority::Critical);
    }
}
