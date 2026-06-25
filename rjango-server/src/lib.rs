//! rjango-server — HTTP server.
//! Wires together request handling, URL routing, middleware, and static file serving.
//! Synchronous middleware chain, async server runtime with raw TCP/HTTP parsing.

use std::net::SocketAddr;
use std::sync::Arc;
use std::path::PathBuf;

use rjango_core::{Request, Response, HttpMethod, StatusCode};
use rjango_middleware::MiddlewareStack;
use rjango_urls::URLResolver;
use rjango_templates::Engine as TemplateEngine;

/// The main Rjango application — holds all configuration.
pub struct Application {
    pub middleware: MiddlewareStack,
    pub url_resolver: Option<URLResolver>,
    pub template_engine: Option<TemplateEngine>,
    pub settings: Option<rjango_core::Settings>,
    pub static_dir: Option<PathBuf>,
    pub media_dir: Option<PathBuf>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            middleware: MiddlewareStack::new(),
            url_resolver: None,
            template_engine: None,
            settings: None,
            static_dir: None,
            media_dir: None,
        }
    }

    pub fn configure(mut self, settings: rjango_core::Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn with_urls(mut self, resolver: URLResolver) -> Self {
        self.url_resolver = Some(resolver);
        self
    }

    pub fn with_middleware(mut self, mw: MiddlewareStack) -> Self {
        self.middleware = mw;
        self
    }

    pub fn with_templates(mut self, engine: TemplateEngine) -> Self {
        self.template_engine = Some(engine);
        self
    }

    /// Set the static files directory (served at /static/).
    pub fn with_static_dir(mut self, path: PathBuf) -> Self {
        self.static_dir = Some(path);
        self
    }

    /// Set the media files directory (served at /media/).
    pub fn with_media_dir(mut self, path: PathBuf) -> Self {
        self.media_dir = Some(path);
        self
    }

    /// Handle a single request synchronously through the full stack.
    pub fn handle_request(&self, request: Request) -> Response {
        // Try static file first
        if let Some(resp) = self.serve_static(&request) {
            return resp;
        }

        if let Some(ref resolver) = self.url_resolver {
            rjango_urls::set_urlconf(resolver.clone());
        }
        let view = self.find_view(&request);
        if let Some(view_fn) = view {
            self.middleware.process(request, move |req| {
                (view_fn)(req)
            })
        } else {
            Response::not_found()
        }
    }

    /// Serve static or media files.
    fn serve_static(&self, request: &Request) -> Option<Response> {
        let path = &request.path;

        // Serve /static/ files
        if path.starts_with("/static/") {
            if let Some(ref static_dir) = self.static_dir {
                let relative = path.trim_start_matches("/static/");
                let file_path = static_dir.join(relative);
                if file_path.exists() && file_path.is_file() {
                    let contents = std::fs::read(&file_path).ok()?;
                    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let mime = match ext {
                        "css" => "text/css",
                        "js" => "application/javascript",
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "svg" => "image/svg+xml",
                        "ico" => "image/x-icon",
                        "woff2" => "font/woff2",
                        "ttf" => "font/ttf",
                        "json" => "application/json",
                        "txt" => "text/plain",
                        _ => "application/octet-stream",
                    };
                    let mut resp = Response::new(StatusCode::OK, contents);
                    resp.headers.insert("content-type".into(), mime.into());
                    return Some(resp);
                }
            }
            return Some(Response::not_found());
        }

        // Serve /media/ files
        if path.starts_with("/media/") {
            if let Some(ref media_dir) = self.media_dir {
                let relative = path.trim_start_matches("/media/");
                let file_path = media_dir.join(relative);
                if file_path.exists() && file_path.is_file() {
                    let contents = std::fs::read(&file_path).ok()?;
                    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let mime = match ext {
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "pdf" => "application/pdf",
                        _ => "application/octet-stream",
                    };
                    let mut resp = Response::new(StatusCode::OK, contents);
                    resp.headers.insert("content-type".into(), mime.into());
                    return Some(resp);
                }
            }
            return Some(Response::not_found());
        }

        None
    }

    fn find_view(&self, request: &Request) -> Option<Arc<dyn Fn(Request) -> Response + Send + Sync>> {
        let resolver = self.url_resolver.as_ref()?;
        let matched = resolver.resolve(&request.path)?;
        Some(matched.view)
    }

    /// Parse an incoming HTTP request to a Rjango request.
    pub fn parse_request(&self, method: &str, uri: &str, headers: &[(String, String)], body: Vec<u8>) -> Request {
        let rj_method = match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        };

        let mut rj_req = Request::new(rj_method, uri);
        for (name, value) in headers {
            rj_req.headers.insert(name.to_lowercase(), value.clone());
        }
        rj_req.body = body;

        // Parse cookies from header
        let cookie_header = rj_req.headers.get("cookie").cloned();
        if let Some(cookie_str) = cookie_header {
            for pair in cookie_str.split(';') {
                let mut parts = pair.splitn(2, '=');
                if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                    rj_req.cookies.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }

        rj_req
    }

    /// Convert a Rjango response to bytes.
    pub fn to_http_response(&self, resp: Response) -> (u16, Vec<(String, String)>, Vec<u8>) {
        let mut headers: Vec<(String, String)> = resp.headers.clone().into_iter().collect();
        for (_, value) in &resp.cookies {
            headers.push(("set-cookie".to_string(), value.clone()));
        }
        (resp.status_code(), headers, resp.body)
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the server on a given address using raw TCP + HTTP parsing.
pub async fn run_server(app: Arc<Application>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::net::TcpListener;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Rjango server running on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let app = app.clone();

        tokio::spawn(async move {
            let (reader, mut writer) = tokio::io::split(stream);
            let mut buf_reader = BufReader::new(reader);
            let mut request_line = String::new();

            if buf_reader.read_line(&mut request_line).await.is_err() {
                return;
            }
            let request_line = request_line.trim();
            if request_line.is_empty() {
                return;
            }

            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() < 2 {
                return;
            }
            let method = parts[0];
            let path = parts[1];

            // Read headers
            let mut headers = Vec::new();
            let mut content_length: usize = 0;
            loop {
                let mut line = String::new();
                if buf_reader.read_line(&mut line).await.is_err() {
                    break;
                }
                let line = line.trim().to_string();
                if line.is_empty() {
                    break;
                }
                if let Some(pos) = line.find(':') {
                    let name = line[..pos].trim().to_string();
                    let value = line[pos + 1..].trim().to_string();
                    if name.eq_ignore_ascii_case("content-length") {
                        content_length = value.parse().unwrap_or(0);
                    }
                    headers.push((name, value));
                }
            }

            // Read body if present
            let body = if content_length > 0 {
                let mut buf = vec![0u8; content_length];
                let _ = buf_reader.read_exact(&mut buf).await;
                buf
            } else {
                Vec::new()
            };

            let rj_req = app.parse_request(method, path, &headers, body);
            let rj_resp = app.handle_request(rj_req);
            let (status, resp_headers, resp_body) = app.to_http_response(rj_resp);

            // Write HTTP response
            let mut response = format!(
                "HTTP/1.1 {} {}\r\n",
                status,
                StatusCode::from(status as u16).reason_phrase()
            );
            for (name, value) in &resp_headers {
                response.push_str(&format!("{}: {}\r\n", name, value));
            }
            response.push_str(&format!("Content-Length: {}\r\n\r\n", resp_body.len()));

            let _ = writer.write_all(response.as_bytes()).await;
            if !resp_body.is_empty() {
                let _ = writer.write_all(&resp_body).await;
            }
        });
    }
}
