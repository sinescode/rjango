# Django vs Rjango: Middleware, Security, Server, HTTP & URL Routing Comparison

**Date**: 2026-06-25 (Updated)  
**Django Version**: 6.0.6  
**Rjango Version**: 0.1.0  

---

## django.middleware vs rjango-middleware

Rjango Location: `rjango-middleware/src/` (321 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| Middleware trait | Middleware API | `Middleware` trait | ✅ YES | Sync only (no async) |
| MiddlewareStack | Chain middlewares | `MiddlewareStack` struct | ✅ YES | process_request → view → process_response |
| CommonMiddleware | URL normalization | — | ❌ NO | |
| SecurityMiddleware | Security headers | `SecurityMiddleware` | ✅ YES | X-Frame-Options, XSS, X-Content-Type |
| SessionMiddleware | Session support | `SessionMiddleware` | ✅ YES | File-based session |
| CSRFMiddleware | CSRF protection | `CsrfMiddleware` | ✅ YES | Real crypto tokens, body+header+query check |
| AuthenticationMiddleware | User attach | `AuthMiddleware` (in rjango-auth) | ✅ YES | Attaches user to request |
| MessageMiddleware | Flash messages | `MessageMiddleware` | ✅ YES | |
| GZipMiddleware | Compression | — | ❌ NO | |
| CacheMiddleware | Page cache | — | ❌ NO | |
| LocaleMiddleware | Language | — | ❌ NO | |
| ClickjackingMiddleware | X-Frame-Options | In SecurityMiddleware | ✅ YES | |
| ConditionalGetMiddleware | ETag | — | ❌ NO | |
| UpdateCacheMiddleware | Cache writing | — | ❌ NO | |
| FetchFromCacheMiddleware | Cache reading | — | ❌ NO | |
| CSPMiddleware | Content security | — | ❌ NO | |
| **Tests** | — | 3 tests | ✅ | |

### CSRF Middleware (Detailed)

| Sub-feature | Status | Details |
|------------|--------|---------|
| Token generation (real crypto) | ✅ | Uses `rjango_utils::crypto::get_random_secret_key()` |
| Token format | ✅ | `rjcsrf-<32-hex-chars>` |
| Cookie-based validation | ✅ | Reads `csrftoken` cookie |
| Body param validation | ✅ | Reads `csrfmiddlewaretoken` from form body |
| Query param validation | ✅ | Reads `csrfmiddlewaretoken` from query string |
| Header validation | ✅ | Reads `x-csrftoken` header |
| GET requests pass through | ✅ | Only POST/PUT/PATCH/DELETE checked |
| Token auto-set in response | ✅ | `process_response()` sets cookie if absent |
| **Tests** | ✅ | 4 tests |

---

## django.http vs rjango-core HTTP

Rjango Location: `rjango-core/src/http.rs` (347 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| HttpMethod enum | GET, POST, etc | `HttpMethod` enum | ✅ YES | GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS, CONNECT, TRACE |
| Request struct | Full request | `Request` struct | ✅ YES | method, path, query, headers, body, cookies |
| Response struct | Full response | `Response` struct | ✅ YES | status, body, headers, cookies |
| QueryDict | Multi-value params | `QueryDict` struct | ✅ YES | get(), keys(), contains(), iter() |
| StatusCode enum | HTTP status codes | `StatusCode` struct | ✅ YES | All standard codes |
| Headers | Case-insensitive | HashMap | ⚠️ PARTIAL | Case-sensitive |
| Cookie handling | Set/read | `set_cookie()`, `cookie()`, get headers | ✅ YES | |
| JSON response | JsonResponse | `Response::json()` | ✅ YES | |
| HTML response | HttpResponse | `Response::html()` | ✅ YES | |
| Redirect | HttpResponseRedirect | `Response::redirect()` | ✅ YES | |
| FileResponse | File download | — | ❌ NO | |
| StreamingHttpResponse | Stream body | — | ❌ NO | |

---

## rjango-server

Rjango Location: `rjango-server/src/lib.rs` (287 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| HTTP server | manage.py runserver | `run_server()` async fn | ✅ YES | Raw TCP + tokio |
| Request parsing | WSGI environ | Manual HTTP parse | ✅ YES | |
| Response writing | WSGI start_response | Manual HTTP format | ✅ YES | |
| Static file serving | django.contrib.staticfiles | `Application::with_static_dir()` | ✅ YES | /static/ routes |
| Media file serving | MEDIA_URL | `Application::with_media_dir()` | ✅ YES | /media/ routes |
| Application class | WSGI handler | `Application` struct | ✅ YES | with_urls, with_middleware, with_templates |
| URL routing dispatch | URLResolver | App dispatches via resolver | ✅ YES | |
| Middleware integration | process_request/response | `handle_request()` | ✅ YES | Full middleware chain |
| HTTPS support | Secure server | — | ❌ NO | |
| HTTP/2 | HTTP/2 support | — | ❌ NO | |
| Keep-alive | Connection reuse | — | ❌ NO | |
| WebSocket | Async WS | — | ❌ NO | |

---

## django.urls vs rjango-urls

Rjango Location: `rjango-urls/src/` (403 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `path()` function | URL pattern | `path()` function | ✅ YES | |
| `re_path()` | Regex URL | `re_path()` function | ✅ YES | |
| `include()` | Include patterns | — | ❌ NO | |
| URLPattern | Single pattern | `URLPattern` struct | ✅ YES | |
| URLResolver | Pattern group | `URLResolver` struct | ✅ YES | |
| `resolve()` | Match URL | `resolve()` | ✅ YES | Returns matched pattern |
| `reverse()` | Name→URL | `reverse()` | ✅ YES | |
| Converters (int, str, slug, uuid, path) | Type safety | `<int:id>` syntax | ✅ YES | String, Int, Slug, UUID converters |
| Namespaces | URL namespacing | — | ❌ NO | |
| `register_converter()` | Custom converters | — | ❌ NO | |
| **Tests** | — | ~8 tests | ✅ | |
