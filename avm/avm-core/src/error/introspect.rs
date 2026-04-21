//! Introspection-layer errors.

#[derive(Debug, thiserror::Error)]
pub enum IntrospectError {
    #[error("execution context unavailable")]
    ContextUnavailable,
}
