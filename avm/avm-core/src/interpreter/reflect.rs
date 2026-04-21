//! Reflection instruction handler (unsafe operations).

use crate::error::ReflectError;
use crate::instruction::ReflectInstruction;
use crate::interpreter::HandlerResult;
use crate::types::Val;
use crate::vm::State;

#[allow(clippy::needless_pass_by_value)]
pub fn execute(instr: ReflectInstruction, state: &mut State) -> HandlerResult {
    match instr {
        ReflectInstruction::Reflect(id) => {
            let meta = state
                .store
                .metadata
                .get(&id)
                .ok_or(ReflectError::MetadataNotFound(id))?;
            // Encode metadata as a list of values
            let val = Val::list(vec![
                Val::ObjectRef(meta.object_id),
                Val::Str(meta.machine.0.clone()),
                meta.creating_controller
                    .as_ref()
                    .map_or(Val::Nothing, |c| Val::just(Val::Str(c.0.clone()))),
                meta.current_controller
                    .as_ref()
                    .map_or(Val::Nothing, |c| Val::just(Val::Str(c.0.clone()))),
            ]);
            Ok((Val::just(val), None))
        }
        ReflectInstruction::ScryMeta { .. } | ReflectInstruction::ScryDeep { .. } => {
            Err(ReflectError::ScryPredicateFailed.into())
        }
    }
}
