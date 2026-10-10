//! Semantic selection outcomes and the resulting assessment record.

use serde::{Deserialize, Serialize};

use super::catalog::CommandId;
use super::failure::InsufficientEvidenceReason;
use super::request::SelectionRequestId;

/// The semantic outcome of assessing an intent against a supplied catalog.
///
/// Distinct from [`super::failure::SelectionFailure`]: this type represents
/// a valid assessment the provider was able to make, including deliberate
/// abstention. A timeout or invalid response must never be represented here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentSelection {
    Selected { command_id: CommandId },
    NeedsClarification { candidate_ids: Vec<CommandId> },
    NoMatch,
    InsufficientEvidence { reason: InsufficientEvidenceReason },
}

/// A recorded assessment: the accepted semantic outcome plus the
/// provenance needed to validate, record, and replay the decision.
///
/// The accepted outcome is not necessarily the provider's raw answer —
/// acceptance policy may downgrade an under-supported selection to
/// [`IntentSelection::InsufficientEvidence`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentSelectionAssessment {
    pub request_id: SelectionRequestId,
    pub catalog_revision: String,
    pub outcome: IntentSelection,
    pub assessor: String,
    pub model_identity: Option<String>,
    pub question_set_revision: String,
    pub acceptance_policy_revision: String,
    pub score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_selection_serde_roundtrip() {
        let outcome = IntentSelection::Selected {
            command_id: CommandId::new("workspace.verify"),
        };
        let json = serde_json::to_string(&outcome).unwrap();
        let parsed: IntentSelection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, outcome);
    }

    #[test]
    fn test_no_match_and_needs_clarification_are_distinct() {
        let no_match = IntentSelection::NoMatch;
        let clarification = IntentSelection::NeedsClarification {
            candidate_ids: vec![CommandId::new("workspace.verify")],
        };
        assert_ne!(no_match, clarification);
    }
}
