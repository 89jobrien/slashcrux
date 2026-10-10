//! Deterministic fixture selector for tests and replay (ADR0001 Phase 1).
//!
//! Maps exact intent strings to outcomes without invoking any provider.
//! Intended for Crux/Slash integration tests and fixture-driven dispatch —
//! not a substitute for the eventual provider adapter.

use std::collections::HashMap;

use super::SelectionFailure;
use super::outcome::{IntentSelection, IntentSelectionAssessment};
use super::request::{IntentSelectionRequest, SelectionResult};

/// A selector backed by an exact intent -> outcome map, for fixtures and
/// replay. Unknown intents are an operational [`SelectionFailure::InvalidRequest`],
/// not a semantic [`IntentSelection::NoMatch`] — the fixture has no opinion,
/// it simply wasn't configured for that intent.
#[derive(Debug, Default)]
pub struct FixtureSelector {
    outcomes: HashMap<String, IntentSelection>,
}

impl FixtureSelector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_outcome(mut self, intent: impl Into<String>, outcome: IntentSelection) -> Self {
        self.outcomes.insert(intent.into(), outcome);
        self
    }

    /// Assess `request` against the fixture map and the request's own
    /// catalog, applying the same validation a real selection service
    /// would (ADR0001 catalog and binding invariants).
    pub fn select(&self, request: &IntentSelectionRequest) -> SelectionResult {
        request
            .catalog
            .validate()
            .map_err(|e| SelectionFailure::InvalidRequest(e.to_string()))?;

        let outcome = self
            .outcomes
            .get(&request.intent)
            .ok_or_else(|| {
                SelectionFailure::InvalidRequest(format!(
                    "no fixture outcome registered for intent {:?}",
                    request.intent
                ))
            })?
            .clone();

        if let IntentSelection::Selected { command_id } = &outcome {
            if !request.catalog.contains(command_id) {
                return Err(SelectionFailure::InvalidResponse(format!(
                    "fixture selected {command_id:?}, not present in supplied catalog"
                )));
            }
        }

        if let IntentSelection::NeedsClarification { candidate_ids } = &outcome {
            for candidate_id in candidate_ids {
                if !request.catalog.contains(candidate_id) {
                    return Err(SelectionFailure::InvalidResponse(format!(
                        "fixture needs-clarification candidate {candidate_id:?}, not present in supplied catalog"
                    )));
                }
            }
        }

        Ok(IntentSelectionAssessment {
            request_id: request.id.clone(),
            catalog_revision: request.catalog.revision.clone(),
            outcome,
            assessor: "fixture".to_string(),
            model_identity: None,
            question_set_revision: "fixture-v1".to_string(),
            acceptance_policy_revision: "fixture-v1".to_string(),
            score: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::{
        CatalogCandidate, CommandCatalog, CommandId, SelectionContext, SelectionRequestId,
    };

    fn request(intent: &str, catalog: CommandCatalog) -> IntentSelectionRequest {
        IntentSelectionRequest {
            id: SelectionRequestId::new("req-1"),
            intent: intent.to_string(),
            catalog,
            context: Vec::<SelectionContext>::new(),
        }
    }

    fn catalog() -> CommandCatalog {
        CommandCatalog {
            revision: "rev-1".to_string(),
            candidates: vec![CatalogCandidate {
                id: CommandId::new("workspace.verify"),
                description: "run verification".to_string(),
            }],
        }
    }

    #[test]
    fn test_registered_intent_returns_configured_outcome() {
        let selector = FixtureSelector::new().with_outcome(
            "verify the workspace",
            IntentSelection::Selected {
                command_id: CommandId::new("workspace.verify"),
            },
        );
        let assessment = selector
            .select(&request("verify the workspace", catalog()))
            .unwrap();
        assert_eq!(
            assessment.outcome,
            IntentSelection::Selected {
                command_id: CommandId::new("workspace.verify")
            }
        );
    }

    #[test]
    fn test_unregistered_intent_is_operational_failure_not_no_match() {
        let selector = FixtureSelector::new();
        let result = selector.select(&request("do something unmapped", catalog()));
        assert!(matches!(result, Err(SelectionFailure::InvalidRequest(_))));
    }

    #[test]
    fn test_selected_id_outside_catalog_is_invalid_response() {
        let selector = FixtureSelector::new().with_outcome(
            "verify the workspace",
            IntentSelection::Selected {
                command_id: CommandId::new("workspace.nonexistent"),
            },
        );
        let result = selector.select(&request("verify the workspace", catalog()));
        assert!(matches!(result, Err(SelectionFailure::InvalidResponse(_))));
    }

    #[test]
    fn test_needs_clarification_candidate_outside_catalog_is_invalid_response() {
        let selector = FixtureSelector::new().with_outcome(
            "verify the workspace",
            IntentSelection::NeedsClarification {
                candidate_ids: vec![CommandId::new("workspace.nonexistent")],
            },
        );
        let result = selector.select(&request("verify the workspace", catalog()));
        assert!(matches!(result, Err(SelectionFailure::InvalidResponse(_))));
    }

    #[test]
    fn test_invalid_catalog_rejected_before_lookup() {
        let selector = FixtureSelector::new();
        let empty_catalog = CommandCatalog {
            revision: "rev-1".to_string(),
            candidates: vec![],
        };
        let result = selector.select(&request("verify the workspace", empty_catalog));
        assert!(matches!(result, Err(SelectionFailure::InvalidRequest(_))));
    }
}
