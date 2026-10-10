//! Capability catalog snapshot supplied to a selection request.

use serde::{Deserialize, Serialize};

/// Identifier of a registered capability (e.g. `"workspace.verify"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId(String);

impl CommandId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CommandId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A snapshot of candidate capabilities the selection may choose from.
///
/// Descriptions are assessment evidence, not executable instructions —
/// selection never generates commands or arguments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandCatalog {
    /// Revision identifier for this snapshot. Assessments are only valid
    /// against the revision they were produced for.
    pub revision: String,
    pub candidates: Vec<CatalogCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogCandidate {
    pub id: CommandId,
    pub description: String,
}

impl CommandCatalog {
    /// Returns `Ok(())` if candidate IDs are unique, the catalog is
    /// nonempty, and the revision is nonempty (ADR0001 catalog invariants).
    pub fn validate(&self) -> Result<(), CatalogValidationError> {
        if self.revision.trim().is_empty() {
            return Err(CatalogValidationError::EmptyRevision);
        }
        if self.candidates.is_empty() {
            return Err(CatalogValidationError::EmptyCatalog);
        }
        let mut seen = std::collections::HashSet::new();
        for candidate in &self.candidates {
            if candidate.id.as_str().is_empty() {
                return Err(CatalogValidationError::EmptyCandidateId);
            }
            if !seen.insert(&candidate.id) {
                return Err(CatalogValidationError::DuplicateCandidateId(
                    candidate.id.clone(),
                ));
            }
        }
        Ok(())
    }

    /// Whether `id` exists in this catalog snapshot.
    pub fn contains(&self, id: &CommandId) -> bool {
        self.candidates.iter().any(|c| &c.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogValidationError {
    #[error("catalog revision must not be empty")]
    EmptyRevision,
    #[error("catalog must contain at least one candidate")]
    EmptyCatalog,
    #[error("candidate id must not be empty")]
    EmptyCandidateId,
    #[error("duplicate candidate id: {0}")]
    DuplicateCandidateId(CommandId),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str) -> CatalogCandidate {
        CatalogCandidate {
            id: CommandId::new(id),
            description: format!("{id} description"),
        }
    }

    #[test]
    fn test_validate_rejects_empty_catalog() {
        let catalog = CommandCatalog {
            revision: "rev-1".to_string(),
            candidates: vec![],
        };
        assert_eq!(
            catalog.validate(),
            Err(CatalogValidationError::EmptyCatalog)
        );
    }

    #[test]
    fn test_validate_rejects_empty_revision() {
        let catalog = CommandCatalog {
            revision: "".to_string(),
            candidates: vec![candidate("workspace.verify")],
        };
        assert_eq!(
            catalog.validate(),
            Err(CatalogValidationError::EmptyRevision)
        );
    }

    #[test]
    fn test_validate_rejects_duplicate_ids() {
        let catalog = CommandCatalog {
            revision: "rev-1".to_string(),
            candidates: vec![candidate("workspace.verify"), candidate("workspace.verify")],
        };
        assert_eq!(
            catalog.validate(),
            Err(CatalogValidationError::DuplicateCandidateId(
                CommandId::new("workspace.verify")
            ))
        );
    }

    #[test]
    fn test_validate_accepts_well_formed_catalog() {
        let catalog = CommandCatalog {
            revision: "rev-1".to_string(),
            candidates: vec![
                candidate("workspace.verify"),
                candidate("workspace.investigate"),
            ],
        };
        assert_eq!(catalog.validate(), Ok(()));
    }

    #[test]
    fn test_contains() {
        let catalog = CommandCatalog {
            revision: "rev-1".to_string(),
            candidates: vec![candidate("workspace.verify")],
        };
        assert!(catalog.contains(&CommandId::new("workspace.verify")));
        assert!(!catalog.contains(&CommandId::new("workspace.investigate")));
    }
}
