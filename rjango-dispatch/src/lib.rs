//! rjango-dispatch — Signal dispatcher (mirrors `django.dispatch`).
//! Re-exports and wraps rjango-core signal infrastructure.

use std::sync::Arc;
use rjango_core::signals;

/// A type-safe signal wrapper around rjango-core's Signal.
pub struct Signal {
    inner: Arc<signals::Signal>,
}

impl Signal {
    pub fn new(name: &'static str) -> Self {
        Self { inner: Arc::new(signals::Signal::new(name)) }
    }

    /// Connect a receiver callback.
    pub fn connect<F>(&self, f: F)
    where
        F: Fn(&dyn std::any::Any) + Send + Sync + 'static,
    {
        self.inner.connect(f);
    }

    /// Send a signal.
    pub fn send(&self, sender: &dyn std::any::Any) {
        self.inner.send(sender);
    }

    /// Get the inner signal name.
    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Pre-defined signals (matching Django's).
pub fn request_started() -> Signal {
    Signal::new("request_started")
}

pub fn request_finished() -> Signal {
    Signal::new("request_finished")
}

pub fn got_request_exception() -> Signal {
    Signal::new("got_request_exception")
}

pub fn pre_save() -> Signal {
    Signal::new("pre_save")
}

pub fn post_save() -> Signal {
    Signal::new("post_save")
}

pub fn pre_delete() -> Signal {
    Signal::new("pre_delete")
}

pub fn post_delete() -> Signal {
    Signal::new("post_delete")
}

pub fn pre_migrate() -> Signal {
    Signal::new("pre_migrate")
}

pub fn post_migrate() -> Signal {
    Signal::new("post_migrate")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let sig = Signal::new("test");
        assert_eq!(sig.name(), "test");
    }

    #[test]
    fn test_connect_and_send() {
        let sig = Signal::new("test_connect");
        let received = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let r = received.clone();

        sig.connect(move |_sender| {
            r.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        sig.send(&"hello");
        assert!(received.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_predefined_signals() {
        let s = request_started();
        assert_eq!(s.name(), "request_started");

        let s = post_save();
        assert_eq!(s.name(), "post_save");
    }
}
