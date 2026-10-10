//! Shared vocabulary for Slash and Crux projects.
//!
//! This crate provides common types and enums used across the Slash language runtime
//! and Crux execution engine, enabling interoperability and reducing duplication.

pub mod execution_context;
pub mod priority;
pub mod selection;
pub mod step_state;
pub mod urgency;

pub use execution_context::ExecutionContext;
pub use priority::Priority;
pub use selection::{
    CatalogCandidate, CommandCatalog, CommandId, InsufficientEvidenceReason, IntentSelection,
    IntentSelectionAssessment, SelectionContext, SelectionFailure, SelectionRequestId,
    SelectionResult,
};
pub use step_state::StepState;
pub use urgency::Urgency;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_all_types_serialize() {
        let json_priority = serde_json::to_string(&Priority::Critical).unwrap();
        assert!(!json_priority.is_empty());

        let json_urgency = serde_json::to_string(&Urgency::Immediate).unwrap();
        assert!(!json_urgency.is_empty());

        let state = StepState::Running;
        let json_state = serde_json::to_string(&state).unwrap();
        assert!(!json_state.is_empty());

        let ctx = ExecutionContext::new();
        let json_ctx = serde_json::to_string(&ctx).unwrap();
        assert!(!json_ctx.is_empty());
    }
}
