//! Interaction trees — the program model for the AVM.
//!
//! An interaction tree is a (possibly infinite) tree that describes a
//! computation interleaved with observable effects. In the AVM, effects are
//! instructions and the tree structure captures sequencing and branching.
//!
//! This module provides a finite, heap-allocated encoding suitable for a
//! single-threaded interpreter. Programs are built by chaining [`trigger`]
//! calls with [`bind`](monad::bind) or the [`avm_do!`] macro.

pub mod monad;

use crate::types::Val;

/// An interaction tree over effect type `E` producing a result of type `A`.
///
/// This is the Rust encoding of the Agda coinductive `ITree` record. Since
/// our programs are finite, we use `Box` for indirection rather than
/// coinductive laziness.
///
/// # Variants
///
/// - `Ret(a)` — the computation has finished with result `a`.
/// - `Tau(next)` — a silent internal step (no observable effect).
/// - `Vis(event, cont)` — an observable effect: emit `event`, then feed
///   the environment's response (a [`Val`]) into `cont` to get the next tree.
pub enum ITree<E, A> {
    /// Terminal node — computation complete.
    Ret(A),
    /// Silent step — no observable effect, just progress.
    Tau(Box<ITree<E, A>>),
    /// Visible effect — emit an event and continue based on the response.
    Vis(E, Continuation<E, A>),
}

impl<E, A> std::fmt::Debug for ITree<E, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ret(_) => write!(f, "Ret(..)"),
            Self::Tau(_) => write!(f, "Tau(..)"),
            Self::Vis(_, _) => write!(f, "Vis(..)"),
        }
    }
}

/// A boxed continuation: given a response value, produces the next tree.
///
/// The `Send` bound allows interaction trees (and the programs they encode) to
/// be moved across OS-thread boundaries. All concrete continuations built with
/// `avm_do!` or `trigger` satisfy this bound naturally.
pub type Continuation<E, A> = Box<dyn FnOnce(Val) -> ITree<E, A> + Send>;

/// Lift an event into a one-step interaction tree.
///
/// The resulting tree emits the event and returns the response directly.
/// This is the primary way to construct programs: chain `trigger` calls
/// using [`bind`](monad::bind) or [`avm_do!`].
pub fn trigger<E: 'static>(event: E) -> ITree<E, Val> {
    ITree::Vis(event, Box::new(ITree::Ret))
}

/// Convenience: create a terminal tree.
pub fn ret<E, A>(value: A) -> ITree<E, A> {
    ITree::Ret(value)
}

/// Convenience: wrap in a silent step.
pub fn tau<E, A>(next: ITree<E, A>) -> ITree<E, A> {
    ITree::Tau(Box::new(next))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ret_is_terminal() {
        let tree: ITree<String, Val> = ret(Val::Nat(42));
        assert!(matches!(tree, ITree::Ret(Val::Nat(42))));
    }

    #[test]
    fn trigger_creates_vis_node() {
        let tree: ITree<String, Val> = trigger("hello".to_string());
        match tree {
            ITree::Vis(event, cont) => {
                assert_eq!(event, "hello");
                let next = cont(Val::Nat(1));
                assert!(matches!(next, ITree::Ret(Val::Nat(1))));
            }
            _ => panic!("expected Vis"),
        }
    }

    #[test]
    fn tau_wraps() {
        let tree: ITree<String, Val> = tau(ret(Val::Bool(true)));
        match tree {
            ITree::Tau(inner) => assert!(matches!(*inner, ITree::Ret(Val::Bool(true)))),
            _ => panic!("expected Tau"),
        }
    }
}
