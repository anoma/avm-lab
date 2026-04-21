//! Wire protocol for inter-node communication.
//!
//! Messages are framed as length-prefixed JSON: a 4-byte big-endian length
//! followed by a UTF-8 JSON payload. This keeps the protocol simple and
//! debuggable while remaining compatible with tokio's async I/O.

use avm_core::types::{MachineId, ObjectId, Val};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::NodeError;

/// Messages exchanged between AVM nodes over TCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeMessage {
    /// A remote call from one object to another on this node.
    Call {
        /// Unique request ID for correlating responses.
        request_id: u64,
        /// The target object that should handle this call.
        target: ObjectId,
        /// The input message.
        input: Val,
        /// The originating object (sender).
        sender: ObjectId,
        /// The machine the sender is on (for routing the response back).
        sender_machine: MachineId,
    },
    /// The response to a previously received `Call`.
    CallResponse {
        /// Matches the `request_id` from the corresponding `Call`.
        request_id: u64,
        /// The result: `Ok(val)` on success, `Err(msg)` on failure.
        result: Result<Val, String>,
    },
    /// Broadcast when a node creates a new object, so others can update their
    /// location directory.
    CreateNotify {
        object_id: ObjectId,
        machine_id: MachineId,
    },
    /// Broadcast when a node destroys an object.
    DestroyNotify { object_id: ObjectId },
}

/// Write a single [`NodeMessage`] as a length-prefixed JSON frame.
///
/// Frame layout: `[u32 big-endian length][JSON bytes]`
pub async fn write_frame<W>(writer: &mut W, msg: &NodeMessage) -> Result<(), NodeError>
where
    W: AsyncWrite + Unpin,
{
    let payload = serde_json::to_vec(msg).map_err(NodeError::Serialize)?;
    let len = u32::try_from(payload.len()).map_err(|_| NodeError::FrameTooLarge(payload.len()))?;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Read a single [`NodeMessage`] from a length-prefixed JSON frame.
///
/// Returns `None` if the connection was closed cleanly (EOF at frame boundary).
pub async fn read_frame<R>(reader: &mut R) -> Result<Option<NodeMessage>, NodeError>
where
    R: AsyncRead + Unpin,
{
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(NodeError::Io(e)),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(NodeError::FrameTooLarge(len));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let msg = serde_json::from_slice(&buf).map_err(NodeError::Deserialize)?;
    Ok(Some(msg))
}

/// Maximum allowed frame size: 16 MiB.
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn roundtrip_call_message() {
        let msg = NodeMessage::Call {
            request_id: 42,
            target: ObjectId(1),
            input: Val::Nat(99),
            sender: ObjectId(2),
            sender_machine: MachineId("alpha".into()),
        };

        let (mut client, mut server) = duplex(4096);
        write_frame(&mut client, &msg).await.unwrap();
        drop(client);

        let received = read_frame(&mut server).await.unwrap().unwrap();
        match received {
            NodeMessage::Call {
                request_id,
                target,
                input,
                ..
            } => {
                assert_eq!(request_id, 42);
                assert_eq!(target, ObjectId(1));
                assert_eq!(input, Val::Nat(99));
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn roundtrip_call_response_ok() {
        let msg = NodeMessage::CallResponse {
            request_id: 7,
            result: Ok(Val::Bool(true)),
        };

        let (mut client, mut server) = duplex(4096);
        write_frame(&mut client, &msg).await.unwrap();
        drop(client);

        let received = read_frame(&mut server).await.unwrap().unwrap();
        match received {
            NodeMessage::CallResponse { request_id, result } => {
                assert_eq!(request_id, 7);
                assert_eq!(result.unwrap(), Val::Bool(true));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[tokio::test]
    async fn roundtrip_call_response_err() {
        let msg = NodeMessage::CallResponse {
            request_id: 3,
            result: Err("object not found".into()),
        };

        let (mut client, mut server) = duplex(4096);
        write_frame(&mut client, &msg).await.unwrap();
        drop(client);

        let received = read_frame(&mut server).await.unwrap().unwrap();
        match received {
            NodeMessage::CallResponse { request_id, result } => {
                assert_eq!(request_id, 3);
                assert!(result.is_err());
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[tokio::test]
    async fn eof_returns_none() {
        let (client, mut server) = duplex(4096);
        drop(client);
        let result = read_frame(&mut server).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn multiple_frames_in_sequence() {
        let msgs = vec![
            NodeMessage::CreateNotify {
                object_id: ObjectId(10),
                machine_id: MachineId("beta".into()),
            },
            NodeMessage::DestroyNotify {
                object_id: ObjectId(10),
            },
        ];

        let (mut client, mut server) = duplex(4096);
        for msg in &msgs {
            write_frame(&mut client, msg).await.unwrap();
        }
        drop(client);

        for _ in 0..msgs.len() {
            let r = read_frame(&mut server).await.unwrap();
            assert!(r.is_some());
        }
        let eof = read_frame(&mut server).await.unwrap();
        assert!(eof.is_none());
    }
}
