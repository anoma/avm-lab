//! The AVM interpreter — executes interaction trees against VM state.
//!
//! The interpreter is an iterative loop over [`ITree`] nodes. It dispatches
//! each [`Instruction`] to the appropriate handler module, threading
//! [`State`] and [`Trace`] through the execution.

mod controller;
mod fd;
mod helpers;
mod introspect;
mod machine;
mod nondet;
mod obj;
mod pure_fn;
mod reflect;
mod reify;
mod tx;

use crate::error::AVMError;
use crate::instruction::Instruction;
use crate::itree::ITree;
use crate::trace::{LogEntry, Trace};
use crate::transport::Transport;
use crate::types::Val;
use crate::vm::{BehaviorRegistry, State};

/// Result from a single instruction handler (leaf handlers only).
///
/// Leaf handlers emit at most one log entry, so `Option<LogEntry>` is used
/// instead of `Vec<LogEntry>` to avoid small-Vec allocations on the hot path.
pub(crate) type HandlerResult = Result<(Val, Option<LogEntry>), AVMError>;

/// The result of a successful AVM execution.
#[derive(Debug)]
pub struct Success<A> {
    pub value: A,
    pub trace: Trace,
}

/// The result type for the AVM interpreter.
pub type AVMResult<A> = Result<Success<A>, AVMError>;

/// Execute an AVM program (interaction tree) against the given state.
///
/// This is the main entry point. The interpreter iteratively processes the
/// tree, dispatching each instruction to its handler. `Tau` nodes are
/// eliminated in-place without recursion.
///
/// The `registry` provides named behavior resolution for `call` instructions.
/// The `transport` handles calls whose target lives on a different machine.
/// State is mutated in-place — no clone is performed.
pub fn interpret(
    program: ITree<Instruction, Val>,
    state: &mut State,
    registry: &BehaviorRegistry,
    transport: &dyn Transport,
) -> AVMResult<Val> {
    let mut trace = Trace::new();
    let mut current = program;

    loop {
        match current {
            ITree::Ret(value) => {
                return Ok(Success { value, trace });
            }
            ITree::Tau(next) => {
                current = *next;
            }
            ITree::Vis(instruction, cont) => {
                let (result_val, new_entries) =
                    execute_instruction(instruction, state, registry, transport)?;
                trace.extend(new_entries);
                current = cont(result_val);
            }
        }
    }
}

/// Dispatch a single instruction to its handler.
fn execute_instruction(
    instruction: Instruction,
    state: &mut State,
    registry: &BehaviorRegistry,
    transport: &dyn Transport,
) -> Result<(Val, Trace), AVMError> {
    // obj::execute can emit multiple entries (callee trace + its own), so it
    // returns Vec<LogEntry> directly.  All other (leaf) handlers return at
    // most one entry via Option<LogEntry>.
    macro_rules! leaf {
        ($call:expr) => {{
            let (val, entry) = $call?;
            Ok((val, entry.into_iter().collect()))
        }};
    }

    match instruction {
        Instruction::Obj(instr) => obj::execute(instr, state, registry, transport),
        Instruction::Introspect(instr) => leaf!(introspect::execute(instr, state)),
        Instruction::Reflect(instr) => leaf!(reflect::execute(instr, state)),
        Instruction::Reify(instr) => leaf!(reify::execute(instr, state)),
        Instruction::Tx(instr) => leaf!(tx::execute(instr, state)),
        Instruction::Pure(instr) => leaf!(pure_fn::execute(instr, state)),
        Instruction::Machine(instr) => leaf!(machine::execute(instr, state)),
        Instruction::Controller(instr) => leaf!(controller::execute(instr, state)),
        Instruction::Fd(instr) => leaf!(fd::execute(instr)),
        Instruction::Nondet(instr) => leaf!(nondet::execute(instr)),
    }
}
