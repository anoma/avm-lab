//! The universal runtime value type.
//!
//! Every message, state element, and computation result in the AVM is a [`Val`].
//! This mirrors the Agda specification's `Val` type from `Background.BasicTypes`.

use std::fmt;

/// The universal runtime value.
///
/// All AVM data flows through this type: messages between objects, internal
/// state, pure function arguments and results. The variants cover the primitive
/// types needed by the specification.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Val {
    /// Natural number (non-negative integer).
    Nat(u64),
    /// Boolean value.
    Bool(bool),
    /// UTF-8 string.
    Str(String),
    /// Ordered sequence of values.
    List(Vec<Val>),
    /// Pair of two values.
    Pair(Box<Val>, Box<Val>),
    /// Absence of a value (the `nothing` case of `Maybe`).
    Nothing,
    /// Presence of a value (the `just` case of `Maybe`).
    Just(Box<Val>),
    /// Reference to an AVM object.
    ObjectRef(super::ObjectId),
    /// Reference to a transaction.
    TxRef(super::TxId),
}

/// Alias matching the specification: inputs to object behaviors.
pub type Input = Val;

/// Alias matching the specification: outputs from object behaviors.
pub type Output = Val;

impl Val {
    /// Create a string value.
    pub fn str(s: impl Into<String>) -> Self {
        Self::Str(s.into())
    }

    /// Create a list value.
    pub fn list(items: Vec<Val>) -> Self {
        Self::List(items)
    }

    /// Create a pair value.
    pub fn pair(a: Val, b: Val) -> Self {
        Self::Pair(Box::new(a), Box::new(b))
    }

    /// Create a `Just` value.
    pub fn just(v: Val) -> Self {
        Self::Just(Box::new(v))
    }

    /// Returns `true` if this is `Nothing`.
    pub fn is_nothing(&self) -> bool {
        matches!(self, Self::Nothing)
    }

    /// Try to extract a natural number.
    pub fn as_nat(&self) -> Option<u64> {
        match self {
            Self::Nat(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to extract a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to extract a string reference.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract an [`ObjectId`](super::ObjectId) from an `ObjectRef`.
    pub fn as_object_id(&self) -> Option<super::ObjectId> {
        match self {
            Self::ObjectRef(id) => Some(*id),
            _ => None,
        }
    }

    /// Try to extract a [`TxId`](super::TxId) from a `TxRef`.
    pub fn as_tx_id(&self) -> Option<super::TxId> {
        match self {
            Self::TxRef(id) => Some(*id),
            _ => None,
        }
    }

    /// Try to unwrap a `Just`, returning the inner value.
    pub fn unwrap_just(self) -> Option<Val> {
        match self {
            Self::Just(v) => Some(*v),
            _ => None,
        }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nat(n) => write!(f, "{n}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Str(s) => write!(f, "\"{s}\""),
            Self::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Self::Pair(a, b) => write!(f, "({a}, {b})"),
            Self::Nothing => write!(f, "nothing"),
            Self::Just(v) => write!(f, "just {v}"),
            Self::ObjectRef(id) => write!(f, "obj:{}", id.0),
            Self::TxRef(id) => write!(f, "tx:{}", id.0),
        }
    }
}

impl From<u64> for Val {
    fn from(n: u64) -> Self {
        Self::Nat(n)
    }
}

impl From<bool> for Val {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<String> for Val {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<&str> for Val {
    fn from(s: &str) -> Self {
        Self::Str(s.to_owned())
    }
}

impl From<super::ObjectId> for Val {
    fn from(id: super::ObjectId) -> Self {
        Self::ObjectRef(id)
    }
}

impl From<super::TxId> for Val {
    fn from(id: super::TxId) -> Self {
        Self::TxRef(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ObjectId, TxId};

    #[test]
    fn val_equality() {
        assert_eq!(Val::Nat(42), Val::Nat(42));
        assert_ne!(Val::Nat(1), Val::Nat(2));
        assert_eq!(Val::Nothing, Val::Nothing);
        assert_eq!(Val::str("hello"), Val::Str("hello".into()));
    }

    #[test]
    fn val_display() {
        assert_eq!(Val::Nat(42).to_string(), "42");
        assert_eq!(Val::Bool(true).to_string(), "true");
        assert_eq!(Val::str("hi").to_string(), "\"hi\"");
        assert_eq!(Val::Nothing.to_string(), "nothing");
        assert_eq!(Val::just(Val::Nat(1)).to_string(), "just 1");
        assert_eq!(
            Val::list(vec![Val::Nat(1), Val::Nat(2)]).to_string(),
            "[1, 2]"
        );
        assert_eq!(Val::ObjectRef(ObjectId(7)).to_string(), "obj:7");
        assert_eq!(Val::TxRef(TxId(3)).to_string(), "tx:3");
    }

    #[test]
    fn val_conversions() {
        assert_eq!(Val::from(10u64), Val::Nat(10));
        assert_eq!(Val::from(true), Val::Bool(true));
        assert_eq!(Val::from("test"), Val::str("test"));
        assert_eq!(Val::from(ObjectId(5)), Val::ObjectRef(ObjectId(5)));
        assert_eq!(Val::from(TxId(2)), Val::TxRef(TxId(2)));
    }

    #[test]
    fn val_accessors() {
        assert_eq!(Val::Nat(5).as_nat(), Some(5));
        assert_eq!(Val::Bool(true).as_bool(), Some(true));
        assert_eq!(Val::str("x").as_str(), Some("x"));
        assert_eq!(Val::Nat(5).as_bool(), None);
        assert!(Val::Nothing.is_nothing());
        assert!(!Val::Nat(0).is_nothing());
        assert_eq!(
            Val::ObjectRef(ObjectId(9)).as_object_id(),
            Some(ObjectId(9))
        );
        assert_eq!(Val::TxRef(TxId(4)).as_tx_id(), Some(TxId(4)));
        assert_eq!(Val::Nat(0).as_object_id(), None);
        assert_eq!(Val::Nat(0).as_tx_id(), None);
    }

    #[test]
    fn val_unwrap_just() {
        assert_eq!(Val::just(Val::Nat(7)).unwrap_just(), Some(Val::Nat(7)));
        assert_eq!(Val::Nothing.unwrap_just(), None);
    }

    #[test]
    fn val_nested_structures() {
        let nested = Val::list(vec![
            Val::pair(Val::Nat(1), Val::str("a")),
            Val::just(Val::Bool(false)),
        ]);
        assert_eq!(nested.to_string(), "[(1, \"a\"), just false]");
    }
}
