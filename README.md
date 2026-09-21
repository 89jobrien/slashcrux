# slashcrux

`slashcrux` provides shared, serializable vocabulary types for the Slash language runtime and the
Crux execution engine. It keeps common concepts such as priority, urgency, step lifecycle, and
ordered execution variables in one small Rust crate.

## Status

The crate is an active, early-stage `0.1.2` library with a deliberately narrow API. It contains no
runtime, parser, workflow engine, procedural macros, or CLI. Those behaviors belong to Slash and
Crux; this crate only defines interoperable data types. As a pre-1.0 crate, its API and serialized
representations should still be treated as subject to change.

## Types

| Type               | Purpose                                                                | Default    |
| ------------------ | ---------------------------------------------------------------------- | ---------- |
| `Priority`         | Importance: `Critical`, `High`, `Medium`, `Low`, or `Deferred`.        | `Medium`   |
| `Urgency`          | Scheduling pressure: `Immediate`, `Soon`, `Whenever`, or `Never`.      | `Whenever` |
| `StepState`        | Lifecycle: pending, running, completed, failed, skipped, or cancelled. | `Pending`  |
| `ExecutionContext` | Ordered JSON bindings, including explicit unset values.                | Empty      |

All four types are re-exported from the crate root. `Priority`, `Urgency`, and `StepState` use
snake-case strings in Serde formats. Priority and urgency expose numeric `score()` methods where a
higher number means higher priority or urgency.

`StepState` models lifecycle, not the engine-specific outcome of a step. Its helpers distinguish
waiting, active, and terminal states:

- `Pending` is waiting.
- `Running` is active.
- `Completed`, `Failed`, `Skipped`, and `Cancelled` are terminal.

## Installation

The crate uses Rust 2024 edition and declares Rust 1.85 as its minimum supported version.

Add the released crate version:

```toml
[dependencies]
slashcrux = "0.1.2"
serde_json = "1"
```

To consume the repository checkout directly instead:

```toml
[dependencies]
slashcrux = { git = "https://github.com/89jobrien/slashcrux" }
serde_json = "1"
```

There are no optional Cargo features.

## Quick start

```rust
use serde_json::json;
use slashcrux::{ExecutionContext, Priority, StepState, Urgency};

fn main() {
    let priority = Priority::High;
    let urgency = Urgency::Soon;
    assert!(priority.score() > Priority::Low.score());
    assert!(urgency.score() > Urgency::Whenever.score());

    let mut context = ExecutionContext::new();
    context.set("task_id", json!("task-42"));
    context.set("attempt", json!(1));

    assert!(context.contains("task_id"));
    assert_eq!(context.get("attempt"), Some(&json!(1)));
    assert!(StepState::Completed.is_terminal());

    println!("{}", context.to_json());
}
```

Because `ExecutionContext` uses Serde flattening, it serializes directly as an object rather than
under a `variables` field:

```json
{ "task_id": "task-42", "attempt": 1 }
```

## ExecutionContext behavior

The context stores `IndexMap<String, Option<serde_json::Value>>`, preserving insertion order.

```rust
use serde_json::json;
use slashcrux::ExecutionContext;

let mut context = ExecutionContext::new();
context.set("first", json!(true));
context.unset("optional");

assert_eq!(context.len(), 2);
assert!(context.contains("first"));
assert!(!context.contains("optional"));
assert!(context.all().contains_key("optional"));

let removed = context.remove("optional");
assert_eq!(removed, Some(None));
```

`unset` retains a key with a `None` value, which serializes as JSON `null`. In contrast, `remove`
deletes the binding entirely. `contains` returns `true` only when a key exists with a non-`None`
value. Use `all` or `all_mut` when the distinction matters.

Other context operations are:

- `from_map` to adopt an existing ordered map.
- `clear`, `len`, and `is_empty` for collection management.
- `to_json` for compact JSON serialization; it returns `{}` if serialization fails.

## Serialization contract

```rust
use slashcrux::{Priority, StepState, Urgency};

fn main() -> Result<(), serde_json::Error> {
    assert_eq!(serde_json::to_string(&Priority::Critical)?, "\"critical\"");
    assert_eq!(serde_json::to_string(&Urgency::Immediate)?, "\"immediate\"");
    assert_eq!(serde_json::to_string(&StepState::Running)?, "\"running\"");
    Ok(())
}
```

The enums derive `Debug`, `Clone`, equality, hashing, serialization, and deserialization. Priority
and urgency are also `Copy`; `StepState` is cloneable but not `Copy`.

## Layout

```text
slashcrux/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── execution_context.rs
    ├── priority.rs
    ├── step_state.rs
    └── urgency.rs
```

The crate has three runtime dependencies: `serde`, `serde_json`, and `indexmap` with Serde support.

## Development

```console
cargo fmt -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
```

Tests are colocated with each module and cover scores, lifecycle helpers, insertion order, context
mutation, and Serde round trips.

## License

Cargo metadata declares `MIT OR Apache-2.0`, but the repository does not currently include the
corresponding license text files.
