# ADR: Bounded Intent Selection and Deterministic Branch Dispatch

- Status: Proposed
- Scope: `slashcrux`, Slash orchestration, Crux runtime, decision-provider adapters
- Decision date: October 10, 2026

## Context

Slash and Crux need a shared way to translate natural-language intent into a selection from a finite catalog of registered capabilities.

A decision provider such as Jev can assess an intent against supplied candidates. However, that assessment must not generate executable commands, invent workflow targets, bypass approvals, or modify pipeline structure.

The proposed boundary separates:

1. Shared selection vocabulary.
2. Provider inference and assessment acceptance.
3. Consumer-owned routing and input preparation.
4. Authorization and execution.

This ADR defines a proposed architecture. It does not assume these APIs or branching semantics already exist in the repositories.

## Decision

Use `slashcrux` to define **provider-neutral** intent-selection vocabulary. Implement inference behind a replaceable selection service. Let Slash and Crux independently resolve accepted outcomes through deterministic, consumer-owned bindings.

For Crux, an accepted outcome selects exactly one branch from a routing table declared before assessment.

```text
Intent + catalog snapshot + context
                 │
                 ▼
         Selection service
         ├── validate request
         ├── invoke provider
         ├── validate response
         └── apply acceptance policy
                 │
                 ▼
     Shared assessment or operational failure
                 │
                 ▼
       Consumer-owned deterministic policy
           ├── Slash invocation preparation
           └── Crux branch dispatch
                 │
                 ▼
       Input validation and authorization
                 │
                 ▼
             Execution
```

## Goals

- Share selection semantics between Slash and Crux.
- Restrict selection to supplied capability IDs.
- Distinguish semantic abstention from operational failure.
- Keep routing deterministic and inspectable.
- Support fixtures, shadow mode, and replay.
- Preserve existing execution, approval, and budget controls.
- Avoid requiring a provider call for explicit commands.

## Non-goals

- Generating shell commands or pipeline definitions.
- Filling command arguments through the selection operation.
- Replacing command parsing or schema validation.
- Authorizing execution.
- Proving task completion or correctness.
- Introducing a new autonomous orchestrator.
- Building general-purpose conversational clarification in the first release.

## Ownership boundaries

| Component           | Owns                                                                                      | Does not own                                  |
| ------------------- | ----------------------------------------------------------------------------------------- | --------------------------------------------- |
| `slashcrux`         | IDs, catalogs, requests, semantic outcomes, assessment records, failure vocabulary        | Networking, inference, policy execution       |
| Provider adapter    | Request translation, transport, provider-response validation                              | Capability authorization or workflow mutation |
| Selection service   | Request validation, adapter invocation, acceptance policy, shared assessment construction | Execution                                     |
| Slash orchestration | Command resolution, input preparation, invocation policy                                  | Provider-specific interpretation              |
| Crux runtime        | Branch resolution, scheduling, fallback, budgets, recording                               | Model-generated workflow targets              |
| Registry owner      | Catalog snapshots and authoritative bindings                                              | Treating relevance as permission              |

The callable selection port belongs in an appropriate ports layer, not necessarily in the shared vocabulary crate.

## Shared contract

The vocabulary includes:

```rust
pub struct IntentSelectionRequest {
    pub id: SelectionRequestId,
    pub intent: String,
    pub catalog: CommandCatalog,
    pub context: Vec<SelectionContext>,
}

pub enum IntentSelection {
    Selected {
        command_id: CommandId,
    },
    NeedsClarification {
        candidate_ids: Vec<CommandId>,
    },
    NoMatch,
    InsufficientEvidence {
        reason: InsufficientEvidenceReason,
    },
}
```

An assessment records:

- Selection request ID.
- Catalog revision.
- Accepted semantic outcome.
- Assessor and optional model identity.
- Question-set revision.
- Acceptance-policy revision.
- Optional typed probability or score signals.

The accepted outcome is not necessarily the provider’s raw answer. For example, acceptance policy may turn an insufficiently supported candidate choice into `InsufficientEvidence`.

Operational failures are represented separately:

```rust
type SelectionResult =
    Result<IntentSelectionAssessment, SelectionFailure>;
```

A timeout must not become `NoMatch`. Invalid output must not become a fabricated selection.

## Catalog and binding invariants

The caller constructs a catalog snapshot before selection.

The same snapshot must support subsequent resolution, either through retained bindings or a retrievable immutable revision.

Required invariants:

1. Candidate IDs are unique and nonempty.
2. The catalog is nonempty and has a revision.
3. Returned IDs exist in the supplied catalog.
4. Assessment request ID and revision match the request.
5. Selected capabilities have valid consumer-owned bindings.
6. Current authorization is checked separately before execution.

Descriptions are assessment evidence, not executable instructions.

If the authoritative bindings change before dispatch, the consumer must either use the retained snapshot when permitted or reject the stale result and rebuild the request. It must not silently resolve against unrelated current bindings.

## Crux branch dispatch

Crux declares an exclusive route map:

