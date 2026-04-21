//! Compiler from [`ITree`] to [`Tape`].
//!
//! Walks a linear `ITree` (as produced by `avm_do!`) and emits flat ops.
//! Continuations are evaluated eagerly with sentinel values so that the
//! resulting tape captures the full instruction sequence.

use crate::instruction::Instruction;
use crate::itree::ITree;
use crate::tape::{Tape, TapeBuilder};
use crate::types::{ObjectId, Val};

/// Compile a linear `ITree` into a flat `Tape`.
///
/// This works for programs built with `avm_do!` where each continuation
/// is a straightforward chain (no branching on the response value).
/// Continuations receive a `Val::ObjectRef` sentinel encoding the
/// destination register so the compiler can track data flow.
///
/// # Panics
///
/// Panics if the `ITree` contains branches that depend on runtime values
/// in ways the compiler cannot trace (i.e., continuations that inspect
/// the sentinel value and take different paths).
pub fn compile(tree: ITree<Instruction, Val>) -> Tape {
    let mut builder = TapeBuilder::new();
    compile_inner(tree, &mut builder);
    builder.build()
}

fn compile_inner(mut current: ITree<Instruction, Val>, builder: &mut TapeBuilder) {
    loop {
        match current {
            ITree::Ret(val) => {
                let r = builder.load_const(val);
                builder.ret(r);
                return;
            }
            ITree::Tau(next) => {
                current = *next;
            }
            ITree::Vis(instr, cont) => {
                let result_reg = builder.effect(instr);
                // Feed a sentinel value into the continuation.
                // The sentinel encodes which register holds the result.
                // Continuations built by avm_do! don't inspect the value
                // at construction time — they just capture it and pass it
                // along to the next trigger() call.
                let sentinel = Val::ObjectRef(ObjectId(u64::from(result_reg)));
                current = cont(sentinel);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm_do;
    use crate::instruction;
    use crate::itree::{ret, trigger};
    use crate::tape::Op;

    #[test]
    fn compile_single_ret() {
        let tree = ret(Val::Nat(42));
        let tape = compile(tree);
        assert_eq!(tape.len(), 2); // LoadConst + Return
        assert!(matches!(tape.ops[0], Op::LoadConst { .. }));
        assert!(matches!(tape.ops[1], Op::Return { .. }));
    }

    #[test]
    fn compile_single_effect() {
        let tree: ITree<Instruction, Val> = avm_do! {
            let _self_id <- trigger(instruction::get_self());
            ret(Val::Bool(true))
        };
        let tape = compile(tree);
        // Effect(get_self) + LoadConst(true) + Return
        assert_eq!(tape.len(), 3);
        assert!(matches!(tape.ops[0], Op::Effect { result: 0, .. }));
        assert!(matches!(tape.ops[1], Op::LoadConst { .. }));
        assert!(matches!(tape.ops[2], Op::Return { reg: 1 }));
    }

    #[test]
    fn compile_sequence() {
        let tree: ITree<Instruction, Val> = avm_do! {
            let _tx <- trigger(instruction::begin_tx(None));
            let _obj <- trigger(instruction::create_obj("test", None));
            ret(Val::str("done"))
        };
        let tape = compile(tree);
        // 2 Effects + LoadConst + Return
        assert_eq!(tape.len(), 4);
        assert!(matches!(tape.ops[0], Op::Effect { result: 0, .. }));
        assert!(matches!(tape.ops[1], Op::Effect { result: 1, .. }));
    }

    #[test]
    fn compile_tau_eliminated() {
        let tree: ITree<Instruction, Val> = crate::itree::tau(crate::itree::tau(ret(Val::Nat(7))));
        let tape = compile(tree);
        // Tau nodes are skipped, just LoadConst + Return
        assert_eq!(tape.len(), 2);
    }

    #[test]
    fn compile_registers_are_sequential() {
        let tree: ITree<Instruction, Val> = avm_do! {
            let _a <- trigger(instruction::get_self());
            let _b <- trigger(instruction::get_input());
            let _c <- trigger(instruction::get_sender());
            ret(Val::Nothing)
        };
        let tape = compile(tree);
        assert_eq!(tape.register_count, 4); // 3 effects + 1 const
    }
}
