//! Nondeterminism instruction handler (stub).

use crate::error::NondetError;
use crate::instruction::NondetInstruction;
use crate::interpreter::HandlerResult;

#[allow(clippy::needless_pass_by_value)]
pub fn execute(instr: NondetInstruction) -> HandlerResult {
    let name = match &instr {
        NondetInstruction::Choose(_) => "choose",
        NondetInstruction::Require(_) => "require",
    };
    Err(NondetError::NotImplemented(name.to_string()).into())
}
