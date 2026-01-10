//! Request handler for mapping WebSocket request IDs to response channels.
//!
//! This module provides a mechanism to track pending WebSocket requests and
//! route responses back to the appropriate callers via oneshot channels.

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tokio::sync::oneshot;
use tracing::{debug, error, warn};

/// Handles pending WebSocket requests by mapping request IDs to response channels.
///
/// When a request is sent over the WebSocket, a oneshot channel sender is registered
/// with the request ID. When a response arrives, it is routed to the appropriate
/// sender based on the request ID.
#[derive(Debug, Default)]
pub struct RequestHandler {
    /// Map of pending request IDs to their oneshot response senders.
    pending: HashMap<u64, oneshot::Sender<JsonValue>>,
}

impl RequestHandler {
    /// Creates a new empty request handler.
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// Registers a pending request with its response channel.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The unique identifier for this request.
    /// * `sender` - The oneshot channel sender to receive the response.
    pub fn register(&mut self, request_id: u64, sender: oneshot::Sender<JsonValue>) {
        debug!("Registering request with id: {}", request_id);
        if self.pending.contains_key(&request_id) {
            warn!(
                "Request ID {} already registered, overwriting previous sender",
                request_id
            );
        }
        self.pending.insert(request_id, sender);
    }

    /// Handles an incoming response by routing it to the appropriate pending request.
    ///
    /// If a matching request ID is found, the response is sent to the registered
    /// oneshot channel and the request is removed from the pending map.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The request ID from the response.
    /// * `response` - The JSON response value to send.
    ///
    /// # Returns
    ///
    /// `true` if the response was successfully routed to a pending request,
    /// `false` if no matching request ID was found.
    pub fn handle_response(&mut self, request_id: u64, response: JsonValue) -> bool {
        if let Some(sender) = self.pending.remove(&request_id) {
            debug!("Handling response for request id: {}", request_id);
            if sender.send(response).is_err() {
                error!(
                    "Failed to send response for request id {}: receiver dropped",
                    request_id
                );
                return false;
            }
            true
        } else {
            warn!("No pending request found for request id: {}", request_id);
            false
        }
    }

    /// Returns the number of pending requests.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Cancels all pending requests by dropping their senders.
    ///
    /// This will cause all waiting receivers to receive an error when they
    /// try to await the response.
    pub fn cancel_all(&mut self) {
        let count = self.pending.len();
        if count > 0 {
            debug!("Cancelling {} pending request(s)", count);
            self.pending.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_handle_response() {
        let mut handler = RequestHandler::new();
        let (tx, rx) = oneshot::channel();

        handler.register(1, tx);
        assert_eq!(handler.pending_count(), 1);

        let response = serde_json::json!({"id": 1, "result": "success"});
        let handled = handler.handle_response(1, response.clone());

        assert!(handled);
        assert_eq!(handler.pending_count(), 0);

        let received = rx.await.unwrap();
        assert_eq!(received, response);
    }

    #[tokio::test]
    async fn test_handle_response_unknown_id() {
        let mut handler = RequestHandler::new();
        let response = serde_json::json!({"id": 999, "result": "unknown"});

        let handled = handler.handle_response(999, response);
        assert!(!handled);
    }

    #[tokio::test]
    async fn test_cancel_all() {
        let mut handler = RequestHandler::new();
        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();

        handler.register(1, tx1);
        handler.register(2, tx2);
        assert_eq!(handler.pending_count(), 2);

        handler.cancel_all();
        assert_eq!(handler.pending_count(), 0);

        // Receivers should get errors since senders were dropped
        assert!(rx1.await.is_err());
        assert!(rx2.await.is_err());
    }
}
