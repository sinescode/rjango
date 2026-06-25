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
