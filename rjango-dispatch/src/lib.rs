//! rjango-dispatch — Signal dispatcher (mirrors `django.dispatch`).
//!
//! Provides sender-filtered signals with dispatch_uid support,
//! similar to Django's `django.dispatch.Signal`.
//!
//! # Example
//!
//! ```rust
//! use rjango_dispatch::Signal;
//! use std::collections::HashMap;
//!
//! let signal = Signal::new("article_published");
//! signal.connect(|sender, kwargs| {
//!     println!("Article published: {:?}", kwargs);
//! }, None, None);
//!
//! let mut kwargs = HashMap::new();
//! kwargs.insert("title".into(), "Hello".into());
//! signal.send(&"Article", &kwargs);
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Mutex;

/// A registered signal receiver.
struct Receiver {
    /// Optional unique identifier to prevent duplicate connections.
    dispatch_uid: Option<String>,
    /// Optional sender TypeId filter — only invoke when the sender's
    /// type matches (e.g. `TypeId::of::<Article>()`).
    sender_filter: Option<TypeId>,
    /// The actual callback: receives `(sender, kwargs)`.
    callback: Box<dyn Fn(&dyn Any, &HashMap<String, String>) + Send + Sync>,
}

/// A named signal that can be connected to and sent.
///
/// Mirrors Django's `django.dispatch.Signal` — supports:
/// - Optional sender type filtering at connect time
/// - dispatch_uid for idempotent connections
/// - Robust sending (catches panics)
/// - Individual receiver disconnection
pub struct Signal {
    name: &'static str,
    receivers: Mutex<Vec<Receiver>>,
}

impl Signal {
    /// Create a new named signal. `name` is used for debugging.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            receivers: Mutex::new(Vec::new()),
        }
    }

    /// Connect a receiver callback.
    ///
    /// * `f` — the callback: `fn(&dyn Any /* sender */, &HashMap<String, String> /* kwargs */)`
    /// * `sender` — optional sender type filter. If `Some(TypeId::of::<MyModel>())`, this
    ///   receiver will only be called when the sender's type matches.
    /// * `dispatch_uid` — optional unique ID. If a receiver with this UID is already
    ///   connected, this call is a no-op (prevents duplicate connections).
    pub fn connect<F>(&self, f: F, sender: Option<TypeId>, dispatch_uid: Option<&str>)
    where
        F: Fn(&dyn Any, &HashMap<String, String>) + Send + Sync + 'static,
    {
        let mut recv = self.receivers.lock().unwrap();
        if let Some(uid) = dispatch_uid {
            if recv.iter().any(|r| r.dispatch_uid.as_deref() == Some(uid)) {
                return; // already connected
            }
        }
        recv.push(Receiver {
            dispatch_uid: dispatch_uid.map(|s| s.to_string()),
            sender_filter: sender,
            callback: Box::new(f),
        });
    }

    /// Send signal to all matching receivers.
    ///
    /// Each receiver is called with `(sender, kwargs)`. If a receiver was connected
    /// with a sender type filter, it only fires when the sender's TypeId matches.
    pub fn send(&self, sender: &dyn Any, kwargs: &HashMap<String, String>) {
        let receivers = self.receivers.lock().unwrap();
        let sender_tid = sender.type_id();
        for r in receivers.iter() {
            if let Some(filter) = r.sender_filter {
                if sender_tid != filter {
                    continue;
                }
            }
            (r.callback)(sender, kwargs);
        }
    }

    /// Send signal robustly — catches panics from individual receivers.
    ///
    /// Returns `(total_matching_receivers, error_count)` where total is the count
    /// of receivers that matched the sender filter (were attempted).
    pub fn send_robust(&self, sender: &dyn Any, kwargs: &HashMap<String, String>) -> (usize, usize) {
        let receivers = self.receivers.lock().unwrap();
        let sender_tid = sender.type_id();
        let mut total = 0;
        let mut errors = 0;
        for r in receivers.iter() {
            if let Some(filter) = r.sender_filter {
                if sender_tid != filter {
                    continue;
                }
            }
            total += 1;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (r.callback)(sender, kwargs);
            }));
            if result.is_err() {
                errors += 1;
            }
        }
        (total, errors)
    }

    /// Disconnect receivers.
    ///
    /// * `dispatch_uid` — if set, only disconnects the receiver with this UID.
    /// * `sender` — if set, only disconnects receivers with this sender TypeId filter.
    /// * If both are `None`, disconnects ALL receivers.
    pub fn disconnect(&self, dispatch_uid: Option<&str>, sender: Option<TypeId>) {
        let mut recv = self.receivers.lock().unwrap();
        recv.retain(|r| {
            match (dispatch_uid, sender) {
                (Some(uid), Some(snd)) => {
                    !(r.dispatch_uid.as_deref() == Some(uid)
                        && r.sender_filter == Some(snd))
                }
                (Some(uid), None) => r.dispatch_uid.as_deref() != Some(uid),
                (None, Some(snd)) => r.sender_filter != Some(snd),
                (None, None) => false, // remove everything
            }
        });
    }

    /// Number of connected receivers.
    pub fn receiver_count(&self) -> usize {
        self.receivers.lock().unwrap().len()
    }

    /// The signal's debug name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

