/// LocaleMiddleware — detect user language from Accept-Language.
/// Mirrors `django.middleware.locale.LocaleMiddleware`.
use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

pub struct LocaleMiddleware;

impl Middleware for LocaleMiddleware {
    fn process_response(&self, request: &Request, response: &mut Response) -> Result<(), RjangoError> {
        let lang = request.headers.get("accept-language")
            .and_then(|v| v.split(',').next())
            .unwrap_or("en-us");
        response.headers.insert("Content-Language".into(), lang.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request, Response};

    #[test]
    fn test_locale_sets_content_language() {
        let mut req = rjango_core::Request::new(rjango_core::HttpMethod::GET, "/");
        req.headers.insert("accept-language".into(), "fr-FR,fr;q=0.9".into());
        let mut res = Response::html("bonjour");
        let mw = LocaleMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.headers.get("Content-Language").unwrap(), "fr-FR");
    }

    #[test]
    fn test_locale_default_without_header() {
        let req = rjango_core::Request::new(rjango_core::HttpMethod::GET, "/");
        let mut res = Response::html("hello");
        let mw = LocaleMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.headers.get("Content-Language").unwrap(), "en-us");
    }
}
