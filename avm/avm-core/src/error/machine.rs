//! Machine-layer errors.

use crate::types::MachineId;

#[derive(Debug, thiserror::Error)]
pub enum MachineError {
    #[error("machine unreachable: {0}")]
    Unreachable(MachineId),

    #[error("invalid machine transfer")]
    InvalidTransfer,

    #[error("teleport forbidden during active transaction")]
    TeleportDuringTx,
}
