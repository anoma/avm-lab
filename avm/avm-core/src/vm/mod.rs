//! VM state and runtime support.

pub mod pure_fn;
pub mod registry;
pub mod state;

pub use pure_fn::{FunctionEntry, PureFunctions};
pub use registry::BehaviorRegistry;
pub use state::State;
