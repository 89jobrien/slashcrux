# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-10-10

### Added

- ADR0001 Phase 1: shared intent-selection vocabulary (`src/selection/`) —
  `CommandId`/`CommandCatalog`, `IntentSelection`, `SelectionFailure`,
  `IntentSelectionAssessment`, `IntentSelectionRequest`, and a
  `FixtureSelector` test double. Enforces catalog-membership validation for
  both `Selected` and `NeedsClarification` outcomes before returning a
  successful assessment.
- `thiserror = "2.0"` dependency for `SelectionFailure`'s error variants.

## [0.1.2] - earlier

- Prior release — see git history for changes before changelog tracking began.
