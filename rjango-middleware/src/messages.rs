//! Message middleware — enables one-time flash messages stored in sessions.
//! Like Django's `django.contrib.messages.middleware.MessageMiddleware`.

use rjango_core::{Request, Response, RjangoError};
use super::Middleware;

/// Message middleware — processes flash messages via session storage.
pub struct MessageMiddleware;

impl Middleware for MessageMiddleware {
    fn process_request(&self, _request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        Ok(None)
    }

    fn process_response(&self, _request: &Request, _response: &mut Response) -> std::result::Result<(), RjangoError> {
        Ok(())
    }
}
