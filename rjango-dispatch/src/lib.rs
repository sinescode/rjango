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
    use std::any::Any;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_signal_creation() {
        let sig = Signal::new("test");
        assert_eq!(sig.name(), "test");
    }

    #[test]
    fn test_connect_and_send() {
        let sig = Signal::new("test_connect");
        let received = Arc::new(AtomicBool::new(false));
        let r = received.clone();

        sig.connect(move |_sender| {
            r.store(true, Ordering::SeqCst);
        });

        sig.send(&"hello");
        assert!(received.load(Ordering::SeqCst));
    }

    #[test]
    fn test_predefined_signals() {
        let s = request_started();
        assert_eq!(s.name(), "request_started");

        let s = post_save();
        assert_eq!(s.name(), "post_save");
    }

    #[test]
    fn test_send_no_receivers() {
        let sig = Signal::new("orphan");
        sig.send(&"should not panic");
        // If we reach here, no crash
    }

    #[test]
    fn test_send_with_int_sender() {
        let sig = Signal::new("int_sender");
        let received = Arc::new(AtomicBool::new(false));
        let r = received.clone();
        sig.connect(move |sender| {
            if let Some(val) = (sender as &dyn Any).downcast_ref::<i32>() {
                if *val == 42 {
                    r.store(true, Ordering::SeqCst);
                }
            }
        });
        sig.send(&42);
        assert!(received.load(Ordering::SeqCst));
    }

    #[test]
    fn test_multiple_receivers_count() {
        let sig = Signal::new("multi_count");
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        sig.connect(move |_| { c1.fetch_add(1, Ordering::SeqCst); });
        sig.connect(move |_| { c2.fetch_add(1, Ordering::SeqCst); });
        sig.send(&"ping");
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_all_predefined_signals() {
        let signals = vec![
            request_started(),
            request_finished(),
            got_request_exception(),
            pre_save(),
            post_save(),
            pre_delete(),
            post_delete(),
            pre_migrate(),
            post_migrate(),
        ];
        let names: Vec<&str> = signals.iter().map(|s| s.name()).collect();
        assert!(names.contains(&"request_started"));
        assert!(names.contains(&"post_delete"));
        assert!(names.contains(&"got_request_exception"));
        assert_eq!(names.len(), 9);
    }

    #[test]
    fn test_signal_sends_string_sender() {
        let sig = Signal::new("string_sender");
        let val = Arc::new(std::sync::Mutex::new(String::new()));
        let v = val.clone();
        sig.connect(move |sender| {
            if let Some(s) = (sender as &dyn Any).downcast_ref::<String>() {
                *v.lock().unwrap() = s.clone();
            }
        });
        sig.send(&"hello".to_string());
        assert_eq!(*val.lock().unwrap(), "hello");
    }

    #[test]
    fn test_dispatch_signal_clone() {
        let sig = Signal::new("cloneable");
        let _cloned = sig; // move, not clone — Signal doesn't implement Clone
    }
}
