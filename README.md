# 🦀 Rjango

A Django-like web framework for Rust. Mirrors Django 6.0.6's architecture with 16 crates covering URL routing, ORM, server, admin, templates, forms, auth, middleware, migrations, and CLI.

## Quick Start

```bash
# Build everything
cargo build

# Run tests
cargo test

# See available CLI commands
cargo run -- --help

# Start a dev server
cargo run -- runserver 0.0.0.0:8080
```

## Project Structure (16 crates)

| Crate | Lines | Description |
|-------|-------|-------------|
| `rjango-core` | 1564 | Core types: Request, Response, Settings, Signals, Paginator, Validators |
| `rjango-orm` | 771 | Models, fields, query builder, relationships, database config |
| `rjango-server` | 206 | HTTP server with raw TCP, static file serving |
| `rjango-cli` | 607 | CLI: runserver, startproject, migrate, shell, createsuperuser |
| `rjango-admin` | 460 | Auto-generated admin CRUD with styled HTML |
| `rjango-templates` | 378 | Template engine, loaders, filters, tags, context processors |
| `rjango-forms` | 671 | Form fields, validation, rendering (as_table, as_p, as_div) |
| `rjango-auth` | 270 | User model, auth backends, login/logout views |
| `rjango-middleware` | 216 | CSRF, Security, Session, Message middleware |
| `rjango-urls` | 400 | URL patterns, resolvers, path converters |
| `rjango-views` | 360 | TemplateView, ListView, DetailView, FormView, CRUD views |
| `rjango-migrations` | 292 | Migration detection, operations, runner |
| `rjango-dispatch` | 105 | Signal system |
| `rjango-conf` | 37 | Global settings access |
| `rjango-utils` | 274 | Crypto, text processing, HTTP utilities |
| `rjango-test` | 160 | Test client, test runner, test cases |
| **Total** | **~6800** | |

## Architecture

```
Request → Middleware Stack → URL Resolver → View → Response
                            ↓
                    Template Engine / ORM
```

- **Middleware** is synchronous (not async) for dyn-compatibility
- **Views** are sync: `fn view(request: Request) -> Response`
- **Server** uses raw TCP + manual HTTP parsing (no hyper)
- **All crates** use `#![forbid(unsafe_code)]`

## API Example

```rust
use rjango_core::{Request, Response};
use rjango_urls::{path, URLResolver};
use rjango_server::Application;
use rjango_middleware::{MiddlewareStack, csrf::CsrfMiddleware};
use std::sync::Arc;

fn hello(req: Request) -> Response {
    Response::html("<h1>Hello, Rjango!</h1>")
}

fn main() {
    let patterns = vec![path("/", hello, Some("home"))];
    let resolver = URLResolver::new(patterns);

    let mut mw = MiddlewareStack::new();
    mw.add(Box::new(CsrfMiddleware));

    let app = Application::new()
        .with_urls(resolver)
        .with_middleware(mw);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(rjango_server::run_server(
        Arc::new(app),
        "127.0.0.1:8080".parse().unwrap()
    )).unwrap();
}
```

## Status

- [x] URL routing with path converters
- [x] Middleware stack (CSRF, Security, Sessions, Messages)
- [x] Admin CRUD with styled HTML
- [x] ORM with model definitions, query builder, field types
- [x] Template engine with loaders, filters, tags
- [x] Form handling with validation and rendering
- [x] Auth: login/logout views, user model, backends
- [x] CLI commands: runserver, migrate, startapp, shell
- [x] Signal/dispatch system
- [x] Database migration detection
- [x] SQLite database execution layer
- [x] Static file serving
- [ ] Database-backed ORM CRUD (sqlx integration)
- [ ] Comprehensive end-to-end test suite
- [ ] HTTPS support
- [ ] ASGI/WSGI compatibility layer

## License

MIT
