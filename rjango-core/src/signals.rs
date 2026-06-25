use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use std::any::Any;

/// A signal with connected receivers.
#[derive(Clone)]
pub struct Signal {
    name: &'static str,
    receivers: Arc<RwLock<Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>,
}

impl Signal {
    pub fn new(name: &'static str) -> Self {
        Self { name, receivers: Arc::new(RwLock::new(Vec::new())) }
    }

    pub fn connect<F: Fn(&dyn Any) + Send + Sync + 'static>(&self, f: F) {
        self.receivers.write().unwrap().push(Box::new(f));
    }

    pub fn send(&self, sender: &dyn Any) {
        let receivers = self.receivers.read().unwrap();
        for r in receivers.iter() {
            r(sender);
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

fn signal_registry() -> &'static RwLock<HashMap<&'static str, Signal>> {
    static REGISTRY: LazyLock<RwLock<HashMap<&'static str, Signal>>> =
        LazyLock::new(|| RwLock::new(HashMap::new()));
    &REGISTRY
}

pub fn get_signal(name: &'static str) -> Signal {
    let mut registry = signal_registry().write().unwrap();
    registry
        .entry(name)
        .or_insert_with(|| Signal::new(name))
        .clone()
}

pub fn pre_save() -> Signal { get_signal("pre_save") }
pub fn post_save() -> Signal { get_signal("post_save") }
pub fn pre_delete() -> Signal { get_signal("pre_delete") }
pub fn post_delete() -> Signal { get_signal("post_delete") }
pub fn pre_migrate() -> Signal { get_signal("pre_migrate") }
pub fn post_migrate() -> Signal { get_signal("post_migrate") }
pub fn request_started() -> Signal { get_signal("request_started") }
pub fn request_finished() -> Signal { get_signal("request_finished") }

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_signal_connect_and_send() {
        let signal = Signal::new("test_signal");
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        signal.connect(move |_| { called_clone.store(true, Ordering::SeqCst); });
        signal.send(&42);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_signal_name() {
        let signal = Signal::new("my_signal");
        assert_eq!(signal.name(), "my_signal");
    }

    #[test]
    fn test_get_signal_reuses() {
        let s1 = get_signal("shared");
        let s2 = get_signal("shared");
        assert_eq!(s1.name(), s2.name());
    }

    #[test]
    fn test_predefined_signals() {
        assert_eq!(pre_save().name(), "pre_save");
        assert_eq!(post_save().name(), "post_save");
        assert_eq!(pre_delete().name(), "pre_delete");
        assert_eq!(post_delete().name(), "post_delete");
        assert_eq!(pre_migrate().name(), "pre_migrate");
        assert_eq!(post_migrate().name(), "post_migrate");
        assert_eq!(request_started().name(), "request_started");
        assert_eq!(request_finished().name(), "request_finished");
    }

    #[test]
    fn test_signal_multiple_receivers() {
        let signal = Signal::new("multi");
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        signal.connect(move |_| { c1.fetch_add(1, Ordering::SeqCst); });
        signal.connect(move |_| { c2.fetch_add(1, Ordering::SeqCst); });
        signal.send(&"test");
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
