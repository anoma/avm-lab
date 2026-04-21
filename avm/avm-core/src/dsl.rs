//! High-level DSL for common AVM patterns.

use crate::instruction::{self, Instruction};
use crate::itree::{ret, trigger, ITree};
use crate::types::{ControllerId, ObjectId, Val};

/// Run a block inside a transaction. Automatically begins and commits.
/// If the block returns an error-like value, the transaction is NOT auto-aborted
/// (the user controls that via the block's logic).
pub fn with_transaction<F>(controller: Option<ControllerId>, body: F) -> ITree<Instruction, Val>
where
    F: FnOnce(Val) -> ITree<Instruction, Val> + 'static,
{
    crate::avm_do! {
        let tx_ref <- trigger(instruction::begin_tx(controller));
        let result <- body(tx_ref.clone());
        let tx_id = tx_ref.as_tx_id().expect("begin_tx returns TxRef");
        trigger(instruction::commit_tx(tx_id));
        ret(result)
    }
}

/// Create an object and immediately call it with an initial message.
pub fn create_and_call(
    behavior_name: impl Into<String> + 'static,
    controller: Option<ControllerId>,
    initial_message: Val,
) -> ITree<Instruction, Val> {
    let name = behavior_name.into();
    crate::avm_do! {
        let obj_ref <- trigger(instruction::create_obj(name, controller));
        let obj_id = obj_ref.as_object_id().expect("create_obj returns ObjectRef");
        let result <- trigger(instruction::call(obj_id, initial_message));
        ret(result)
    }
}

/// Send a message to an object and unwrap the response (returns `Val::Nothing` on failure).
pub fn send(target: ObjectId, message: Val) -> ITree<Instruction, Val> {
    crate::avm_do! {
        let response <- trigger(instruction::call(target, message));
        let unwrapped = match response {
            Val::Just(inner) => *inner,
            other => other,
        };
        ret(unwrapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction;
    use crate::itree::ITree;
    use crate::types::{ObjectId, TxId, Val};

    /// Walk a `with_transaction` tree, feeding synthetic responses, and verify
    /// the sequence of instructions: Begin → (body) → Commit → Ret.
    #[test]
    fn with_transaction_creates_and_commits() {
        let tx_id = TxId(1);
        let program = with_transaction(None, |_tx_ref| ret(Val::Nat(42)));

        // First node: Begin
        let ITree::Vis(instr, cont) = program else {
            panic!("expected Vis(Begin, ...)");
        };
        assert!(matches!(instr, Instruction::Tx(_)), "expected Tx(Begin)");

        // Feed back a TxRef
        let next = cont(Val::TxRef(tx_id));

        // Body returns immediately (ret), so next node is Commit
        let ITree::Vis(instr2, cont2) = next else {
            panic!("expected Vis(Commit, ...)");
        };
        assert!(matches!(instr2, Instruction::Tx(_)), "expected Tx(Commit)");

        // Feed commit response; should get Ret(42)
        let final_tree = cont2(Val::Nothing);
        assert!(matches!(final_tree, ITree::Ret(Val::Nat(42))));
    }

    /// Verify `create_and_call` emits Create then Call.
    #[test]
    fn create_and_call_emits_create_then_call() {
        let obj_id = ObjectId(7);
        let program = create_and_call("MyBehavior", None, Val::Nat(0));

        // First: Create
        let ITree::Vis(instr, cont) = program else {
            panic!("expected Vis(Create, ...)");
        };
        assert!(matches!(instr, Instruction::Obj(_)), "expected Obj(Create)");

        // Feed back an ObjectRef
        let next = cont(Val::ObjectRef(obj_id));

        // Second: Call
        let ITree::Vis(instr2, cont2) = next else {
            panic!("expected Vis(Call, ...)");
        };
        assert!(matches!(instr2, Instruction::Obj(_)), "expected Obj(Call)");

        // Feed call result
        let final_tree = cont2(Val::Bool(true));
        assert!(matches!(final_tree, ITree::Ret(Val::Bool(true))));
    }

    /// Verify `send` unwraps `Val::Just`.
    #[test]
    fn send_unwraps_just() {
        let target = ObjectId(3);
        let program = send(target, Val::Nat(1));

        let ITree::Vis(instr, cont) = program else {
            panic!("expected Vis(Call, ...)");
        };
        assert!(matches!(instr, Instruction::Obj(_)));

        // Feed back a Just-wrapped value
        let inner = Val::Nat(99);
        let final_tree = cont(Val::just(inner));
        assert!(matches!(final_tree, ITree::Ret(Val::Nat(99))));
    }

    /// Verify `send` passes through non-Just values unchanged.
    #[test]
    fn send_passes_through_non_just() {
        let target = ObjectId(5);
        let program = send(target, Val::Nat(0));

        let ITree::Vis(_, cont) = program else {
            panic!("expected Vis");
        };

        let final_tree = cont(Val::Bool(false));
        assert!(matches!(final_tree, ITree::Ret(Val::Bool(false))));
    }
}
