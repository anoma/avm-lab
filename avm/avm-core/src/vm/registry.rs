//! Behavior registry for resolving object behavior names to programs.
//!
//! Replaces the raw `BehaviorResolver` closure with a structured registry
//! that supports registration, lookup, listing, and clear error reporting.

use crate::error::{AVMError, ObjError};
use crate::instruction::Instruction;
use crate::itree::ITree;
use crate::types::Val;
use rustc_hash::FxHashMap;

/// A behavior factory: given an input message, produces the program tree.
pub type BehaviorFn = Box<dyn Fn(Val) -> ITree<Instruction, Val>>;

/// A structured registry of named object behaviors.
///
/// Use this instead of raw closures to get:
/// - Clear error messages on unknown behavior names
/// - Ability to list available behaviors
/// - Compile-time-like safety through startup-time registration
pub struct BehaviorRegistry {
    behaviors: FxHashMap<String, BehaviorFn>,
}

impl BehaviorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            behaviors: FxHashMap::default(),
        }
    }

    /// Register a named behavior. Panics if the name is already registered.
    pub fn register(&mut self, name: impl Into<String>, factory: BehaviorFn) {
        let name = name.into();
        assert!(
            !self.behaviors.contains_key(&name),
            "behavior '{name}' already registered"
        );
        self.behaviors.insert(name, factory);
    }

    /// Look up a behavior by name and produce its program tree.
    pub fn resolve(&self, name: &str, input: Val) -> Result<ITree<Instruction, Val>, AVMError> {
        let factory = self
            .behaviors
            .get(name)
            .ok_or_else(|| ObjError::BehaviorNotFound(name.to_string()))?;
        Ok(factory(input))
    }

    /// List all registered behavior names.
    pub fn names(&self) -> Vec<&str> {
        self.behaviors.keys().map(String::as_str).collect()
    }

    /// Check whether a behavior name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.behaviors.contains_key(name)
    }

    /// Number of registered behaviors.
    pub fn len(&self) -> usize {
        self.behaviors.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }
}

impl Default for BehaviorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BehaviorRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehaviorRegistry")
            .field("behaviors", &self.behaviors.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::itree::ret;

    #[test]
    fn register_and_resolve() {
        let mut reg = BehaviorRegistry::new();
        reg.register("echo", Box::new(ret));
        assert!(reg.contains("echo"));
        assert_eq!(reg.len(), 1);

        let tree = reg.resolve("echo", Val::Nat(42)).unwrap();
        assert!(matches!(tree, ITree::Ret(Val::Nat(42))));
    }

    #[test]
    fn unknown_behavior_returns_error() {
        let reg = BehaviorRegistry::new();
        let result = reg.resolve("nonexistent", Val::Nothing);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("nonexistent"),
            "error should name the behavior"
        );
    }

    #[test]
    fn list_names() {
        let mut reg = BehaviorRegistry::new();
        reg.register("alpha", Box::new(|_| ret(Val::Nothing)));
        reg.register("beta", Box::new(|_| ret(Val::Nothing)));
        let mut names = reg.names();
        names.sort_unstable();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    #[should_panic(expected = "already registered")]
    fn duplicate_registration_panics() {
        let mut reg = BehaviorRegistry::new();
        reg.register("dup", Box::new(|_| ret(Val::Nothing)));
        reg.register("dup", Box::new(|_| ret(Val::Nothing)));
    }
}
