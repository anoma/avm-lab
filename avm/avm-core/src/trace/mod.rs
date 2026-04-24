//! Observability and event logging.
//!
//! Every state-modifying instruction emits a [`LogEntry`] into the execution
//! [`Trace`]. This provides a complete audit trail for debugging, testing,
//! and formal verification.

use crate::types::{ControllerId, Input, MachineId, ObjectId, Output, TxId};

/// The type of event that occurred during execution.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventType {
    ObjectCreated {
        id: ObjectId,
        behavior_name: String,
    },
    ObjectDestroyed(ObjectId),
    ObjectCalled {
        id: ObjectId,
        input: Input,
        output: Option<Output>,
    },
    MessageReceived {
        id: ObjectId,
        input: Input,
    },
    ObjectMoved {
        id: ObjectId,
        from: MachineId,
        to: MachineId,
    },
    ExecutionMoved {
        from: MachineId,
        to: MachineId,
    },
    ObjectFetched {
        id: ObjectId,
        machine: MachineId,
    },
    ObjectTransferred {
        id: ObjectId,
        from: ControllerId,
        to: ControllerId,
    },
    ObjectFrozen {
        id: ObjectId,
        controller: ControllerId,
    },
    FunctionUpdated(String),
    TransactionStarted(TxId),
    TransactionCommitted(TxId),
    TransactionAborted(TxId),
    StateUpdated(ObjectId),
    ErrorOccurred(String),
}

/// A single log entry in the execution trace.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogEntry {
    pub timestamp: u64,
    pub event_type: EventType,
    pub executing_controller: Option<ControllerId>,
}

/// An ordered sequence of log entries from one execution.
pub type Trace = Vec<LogEntry>;

impl LogEntry {
    /// Create a new log entry with the given timestamp and event.
    pub fn new(timestamp: u64, event_type: EventType, controller: Option<ControllerId>) -> Self {
        Self {
            timestamp,
            event_type,
            executing_controller: controller,
        }
    }
}

/// Count occurrences of a specific event kind in a trace.
pub fn count_events(trace: &Trace, predicate: impl Fn(&EventType) -> bool) -> usize {
    trace.iter().filter(|e| predicate(&e.event_type)).count()
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectCreated { id, behavior_name } => {
                write!(f, "created {id} (behavior: {behavior_name})")
            }
            Self::ObjectDestroyed(id) => write!(f, "destroyed {id}"),
            Self::ObjectCalled { id, input, output } => {
                write!(f, "called {id} with {input}")?;
                if let Some(out) = output {
                    write!(f, " -> {out}")?;
                }
                Ok(())
            }
            Self::MessageReceived { id, input } => {
                write!(f, "{id} received {input}")
            }
            Self::ObjectMoved { id, from, to } => {
                write!(f, "moved {id} from {from} to {to}")
            }
            Self::ExecutionMoved { from, to } => {
                write!(f, "execution moved from {from} to {to}")
            }
            Self::ObjectFetched { id, machine } => {
                write!(f, "fetched {id} to {machine}")
            }
            Self::ObjectTransferred { id, from, to } => {
                write!(f, "transferred {id} from {from} to {to}")
            }
            Self::ObjectFrozen { id, controller } => {
                write!(f, "froze {id} via {controller}")
            }
            Self::FunctionUpdated(name) => write!(f, "updated function {name}"),
            Self::TransactionStarted(id) => write!(f, "tx started: {id}"),
            Self::TransactionCommitted(id) => write!(f, "tx committed: {id}"),
            Self::TransactionAborted(id) => write!(f, "tx aborted: {id}"),
            Self::StateUpdated(id) => write!(f, "state updated: {id}"),
            Self::ErrorOccurred(msg) => write!(f, "error: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_entry_display() {
        let entry = LogEntry::new(
            0,
            EventType::ObjectCreated {
                id: ObjectId(0),
                behavior_name: "ping".into(),
            },
            None,
        );
        let msg = entry.event_type.to_string();
        assert!(msg.contains("created"));
        assert!(msg.contains("ping"));
    }

    #[test]
    fn count_events_filters() {
        let trace = vec![
            LogEntry::new(0, EventType::TransactionStarted(TxId(0)), None),
            LogEntry::new(1, EventType::TransactionCommitted(TxId(0)), None),
            LogEntry::new(2, EventType::TransactionStarted(TxId(1)), None),
        ];
        let starts = count_events(&trace, |e| matches!(e, EventType::TransactionStarted(_)));
        assert_eq!(starts, 2);
    }
}
