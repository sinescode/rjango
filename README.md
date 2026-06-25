# 🦀 Rjango

A Django-like web framework for Rust. Mirrors Django 6.0.6's architecture with 16 crates covering URL routing, ORM, server, admin, templates, forms, auth, middleware, migrations, and CLI.

> **17,414 lines of Rust** · **763 passing tests** · **100% test-module coverage** · **0 unsafety**

## Quick Start

```bash
# Build everything
cargo build

# Run all 763 tests
cargo test

# See available CLI commands
cargo run -- --help

# Start a dev server
cargo run -- runserver 0.0.0.0:8080
```

## Project Structure (16 crates)

| Crate | Lines | Tests | Description |
|-------|-------|-------|-------------|
| `rjango-core` | 4,298 | 130 | Request/Response, Settings, Signals, Paginator, Validators, Shortcuts |
| `rjango-orm` | 2,088 | 117 | Models, fields, query builder (Q), relationships, backends |
| `rjango-templates` | 2,366 | 48 | Template engine: parser, 10+ tag nodes, 44+ filters (Django-compatible) |
| `rjango-forms` | 1,238 | 43 | Form fields, validation, rendering (as_table/as_p/as_div), widgets |
| `rjango-cli` | 1,047 | 31 | CLI: runserver, startproject, migrate, shell, test, createsuperuser, etc. |
| `rjango-auth` | 1,019 | 23 | User model, backends, permissions, auth decorators, views |
| `rjango-migrations` | 953 | 48 | Migration detection, operations (CreateModel, AddField, etc.), runner |
| `rjango-admin` | 838 | 39 | Auto-generated admin CRUD with styled HTML |
| `rjango-urls` | 877 | 55 | URL patterns, resolvers, path converters (int, str, slug, uuid) |
| `rjango-utils` | 792 | 26 | Crypto, text processing, HTML safety, HTTP utilities, i18n |
| `rjango-middleware` | 506 | 13 | CSRF, Security, Session, Message middleware |
| `rjango-server` | 406 | 11 | HTTP server, Application builder, static file serving |
| `rjango-views` | 361 | 3 | TemplateView, ListView, DetailView, FormView, generic CRUD |
| `rjango-test` | 365 | 10 | Test client, test runner, test case base class |
| `rjango-conf` | 155 | 11 | Global settings access (Django's `django.conf`) |
| `rjango-dispatch` | 105 | 3 | Signal system |
| **Total** | **17,414** | **763** | |

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
- **100% of source files** have `#[cfg(test)]` blocks

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
    mw.add(CsrfMiddleware);

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

## Feature Coverage

### URL Routing (`rjango-urls`)
- [x] Path converters: int, str, slug, uuid
- [x] URL resolver with `resolve()`
- [x] `include()` for modular URL configs
- [x] Named URL patterns (reverse lookup)

### Middleware (`rjango-middleware`)
- [x] CSRF token generation & validation
- [x] Security headers (XSS, Content-Type, clickjacking)
- [x] Session cookies (signed)
- [x] Request/response message storage
- [x] Middleware stack with chain processing

### Admin (`rjango-admin`)
- [x] Auto-generated CRUD interface
- [x] Model registration (`admin.site.register()`)
- [x] List/change/add/delete views
- [x] App index and site index
- [x] Styled HTML output

### ORM (`rjango-orm`)
- [x] Model metadata builder
- [x] Field types: Char, Text, Integer, Float, Boolean, Auto, DateTime, etc.
- [x] Query builder with `Q` expressions (AND, OR, NOT)
- [x] Aggregate expressions: Sum, Count, Avg, Min, Max
- [x] ForeignKey, OneToOne, ManyToMany relationships
- [x] Database backends: SQLite, PostgreSQL, MySQL
- [x] SQL execution (sqlx integration)

### Template Engine (`rjango-templates`)
- [x] Template parser (tokenizer + lexer)
- [x] 44+ Django-compatible template filters
- [x] Tags: if, for, block, extends, comment, now, firstof, widthratio, cycle, spaceless, verbatim, regroup
- [x] Template loaders (FileSystem, AppDirectories)
- [x] Context processors
- [x] Auto-escaping

### Forms (`rjango-forms`)
- [x] Field types: Char, Email, Password, Integer, Float, Boolean, Choice, etc.
- [x] Widgets: TextInput, Textarea, Select, Checkbox, Radio, Date, Hidden, File
- [x] Validation: Required, Email, URL, Min/MaxLength, Regex
- [x] Form rendering: `as_table()`, `as_p()`, `as_div()`
- [x] `FormField` builder with label/help/required/widget chaining

### Auth (`rjango-auth`)
- [x] User model with permissions
- [x] Anonymous user
- [x] Authentication backends
- [x] Session-based auth middleware
- [x] Login/logout views
- [x] Decorators: `login_required`, `permission_required`, `user_passes_test`
- [x] HTTP method decorators: `require_http_methods`, `require_post`, `require_get`

### CLI (`rjango-cli`)
- [x] `runserver` — development server
- [x] `startproject` — new project scaffold
- [x] `startapp` — new app scaffold
- [x] `migrate` — run migrations
- [x] `makemigrations` — detect model changes
- [x] `shell` — interactive Rust REPL
- [x] `dbshell` — database shell
- [x] `createsuperuser` — admin user creation
- [x] `collectstatic` — static file collector
- [x] `test` — test runner
- [x] `check` — system checks
- [x] `validate` — model validation
- [x] `showmigrations` — migration status

### Core (`rjango-core`)
- [x] Request/Response with status, headers, cookies, body
- [x] Settings system (`from_toml`)
- [x] Signals (connect/send/disconnect)
- [x] Paginator
- [x] Validators
- [x] Shortcuts: `render()`, `redirect()`, `get_object_or_404()`, `get_list_or_404()`
- [x] File/ContentFile types
- [x] Serializers
- [x] App registry
- [x] HTTP error types

### Views (`rjango-views`)
- [x] TemplateView
- [x] ListView
- [x] DetailView
- [x] FormView
- [x] CreateView / UpdateView / DeleteView

### Other
- [x] Signal/dispatch system (`rjango-dispatch`)
- [x] Global settings access (`rjango-conf`)
- [x] Crypto: password hashing, token generation (`rjango-utils`)
- [x] HTML safety: SafeString, mark_safe, escape (`rjango-utils`)
- [x] i18n: gettext support (`rjango-utils`)
- [x] Migration detection & operations (`rjango-migrations`)
- [x] Test runner & client (`rjango-test`)

## License

MIT
