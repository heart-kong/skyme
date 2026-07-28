use crate::event::Event;

/// Listener trait — any module that wants to receive events implements this.
pub trait EventListener: Send + 'static {
    fn on_event(&mut self, event: &Event);
}

/// Simple synchronous event bus.
///
/// Dispatches events to all registered listeners in registration order.
/// This is intentionally simple — no filtering, no async, no priority.
/// Those can be added later if profiling shows they're needed.
pub struct EventBus {
    listeners: Vec<Box<dyn EventListener>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn register(&mut self, listener: Box<dyn EventListener>) {
        self.listeners.push(listener);
    }

    pub fn dispatch(&mut self, event: &Event) {
        for listener in &mut self.listeners {
            listener.on_event(event);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
