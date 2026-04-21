//! TCP-backed [`Transport`] implementation.
//!
//! `TcpTransport` implements the `avm_core::transport::Transport` trait so the
//! AVM interpreter can transparently call objects on remote machines over TCP.
//!
//! # Threading model
//!
//! `remote_call` is a *blocking* method (called from `spawn_blocking`).  It:
//! 1. Registers a oneshot channel in the pending map.
//! 2. Sends a `Call` message over the outbound mpsc channel (non-blocking).
//! 3. Blocks on the oneshot receiver via `blocking_recv`.
//!
//! The async inbound-dispatch task delivers the `CallResponse` and wakes us.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use avm_core::error::{AVMError, MachineError};
use avm_core::transport::Transport;
use avm_core::types::{MachineId, ObjectId, Val};
use tokio::sync::mpsc;

use crate::directory::LocationDirectory;
use crate::remote_call::{register_pending, PendingMap};

/// An outbound message queued by the transport and routed by the node.
#[derive(Debug)]
pub struct OutboundMsg {
    /// Destination machine.
    pub target_machine: MachineId,
    /// The wire message to send.
    pub message: crate::protocol::NodeMessage,
}

/// TCP-backed transport for cross-node object calls.
pub struct TcpTransport {
    /// This node's machine identifier.
    pub local_machine: MachineId,
    /// Channel to hand outbound messages to the router task.
    pub outbound_tx: mpsc::UnboundedSender<OutboundMsg>,
    /// Shared map of in-flight request IDs → oneshot senders.
    pub pending_map: Arc<PendingMap>,
    /// Monotonic counter for generating unique request IDs.
    pub request_counter: Arc<AtomicU64>,
    /// Location directory for resolving object → machine.
    #[allow(dead_code)]
    pub directory: LocationDirectory,
}

impl Transport for TcpTransport {
    fn remote_call(
        &self,
        target_machine: &MachineId,
        target: ObjectId,
        input: Val,
        sender: ObjectId,
    ) -> Result<Val, AVMError> {
        let request_id = self.request_counter.fetch_add(1, Ordering::Relaxed);

        // Register the pending slot before enqueuing the message, to avoid a
        // race where the response arrives before we've registered.
        let rx = register_pending(&self.pending_map, request_id);

        let msg = crate::protocol::NodeMessage::Call {
            request_id,
            target,
            input,
            sender,
            sender_machine: self.local_machine.clone(),
        };

        self.outbound_tx
            .send(OutboundMsg {
                target_machine: target_machine.clone(),
                message: msg,
            })
            .map_err(|_| MachineError::Unreachable(target_machine.clone()))?;

        // Block the current (spawn_blocking) thread until the response arrives.
        let result = rx
            .blocking_recv()
            .map_err(|_| MachineError::Unreachable(target_machine.clone()))?;

        result.map_err(|e| {
            AVMError::Machine(MachineError::Unreachable(MachineId(format!(
                "{target_machine} — remote error: {e}"
            ))))
        })
    }
}
