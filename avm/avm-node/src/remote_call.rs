//! Pending remote-call tracking.
//!
//! When the transport fires a remote call, it registers a oneshot channel
//! in the pending map keyed by request ID. The inbound dispatch task
//! completes the pending entry when the `CallResponse` arrives.
//!
//! We use `std::sync::Mutex` (not tokio's) because these maps are accessed
//! from `spawn_blocking` threads as well as async tasks.

use std::sync::Mutex;

use avm_core::types::Val;
use rustc_hash::FxHashMap;
use tokio::sync::oneshot;

/// Sender half stored while a remote call is in flight.
type Tx = oneshot::Sender<Result<Val, String>>;

/// The shared map from request ID → waiting oneshot sender.
pub type PendingMap = Mutex<FxHashMap<u64, Tx>>;

/// Create a new empty pending map.
pub fn new_pending_map() -> PendingMap {
    Mutex::new(FxHashMap::default())
}

/// Register a new pending call and return the receiver end to block on.
///
/// The caller should send [`OutboundMsg::Call`] over the outbound channel
/// immediately after calling this function.
pub fn register_pending(
    pending: &PendingMap,
    request_id: u64,
) -> oneshot::Receiver<Result<Val, String>> {
    let (tx, rx) = oneshot::channel();
    pending
        .lock()
        .expect("pending map lock poisoned")
        .insert(request_id, tx);
    rx
}

/// Complete a pending call by delivering its result.
///
/// If no pending entry exists for `request_id` the response is silently
/// dropped (the caller may have timed out or been cancelled).
pub fn complete_pending(pending: &PendingMap, request_id: u64, result: Result<Val, String>) {
    let tx = pending
        .lock()
        .expect("pending map lock poisoned")
        .remove(&request_id);
    if let Some(sender) = tx {
        // Ignore send errors — receiver dropped means caller gave up.
        let _ = sender.send(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avm_core::types::Val;

    #[tokio::test]
    async fn register_and_complete_ok() {
        let map = new_pending_map();
        let rx = register_pending(&map, 1);
        complete_pending(&map, 1, Ok(Val::Nat(42)));
        let result = rx.await.expect("channel closed");
        assert_eq!(result.unwrap(), Val::Nat(42));
    }

    #[tokio::test]
    async fn register_and_complete_err() {
        let map = new_pending_map();
        let rx = register_pending(&map, 2);
        complete_pending(&map, 2, Err("oops".into()));
        let result = rx.await.expect("channel closed");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn complete_unknown_request_is_noop() {
        let map = new_pending_map();
        // Completing a non-existent request must not panic.
        complete_pending(&map, 999, Ok(Val::Nothing));
    }

    #[tokio::test]
    async fn map_is_empty_after_completion() {
        let map = new_pending_map();
        let _rx = register_pending(&map, 10);
        complete_pending(&map, 10, Ok(Val::Bool(true)));
        assert!(map.lock().unwrap().is_empty());
    }
}
