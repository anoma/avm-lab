//! Introspection instruction handler.

use crate::instruction::IntrospectInstruction;
use crate::interpreter::helpers::log_event;
use crate::interpreter::HandlerResult;
use crate::trace::EventType;
use crate::types::Val;
use crate::vm::State;

#[allow(clippy::unnecessary_wraps)]
pub fn execute(instr: IntrospectInstruction, state: &mut State) -> HandlerResult {
    match instr {
        IntrospectInstruction::GetSelf => Ok((Val::ObjectRef(state.self_id), None)),
        IntrospectInstruction::GetInput => Ok((state.input.clone(), None)),
        IntrospectInstruction::GetCurrentMachine => {
            Ok((Val::Str(state.machine_id.0.clone()), None))
        }
        IntrospectInstruction::GetState => {
            let obj_state = state
                .store
                .states
                .get(&state.self_id)
                .cloned()
                .unwrap_or_default();
            Ok((Val::List(obj_state), None))
        }
        IntrospectInstruction::SetState(new_state) => {
            if state.in_transaction() {
                state.pending_states.push((state.self_id, new_state));
            } else {
                state.store.states.insert(state.self_id, new_state);
            }
            let entry = log_event(state, EventType::StateUpdated(state.self_id));
            Ok((Val::Nothing, Some(entry)))
        }
        IntrospectInstruction::GetSender => {
            let val = match state.sender {
                Some(oid) => Val::just(Val::ObjectRef(oid)),
                None => Val::Nothing,
            };
            Ok((val, None))
        }
    }
}
