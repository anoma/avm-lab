# Tracing

Every state-modifying instruction emits a `LogEntry` into the execution trace.
The trace provides a complete audit trail for debugging, testing, and formal
verification.

## Log entry structure

```rust
struct LogEntry {
    timestamp: u64,              // monotonic counter
    event_type: EventType,       // what happened
    executing_controller: Option<ControllerId>,
}
```

## Event types

| Event | Emitted by |
|---|---|
| `ObjectCreated { id, behavior_name }` | `create_obj` |
| `ObjectDestroyed(id)` | `destroy_obj` |
| `ObjectCalled { id, input, output }` | `call` |
| `MessageReceived { id, input }` | `receive` |
| `StateUpdated(id)` | `set_state` |
| `ObjectMoved { id, from, to }` | `move_object` |
| `ExecutionMoved { from, to }` | `teleport` |
| `ObjectFetched { id, machine }` | `fetch` |
| `ObjectTransferred { id, from, to }` | `transfer_object` |
| `ObjectFrozen { id, controller }` | `freeze` |
| `FunctionUpdated(name)` | `update_pure` |
| `TransactionStarted(id)` | `begin_tx` |
| `TransactionCommitted(id)` | `commit_tx` |
| `TransactionAborted(id)` | `abort_tx` |
| `ErrorOccurred(msg)` | error paths |

## Using traces in tests

```rust
use avm_core::trace::{count_events, EventType};

let result = run_ping_pong(3).unwrap();

let creates = count_events(&result.trace, |e| {
    matches!(e, EventType::ObjectCreated { .. })
});
assert_eq!(creates, 2);
```

## Timestamps

Timestamps are monotonically increasing within a single `interpret()` call.
They do not represent wall-clock time — they are logical event counters.
