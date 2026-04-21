//! Monadic operations on interaction trees.
//!
//! Provides `bind` (sequencing), `map`, and the `avm_do!` macro for
//! ergonomic program construction.

use super::{Continuation, ITree};

/// Monadic bind: sequence two computations.
///
/// Runs `tree` to completion, then feeds its result into `f` to get the
/// next computation. This is the fundamental sequencing combinator.
pub fn bind<E, A, B>(
    tree: ITree<E, A>,
    f: impl FnOnce(A) -> ITree<E, B> + Send + 'static,
) -> ITree<E, B>
where
    E: 'static,
    A: 'static + Send,
    B: 'static,
{
    match tree {
        ITree::Ret(a) => f(a),
        ITree::Tau(next) => ITree::Tau(Box::new(bind(*next, f))),
        ITree::Vis(event, cont) => {
            let new_cont: Continuation<E, B> = Box::new(move |val| bind(cont(val), f));
            ITree::Vis(event, new_cont)
        }
    }
}

/// Functor map: transform the result of a computation.
pub fn map<E, A, B>(tree: ITree<E, A>, f: impl FnOnce(A) -> B + Send + 'static) -> ITree<E, B>
where
    E: 'static,
    A: 'static + Send,
    B: 'static,
{
    bind(tree, move |a| ITree::Ret(f(a)))
}

/// Do-notation macro for building AVM programs.
///
/// # Example
///
/// ```ignore
/// use avm_core::avm_do;
///
/// let program = avm_do! {
///     let x <- trigger(some_instruction());
///     let y <- trigger(another_instruction());
///     ret(Val::pair(x, y))
/// };
/// ```
#[macro_export]
macro_rules! avm_do {
    // Bind: let x <- expr; rest
    (let $x:ident <- $e:expr; $($rest:tt)*) => {
        $crate::itree::monad::bind($e, move |$x| $crate::avm_do!($($rest)*))
    };
    // Pure let: let x = expr; rest (plain variable binding, no monadic bind)
    (let $x:ident = $e:expr; $($rest:tt)*) => {{
        let $x = $e;
        $crate::avm_do!($($rest)*)
    }};
    // Mutable let: let mut x = expr; rest
    (let mut $x:ident = $e:expr; $($rest:tt)*) => {{
        let mut $x = $e;
        $crate::avm_do!($($rest)*)
    }};
    // Sequence: expr; rest (discard result)
    ($e:expr; $($rest:tt)*) => {
        $crate::itree::monad::bind($e, move |_| $crate::avm_do!($($rest)*))
    };
    // Terminal expression
    ($e:expr) => {
        $e
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::itree::{ret, trigger};
    use crate::types::Val;

    #[test]
    fn bind_ret_is_f() {
        // Left identity: bind(ret(a), f) == f(a)
        let tree: ITree<String, Val> = bind(ret(Val::Nat(5)), ret);
        assert!(matches!(tree, ITree::Ret(Val::Nat(5))));
    }

    #[test]
    fn bind_tree_ret_is_tree() {
        // Right identity: bind(tree, ret) == tree
        let tree: ITree<String, Val> = trigger("event".to_string());
        let bound = bind(tree, ret);
        // Should be Vis("event", ...) where cont returns Ret
        match bound {
            ITree::Vis(event, cont) => {
                assert_eq!(event, "event");
                let result = cont(Val::Nat(99));
                assert!(matches!(result, ITree::Ret(Val::Nat(99))));
            }
            _ => panic!("expected Vis"),
        }
    }

    #[test]
    fn map_transforms_result() {
        let tree: ITree<String, Val> = ret(Val::Nat(10));
        let mapped = map(tree, |v| {
            let n = v.as_nat().unwrap();
            Val::Nat(n * 2)
        });
        assert!(matches!(mapped, ITree::Ret(Val::Nat(20))));
    }

    #[test]
    fn avm_do_sequencing() {
        let program: ITree<String, Val> = avm_do! {
            let x <- trigger("first".to_string());
            let _y <- trigger("second".to_string());
            ret(x)
        };
        // First node should be Vis("first", ...)
        match program {
            ITree::Vis(event, cont) => {
                assert_eq!(event, "first");
                // Feed response, get second Vis
                let next = cont(Val::Nat(1));
                match next {
                    ITree::Vis(event2, cont2) => {
                        assert_eq!(event2, "second");
                        let final_tree = cont2(Val::Nat(2));
                        // Should return the first value (1), not the second
                        assert!(matches!(final_tree, ITree::Ret(Val::Nat(1))));
                    }
                    _ => panic!("expected second Vis"),
                }
            }
            _ => panic!("expected first Vis"),
        }
    }

    #[test]
    fn avm_do_discard() {
        let program: ITree<String, Val> = avm_do! {
            trigger("ignored".to_string());
            ret(Val::Bool(true))
        };
        match program {
            ITree::Vis(_, cont) => {
                let next = cont(Val::Nothing);
                assert!(matches!(next, ITree::Ret(Val::Bool(true))));
            }
            _ => panic!("expected Vis"),
        }
    }
}
