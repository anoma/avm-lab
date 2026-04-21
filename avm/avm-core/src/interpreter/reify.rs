//! Reification instruction handler.

use crate::error::ReifyError;
use crate::instruction::ReifyInstruction;
use crate::interpreter::HandlerResult;
use crate::types::Val;
use crate::vm::State;

#[allow(clippy::needless_pass_by_value)]
pub fn execute(instr: ReifyInstruction, state: &mut State) -> HandlerResult {
    match instr {
        ReifyInstruction::ReifyContext => {
            let ctx = Val::list(vec![
                Val::ObjectRef(state.self_id),
                state.input.clone(),
                state
                    .sender
                    .map_or(Val::Nothing, |s| Val::just(Val::ObjectRef(s))),
                Val::Str(state.machine_id.0.clone()),
                state
                    .tx_controller
                    .as_ref()
                    .map_or(Val::Nothing, |c| Val::just(Val::Str(c.0.clone()))),
            ]);
            Ok((ctx, None))
        }
        ReifyInstruction::ReifyTxState => {
            if let Some(tx_id) = state.tx {
                let tx_state = Val::list(vec![
                    Val::TxRef(tx_id),
                    Val::Nat(state.tx_log.len() as u64),
                    Val::Nat(state.creates.len() as u64),
                    Val::Nat(state.destroys.len() as u64),
                    Val::Nat(state.observed.len() as u64),
                ]);
                Ok((Val::just(tx_state), None))
            } else {
                Err(ReifyError::NoTransaction.into())
            }
        }
        ReifyInstruction::ReifyConstraints => Err(ReifyError::ConstraintStoreUnavailable.into()),
    }
}
