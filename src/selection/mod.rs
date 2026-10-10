//! Shared intent-selection vocabulary (ADR0001).
//!
//! Provider-neutral types for translating natural-language intent into a
//! selection from a finite, caller-supplied capability catalog. See
//! `docs/architecture/ADR0001-bounded-intent-selection.md`.

mod catalog;
mod failure;
mod fixture;
mod outcome;
mod request;

pub use catalog::{CatalogCandidate, CatalogValidationError, CommandCatalog, CommandId};
pub use failure::{InsufficientEvidenceReason, SelectionFailure};
pub use fixture::FixtureSelector;
pub use outcome::{IntentSelection, IntentSelectionAssessment};
pub use request::{IntentSelectionRequest, SelectionContext, SelectionRequestId, SelectionResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_result_distinguishes_outcome_from_failure() {
        let ok: SelectionResult = Ok(IntentSelectionAssessment {
            request_id: SelectionRequestId::new("req-1"),
            catalog_revision: "rev-1".to_string(),
            outcome: IntentSelection::NoMatch,
            assessor: "fixture".to_string(),
            model_identity: None,
            question_set_revision: "qs-1".to_string(),
            acceptance_policy_revision: "ap-1".to_string(),
            score: None,
        });
        assert!(ok.is_ok());

        let err: SelectionResult = Err(SelectionFailure::Timeout);
        assert!(err.is_err());
    }
}