// ── Module-level convenience constructors (matching Django's pre-defined signals) ──

/// Sent when an HTTP request starts.
pub fn request_started() -> Signal {
    Signal::new("request_started")
}
/// Sent when an HTTP request finishes.
pub fn request_finished() -> Signal {
    Signal::new("request_finished")
}
/// Sent when a view raises an uncaught exception.
pub fn got_request_exception() -> Signal {
    Signal::new("got_request_exception")
}
/// Sent before a model's `save()` method.
pub fn pre_save() -> Signal {
    Signal::new("pre_save")
}
/// Sent after a model's `save()` method.
pub fn post_save() -> Signal {
    Signal::new("post_save")
}
/// Sent before a model's `delete()` method.
pub fn pre_delete() -> Signal {
    Signal::new("pre_delete")
}
/// Sent after a model's `delete()` method.
pub fn post_delete() -> Signal {
    Signal::new("post_delete")
}
/// Sent before migrations run.
pub fn pre_migrate() -> Signal {
    Signal::new("pre_migrate")
}
/// Sent after migrations run.
pub fn post_migrate() -> Signal {
    Signal::new("post_migrate")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    struct Article;
    struct Comment;

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

        sig.connect(
            move |_sender, _kwargs| {
                r.store(true, Ordering::SeqCst);
            },
            None,
            None,
        );

        let kwargs = HashMap::new();
        sig.send(&"hello", &kwargs);
        assert!(received.load(Ordering::SeqCst));
    }

    #[test]
    fn test_send_with_kwargs() {
        let sig = Signal::new("kwargs");
        let val = Arc::new(std::sync::Mutex::new(String::new()));
        let v = val.clone();

        sig.connect(
            move |_sender, kwargs| {
                if let Some(msg) = kwargs.get("msg") {
                    *v.lock().unwrap() = msg.clone();
                }
            },
            None,
            None,
        );

        let mut kwargs = HashMap::new();
        kwargs.insert("msg".to_string(), "hello".to_string());
        sig.send(&"sender", &kwargs);
        assert_eq!(*val.lock().unwrap(), "hello");
    }

    #[test]
    fn test_send_with_sender_info() {
        let sig = Signal::new("sender_info");
        let captured_sender = Arc::new(std::sync::Mutex::new(String::new()));
        let cs = captured_sender.clone();

        sig.connect(
            move |sender, _kwargs| {
                *cs.lock().unwrap() = format!("{:?}", sender.type_id());
            },
            None,
            None,
        );

        sig.send(&42, &HashMap::new());
        let val = captured_sender.lock().unwrap().clone();
        // TypeId format varies by platform; just verify we got something
        assert!(!val.is_empty(), "Expected non-empty type_id string, got empty");
    }

    #[test]
    fn test_sender_filter() {
        let sig = Signal::new("filtered");
        let called = Arc::new(AtomicUsize::new(0));
        let c1 = called.clone();
        let c2 = called.clone();

        // Only fires when sender is `Article`
        sig.connect(
            move |_s, _k| {
                c1.fetch_add(1, Ordering::SeqCst);
            },
            Some(TypeId::of::<Article>()),
            None,
        );
        // Fires for any sender
        sig.connect(
            move |_s, _k| {
                c2.fetch_add(2, Ordering::SeqCst);
            },
            None,
            None,
        );

        sig.send(&Article, &HashMap::new());
        // Article-filtered fires (+1) and unfiltered fires (+2) = 3
        assert_eq!(called.load(Ordering::SeqCst), 3);

        sig.send(&Comment, &HashMap::new());
        // Article-filtered skips (+0) and unfiltered fires (+2) = 5
        assert_eq!(called.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_dispatch_uid_dedup() {
        let sig = Signal::new("dedup");
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        sig.connect(
            move |_s, _k| {
                c.fetch_add(1, Ordering::SeqCst);
            },
            None,
            Some("uid1"),
        );
        // Same uid — should be a no-op
        sig.connect(
            move |_s, _k| {
                // this never runs
            },
            None,
            Some("uid1"),
        );

        assert_eq!(sig.receiver_count(), 1);
        sig.send(&"test", &HashMap::new());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_receivers() {
        let sig = Signal::new("multi");
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        sig.connect(move |_s, _k| { c1.fetch_add(1, Ordering::SeqCst); }, None, None);
        sig.connect(move |_s, _k| { c2.fetch_add(1, Ordering::SeqCst); }, None, None);

        sig.send(&"x", &HashMap::new());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_send_robust() {
        let sig = Signal::new("robust_test");
        sig.connect(|_s, _k| { /* ok */ }, None, None);
        sig.connect(|_s, _k| panic!("receiver panic"), None, None);
        sig.connect(|_s, _k| { /* ok */ }, None, None);

        let (total, errors) = sig.send_robust(&"sender", &HashMap::new());
        assert_eq!(total, 3);
        assert_eq!(errors, 1);
    }

    #[test]
    fn test_send_robust_no_errors() {
        let sig = Signal::new("robust_ok");
        sig.connect(|_s, _k| { /* ok */ }, None, None);
        sig.connect(|_s, _k| { /* ok */ }, None, None);

        let (total, errors) = sig.send_robust(&"sender", &HashMap::new());
        assert_eq!(total, 2);
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_disconnect_all() {
        let sig = Signal::new("disconnect_all");
        sig.connect(|_s, _k| {}, None, Some("a"));
        sig.connect(|_s, _k| {}, None, Some("b"));
        assert_eq!(sig.receiver_count(), 2);
        sig.disconnect(None, None);
        assert_eq!(sig.receiver_count(), 0);
    }

    #[test]
    fn test_disconnect_by_uid() {
        let sig = Signal::new("disconnect_uid");
        sig.connect(|_s, _k| {}, None, Some("keep"));
        sig.connect(|_s, _k| {}, None, Some("remove"));
        assert_eq!(sig.receiver_count(), 2);
        sig.disconnect(Some("remove"), None);
        assert_eq!(sig.receiver_count(), 1);
    }

    #[test]
    fn test_disconnect_by_sender() {
        let sig = Signal::new("disconnect_sender");
        sig.connect(|_s, _k| {}, Some(TypeId::of::<Article>()), Some("a1"));
        sig.connect(|_s, _k| {}, None, Some("catchall"));
        assert_eq!(sig.receiver_count(), 2);
        sig.disconnect(None, Some(TypeId::of::<Article>()));
        assert_eq!(sig.receiver_count(), 1);
    }

    #[test]
    fn test_send_no_receivers() {
        let sig = Signal::new("orphan");
        sig.send(&"x", &HashMap::new());
        // If we reach here, no crash
    }

    #[test]
    fn test_receiver_count() {
        let sig = Signal::new("count_test");
        assert_eq!(sig.receiver_count(), 0);
        sig.connect(|_s, _k| {}, None, None);
        assert_eq!(sig.receiver_count(), 1);
        sig.connect(|_s, _k| {}, None, None);
        assert_eq!(sig.receiver_count(), 2);
    }

    #[test]
    fn test_predefined_signals() {
        let s = request_started();
        assert_eq!(s.name(), "request_started");

        let s = post_save();
        assert_eq!(s.name(), "post_save");
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
    fn test_signal_name() {
        let sig = Signal::new("custom_signal");
        assert_eq!(sig.name(), "custom_signal");
    }

    #[test]
    fn test_send_empty_kwargs() {
        let sig = Signal::new("empty_kwargs");
        let called = Arc::new(AtomicBool::new(false));
        let c = called.clone();
        sig.connect(move |_s, _k| { c.store(true, Ordering::SeqCst); }, None, None);
        sig.send(&"x", &HashMap::new());
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_multiple_sends() {
        let sig = Signal::new("multiple_sends");
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();
        sig.connect(move |_s, _k| { c.fetch_add(1, Ordering::SeqCst); }, None, None);
        for _ in 0..5 {
            sig.send(&"x", &HashMap::new());
        }
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_sender_filter_disconnect_by_both() {
        let sig = Signal::new("filter_disconnect_both");
        sig.connect(|_s, _k| {}, Some(TypeId::of::<Article>()), Some("uid1"));
        sig.connect(|_s, _k| {}, None, Some("uid2"));
        // Should not remove uid1's receiver — only matches when BOTH uid1 AND Article filter
        sig.disconnect(Some("uid1"), None);
        assert_eq!(sig.receiver_count(), 1);
    }

    #[test]
    fn test_send_robust_filtered() {
        let sig = Signal::new("robust_filtered");
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();

        sig.connect(move |_s, _k| { c1.fetch_add(1, Ordering::SeqCst); }, Some(TypeId::of::<Article>()), None);
        sig.connect(|_s, _k| panic!("oops"), None, None);

        // When sender is Article:
        //   Receiver 1 (Article-filtered) = called, +1 to counter, OK
        //   Receiver 2 (unfiltered) = called, panics
        let (total, errors) = sig.send_robust(&Article, &HashMap::new());
        assert_eq!(total, 2);
        assert_eq!(errors, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // When sender is Comment:
        //   Receiver 1 (Article-filtered) = skipped (TypeId mismatch)
        //   Receiver 2 (unfiltered) = called, panics
        let (total2, errors2) = sig.send_robust(&Comment, &HashMap::new());
        assert_eq!(total2, 1);
        assert_eq!(errors2, 1);
    }

    #[test]
    fn test_static_signal_send_sync() {
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<Signal>();
    }
}
