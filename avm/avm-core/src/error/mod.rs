//! Compositional error hierarchy for the AVM.
//!
//! The error types mirror the instruction set layers. Each instruction family
//! has its own error enum, and these compose into the top-level [`AVMError`]
//! via `#[from]` conversions that enable ergonomic `?` propagation.

mod composed;
mod controller;
mod fd;
mod introspect;
mod machine;
mod obj;
mod pure_fn;
mod reflect;
mod reify;
mod tx;

pub use composed::{AVMError, BaseError, PureLayerError, TxLayerError};
pub use controller::ControllerError;
pub use fd::{FdError, NondetError};
pub use introspect::IntrospectError;
pub use machine::MachineError;
pub use obj::ObjError;
pub use pure_fn::PureError;
pub use reflect::ReflectError;
pub use reify::ReifyError;
pub use tx::TxError;
