//! Composed error hierarchy mirroring the specification's layered structure.
//!
//! The composition chain enables `?` propagation from any depth:
//! `ObjError` → `BaseError` → `TxLayerError` → `PureLayerError` → `AVMError`.

use super::{
    ControllerError, FdError, IntrospectError, MachineError, NondetError, ObjError, PureError,
    ReflectError, ReifyError, TxError,
};

/// Layer 0 errors: objects, introspection, reflection, reification.
#[derive(Debug, thiserror::Error)]
pub enum BaseError {
    #[error(transparent)]
    Obj(#[from] ObjError),
    #[error(transparent)]
    Introspect(#[from] IntrospectError),
    #[error(transparent)]
    Reflect(#[from] ReflectError),
    #[error(transparent)]
    Reify(#[from] ReifyError),
}

/// Layer 1 errors: base + transactions.
#[derive(Debug, thiserror::Error)]
pub enum TxLayerError {
    #[error(transparent)]
    Base(#[from] BaseError),
    #[error(transparent)]
    Tx(#[from] TxError),
}

/// Layer 2 errors: transactions + pure functions.
#[derive(Debug, thiserror::Error)]
pub enum PureLayerError {
    #[error(transparent)]
    TxLayer(#[from] TxLayerError),
    #[error(transparent)]
    Pure(#[from] PureError),
}

/// The top-level AVM error type encompassing all layers.
#[derive(Debug, thiserror::Error)]
pub enum AVMError {
    #[error(transparent)]
    PureLayer(#[from] PureLayerError),
    #[error(transparent)]
    Machine(#[from] MachineError),
    #[error(transparent)]
    Controller(#[from] ControllerError),
    #[error(transparent)]
    Fd(#[from] FdError),
    #[error(transparent)]
    Nondet(#[from] NondetError),
}

// --- Convenience From impls for direct conversion from leaf errors to AVMError ---

impl From<ObjError> for AVMError {
    fn from(e: ObjError) -> Self {
        Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(BaseError::Obj(
            e,
        ))))
    }
}

impl From<IntrospectError> for AVMError {
    fn from(e: IntrospectError) -> Self {
        Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(
            BaseError::Introspect(e),
        )))
    }
}

impl From<ReflectError> for AVMError {
    fn from(e: ReflectError) -> Self {
        Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(
            BaseError::Reflect(e),
        )))
    }
}

impl From<ReifyError> for AVMError {
    fn from(e: ReifyError) -> Self {
        Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(
            BaseError::Reify(e),
        )))
    }
}

impl From<TxError> for AVMError {
    fn from(e: TxError) -> Self {
        Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Tx(e)))
    }
}

impl From<PureError> for AVMError {
    fn from(e: PureError) -> Self {
        Self::PureLayer(PureLayerError::Pure(e))
    }
}

impl AVMError {
    /// Check if this is an object-not-found error.
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(BaseError::Obj(
                ObjError::NotFound(_)
            ))))
        )
    }

    /// Check if this is a transaction conflict error.
    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Tx(
                TxError::Conflict(_)
            )))
        )
    }

    /// Check if this is a behavior-not-found error.
    pub fn is_behavior_not_found(&self) -> bool {
        matches!(
            self,
            Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(BaseError::Obj(
                ObjError::BehaviorNotFound(_)
            ))))
        )
    }

    /// Check if this is a no-active-transaction error.
    pub fn is_no_active_tx(&self) -> bool {
        matches!(
            self,
            Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Tx(
                TxError::NoActiveTx
            )))
        )
    }

    /// Try to extract the underlying [`ObjError`].
    pub fn as_obj_error(&self) -> Option<&ObjError> {
        match self {
            Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(BaseError::Obj(e)))) => {
                Some(e)
            }
            _ => None,
        }
    }

    /// Try to extract the underlying [`TxError`].
    pub fn as_tx_error(&self) -> Option<&TxError> {
        match self {
            Self::PureLayer(PureLayerError::TxLayer(TxLayerError::Tx(e))) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ObjectId;

    #[test]
    fn leaf_error_converts_to_avm_error() {
        let obj_err = ObjError::NotFound(ObjectId(0));
        let avm_err: AVMError = obj_err.into();
        assert!(matches!(
            avm_err,
            AVMError::PureLayer(PureLayerError::TxLayer(TxLayerError::Base(BaseError::Obj(
                ObjError::NotFound(_)
            ))))
        ));
    }

    #[test]
    fn tx_error_converts() {
        let tx_err = TxError::NoActiveTx;
        let avm_err: AVMError = tx_err.into();
        let msg = avm_err.to_string();
        assert_eq!(msg, "no active transaction");
    }

    #[test]
    fn machine_error_converts() {
        let err = MachineError::TeleportDuringTx;
        let avm_err: AVMError = err.into();
        assert!(matches!(avm_err, AVMError::Machine(_)));
    }

    #[test]
    fn error_display_messages() {
        let err = ObjError::NotFound(ObjectId(42));
        assert_eq!(err.to_string(), "object not found: obj:42");
    }

    #[test]
    fn is_not_found_true_for_obj_not_found() {
        let err: AVMError = ObjError::NotFound(ObjectId(1)).into();
        assert!(err.is_not_found());
        assert!(!err.is_conflict());
        assert!(!err.is_behavior_not_found());
        assert!(!err.is_no_active_tx());
    }

    #[test]
    fn is_not_found_false_for_other_errors() {
        let err: AVMError = TxError::NoActiveTx.into();
        assert!(!err.is_not_found());
    }

    #[test]
    fn is_conflict_true_for_tx_conflict() {
        use crate::types::TxId;
        let err: AVMError = TxError::Conflict(TxId(9)).into();
        assert!(err.is_conflict());
        assert!(!err.is_not_found());
    }

    #[test]
    fn is_behavior_not_found_true() {
        let err: AVMError = ObjError::BehaviorNotFound("foo".into()).into();
        assert!(err.is_behavior_not_found());
        assert!(!err.is_not_found());
    }

    #[test]
    fn is_no_active_tx_true() {
        let err: AVMError = TxError::NoActiveTx.into();
        assert!(err.is_no_active_tx());
        assert!(!err.is_conflict());
    }

    #[test]
    fn as_obj_error_extracts_obj_error() {
        let err: AVMError = ObjError::NotFound(ObjectId(5)).into();
        assert!(matches!(err.as_obj_error(), Some(ObjError::NotFound(_))));
        assert!(err.as_tx_error().is_none());
    }

    #[test]
    fn as_tx_error_extracts_tx_error() {
        let err: AVMError = TxError::NoActiveTx.into();
        assert!(matches!(err.as_tx_error(), Some(TxError::NoActiveTx)));
        assert!(err.as_obj_error().is_none());
    }

    #[test]
    fn is_not_found_used_in_error_check() {
        // Verify is_not_found() can be used as a predicate in real error handling.
        let err: AVMError = ObjError::NotFound(ObjectId(42)).into();
        let handled = if err.is_not_found() {
            "missing"
        } else {
            "other"
        };
        assert_eq!(handled, "missing");
    }
}