```text
Selected("workspace.verify")       → verification
Selected("workspace.investigate")  → investigation
NeedsClarification                → clarification
NoMatch                           → unsupported_request
InsufficientEvidence              → gather_context
Assessment failure                → assessment_fallback
```

An illustrative declaration follows. This is proposed syntax, not an existing API:

```yaml
- id: dispatch
  kind: selection_branch
  from: select_capability
  routes:
    selected:
      workspace.verify: verification
      workspace.investigate: investigation
    needs_clarification: clarification
    no_match: unsupported_request
    insufficient_evidence: gather_context
    unavailable: assessment_fallback
```

The runtime:

1. Validates the assessment against its request.
2. Resolves the outcome through the declared map.
3. Verifies the target is allowed.
4. Records the decision.
5. Activates the selected branch with typed input.

An unbound capability is a routing error. It must not fall through to direct command execution.

### Scheduling semantics

- Exactly one alternative is activated.
- Unselected alternatives are marked skipped, not failed.
- Joins explicitly recognize completion of the selected alternative.
- Unrelated branches are unaffected unless pipeline failure policy requires cancellation.
- All activated work remains subject to normal execution controls.

The selected branch receives the assessment or failure record, plus explicitly declared upstream inputs. Routing must not grant access to otherwise unavailable context.

## Slash consumption

Slash consumes the same assessment but resolves it into a pending invocation:

```text
Selected capability
        │
        ▼
Resolve registered command
        │
        ▼
Collect and validate inputs
        │
        ▼
Authorize and obtain approval where required
        │
        ▼
Execute
```

Explicit commands bypass semantic selection and proceed through deterministic parsing and validation.

## Error and fallback policy

| Condition                                     | Required behavior                                    |
| --------------------------------------------- | ---------------------------------------------------- |
| Timeout, temporary unavailability, rate limit | Bounded adapter retries within the caller’s deadline |
| Invalid request or unsupported schema         | Surface configuration/programming failure            |
| Authentication or authorization failure       | Stop futile retries and surface the failure          |
| Invalid provider response                     | Reject; never manufacture an answer                  |
| Caller cancellation                           | Cancel without retrying                              |
| Valid but inadequate assessment               | Produce semantic abstention                          |
| Unknown capability or stale catalog           | Reject dispatch                                      |
| Repeated evidence gathering                   | Enforce a recovery-cycle limit                       |

One layer owns transport retries.

Assessment attempts, work attempts, and recovery cycles are accounted for separately and also obey the overall workflow budget.

Fallback behavior is declared by the consumer:

- Advisory: retain the existing deterministic path.
- Routing-required: enter a predefined investigation or escalation path.
- Gate-required: block dependent execution when the requirement is unmet.

No fallback may silently authorize a guessed command.

## Recording and replay

Record selection separately from preparation and execution.

Minimum dispatch record:

- Request ID.
- Catalog revision or snapshot identity.
- Assessment.
- Routing-policy revision.
- Resolved branch ID.
- Relevant question and acceptance-policy revisions.

Replay uses the recorded assessment and routing policy without requiring inference. Re-execution remains subject to current authorization and execution controls.

Store sanitized diagnostics. Credentials and unrestricted raw provider responses are excluded from ordinary logs.

## Alternatives considered

### Put the full integration in `slashcrux`

Rejected because networking, retries, and runtime policy would expand a shared vocabulary layer into a service and orchestrator.

### Let the provider return branch IDs

Rejected because it couples inference to pipeline internals and weakens the finite allowed-target boundary.

### Duplicate selection contracts in Slash and Crux

Rejected because outcome meanings, validation, and replay formats would diverge.

### Use deterministic routing only

Retained for explicit commands and unambiguous rules. Semantic selection is optional and should justify its added complexity through evaluation.

## Rollout

### Phase 1: Contract and fixtures

- Introduce shared types.
- Implement validation.
- Add fixture selector.
- Implement Crux deterministic dispatch.
- Implement Slash pending-invocation preparation.

### Phase 2: Provider integration in shadow mode

- Add Jev adapter.
- Record assessments without changing execution.
- Evaluate selection, abstention, latency, and cost.

### Phase 3: Limited activation

- Enable a small, non-destructive capability catalog.
- Require explicit fallback routes.
- Preserve existing authorization boundaries.
- Expand only after measured outcomes justify it.

## Acceptance criteria

- Unknown IDs and mismatched revisions cannot dispatch.
- Every supported outcome resolves to a declared route or explicit routing error.
- Operational failures remain distinct from semantic outcomes.
- Tests assert that only the selected branch is activated.
- Join tests cover skipped alternatives.
- Retry, cancellation, and recovery-limit tests pass.
- Replay can reproduce the recorded branch decision without inference.
- Explicit commands do not require the selection service.
- Selection cannot bypass input validation or approval.
- Shadow mode produces no execution changes.

## Consequences

This design adds contracts, catalog management, validation, and explicit fallback configuration. In return, it creates one reusable semantic-selection boundary while preserving deterministic execution authority.

The central invariant is: a provider assesses relevance; the owning runtime decides what may run.
