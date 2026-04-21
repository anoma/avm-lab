//! Pure function instruction handler.

use crate::error::PureError;
use crate::instruction::PureInstruction;
use crate::interpreter::helpers::log_event;
use crate::interpreter::HandlerResult;
use crate::trace::EventType;
use crate::types::Val;
use crate::vm::State;

pub fn execute(instr: PureInstruction, state: &mut State) -> HandlerResult {
    match instr {
        PureInstruction::Call { name, args } => match state.pure_functions.call(&name, &args) {
            Some(result) => Ok((result.unwrap_or(Val::Nothing), None)),
            None => Err(PureError::NotFound(name).into()),
        },
        PureInstruction::Register { name, .. } => {
            // In this encoding, we can't pass closures through instructions.
            // The register instruction is a placeholder — real registration
            // happens through State::pure_functions directly.
            Err(PureError::AlreadyRegistered(name).into())
        }
        PureInstruction::Update { name, .. } => {
            let entry = log_event(state, EventType::FunctionUpdated(name));
            Ok((Val::Bool(true), Some(entry)))
        }
    }
}
