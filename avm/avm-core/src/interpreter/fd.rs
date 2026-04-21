//! Finite-domain constraint instruction handler (stub).

use crate::error::FdError;
use crate::instruction::FdInstruction;
use crate::interpreter::HandlerResult;

#[allow(clippy::needless_pass_by_value)]
pub fn execute(instr: FdInstruction) -> HandlerResult {
    let name = match &instr {
        FdInstruction::NewVar(_) => "newVar",
        FdInstruction::Narrow { .. } => "narrow",
        FdInstruction::Post(_) => "post",
        FdInstruction::Label(_) => "label",
    };
    Err(FdError::NotImplemented(name.to_string()).into())
}
