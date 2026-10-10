//! Task scheduling urgency levels and their relative scores.

use serde::{Deserialize, Serialize};

/// Urgency level for task execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Immediate,
    Soon,
    #[default]
    Whenever,
    Never,
}

impl Urgency {
    /// Numeric score for scheduling (higher = more urgent).
    pub fn score(&self) -> i32 {
        match self {
            Urgency::Immediate => 4,
            Urgency::Soon => 3,
            Urgency::Whenever => 2,
            Urgency::Never => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urgency_scores() {
        assert!(Urgency::Immediate.score() > Urgency::Soon.score());
        assert!(Urgency::Soon.score() > Urgency::Whenever.score());
        assert!(Urgency::Whenever.score() > Urgency::Never.score());
    }

    #[test]
    fn test_urgency_serde() {
        let u = Urgency::Immediate;
        let json = serde_json::to_string(&u).unwrap();
        assert_eq!(json, "\"immediate\"");
        let parsed: Urgency = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Urgency::Immediate);
    }
}
