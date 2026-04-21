//! Controller instruction handler: logical ownership management.

use crate::error::ControllerError;
use crate::instruction::ControllerInstruction;
use crate::interpreter::helpers::log_event;
use crate::interpreter::HandlerResult;
use crate::trace::EventType;
use crate::types::Val;
use crate::vm::State;

pub fn execute(instr: ControllerInstruction, state: &mut State) -> HandlerResult {
    match instr {
        ControllerInstruction::GetCurrentController => {
            let val = match &state.tx_controller {
                Some(c) => Val::just(Val::Str(c.0.clone())),
                None => Val::Nothing,
            };
            Ok((val, None))
        }
        ControllerInstruction::GetController(id) => {
            let val = state
                .store
                .metadata
                .get(&id)
                .and_then(|m| m.current_controller.as_ref())
                .map_or(Val::Nothing, |c| Val::just(Val::Str(c.0.clone())));
            Ok((val, None))
        }
        ControllerInstruction::Transfer {
            object,
            new_controller,
        } => {
            if !state.store.objects.contains(&object) {
                return Err(ControllerError::NotAvailable(object).into());
            }
            if state.in_transaction() {
                state
                    .pending_transfers
                    .push((object, new_controller.clone()));
                Ok((Val::Bool(true), None))
            } else if let Some(meta) = state.store.metadata.get_mut(&object) {
                let from = meta
                    .current_controller
                    .clone()
                    .unwrap_or(crate::types::ControllerId("none".into()));
                meta.current_controller = Some(new_controller.clone());
                let entry = log_event(
                    state,
                    EventType::ObjectTransferred {
                        id: object,
                        from,
                        to: new_controller,
                    },
                );
                Ok((Val::Bool(true), Some(entry)))
            } else {
                Ok((Val::Bool(true), None))
            }
        }
        ControllerInstruction::Freeze(id) => {
            let controller = state
                .store
                .metadata
                .get(&id)
                .and_then(|m| m.current_controller.clone());
            match controller {
                None => Ok((Val::Nothing, None)),
                Some(ctrl) => {
                    let entry = log_event(
                        state,
                        EventType::ObjectFrozen {
                            id,
                            controller: ctrl,
                        },
                    );
                    Ok((Val::just(Val::Bool(true)), Some(entry)))
                }
            }
        }
    }
}
