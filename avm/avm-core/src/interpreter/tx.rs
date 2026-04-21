//! Transaction instruction handler: begin, commit, abort.

use crate::error::TxError;
use crate::instruction::TxInstruction;
use crate::interpreter::helpers::{
    apply_creates, apply_destroys, apply_states, apply_transfers, apply_writes, log_event,
    validate_observed,
};
use crate::interpreter::HandlerResult;
use crate::trace::EventType;
use crate::types::Val;
use crate::vm::State;

pub fn execute(instr: TxInstruction, state: &mut State) -> HandlerResult {
    match instr {
        TxInstruction::Begin(controller) => execute_begin(controller, state),
        TxInstruction::Commit(id) => execute_commit(id, state),
        TxInstruction::Abort(id) => execute_abort(id, state),
    }
}

fn execute_begin(
    controller: Option<crate::types::ControllerId>,
    state: &mut State,
) -> HandlerResult {
    if state.in_transaction() {
        return Err(TxError::InvalidDuringTx.into());
    }

    let tx_id = state.fresh_ids.next_tx_id();
    state.tx = Some(tx_id);
    state.tx_controller = controller;
    state.tx_snapshot = Some(state.store.clone());

    let entry = log_event(state, EventType::TransactionStarted(tx_id));
    Ok((Val::TxRef(tx_id), Some(entry)))
}

fn execute_commit(id: crate::types::TxId, state: &mut State) -> HandlerResult {
    match &state.tx {
        None => return Err(TxError::NoActiveTx.into()),
        Some(active) if *active != id => return Err(TxError::NotFound(id).into()),
        _ => {}
    }

    // Validate read set
    if let Err(e) = validate_observed(state) {
        state.clear_tx_overlay();
        return Err(e);
    }

    // Discard the snapshot — we are keeping the changes.
    state.tx_snapshot = None;

    // Apply in dependency order (matches the Agda spec)
    apply_creates(state);
    apply_transfers(state);
    apply_writes(state);
    apply_states(state);
    apply_destroys(state);

    // Clear remaining overlay
    state.observed.clear();
    state.tx = None;
    state.tx_controller = None;

    let entry = log_event(state, EventType::TransactionCommitted(id));
    Ok((Val::Bool(true), Some(entry)))
}

fn execute_abort(id: crate::types::TxId, state: &mut State) -> HandlerResult {
    match &state.tx {
        None => return Err(TxError::NoActiveTx.into()),
        Some(active) if *active != id => return Err(TxError::NotFound(id).into()),
        _ => {}
    }

    state.clear_tx_overlay();

    let entry = log_event(state, EventType::TransactionAborted(id));
    Ok((Val::Nothing, Some(entry)))
}
