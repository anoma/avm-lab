//! Battleship example — transactional game with state management.
//!
//! This mirrors the Agda specification's `examples/Battleship/Game.lagda.md`.
//! Two player board objects manage ship placement and attack resolution
//! using `getState`/`setState` within transactions.

use avm_core::avm_do;
use avm_core::instruction::{self, Instruction};
use avm_core::interpreter::{interpret, Success};
use avm_core::itree::{ret, trigger, ITree};
use avm_core::transport::LocalOnlyTransport;
use avm_core::types::{MachineId, Val};
use avm_core::vm::{BehaviorRegistry, State};

/// Ship: (x, y, length).
fn ship(x: u64, y: u64, len: u64) -> Val {
    Val::list(vec![
        Val::str("ship"),
        Val::Nat(x),
        Val::Nat(y),
        Val::Nat(len),
    ])
}

/// Attack coordinate.
fn coord(x: u64, y: u64) -> Val {
    Val::list(vec![Val::str("coord"), Val::Nat(x), Val::Nat(y)])
}

/// The board behavior: handles ship placement and attack queries.
#[allow(clippy::needless_pass_by_value)]
pub fn board_behavior(input: Val) -> ITree<Instruction, Val> {
    match input {
        Val::List(ref items) if !items.is_empty() => {
            let tag = items[0].as_str().unwrap_or("");
            match tag {
                "ship" => {
                    // Place a ship: add to state
                    let input_clone = input.clone();
                    avm_do! {
                        let current_state <- trigger(instruction::get_state());
                        let new_state = {
                            let mut ships = match current_state {
                                Val::List(s) => s,
                                _ => vec![],
                            };
                            ships.push(input_clone);
                            ships
                        };
                        trigger(instruction::set_state(new_state));
                        ret(Val::Bool(true))
                    }
                }
                "coord" if items.len() >= 3 => {
                    // Check if attack hits any ship
                    let ax = items[1].as_nat().unwrap_or(0);
                    let ay = items[2].as_nat().unwrap_or(0);
                    avm_do! {
                        let current_state <- trigger(instruction::get_state());
                        let ships = match current_state {
                            Val::List(s) => s,
                            _ => vec![],
                        };
                        let hit = ships.iter().any(|s| {
                            if let Val::List(parts) = s {
                                if parts.len() >= 4 {
                                    let sx = parts[1].as_nat().unwrap_or(0);
                                    let sy = parts[2].as_nat().unwrap_or(0);
                                    let slen = parts[3].as_nat().unwrap_or(0);
                                    ay == sy && ax >= sx && ax < sx + slen
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });
                        ret(Val::Bool(hit))
                    }
                }
                _ => ret(Val::str("board: unknown command")),
            }
        }
        _ => ret(Val::str("board: expected list input")),
    }
}

/// Create a behavior registry with the board behavior.
pub fn battleship_registry() -> BehaviorRegistry {
    let mut reg = BehaviorRegistry::new();
    reg.register("board", Box::new(board_behavior));
    reg
}

/// Build the game setup program: create two boards and place ships.
pub fn game_setup() -> ITree<Instruction, Val> {
    avm_do! {
        let tx_val <- trigger(instruction::begin_tx(None));
        let board1_val <- trigger(instruction::create_obj("board", None));
        let board2_val <- trigger(instruction::create_obj("board", None));
        let _committed <- trigger(instruction::commit_tx(
            crate::tx_id_from_val(&tx_val).expect("begin_tx must return TxRef")
        ));

        let b1 = crate::object_id_from_val(&board1_val).expect("create_obj must return ObjectRef");
        let b2 = crate::object_id_from_val(&board2_val).expect("create_obj must return ObjectRef");

        // Place ships on board 1
        let _r1 <- trigger(instruction::call(b1, ship(0, 0, 3)));
        let _r2 <- trigger(instruction::call(b1, ship(5, 2, 2)));

        // Place ships on board 2
        let _r3 <- trigger(instruction::call(b2, ship(1, 1, 4)));

        // Attack board 2 at (1, 1) — should hit
        let hit <- trigger(instruction::call(b2, coord(1, 1)));
        // Attack board 2 at (9, 9) — should miss
        let miss <- trigger(instruction::call(b2, coord(9, 9)));

        ret(Val::list(vec![hit, miss]))
    }
}

/// Run the Battleship example.
pub fn run_battleship() -> Result<Success<Val>, avm_core::error::AVMError> {
    let mut state = State::new(MachineId("local".into()));
    let registry = battleship_registry();
    let program = game_setup();
    interpret(program, &mut state, &registry, &LocalOnlyTransport)
}

#[cfg(test)]
mod tests {
    use super::*;
    use avm_core::trace::EventType;

    #[test]
    fn battleship_hit_and_miss() {
        let result = run_battleship().expect("should succeed");
        // Result should be [Just(true), Just(false)]
        if let Val::List(ref items) = result.value {
            assert_eq!(items.len(), 2);
            // Hit at (1,1)
            assert_eq!(items[0], Val::just(Val::Bool(true)));
            // Miss at (9,9)
            assert_eq!(items[1], Val::just(Val::Bool(false)));
        } else {
            panic!("expected list result, got: {}", result.value);
        }
    }

    #[test]
    fn battleship_creates_two_boards() {
        let result = run_battleship().expect("should succeed");
        let creates = result
            .trace
            .iter()
            .filter(|e| matches!(&e.event_type, EventType::ObjectCreated { .. }))
            .count();
        assert_eq!(creates, 2);
    }

    #[test]
    fn battleship_has_transaction() {
        let result = run_battleship().expect("should succeed");
        let tx_starts = result
            .trace
            .iter()
            .filter(|e| matches!(&e.event_type, EventType::TransactionStarted(_)))
            .count();
        assert!(tx_starts >= 1, "should have at least one transaction");
    }
}
