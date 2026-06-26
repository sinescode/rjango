/// Messages — re-exports from rjango-core's messages framework.
/// Like `django.contrib.messages`.
pub use rjango_core::messages::{
    Message, MessageLevel,
    add_message, get_messages,
    info, success, warning, error, debug,
};
