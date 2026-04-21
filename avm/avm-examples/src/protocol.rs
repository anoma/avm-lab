//! Typed protocol messages for the examples.
//!
//! These demonstrate how to use structured types with `Into<Val>` and
//! `TryFrom<Val>` to catch protocol errors at compile time.

use avm_core::types::{ObjectId, Val};
use std::convert::TryFrom;

/// Messages in the `PingPong` protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum PingPongMsg {
    Ping {
        count: u64,
        max: u64,
        partner: ObjectId,
    },
    Pong {
        count: u64,
        max: u64,
        partner: ObjectId,
    },
    Done {
        count: u64,
    },
}

/// Error when decoding a `Val` into a protocol message.
#[derive(Debug, thiserror::Error)]
#[error("invalid protocol message: {0}")]
pub struct ProtocolError(pub String);

impl From<PingPongMsg> for Val {
    fn from(msg: PingPongMsg) -> Self {
        match msg {
            PingPongMsg::Ping {
                count,
                max,
                partner,
            } => Val::list(vec![
                Val::str("Ping"),
                Val::Nat(count),
                Val::Nat(max),
                Val::ObjectRef(partner),
            ]),
            PingPongMsg::Pong {
                count,
                max,
                partner,
            } => Val::list(vec![
                Val::str("Pong"),
                Val::Nat(count),
                Val::Nat(max),
                Val::ObjectRef(partner),
            ]),
            PingPongMsg::Done { count } => Val::list(vec![Val::str("done"), Val::Nat(count)]),
        }
    }
}

impl TryFrom<Val> for PingPongMsg {
    type Error = ProtocolError;

    fn try_from(val: Val) -> Result<Self, Self::Error> {
        match val {
            Val::List(ref items) if items.len() >= 2 => {
                let tag = items[0]
                    .as_str()
                    .ok_or_else(|| ProtocolError("missing tag".into()))?;
                match tag {
                    "done" => {
                        let count = items
                            .get(1)
                            .and_then(Val::as_nat)
                            .ok_or_else(|| ProtocolError("missing count in Done".into()))?;
                        Ok(PingPongMsg::Done { count })
                    }
                    "Ping" | "Pong" if items.len() >= 4 => {
                        let count = items[1]
                            .as_nat()
                            .ok_or_else(|| ProtocolError("missing count".into()))?;
                        let max = items[2]
                            .as_nat()
                            .ok_or_else(|| ProtocolError("missing max".into()))?;
                        let partner = items[3]
                            .as_object_id()
                            .ok_or_else(|| ProtocolError("missing partner ObjectRef".into()))?;
                        if tag == "Ping" {
                            Ok(PingPongMsg::Ping {
                                count,
                                max,
                                partner,
                            })
                        } else {
                            Ok(PingPongMsg::Pong {
                                count,
                                max,
                                partner,
                            })
                        }
                    }
                    other => Err(ProtocolError(format!("unknown tag: {other}"))),
                }
            }
            other => Err(ProtocolError(format!("expected list, got: {other}"))),
        }
    }
}

/// Battleship protocol messages.
#[derive(Debug, Clone, PartialEq)]
pub enum BattleshipMsg {
    PlaceShip { x: u64, y: u64, length: u64 },
    Attack { x: u64, y: u64 },
}

impl From<BattleshipMsg> for Val {
    fn from(msg: BattleshipMsg) -> Self {
        match msg {
            BattleshipMsg::PlaceShip { x, y, length } => Val::list(vec![
                Val::str("ship"),
                Val::Nat(x),
                Val::Nat(y),
                Val::Nat(length),
            ]),
            BattleshipMsg::Attack { x, y } => {
                Val::list(vec![Val::str("coord"), Val::Nat(x), Val::Nat(y)])
            }
        }
    }
}

impl TryFrom<Val> for BattleshipMsg {
    type Error = ProtocolError;

    fn try_from(val: Val) -> Result<Self, Self::Error> {
        match val {
            Val::List(ref items) if !items.is_empty() => {
                let tag = items[0]
                    .as_str()
                    .ok_or_else(|| ProtocolError("missing tag".into()))?;
                match tag {
                    "ship" if items.len() >= 4 => Ok(BattleshipMsg::PlaceShip {
                        x: items[1]
                            .as_nat()
                            .ok_or_else(|| ProtocolError("missing x".into()))?,
                        y: items[2]
                            .as_nat()
                            .ok_or_else(|| ProtocolError("missing y".into()))?,
                        length: items[3]
                            .as_nat()
                            .ok_or_else(|| ProtocolError("missing length".into()))?,
                    }),
                    "coord" if items.len() >= 3 => Ok(BattleshipMsg::Attack {
                        x: items[1]
                            .as_nat()
                            .ok_or_else(|| ProtocolError("missing x".into()))?,
                        y: items[2]
                            .as_nat()
                            .ok_or_else(|| ProtocolError("missing y".into()))?,
                    }),
                    other => Err(ProtocolError(format!("unknown tag: {other}"))),
                }
            }
            other => Err(ProtocolError(format!("expected list, got: {other}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn ping_round_trips() {
        let msg = PingPongMsg::Ping {
            count: 3,
            max: 10,
            partner: ObjectId(42),
        };
        let val: Val = msg.clone().into();
        let decoded = PingPongMsg::try_from(val).expect("should decode Ping");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn pong_round_trips() {
        let msg = PingPongMsg::Pong {
            count: 5,
            max: 10,
            partner: ObjectId(7),
        };
        let val: Val = msg.clone().into();
        let decoded = PingPongMsg::try_from(val).expect("should decode Pong");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn done_round_trips() {
        let msg = PingPongMsg::Done { count: 10 };
        let val: Val = msg.clone().into();
        let decoded = PingPongMsg::try_from(val).expect("should decode Done");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn ping_pong_invalid_val_errors() {
        let val = Val::Nat(42);
        let result = PingPongMsg::try_from(val);
        assert!(result.is_err());
    }

    #[test]
    fn ping_pong_unknown_tag_errors() {
        let val = Val::list(vec![Val::str("unknown"), Val::Nat(1)]);
        let result = PingPongMsg::try_from(val);
        assert!(result.is_err());
    }

    #[test]
    fn battleship_place_ship_round_trips() {
        let msg = BattleshipMsg::PlaceShip {
            x: 2,
            y: 3,
            length: 4,
        };
        let val: Val = msg.clone().into();
        let decoded = BattleshipMsg::try_from(val).expect("should decode PlaceShip");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn battleship_attack_round_trips() {
        let msg = BattleshipMsg::Attack { x: 5, y: 6 };
        let val: Val = msg.clone().into();
        let decoded = BattleshipMsg::try_from(val).expect("should decode Attack");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn battleship_invalid_val_errors() {
        let val = Val::Bool(true);
        let result = BattleshipMsg::try_from(val);
        assert!(result.is_err());
    }

    #[test]
    fn battleship_unknown_tag_errors() {
        let val = Val::list(vec![Val::str("fire"), Val::Nat(1), Val::Nat(2)]);
        let result = BattleshipMsg::try_from(val);
        assert!(result.is_err());
    }
}
