//! `PingPong` example — two objects exchanging messages.
//!
//! This mirrors the Agda specification's `examples/PingPong/Main.lagda.md`.
//! Two objects, "ping" and "pong", exchange messages until a counter reaches
//! the maximum count.

use avm_core::avm_do;
use avm_core::instruction::{self, Instruction};
use avm_core::interpreter::{interpret, Success};
use avm_core::itree::{ret, trigger, ITree};
use avm_core::types::{MachineId, ObjectId, Val};
use avm_core::vm::{BehaviorRegistry, State};

/// Build the main `PingPong` orchestrator program.
///
/// Creates both objects inside a transaction, then sends the initial
/// message to kick off the exchange.
pub fn ping_pong_program(max_count: u64) -> ITree<Instruction, Val> {
    avm_do! {
        let tx_id_val <- trigger(instruction::begin_tx(None));
        let ping_id_val <- trigger(instruction::create_obj("ping", None));
        let pong_id_val <- trigger(instruction::create_obj("pong", None));
        let _committed <- trigger(instruction::commit_tx(
            crate::tx_id_from_val(&tx_id_val).expect("begin_tx must return TxRef")
        ));
        let initial_msg = Val::list(vec![
            Val::str("Ping"),
            Val::Nat(0),
            Val::Nat(max_count),
            pong_id_val.clone(),
        ]);
        let result <- trigger(instruction::call(
            crate::object_id_from_val(&ping_id_val).expect("create_obj must return ObjectRef"),
            initial_msg,
        ));
        ret(result)
    }
}

/// The ping behavior: receives a message, increments the counter,
/// and calls pong if the counter hasn't reached max.
#[allow(clippy::needless_pass_by_value)]
pub fn ping_behavior(input: Val) -> ITree<Instruction, Val> {
    match input {
        Val::List(ref items) if items.len() == 4 => {
            let count = items[1].as_nat().unwrap_or(0);
            let max = items[2].as_nat().unwrap_or(0);
            let pong_id_val = items[3].clone();

            if count >= max {
                ret(Val::list(vec![Val::str("done"), Val::Nat(count)]))
            } else {
                let msg = Val::list(vec![
                    Val::str("Pong"),
                    Val::Nat(count + 1),
                    Val::Nat(max),
                    pong_id_val.clone(),
                ]);
                let pong_id = crate::object_id_from_val(&pong_id_val)
                    .expect("pong_id in message must be ObjectRef");
                avm_do! {
                    let response <- trigger(instruction::call(pong_id, msg));
                    ret(response)
                }
            }
        }
        _ => ret(Val::str("ping: unexpected input")),
    }
}

/// The pong behavior: receives a message, increments the counter,
/// and calls ping back.
#[allow(clippy::needless_pass_by_value)]
pub fn pong_behavior(input: Val) -> ITree<Instruction, Val> {
    match input {
        Val::List(ref items) if items.len() == 4 => {
            let count = items[1].as_nat().unwrap_or(0);
            let max = items[2].as_nat().unwrap_or(0);

            if count >= max {
                ret(Val::list(vec![Val::str("done"), Val::Nat(count)]))
            } else {
                // Get self to pass back as the pong reference
                avm_do! {
                    let self_val <- trigger(instruction::get_self());
                    let caller <- trigger(instruction::get_sender());
                    let caller_id = match caller {
                        Val::Just(ref inner) => crate::object_id_from_val(inner)
                            .unwrap_or(ObjectId(u64::MAX)),
                        _ => ObjectId(u64::MAX),
                    };
                    let msg = Val::list(vec![
                        Val::str("Ping"),
                        Val::Nat(count + 1),
                        Val::Nat(max),
                        self_val,
                    ]);
                    let response <- trigger(instruction::call(caller_id, msg));
                    ret(response)
                }
            }
        }
        _ => ret(Val::str("pong: unexpected input")),
    }
}

/// Create a behavior registry with ping and pong behaviors.
pub fn ping_pong_registry() -> BehaviorRegistry {
    let mut reg = BehaviorRegistry::new();
    reg.register("ping", Box::new(ping_behavior));
    reg.register("pong", Box::new(pong_behavior));
    reg
}

/// Run the full `PingPong` example and return the result.
pub fn run_ping_pong(max_count: u64) -> Result<Success<Val>, avm_core::error::AVMError> {
    let mut state = State::new(MachineId("local".into()));
    let registry = ping_pong_registry();
    let program = ping_pong_program(max_count);
    interpret(program, &mut state, &registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use avm_core::trace::EventType;

    #[test]
    fn ping_pong_zero_rounds() {
        let result = run_ping_pong(0).expect("should succeed");
        // With max=0, ping should immediately return done
        match result.value {
            Val::Just(inner) => match *inner {
                Val::List(ref items) => {
                    assert_eq!(items[0], Val::str("done"));
                    assert_eq!(items[1], Val::Nat(0));
                }
                other => panic!("unexpected inner: {other}"),
            },
            other => panic!("expected Just, got: {other}"),
        }
    }

    #[test]
    fn ping_pong_three_rounds() {
        let result = run_ping_pong(3).expect("should succeed");
        // Should have completed with count reaching 3
        assert!(!result.trace.is_empty());
        // Verify objects were created
        let creates = result
            .trace
            .iter()
            .filter(|e| matches!(&e.event_type, EventType::ObjectCreated { .. }))
            .count();
        assert_eq!(creates, 2, "should create ping and pong");
    }

    #[test]
    fn ping_pong_trace_has_calls() {
        let result = run_ping_pong(2).expect("should succeed");
        let calls = result
            .trace
            .iter()
            .filter(|e| matches!(&e.event_type, EventType::ObjectCalled { .. }))
            .count();
        // At minimum: the initial call to ping, plus back-and-forth
        assert!(calls >= 1, "should have at least one call event");
    }
}
