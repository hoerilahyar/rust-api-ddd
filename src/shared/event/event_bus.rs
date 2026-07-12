use std::fmt::Debug;
use std::sync::Arc;

use tokio::sync::broadcast;

/// Marker trait for domain events published across module boundaries
/// (e.g. `user::UserRegistered` consumed by an audit or notification listener).
pub trait DomainEvent: Clone + Debug + Send + Sync + 'static {}

/// Minimal in-process event bus backed by a broadcast channel. Good enough
/// for decoupling modules within a single process; swap the internals for a
/// message broker (NATS/Kafka) later without touching call sites.
#[derive(Clone)]
pub struct EventBus<E: DomainEvent> {
    sender: broadcast::Sender<E>,
}

impl<E: DomainEvent> EventBus<E> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all current subscribers. Errors only when there
    /// are zero subscribers, which is not a failure worth propagating.
    pub fn publish(&self, event: E) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<E> {
        self.sender.subscribe()
    }
}

pub type SharedEventBus<E> = Arc<EventBus<E>>;
