//! Pure function registry.
//!
//! An extensible registry of named pure functions that can be called from
//! AVM programs via `callPure`. Functions are versioned for consistency
//! across distributed nodes.

use crate::types::Val;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// A pure function: takes a list of values, returns an optional value.
pub type PureFn = Arc<dyn Fn(&[Val]) -> Option<Val> + Send + Sync>;

/// An entry in the pure function registry.
#[derive(Clone)]
pub struct FunctionEntry {
    pub impl_fn: PureFn,
    pub version: u64,
}

impl std::fmt::Debug for FunctionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionEntry")
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
}

/// Registry of named pure functions.
#[derive(Debug, Clone, Default)]
pub struct PureFunctions {
    registry: FxHashMap<String, FunctionEntry>,
}

impl PureFunctions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a function by name.
    pub fn get(&self, name: &str) -> Option<&FunctionEntry> {
        self.registry.get(name)
    }

    /// Register a new function. Returns `false` if the name already exists.
    pub fn register(&mut self, name: String, impl_fn: PureFn) -> bool {
        if self.registry.contains_key(&name) {
            return false;
        }
        self.registry.insert(
            name,
            FunctionEntry {
                impl_fn,
                version: 0,
            },
        );
        true
    }

    /// Update an existing function. Returns `false` if not found.
    pub fn update(&mut self, name: &str, impl_fn: PureFn) -> bool {
        if let Some(entry) = self.registry.get_mut(name) {
            entry.impl_fn = impl_fn;
            entry.version += 1;
            true
        } else {
            false
        }
    }

    /// Call a named function with the given arguments.
    pub fn call(&self, name: &str, args: &[Val]) -> Option<Option<Val>> {
        self.registry.get(name).map(|entry| (entry.impl_fn)(args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_call() {
        let mut fns = PureFunctions::new();
        let add: PureFn = Arc::new(|args| {
            if let [Val::Nat(a), Val::Nat(b)] = args {
                Some(Val::Nat(a + b))
            } else {
                None
            }
        });
        assert!(fns.register("add".into(), add));
        let result = fns.call("add", &[Val::Nat(2), Val::Nat(3)]);
        assert_eq!(result, Some(Some(Val::Nat(5))));
    }

    #[test]
    fn register_duplicate_fails() {
        let mut fns = PureFunctions::new();
        let noop: PureFn = Arc::new(|_| None);
        assert!(fns.register("f".into(), noop.clone()));
        assert!(!fns.register("f".into(), noop));
    }

    #[test]
    fn update_increments_version() {
        let mut fns = PureFunctions::new();
        let v1: PureFn = Arc::new(|_| Some(Val::Nat(1)));
        let v2: PureFn = Arc::new(|_| Some(Val::Nat(2)));
        fns.register("f".into(), v1);
        assert_eq!(fns.get("f").unwrap().version, 0);
        fns.update("f", v2);
        assert_eq!(fns.get("f").unwrap().version, 1);
    }

    #[test]
    fn call_missing_returns_none() {
        let fns = PureFunctions::new();
        assert_eq!(fns.call("missing", &[]), None);
    }
}
