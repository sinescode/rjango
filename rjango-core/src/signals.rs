#![allow(non_upper_case_globals)]

use std::collections::HashMap;
use std::sync::Mutex;

/// A signal with connected receivers.
pub struct Signal {
    name: &'static str,
    receivers: Mutex<Vec<Box<dyn Fn(&HashMap<String, String>) + Send + Sync>>>,
}

impl Signal {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            receivers: Mutex::new(Vec::new()),
        }
    }

    pub fn connect<F: Fn(&HashMap<String, String>) + Send + Sync + 'static>(&self, f: F) {
        self.receivers.lock().unwrap().push(Box::new(f));
    }

    /// Send signal — all receivers run. Panics are NOT caught.
    pub fn send(&self, kwargs: &HashMap<String, String>) {
        let receivers = self.receivers.lock().unwrap();
        for r in receivers.iter() {
            r(kwargs);
        }
    }

    /// Send signal robustly — catches panics from individual receivers.
    /// Returns (receiver_count, error_count).
    pub fn send_robust(&self, kwargs: &HashMap<String, String>) -> (usize, usize) {
        let receivers = self.receivers.lock().unwrap();
        let total = receivers.len();
        let mut errors = 0;
        for r in receivers.iter() {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                r(kwargs);
            }));
            if result.is_err() {
                errors += 1;
            }
        }
        (total, errors)
    }

    /// Disconnect all receivers.
    pub fn disconnect(&self) {
        self.receivers.lock().unwrap().clear();
    }

    /// Number of connected receivers.
    pub fn receiver_count(&self) -> usize {
        self.receivers.lock().unwrap().len()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Create a boxed receiver function.
pub fn receiver<F>(f: F) -> Box<dyn Fn(&HashMap<String, String>) + Send + Sync>
where
    F: Fn(&HashMap<String, String>) + Send + Sync + 'static,
{
    Box::new(f)
}

// ── Module-level Signal instances ──

/// Sent before a model's `__init__()` method.
pub static pre_init: Signal = Signal::new("pre_init");
/// Sent after a model's `__init__()` method.
pub static post_init: Signal = Signal::new("post_init");
/// Sent before a model's `save()` method.
pub static pre_save: Signal = Signal::new("pre_save");
/// Sent after a model's `save()` method.
pub static post_save: Signal = Signal::new("post_save");
/// Sent before a model's `delete()` method.
pub static pre_delete: Signal = Signal::new("pre_delete");
/// Sent after a model's `delete()` method.
pub static post_delete: Signal = Signal::new("post_delete");
/// Sent when a ManyToManyField is changed.
pub static m2m_changed: Signal = Signal::new("m2m_changed");
/// Sent when an HTTP request starts.
pub static request_started: Signal = Signal::new("request_started");
/// Sent when an HTTP request finishes.
pub static request_finished: Signal = Signal::new("request_finished");
/// Sent when a view raises an uncaught exception.
pub static got_request_exception: Signal = Signal::new("got_request_exception");
/// Sent when a setting is changed.
pub static setting_changed: Signal = Signal::new("setting_changed");

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_signal_connect_and_send() {
        let signal = Signal::new("test_signal");
        let called = Arc::new(AtomicBool::new(false));
        let c = called.clone();
        signal.connect(move |_| { c.store(true, Ordering::SeqCst); });
        signal.send(&HashMap::new());
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_signal_with_kwargs() {
        let signal = Signal::new("kwargs_test");
        let captured = Arc::new(Mutex::new(String::new()));
        let cap = captured.clone();
        signal.connect(move |kwargs| {
            if let Some(val) = kwargs.get("key") {
                *cap.lock().unwrap() = val.clone();
            }
        });
        let mut kwargs = HashMap::new();
        kwargs.insert("key".to_string(), "value".to_string());
        signal.send(&kwargs);
        assert_eq!(*captured.lock().unwrap(), "value");
    }

    #[test]
    fn test_signal_name() {
        let signal = Signal::new("my_signal");
        assert_eq!(signal.name(), "my_signal");
    }

    #[test]
    fn test_static_signal_instances() {
        assert_eq!(pre_init.name(), "pre_init");
        assert_eq!(post_init.name(), "post_init");
        assert_eq!(pre_save.name(), "pre_save");
        assert_eq!(post_save.name(), "post_save");
        assert_eq!(pre_delete.name(), "pre_delete");
        assert_eq!(post_delete.name(), "post_delete");
        assert_eq!(m2m_changed.name(), "m2m_changed");
        assert_eq!(request_started.name(), "request_started");
        assert_eq!(request_finished.name(), "request_finished");
        assert_eq!(got_request_exception.name(), "got_request_exception");
        assert_eq!(setting_changed.name(), "setting_changed");
    }

    #[test]
    fn test_signal_multiple_receivers() {
        let signal = Signal::new("multi");
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = Arc::clone(&counter);
        let c2 = Arc::clone(&counter);
        signal.connect(move |_| { c1.fetch_add(1, Ordering::SeqCst); });
        signal.connect(move |_| { c2.fetch_add(1, Ordering::SeqCst); });
        signal.send(&HashMap::new());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_send_robust() {
        let signal = Signal::new("robust_test");
        signal.connect(|_| { /* ok */ });
        signal.connect(|_| panic!("receiver panic"));
        signal.connect(|_| { /* ok */ });
        let (total, errors) = signal.send_robust(&HashMap::new());
        assert_eq!(total, 3);
        assert_eq!(errors, 1);
    }

    #[test]
    fn test_disconnect() {
        let signal = Signal::new("disconnect_test");
        signal.connect(|_| { });
        signal.connect(|_| { });
        assert_eq!(signal.receiver_count(), 2);
        signal.disconnect();
        assert_eq!(signal.receiver_count(), 0);
    }

    #[test]
    fn test_receiver_function() {
        let signal = Signal::new("receiver_test");
        let called = Arc::new(AtomicBool::new(false));
        let c = called.clone();
        signal.connect(move |_| { c.store(true, Ordering::SeqCst); });
        signal.send(&HashMap::new());
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_receiver_count() {
        let signal = Signal::new("count_test");
        assert_eq!(signal.receiver_count(), 0);
        signal.connect(|_| { });
        assert_eq!(signal.receiver_count(), 1);
    }

    #[test]
    fn test_send_robust_all_ok() {
        let signal = Signal::new("all_ok");
        signal.connect(|_| { });
        signal.connect(|_| { });
        let (total, errors) = signal.send_robust(&HashMap::new());
        assert_eq!(total, 2);
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_receiver_fn_returns_box() {
        let _r = receiver(|kwargs: &HashMap<String, String>| {
            let _ = kwargs;
        });
    }

    #[test]
    fn test_static_signal_is_send_sync() {
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<Signal>();
    }
}
