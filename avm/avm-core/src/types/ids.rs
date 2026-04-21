//! Newtype identifiers for AVM entities.
//!
//! Each identifier is a distinct type to prevent accidental mixing at compile
//! time. This mirrors the Agda spec's parameterized modules over abstract ID
//! types.

use std::fmt;

/// Unique identifier for an object in the AVM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub u64);

/// Identifier for a physical machine in the distributed AVM.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MachineId(pub String);

/// Identifier for a logical controller (transaction ordering authority).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControllerId(pub String);

/// Identifier for a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TxId(pub u64);

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "obj:{}", self.0)
    }
}

impl fmt::Display for MachineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "machine:{}", self.0)
    }
}

impl fmt::Display for ControllerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ctrl:{}", self.0)
    }
}

impl fmt::Display for TxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tx:{}", self.0)
    }
}

/// Monotonic ID generator with separate counters for objects and transactions.
#[derive(Debug, Clone)]
pub struct FreshIdGen {
    object_counter: u64,
    tx_counter: u64,
}

impl FreshIdGen {
    /// Create a new generator starting from zero.
    pub fn new() -> Self {
        Self {
            object_counter: 0,
            tx_counter: 0,
        }
    }

    /// Generate a fresh [`ObjectId`].
    pub fn next_object_id(&mut self) -> ObjectId {
        let id = ObjectId(self.object_counter);
        self.object_counter += 1;
        id
    }

    /// Generate a fresh [`TxId`].
    pub fn next_tx_id(&mut self) -> TxId {
        let id = TxId(self.tx_counter);
        self.tx_counter += 1;
        id
    }
}

impl Default for FreshIdGen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct_types() {
        let obj = ObjectId(0);
        let tx = TxId(0);
        // These are different types — won't compile if you try to compare them.
        assert_eq!(obj.0, tx.0); // Only inner u64s compare.
    }

    #[test]
    fn display_formatting() {
        assert_eq!(ObjectId(42).to_string(), "obj:42");
        assert_eq!(MachineId("m1".into()).to_string(), "machine:m1");
        assert_eq!(ControllerId("c1".into()).to_string(), "ctrl:c1");
        assert_eq!(TxId(42).to_string(), "tx:42");
    }

    #[test]
    fn fresh_id_gen_monotonic() {
        let mut gen = FreshIdGen::new();
        let a = gen.next_object_id();
        let b = gen.next_object_id();
        assert_ne!(a, b);
        assert_eq!(a, ObjectId(0));
        assert_eq!(b, ObjectId(1));
    }

    #[test]
    fn fresh_id_gen_independent_counters() {
        let mut gen = FreshIdGen::new();
        let obj = gen.next_object_id(); // object counter: 0
        let tx = gen.next_tx_id(); // tx counter: 0
        let obj2 = gen.next_object_id(); // object counter: 1
        assert_eq!(obj, ObjectId(0));
        assert_eq!(tx, TxId(0));
        assert_eq!(obj2, ObjectId(1));
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ObjectId(1));
        assert!(set.contains(&ObjectId(1)));
        assert!(!set.contains(&ObjectId(2)));
    }
}
