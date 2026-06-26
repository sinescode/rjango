/// Demo blog app: runs on rjango with raw TCP server.
/// Usage: cargo run --example blog -p rjango-test

use rjango_core::{Request, Response, QueryDict, HttpMethod};
use rjango_server::Application;
use rjango_urls::{path, URLResolver};
use rjango_middleware::MiddlewareStack;
use std::sync::{Arc, Mutex};

// ── In-memory database ────────────────────────────────────────────
#[derive(Clone, Debug)]
struct Post {
    id: u64,
    title: String,
    content: String,
    _published: bool,
}

struct BlogDB {
    posts: Vec<Post>,
    next_id: u64,
}

impl BlogDB {
    fn new() -> Self {
        Self { posts: vec![], next_id: 1 }
    }

    fn create(&mut self, title: String, content: String, published: bool) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.posts.push(Post { id, title, content, _published: published });
        id
    }

    fn list(&self) -> &[Post] {
        &self.posts
    }

    fn get(&self, id: u64) -> Option<&Post> {
        self.posts.iter().find(|p| p.id == id)
    }
}

static DB: std::sync::OnceLock<Mutex<BlogDB>> = std::sync::OnceLock::new();
fn get_db() -> &'static Mutex<BlogDB> {
    DB.get_or_init(|| Mutex::new(BlogDB::new()))
}

