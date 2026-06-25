/// Messages framework — one-time flash messages (like Django's `django.contrib.messages`).
/// Messages are stored in the session under `_messages`.

use std::collections::HashMap;

/// Message severity levels (matching Django's constants).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Debug = 10,
    Info = 20,
    Success = 25,
    Warning = 30,
    Error = 40,
}

impl MessageLevel {
    pub fn from_int(n: i32) -> Self {
        match n {
            10 => MessageLevel::Debug,
            25 => MessageLevel::Success,
            30 => MessageLevel::Warning,
            40 => MessageLevel::Error,
            _ => MessageLevel::Info,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MessageLevel::Debug => "debug",
            MessageLevel::Info => "info",
            MessageLevel::Success => "success",
            MessageLevel::Warning => "warning",
            MessageLevel::Error => "error",
        }
    }
}

/// A flash message.
#[derive(Debug, Clone)]
pub struct Message {
    pub level: MessageLevel,
    pub message: String,
}

impl Message {
    pub fn new(level: MessageLevel, message: &str) -> Self {
        Self { level, message: message.to_string() }
    }
}

/// Add a flash message to the request's session (like Django's `messages.add_message()`).
pub fn add_message(session: &mut HashMap<String, serde_json::Value>, level: MessageLevel, message: &str) {
    let msgs = session.entry("_messages".into())
        .or_insert_with(|| serde_json::Value::Array(vec![]));
    if let serde_json::Value::Array(ref mut arr) = msgs {
        arr.push(serde_json::json!({
            "level": level as i32,
            "message": message,
        }));
    }
}

/// Retrieve and clear flash messages from the session (like Django's `messages.get_messages()`).
pub fn get_messages(session: &mut HashMap<String, serde_json::Value>) -> Vec<Message> {
    let msgs = session.remove("_messages")
        .unwrap_or(serde_json::Value::Array(vec![]));
    let mut result = vec![];
    if let serde_json::Value::Array(arr) = msgs {
        for item in arr {
            if let serde_json::Value::Object(map) = item {
                let level = map.get("level").and_then(|v| v.as_i64()).unwrap_or(20) as i32;
                let message = map.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                result.push(Message::new(MessageLevel::from_int(level), &message));
            }
        }
    }
    result
}

/// Convenience: add an info message.
pub fn info(session: &mut HashMap<String, serde_json::Value>, message: &str) {
    add_message(session, MessageLevel::Info, message);
}

/// Convenience: add a success message.
pub fn success(session: &mut HashMap<String, serde_json::Value>, message: &str) {
    add_message(session, MessageLevel::Success, message);
}

/// Convenience: add a warning message.
pub fn warning(session: &mut HashMap<String, serde_json::Value>, message: &str) {
    add_message(session, MessageLevel::Warning, message);
}

/// Convenience: add an error message.
pub fn error(session: &mut HashMap<String, serde_json::Value>, message: &str) {
    add_message(session, MessageLevel::Error, message);
}

/// Convenience: add a debug message.
pub fn debug(session: &mut HashMap<String, serde_json::Value>, message: &str) {
    add_message(session, MessageLevel::Debug, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_messages() {
        let mut session = HashMap::new();
        add_message(&mut session, MessageLevel::Info, "Hello");
        add_message(&mut session, MessageLevel::Warning, "Watch out");
        add_message(&mut session, MessageLevel::Error, "Something broke");

        let msgs = get_messages(&mut session);
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].message, "Hello");
        assert_eq!(msgs[0].level, MessageLevel::Info);
        assert_eq!(msgs[1].message, "Watch out");
        assert_eq!(msgs[1].level, MessageLevel::Warning);
        assert_eq!(msgs[2].message, "Something broke");
        assert_eq!(msgs[2].level, MessageLevel::Error);
        // Messages should be cleared after get_messages
        assert_eq!(get_messages(&mut session).len(), 0);
    }

    #[test]
    fn test_convenience_functions() {
        let mut session = HashMap::new();
        info(&mut session, "Info msg");
        success(&mut session, "Success msg");
        warning(&mut session, "Warning msg");
        error(&mut session, "Error msg");
        debug(&mut session, "Debug msg");

        let msgs = get_messages(&mut session);
        assert_eq!(msgs.len(), 5);
        assert_eq!(msgs[0].level, MessageLevel::Info);
        assert_eq!(msgs[1].level, MessageLevel::Success);
        assert_eq!(msgs[2].level, MessageLevel::Warning);
        assert_eq!(msgs[3].level, MessageLevel::Error);
        assert_eq!(msgs[4].level, MessageLevel::Debug);
    }

    #[test]
    fn test_no_messages() {
        let mut session = HashMap::new();
        let msgs = get_messages(&mut session);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_message_level_names() {
        assert_eq!(MessageLevel::Debug.name(), "debug");
        assert_eq!(MessageLevel::Info.name(), "info");
        assert_eq!(MessageLevel::Success.name(), "success");
        assert_eq!(MessageLevel::Warning.name(), "warning");
        assert_eq!(MessageLevel::Error.name(), "error");
    }

    #[test]
    fn test_message_level_roundtrip() {
        assert_eq!(MessageLevel::from_int(10), MessageLevel::Debug);
        assert_eq!(MessageLevel::from_int(20), MessageLevel::Info);
        assert_eq!(MessageLevel::from_int(25), MessageLevel::Success);
        assert_eq!(MessageLevel::from_int(30), MessageLevel::Warning);
        assert_eq!(MessageLevel::from_int(40), MessageLevel::Error);
        assert_eq!(MessageLevel::from_int(99), MessageLevel::Info); // fallback
    }
}
