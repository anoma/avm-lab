//! Machine instruction handler: physical location management.

use crate::error::MachineError;
use crate::instruction::MachineInstruction;
use crate::interpreter::helpers::log_event;
use crate::interpreter::HandlerResult;
use crate::trace::EventType;
use crate::types::Val;
use crate::vm::State;

pub fn execute(instr: MachineInstruction, state: &mut State) -> HandlerResult {
    match instr {
        MachineInstruction::GetMachine(id) => {
            let machine = state.store.metadata.get(&id).map(|m| m.machine.clone());
            let val = match machine {
                Some(m) => Val::just(Val::Str(m.0)),
                None => Val::Nothing,
            };
            Ok((val, None))
        }
        MachineInstruction::Teleport(target) => {
            if state.in_transaction() {
                return Err(MachineError::TeleportDuringTx.into());
            }
            let from = state.machine_id.clone();
            state.machine_id = target.clone();
            let entry = log_event(state, EventType::ExecutionMoved { from, to: target });
            Ok((Val::Bool(true), Some(entry)))
        }
        MachineInstruction::MoveObject { object, target } => {
            if let Some(meta) = state.store.metadata.get_mut(&object) {
                let from = meta.machine.clone();
                meta.machine = target.clone();
                let entry = log_event(
                    state,
                    EventType::ObjectMoved {
                        id: object,
                        from,
                        to: target,
                    },
                );
                Ok((Val::Bool(true), Some(entry)))
            } else {
                Ok((Val::Bool(false), None))
            }
        }
        MachineInstruction::Fetch(id) => {
            let entry = log_event(
                state,
                EventType::ObjectFetched {
                    id,
                    machine: state.machine_id.clone(),
                },
            );
            Ok((Val::Bool(true), Some(entry)))
        }
    }
}