// ── Styled HTML templates ─────────────────────────────────────────
const STYLES: &str = "<style>
body{font-family:sans-serif;max-width:800px;margin:auto;padding:20px;line-height:1.6}
.post{border:1px solid #e0e0e0;padding:15px;margin:10px 0;border-radius:6px}
.post:hover{box-shadow:0 2px 8px rgba(0,0,0,0.1)}
h1{color:#2c3e50} h2{color:#3498db}
a{color:#3498db;text-decoration:none} a:hover{text-decoration:underline}
.btn{display:inline-block;padding:8px 16px;background:#3498db;color:#fff;
border-radius:4px;text-decoration:none;font-weight:bold}
.btn:hover{background:#2980b9;text-decoration:none}
label{display:block;margin:10px 0 4px;font-weight:bold}
input,textarea{width:100%;padding:8px;border:1px solid #ddd;border-radius:4px;font-size:14px}
button{padding:10px 20px;background:#3498db;color:#fff;border:none;border-radius:4px;cursor:pointer;font-size:14px}
button:hover{background:#2980b9}
.card{border:1px solid #ddd;border-radius:8px;padding:20px;margin:20px 0}
</style>";

// ── Blog Views ────────────────────────────────────────────────────
fn blog_index(_req: Request) -> Response {
    let db = get_db().lock().unwrap();
    let posts = db.list();
    let mut html = format!(
        "<!DOCTYPE html><html><head><title>Blog</title>{}</head><body>", STYLES
    );
    html.push_str("<h1>Rjango Blog</h1>");
    html.push_str("<a href='/blog/new/' class='btn'>+ New Post</a><hr>");
    if posts.is_empty() {
        html.push_str("<p>No posts yet. Create one!</p>");
    }
    for post in posts {
        let preview: String = post.content.chars().take(100).collect();
        html.push_str(&format!(
            "<div class='post'><h2><a href='/blog/{}/'>{}</a></h2>\
             <p>{}</p><small>ID: {}</small></div>",
            post.id, post.title, preview, post.id
        ));
    }
    html.push_str("</body></html>");
    Response::html(&html)
}

fn blog_detail(req: Request) -> Response {
    let id_str = req.path.trim_start_matches("/blog/").trim_end_matches('/');
    let id: u64 = match id_str.parse() {
        Ok(n) => n,
        Err(_) => return Response::not_found(),
    };

    let db = get_db().lock().unwrap();
    match db.get(id) {
        Some(post) => {
            let html = format!(
                "<!DOCTYPE html><html><head><title>{}</title>{}</head><body>\
                 <h1>{}</h1><p>{}</p><hr>\
                 <a href='/blog/'>Back to blog</a>\
                 </body></html>",
                post.title, STYLES, post.title, post.content
            );
            Response::html(&html)
        }
        None => Response::not_found(),
    }
}

fn blog_new(req: Request) -> Response {
    if req.method == HttpMethod::POST {
        let body_str = String::from_utf8_lossy(&req.body);
        let params = QueryDict::from_query(&body_str);
        let title = params.get("title").unwrap_or("Untitled").to_string();
        let content = params.get("content").unwrap_or("").to_string();
        let published = params.get("published").unwrap_or("off") == "on";

        let mut db = get_db().lock().unwrap();
        let id = db.create(title, content, published);
        return Response::redirect(&format!("/blog/{}/", id), false);
    }

    // Generate CSRF token, embed in hidden input and set cookie
    let csrf_token = rjango_middleware::csrf::CsrfMiddleware::generate_token();
    let html = format!(
        "<!DOCTYPE html><html><head><title>New Post</title>{}</head><body>\
         <h1>New Blog Post</h1>\
         <form method='post'>\
         <input type='hidden' name='csrfmiddlewaretoken' value='{}'>\
         <label>Title:</label>\
         <input type='text' name='title' required><br>\
         <label>Content:</label>\
         <textarea name='content' rows='8'></textarea><br>\
         <label><input type='checkbox' name='published' checked> Published</label><br><br>\
         <button type='submit'>Create Post</button>\
         <a href='/blog/' style='margin-left:10px;color:#666'>Cancel</a>\
         </form></body></html>",
        STYLES, csrf_token
    );
    let mut resp = Response::html(&html);
    resp.set_cookie("csrftoken", &csrf_token);
    resp
}

// ── Home View ─────────────────────────────────────────────────────
fn home(_req: Request) -> Response {
    let html = format!(
        "<!DOCTYPE html><html><head><title>Rjango Demo</title>{}</head><body>\
         <h1>Rjango</h1>\
         <p style='font-size:1.2em;color:#666'>A Django-like web framework for Rust</p>\
         <div class='card'>
            <h2>Quick Links</h2>\
            <a href='/blog/' class='btn'>Blog Demo</a>\
            <a href='/admin/' class='btn'>Admin</a>
         </div>\
         <div class='card'>
            <h2>Features</h2><ul>\
            <li>URL routing with path converters</li>\
            <li>Middleware stack (CSRF, Security, Sessions)</li>\
            <li>Sync view functions</li>\
            <li>Admin CRUD interface</li>\
            <li>ORM with query builder</li>\
            <li>Form validation and rendering</li>\
            <li>Template engine</li>\
            <li>Auth (login/logout)</li>\
            <li>Static file serving</li>\
            <li>CLI: runserver, migrate, startapp, shell</li></ul>\
         </div>\
         <p style='color:#999'>Running on Rjango 0.1.0</p>\
         </body></html>",
        STYLES
    );
    Response::html(&html)
}

// ── Application Setup ─────────────────────────────────────────────
fn main() {
    let patterns = vec![
        path("/", home, Some("home")),
        path("/blog/", blog_index, Some("blog:index")),
        path("/blog/new/", blog_new, Some("blog:new")),
        path("/blog/<int:id>/", blog_detail, Some("blog:detail")),
    ];

    let resolver = URLResolver::new(patterns);

    // Set up middleware stack
    let mut mw = MiddlewareStack::new();
    mw.add(rjango_middleware::csrf::CsrfMiddleware);
    mw.add(rjango_middleware::security::SecurityMiddleware);
    mw.add(rjango_middleware::session::SessionMiddleware::new());
    mw.add(rjango_middleware::messages::MessageMiddleware);

    let app = Application::new()
        .with_urls(resolver)
        .with_middleware(mw);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    println!("Rjango blog server starting on http://{}:{}", host, port);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(rjango_server::run_server(Arc::new(app), addr)).unwrap();
}
