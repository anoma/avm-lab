//! Server-Sent Events (SSE) HTTP server for streaming AVM trace events.
//!
//! External dashboards and frontends can subscribe to real-time AVM events by
//! connecting to `GET /events`. Each event is a JSON-serialised
//! [`avm_core::trace::LogEntry`] sent as an SSE `data:` line.
//!
//! A `GET /health` endpoint is also provided for liveness checks.
//!
//! # Example
//!
//! ```text
//! curl -N http://localhost:8080/events
//! ```

use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::routing::get;
use axum::Router;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

/// Broadcasts serialised [`avm_core::trace::LogEntry`] JSON strings to all
/// active SSE subscribers.
///
/// The channel is a `tokio::sync::broadcast` so that slow clients do not block
/// the interpreter; lagged receivers simply skip missed messages.
pub struct EventBroadcaster {
    tx: broadcast::Sender<String>,
}

impl EventBroadcaster {
    /// Create a new broadcaster with the given channel capacity.
    ///
    /// Subscribers that fall more than `capacity` messages behind will receive
    /// a [`broadcast::error::RecvError::Lagged`] error and miss those entries.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish a serialised event JSON string to all active SSE subscribers.
    ///
    /// Silently drops the message if there are no current subscribers.
    pub fn publish(&self, event_json: String) {
        // Ignore send errors: they mean no active receivers, which is fine.
        let _ = self.tx.send(event_json);
    }

    /// Subscribe to the broadcast stream.
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

/// Build the axum [`Router`] from an already-`Arc`-wrapped [`EventBroadcaster`].
///
/// This is useful when the caller already holds the broadcaster behind an `Arc`
/// and wants to share it with the node runtime.
pub fn sse_router_arc(broadcaster: Arc<EventBroadcaster>) -> Router {
    Router::new()
        .route("/events", get(sse_handler))
        .route("/health", get(health))
        .with_state(broadcaster)
}

/// `GET /health` — returns `"ok"` with HTTP 200.
async fn health() -> &'static str {
    "ok"
}

/// `GET /events` — streams AVM trace events as SSE.
///
/// Each `data:` field contains a JSON-serialised [`avm_core::trace::LogEntry`].
async fn sse_handler(
    State(broadcaster): State<Arc<EventBroadcaster>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = broadcaster.subscribe();
    let stream = BroadcastStream::new(rx)
        // BroadcastStream yields `Result<T, BroadcastStreamRecvError>`.
        // Lagged messages are skipped; other items are forwarded.
        .filter_map(Result::ok)
        .map(|data| Ok::<Event, Infallible>(Event::default().data(data)));
    Sse::new(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that publish → subscribe → receive works end-to-end.
    #[tokio::test]
    async fn broadcaster_publish_and_receive() {
        let broadcaster = EventBroadcaster::new(16);
        let mut rx = broadcaster.subscribe();

        broadcaster.publish(r#"{"test":"value"}"#.to_string());

        let received = rx
            .recv()
            .await
            .expect("should receive the published message");
        assert_eq!(received, r#"{"test":"value"}"#);
    }

    /// Publishing with no subscribers must not panic.
    #[tokio::test]
    async fn broadcaster_publish_no_subscribers_is_noop() {
        let broadcaster = EventBroadcaster::new(16);
        // No subscriber — this must not panic or error.
        broadcaster.publish("hello".to_string());
    }

    /// Multiple subscribers each receive the same message.
    #[tokio::test]
    async fn broadcaster_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new(16);
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        broadcaster.publish("event".to_string());

        let r1 = rx1.recv().await.expect("rx1 should receive");
        let r2 = rx2.recv().await.expect("rx2 should receive");
        assert_eq!(r1, "event");
        assert_eq!(r2, "event");
    }

    /// The SSE router responds with 200 on /health.
    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt as _;

        let broadcaster = Arc::new(EventBroadcaster::new(16));
        let router = sse_router_arc(broadcaster);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
