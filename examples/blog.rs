// Demo blog app: a complete usage example for rjango

use rjango_core::{Request, Response, StatusCode, QueryDict, HttpMethod};
use rjango_server::Application;
use rjango_urls::{path, URLResolver};
use rjango_middleware::{MiddlewareStack, Middleware, RjangoError};
use rjango_forms::{Form, FieldType, FormField};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ── In-memory database ────────────────────────────────────────────
#[derive(Clone, Debug)]
struct Post {
    id: u64,
    title: String,
    content: String,
    published: bool,
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
        self.posts.push(Post { id, title, content, published });
        id
    }

    fn list(&self) -> &[Post] {
        &self.posts
    }

    fn get(&self, id: u64) -> Option<&Post> {
        self.posts.iter().find(|p| p.id == id)
    }

    fn delete(&mut self, id: u64) -> bool {
        let len = self.posts.len();
        self.posts.retain(|p| p.id != id);
        self.posts.len() < len
    }
}

static DB: std::sync::OnceLock<Mutex<BlogDB>> = std::sync::OnceLock::new();
fn get_db() -> &'static Mutex<BlogDB> {
    DB.get_or_init(|| Mutex::new(BlogDB::new()))
}

// ── Blog Views ────────────────────────────────────────────────────
fn blog_index(req: Request) -> Response {
    let db = get_db().lock().unwrap();
    let posts = db.list();
    let mut html = String::from(
        "<!DOCTYPE html><html><head><title>Blog</title>
        <style>body{font-family:sans-serif;max-width:800px;margin:auto;padding:20px}
        .post{border:1px solid #ddd;padding:15px;margin:10px 0;border-radius:5px}
        h1{color:#333} a{color:#3498db}</style></head><body>"
    );
    html.push_str("<h1>📝 Rjango Blog</h1>");
    html.push_str("<a href='/blog/new/' style='display:inline-block;padding:8px 16px;background:#3498db;color:#fff;border-radius:4px;text-decoration:none'>+ New Post</a>");
    html.push_str("<hr>");
    if posts.is_empty() {
        html.push_str("<p>No posts yet. Create one!</p>");
    }
    for post in posts {
        html.push_str(&format!(
            "<div class='post'><h2><a href='/blog/{}/'>{}</a></h2><p>{}</p><small>ID: {}</small></div>",
            post.id, post.title, &post.content[..post.content.len().min(100)], post.id
        ));
    }
    html.push_str("</body></html>");
    Response::html(&html)
}

fn blog_detail(req: Request) -> Response {
    // Extract ID from path like /blog/1/
    let id_str = req.path
        .trim_start_matches("/blog/")
        .trim_end_matches('/');
    let id: u64 = match id_str.parse() {
        Ok(n) => n,
        Err(_) => return Response::not_found(),
    };

    let db = get_db().lock().unwrap();
    match db.get(id) {
        Some(post) => {
            let html = format!(
                "<!DOCTYPE html><html><head><title>{}</title>
                <style>body{{font-family:sans-serif;max-width:800px;margin:auto;padding:20px}}</style>
                </head><body>
                <h1>{}</h1>
                <p>{}</p>
                <hr>
                <a href='/blog/'>← Back to blog</a>
                </body></html>",
                post.title, post.title, post.content
            );
            Response::html(&html)
        }
        None => Response::not_found(),
    }
}

fn blog_new(req: Request) -> Response {
    if req.method == HttpMethod::POST {
        // Handle form submission
        let body_str = String::from_utf8_lossy(&req.body);
        let params = QueryDict::from_query(&body_str);
        let title = params.get("title").unwrap_or("Untitled").to_string();
        let content = params.get("content").unwrap_or("").to_string();
        let published = params.get("published").unwrap_or("off") == "on";

        let mut db = get_db().lock().unwrap();
        let id = db.create(title, content, published);
        return Response::redirect(&format!("/blog/{}/", id), false);
    }

    // GET: show form
    let html = String::from(
        "<!DOCTYPE html><html><head><title>New Post</title>
        <style>body{font-family:sans-serif;max-width:800px;margin:auto;padding:20px}
        label{display:block;margin:10px 0 4px}
        input,textarea{width:100%;padding:8px;border:1px solid #ddd;border-radius:4px}
        button{padding:10px 20px;background:#3498db;color:#fff;border:none;border-radius:4px;cursor:pointer}
        </style></head><body>
        <h1>New Blog Post</h1>
        <form method='post'>
            <input type='hidden' name='csrfmiddlewaretoken' value='rjango-csrf-token'>
            <label>Title:</label>
            <input type='text' name='title' required><br>
            <label>Content:</label>
            <textarea name='content' rows='8'></textarea><br>
            <label><input type='checkbox' name='published' checked> Published</label><br><br>
            <button type='submit'>Create Post</button>
            <a href='/blog/' style='margin-left:10px;color:#666'>Cancel</a>
        </form>
        </body></html>"
    );
    Response::html(&html)
}

// ── Home View ─────────────────────────────────────────────────────
fn home(req: Request) -> Response {
    let html = String::from(
        "<!DOCTYPE html><html><head><title>Rjango Demo</title>
        <style>body{font-family:sans-serif;max-width:800px;margin:auto;padding:40px;text-align:center}
        h1{font-size:3em;color:#333} .sub{color:#666;font-size:1.2em}
        .btn{display:inline-block;padding:12px 24px;margin:10px;background:#3498db;color:#fff;
        border-radius:6px;text-decoration:none;font-weight:bold}
        .btn:hover{background:#2980b9}
        .card{border:1px solid #ddd;border-radius:8px;padding:20px;margin:20px 0;text-align:left}
        </style></head><body>
        <h1>🦀 Rjango</h1>
        <p class='sub'>A Django-like web framework for Rust</p>
        <div class='card'>
            <h3>Quick Links</h3>
            <a href='/blog/' class='btn'>📝 Blog Demo</a>
            <a href='/admin/' class='btn'>⚙️ Admin</a>
        </div>
        <div class='card'>
            <h3>Features</h3>
            <ul>
                <li>URL routing with path converters</li>
                <li>Middleware stack (CSRF, Security, Sessions)</li>
                <li>Sync view functions</li>
                <li>Admin CRUD interface</li>
                <li>ORM with query builder</li>
                <li>Form validation & rendering</li>
                <li>Template engine with inheritance</li>
                <li>Auth (login/logout)</li>
                <li>Static file serving</li>
                <li>CLI: runserver, migrate, startapp, shell</li>
            </ul>
        </div>
        <p style='color:#999'>Running on Rjango 0.1.0</p>
        </body></html>"
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
    mw.add(Box::new(rjango_middleware::csrf::CsrfMiddleware));
    mw.add(Box::new(rjango_middleware::security::SecurityMiddleware));
    mw.add(Box::new(rjango_middleware::session::SessionMiddleware));

    let app = Application::new()
        .with_urls(resolver)
        .with_middleware(mw);

    // Run via server
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let addr = "127.0.0.1:8080".parse().unwrap();
    runtime.block_on(rjango_server::run_server(Arc::new(app), addr)).unwrap();
}
