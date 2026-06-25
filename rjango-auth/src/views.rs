//! Auth views — login, logout, password management.
//! Mirrors Django's `django.contrib.auth.views`.

use rjango_core::{Request, Response};

/// Render the login page with a form.
pub fn login_view(request: &Request) -> Response {
    let next = request.query.get("next").map(|s| s.to_string()).unwrap_or_else(|| "/".to_string());
    let error = request.query.get("error").map(|s| s.to_string()).unwrap_or_default();

    let error_html = if error.is_empty() {
        String::new()
    } else {
        format!("<ul class=\"errorlist\"><li>{}</li></ul>", html_escape(&error))
    };

    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Log in | Rjango</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; display: flex; justify-content: center; align-items: center; min-height: 100vh; }}
        .login-container {{ background: #fff; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); padding: 40px; width: 360px; }}
        h1 {{ font-size: 24px; margin-bottom: 8px; color: #333; }}
        .subtitle {{ color: #666; margin-bottom: 24px; font-size: 14px; }}
        .errorlist {{ list-style: none; background: #fff0f0; border: 1px solid #e74c3c; border-radius: 4px; padding: 8px 12px; margin-bottom: 16px; color: #c0392b; font-size: 13px; }}
        .form-row {{ margin-bottom: 16px; }}
        label {{ display: block; margin-bottom: 4px; font-size: 14px; color: #333; font-weight: 500; }}
        input[type="text"], input[type="password"] {{ width: 100%; padding: 10px 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px; transition: border-color 0.2s; }}
        input:focus {{ outline: none; border-color: #3498db; box-shadow: 0 0 0 2px rgba(52,152,219,0.2); }}
        button {{ width: 100%; padding: 10px; background: #3498db; color: #fff; border: none; border-radius: 4px; font-size: 15px; cursor: pointer; transition: background 0.2s; }}
        button:hover {{ background: #2980b9; }}
        .helptext {{ display: block; font-size: 12px; color: #999; margin-top: 4px; }}
        .footer {{ text-align: center; margin-top: 16px; font-size: 13px; color: #999; }}
    </style>
</head>
<body>
    <div class="login-container">
        <h1>Log in</h1>
        <p class="subtitle">Enter your credentials to continue</p>
        {error_html}
        <form action="/accounts/login/" method="post">
            <input type="hidden" name="csrfmiddlewaretoken" value="rjango-csrf-token">
            <input type="hidden" name="next" value="{next}">
            <div class="form-row">
                <label for="id_username">Username:</label>
                <input type="text" name="username" id="id_username" placeholder="Username" required autofocus>
            </div>
            <div class="form-row">
                <label for="id_password">Password:</label>
                <input type="password" name="password" id="id_password" placeholder="Password" required>
            </div>
            <button type="submit">Log in</button>
        </form>
        <div class="footer">
            <p>Forgot your password? Contact an administrator.</p>
        </div>
    </div>
</body>
</html>
"#);
    Response::html(&html)
}

/// Handle login form submission.
pub fn handle_login(request: &Request) -> Response {
    let body_str = String::from_utf8_lossy(&request.body);
    let params = rjango_core::QueryDict::from_query(&body_str);

    let username = params.get("username").unwrap_or("").to_string();
    let password = params.get("password").unwrap_or("").to_string();
    let next = params.get("next").unwrap_or("/");

    if username.is_empty() || password.is_empty() {
        return Response::redirect(
            &format!("/accounts/login/?error={}", urlencode("Username and password are required.")),
            false,
        );
    }

    // Try authentication against registered backends
    let authenticated = {
        let backends = crate::get_backends();
        backends.iter().any(|b| b.authenticate(request, &username, &password).is_some())
    };

    if authenticated {
        let mut resp = Response::redirect(next, false);
        resp.set_cookie("sessionid", &format!("rjango-{}-session", username));
        resp
    } else {
        Response::redirect(
            &format!("/accounts/login/?error={}", urlencode("Invalid username or password.")),
            false,
        )
    }
}

/// Logout view.
pub fn logout_view(_request: &Request) -> Response {
    let mut resp = Response::redirect("/accounts/login/", false);
    resp.set_cookie("sessionid", "");
    resp
}

/// Minimal URL-encoding (no crate dep).
fn urlencode(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
        b' ' => "+".to_string(),
        _ => format!("%{:02X}", b),
    }).collect()
}

/// Minimal HTML escaping.
fn html_escape(s: &str) -> String {
    s.chars().map(|c| match c {
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '&' => "&amp;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&#x27;".to_string(),
        _ => c.to_string(),
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_view_renders() {
        let req = Request::new(rjango_core::HttpMethod::GET, "/accounts/login/");
        let resp = login_view(&req);
        assert_eq!(resp.status_code(), 200);
        let html = resp.body_str();
        assert!(html.contains("Log in"));
        assert!(html.contains("csrfmiddlewaretoken"));
        assert!(html.contains("username"));
        assert!(html.contains("password"));
    }

    #[test]
    fn test_login_view_with_error() {
        let req = Request::new(rjango_core::HttpMethod::GET, "/accounts/login/?error=Bad%20credentials");
        let resp = login_view(&req);
        let html = resp.body_str();
        assert!(html.contains("Bad credentials"));
    }

    #[test]
    fn test_logout_view_redirects() {
        let req = Request::new(rjango_core::HttpMethod::GET, "/accounts/logout/");
        let resp = logout_view(&req);
        assert_eq!(resp.status_code(), 302);
        assert!(resp.header("location") == Some("/accounts/login/"));
    }

    #[test]
    fn test_urlencode() {
        assert_eq!(urlencode("hello world"), "hello+world");
        assert_eq!(urlencode("a=b&c"), "a%3Db%26c");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("safe"), "safe");
    }
}
