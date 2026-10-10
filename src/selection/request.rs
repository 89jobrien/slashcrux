//! Selection request vocabulary and the shared result type.

use serde::{Deserialize, Serialize};

use super::catalog::CommandCatalog;
use super::failure::SelectionFailure;
use super::outcome::IntentSelectionAssessment;

/// Identifier correlating a request with its eventual assessment record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SelectionRequestId(String);

impl SelectionRequestId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Caller-supplied context accompanying an intent (e.g. prior turns,
/// workspace state). Context informs assessment; it is not itself
/// executable and does not expand the candidate set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionContext {
    pub key: String,
    pub value: String,
}

/// A request to assess `intent` against a supplied, immutable catalog
/// snapshot. The same snapshot must remain resolvable for subsequent
/// dispatch (ADR0001 catalog and binding invariants).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentSelectionRequest {
    pub id: SelectionRequestId,
    pub intent: String,
    pub catalog: CommandCatalog,
    pub context: Vec<SelectionContext>,
}

/// Outcome of attempting a selection: either a valid assessment (which may
/// itself be a semantic abstention) or an operational failure. The two
/// must never be conflated.
pub type SelectionResult = Result<IntentSelectionAssessment, SelectionFailure>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::{CatalogCandidate, CommandId};

    #[test]
    fn test_selection_request_serde_roundtrip() {
        let request = IntentSelectionRequest {
            id: SelectionRequestId::new("req-1"),
            intent: "verify the workspace".to_string(),
            catalog: CommandCatalog {
                revision: "rev-1".to_string(),
                candidates: vec![CatalogCandidate {
                    id: CommandId::new("workspace.verify"),
                    description: "run verification".to_string(),
                }],
            },
            context: vec![SelectionContext {
                key: "branch".to_string(),
                value: "main".to_string(),
            }],
        };
        let json = serde_json::to_string(&request).unwrap();
        let parsed: IntentSelectionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, request);
    }
}
