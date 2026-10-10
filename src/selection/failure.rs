//! Operational failure vocabulary, kept distinct from semantic outcomes.

use serde::{Deserialize, Serialize};

/// Why a provider assessment was accepted as semantically inconclusive.
///
/// This is a *semantic* outcome ([`super::outcome::IntentSelection::InsufficientEvidence`]),
/// not an operational failure — the provider responded, but acceptance
/// policy determined the response did not justify a selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsufficientEvidenceReason {
    LowConfidence,
    AmbiguousIntent,
    MissingContext,
}

/// Operational failure of the selection pipeline itself.
///
/// A timeout, transport error, or invalid provider response must surface
/// here, never be coerced into [`super::outcome::IntentSelection::NoMatch`]
/// or a fabricated selection (ADR0001).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionFailure {
    #[error("selection timed out")]
    Timeout,
    #[error("provider temporarily unavailable")]
    Unavailable,
    #[error("rate limited")]
    RateLimited,
    #[error("invalid selection request: {0}")]
    InvalidRequest(String),
    #[error("authentication or authorization failed")]
    AuthFailed,
    #[error("provider returned an invalid response: {0}")]
    InvalidResponse(String),
    #[error("cancelled by caller")]
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_variants_are_distinct() {
        assert_ne!(SelectionFailure::Timeout, SelectionFailure::Unavailable);
    }

    #[test]
    fn test_failure_serde_roundtrip() {
        let failure = SelectionFailure::InvalidResponse("missing field".to_string());
        let json = serde_json::to_string(&failure).unwrap();
        let parsed: SelectionFailure = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, failure);
    }
}
