//! Flat interpreter for [`Tape`] programs.
//!
//! Executes a tape using a fixed register file and a program counter.
//! Zero heap allocation during the main loop — all values live in the
//! register array.

use crate::error::AVMError;
use crate::interpreter::Success;
use crate::tape::{Op, Tape, MAX_REGS};
use crate::trace::Trace;
use crate::types::Val;
use crate::vm::{BehaviorRegistry, State};

/// Execute a tape program against the given state.
///
/// This is the flat alternative to [`crate::interpreter::interpret`]. It uses a
/// program counter and a fixed register file instead of walking an `ITree`.
///
/// `Effect` ops are currently skipped (the result register is left as
/// `Nothing`). To execute effects, refactor the instruction handlers to
/// accept `&Instruction` and use [`execute_effect`] as a bridge. The
/// primary value of the tape interpreter today is for pure data-flow
/// programs and for CFG analysis.
#[allow(clippy::too_many_lines)]
pub fn interpret_tape(
    tape: &Tape,
    _state: &mut State,
    _registry: &BehaviorRegistry,
) -> Result<Success<Val>, AVMError> {
    let mut regs: Vec<Val> = vec![Val::Nothing; MAX_REGS];
    let mut pc: usize = 0;
    let trace = Trace::new();

    loop {
        if pc >= tape.ops.len() {
            return Ok(Success {
                value: Val::Nothing,
                trace,
            });
        }

        match &tape.ops[pc] {
            Op::Effect { result, .. } => {
                // Effect dispatch requires ownership of the Instruction.
                // For now, leave the result register as Nothing.
                // The real win of the Tape IR is zero-allocation construction
                // and CFG analysis; effect execution is bridged via ITree.
                let _ = result;
                pc += 1;
            }

            Op::LoadConst { val, result } => {
                regs[*result as usize] = val.clone();
                pc += 1;
            }

            Op::Move { src, dst } => {
                regs[*dst as usize] = regs[*src as usize].clone();
                pc += 1;
            }

            Op::MakeList {
                start,
                count,
                result,
            } => {
                let s = *start as usize;
                let c = *count as usize;
                let items: Vec<Val> = regs[s..s + c].to_vec();
                regs[*result as usize] = Val::List(items);
                pc += 1;
            }

            Op::UnwrapJust { src, result } => {
                let val = regs[*src as usize].clone();
                regs[*result as usize] = match val {
                    Val::Just(inner) => *inner,
                    other => other,
                };
                pc += 1;
            }

            Op::BranchNothing { reg, target } => {
                if regs[*reg as usize].is_nothing() {
                    pc = *target;
                } else {
                    pc += 1;
                }
            }

            Op::BranchTrue { reg, target } => {
                if regs[*reg as usize].as_bool() == Some(true) {
                    pc = *target;
                } else {
                    pc += 1;
                }
            }

            Op::Jump { target } => {
                pc = *target;
            }

            Op::Return { reg } => {
                return Ok(Success {
                    value: regs[*reg as usize].clone(),
                    trace,
                });
            }

            Op::Halt { .. } => {
                return Err(
                    crate::error::ObjError::RejectedCall(crate::types::ObjectId(u64::MAX)).into(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tape::TapeBuilder;
    use crate::types::MachineId;

    fn test_state() -> State {
        State::new(MachineId("local".into()))
    }

    fn empty_registry() -> BehaviorRegistry {
        BehaviorRegistry::new()
    }

    #[test]
    fn interpret_load_and_return() {
        let mut b = TapeBuilder::new();
        let r = b.load_const(Val::Nat(42));
        b.ret(r);
        let tape = b.build();

        let mut state = test_state();
        let result = interpret_tape(&tape, &mut state, &empty_registry()).unwrap();
        assert_eq!(result.value, Val::Nat(42));
    }

    #[test]
    fn interpret_branch_nothing_taken() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::Nothing);
        let br = b.branch_nothing(r0);
        let r1 = b.load_const(Val::Nat(1));
        b.ret(r1);
        b.patch(br);
        let r2 = b.load_const(Val::Nat(2));
        b.ret(r2);
        let tape = b.build();

        let mut state = test_state();
        let result = interpret_tape(&tape, &mut state, &empty_registry()).unwrap();
        assert_eq!(result.value, Val::Nat(2));
    }

    #[test]
    fn interpret_branch_nothing_not_taken() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::Nat(99));
        let br = b.branch_nothing(r0);
        let r1 = b.load_const(Val::Nat(1));
        b.ret(r1);
        b.patch(br);
        let r2 = b.load_const(Val::Nat(2));
        b.ret(r2);
        let tape = b.build();

        let mut state = test_state();
        let result = interpret_tape(&tape, &mut state, &empty_registry()).unwrap();
        assert_eq!(result.value, Val::Nat(1));
    }

    #[test]
    fn interpret_make_list() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::Nat(1));
        let _r1 = b.load_const(Val::Nat(2));
        let _r2 = b.load_const(Val::Nat(3));
        let rlist = b.make_list(r0, 3);
        b.ret(rlist);
        let tape = b.build();

        let mut state = test_state();
        let result = interpret_tape(&tape, &mut state, &empty_registry()).unwrap();
        assert_eq!(
            result.value,
            Val::List(vec![Val::Nat(1), Val::Nat(2), Val::Nat(3)])
        );
    }

    #[test]
    fn interpret_unwrap_just() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::just(Val::Nat(7)));
        let r1 = b.unwrap_just(r0);
        b.ret(r1);
        let tape = b.build();

        let mut state = test_state();
        let result = interpret_tape(&tape, &mut state, &empty_registry()).unwrap();
        assert_eq!(result.value, Val::Nat(7));
    }

    #[test]
    fn interpret_move_register() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::str("hello"));
        let r1 = b.mov(r0);
        b.ret(r1);
        let tape = b.build();

        let mut state = test_state();
        let result = interpret_tape(&tape, &mut state, &empty_registry()).unwrap();
        assert_eq!(result.value, Val::str("hello"));
    }

    #[test]
    fn interpret_jump() {
        let mut b = TapeBuilder::new();
        let jmp = b.jump();
        let r_skip = b.load_const(Val::Nat(999));
        b.ret(r_skip);
        b.patch(jmp);
        let r_ok = b.load_const(Val::Nat(42));
        b.ret(r_ok);
        let tape = b.build();

        let mut state = test_state();
        let result = interpret_tape(&tape, &mut state, &empty_registry()).unwrap();
        assert_eq!(result.value, Val::Nat(42));
    }
}
