//! Interpreter helper functions.
//!
//! Shared utilities for building log entries, applying transaction
//! commits, and managing the execution context during object calls.

use crate::error::{AVMError, TxError};
use crate::store::ObjectBehavior;
use crate::trace::{EventType, LogEntry};
use crate::types::{MachineId, ObjectId, ObjectMeta, Val};
use crate::vm::State;

/// Create a log entry with the current timestamp and controller.
pub fn log_event(state: &mut State, event: EventType) -> LogEntry {
    let ts = state.next_timestamp();
    LogEntry::new(ts, event, state.tx_controller.clone())
}

/// Validate the read set: check that no observed object was destroyed.
pub fn validate_observed(state: &State) -> Result<(), AVMError> {
    for oid in &state.observed {
        if state.destroys.contains(oid) {
            continue; // Destroyed by this tx — that's fine, we know about it.
        }
        if !state.store.objects.contains(oid) && !state.creates.contains_key(oid) {
            return Err(TxError::Conflict(state.tx.unwrap_or(crate::types::TxId(0))).into());
        }
    }
    Ok(())
}

/// Apply all pending creates to the store.
pub fn apply_creates(state: &mut State) {
    for (_, entry) in std::mem::take(&mut state.creates) {
        let meta = ObjectMeta {
            object_id: entry.id,
            machine: state.machine_id.clone(),
            creating_controller: entry.controller.clone(),
            current_controller: entry.controller,
        };
        state
            .store
            .objects
            .insert(entry.id, ObjectBehavior::named(&entry.behavior_name));
        state.store.metadata.insert(entry.id, meta);
        state.store.states.insert(entry.id, vec![]);
    }
}

/// Apply all pending writes (`tx_log` entries trigger object calls).
pub fn apply_writes(state: &mut State) {
    // Writes in the tx_log represent calls that were already executed.
    // Their effects are already in the store. We just clear the log.
    state.tx_log.clear();
}

/// Apply all pending state updates.
pub fn apply_states(state: &mut State) {
    for (oid, new_state) in std::mem::take(&mut state.pending_states) {
        state.store.states.insert(oid, new_state);
    }
}

/// Apply all pending transfers.
pub fn apply_transfers(state: &mut State) {
    for (oid, new_controller) in std::mem::take(&mut state.pending_transfers) {
        if let Some(meta) = state.store.metadata.get_mut(&oid) {
            meta.current_controller = Some(new_controller);
        }
    }
}

/// Apply all pending destroys.
pub fn apply_destroys(state: &mut State) {
    for oid in std::mem::take(&mut state.destroys) {
        state.store.remove(&oid);
    }
}

/// Save the current execution frame, run a call, then restore.
pub fn with_call_context<F, R>(
    state: &mut State,
    target_id: ObjectId,
    input: Val,
    sender: ObjectId,
    machine: MachineId,
    body: F,
) -> R
where
    F: FnOnce(&mut State) -> R,
{
    // Save caller context
    let saved_self = std::mem::replace(&mut state.self_id, target_id);
    let saved_input = std::mem::replace(&mut state.input, input);
    let saved_sender = state.sender.replace(sender);
    let saved_machine = std::mem::replace(&mut state.machine_id, machine);

    let result = body(state);

    // Restore caller context
    state.self_id = saved_self;
    state.input = saved_input;
    state.sender = saved_sender;
    state.machine_id = saved_machine;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::state::CreateEntry;
    use crate::vm::State;

    fn test_state() -> State {
        State::new(MachineId("local".into()))
    }

    #[test]
    fn log_event_increments_timestamp() {
        let mut state = test_state();
        let e1 = log_event(
            &mut state,
            EventType::TransactionStarted(crate::types::TxId(0)),
        );
        let e2 = log_event(
            &mut state,
            EventType::TransactionAborted(crate::types::TxId(0)),
        );
        assert_eq!(e1.timestamp, 0);
        assert_eq!(e2.timestamp, 1);
    }

    #[test]
    fn apply_creates_populates_store() {
        let mut state = test_state();
        state.creates.insert(
            ObjectId(1),
            CreateEntry {
                id: ObjectId(1),
                behavior_name: "test".into(),
                controller: None,
            },
        );
        apply_creates(&mut state);
        assert!(state.store.contains(&ObjectId(1)));
        assert!(state.creates.is_empty());
    }

    #[test]
    fn apply_destroys_removes_from_store() {
        let mut state = test_state();
        // First create an object
        state.creates.insert(
            ObjectId(2),
            CreateEntry {
                id: ObjectId(2),
                behavior_name: "x".into(),
                controller: None,
            },
        );
        apply_creates(&mut state);
        assert!(state.store.contains(&ObjectId(2)));

        // Then destroy it
        state.destroys.insert(ObjectId(2));
        apply_destroys(&mut state);
        assert!(!state.store.contains(&ObjectId(2)));
    }

    #[test]
    fn with_call_context_saves_and_restores() {
        let mut state = test_state();
        state.self_id = ObjectId(10);
        state.input = Val::Nat(1);

        with_call_context(
            &mut state,
            ObjectId(20),
            Val::Nat(99),
            ObjectId(10),
            MachineId("local".into()),
            |s| {
                assert_eq!(s.self_id, ObjectId(20));
                assert_eq!(s.input, Val::Nat(99));
                assert_eq!(s.sender, Some(ObjectId(10)));
            },
        );

        assert_eq!(state.self_id, ObjectId(10));
        assert_eq!(state.input, Val::Nat(1));
    }
}
