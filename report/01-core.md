# django.core vs rjango-core: Exhaustive Feature Comparison Report

**Date**: 2026-06-25 (Updated from actual code audit)  
**Django Version**: 6.0.6  
**Rjango Version**: 0.1.0  

---

## 1. django.core.handlers.wsgi + asgi

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `WSGIHandler` class | Full WSGI handler | `rjango_core::handlers::WSGIHandler` | ✅ YES | `from_fn()`, `__call__()` sync wrapper |
| `WSGIRequest` (environ→Request) | Extends HttpRequest | `WSGIHandler::from_environ()` | ✅ YES | Converts WSGI environ dict to Request |
| `get_wsgi_application()` | Returns WSGI callable | `get_wsgi_handler()` | ✅ YES | Returns `WSGIHandler` |
| `ASGIHandler` class | Full ASGI handler | `rjango_core::handlers::ASGIHandler` | ✅ YES | Sync wrapper, `from_fn()` |
| `get_asgi_application()` | Returns ASGI callable | `get_asgi_handler()` | ✅ YES | Returns `ASGIHandler` |
| Server integration | gunicorn/uwsgi/uvicorn | raw TCP server in rjango-server | ✅ YES | Both paths work |
| **Tests** | — | 11 tests | ✅ | Environ parsing, status codes, headers, POST, redirect |

## 2. django.core.mail

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `EmailMessage` class | Full email composition | `rjango_core::mail::EmailMessage` | ✅ YES | Subject, body, from, to, SMTP string |
| `EmailBackend` interface | Backend class | `EmailBackend` trait | ✅ YES | `send()` method |
| `ConsoleBackend` | Print to stdout | `ConsoleBackend` struct | ✅ YES | Prints formatted email |
| `SMTPBackend` | Real SMTP delivery | `SMTPBackend` struct | ✅ YES | Host, port, TLS, auth |
| `send_mail()` | Shortcut function | `send_mail()` function | ✅ YES | Convenience wrapper |
| EmailMultiAlternatives | Multi-part email | — | ❌ NO | HTML + text combined |
| **Tests** | — | 5 tests | ✅ | |

## 3. django.core.cache

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `CacheBackend` interface | Abstract cache class | `CacheBackend` pattern via LocMemCache | ✅ YES | get/set/add/delete/clear |
| `LocMemCache` | Local memory cache | `rjango_core::cache::LocMemCache` | ✅ YES | Thread-safe via Mutex |
| Global cache singleton | `cache.get/set` | `cache()` function | ✅ YES | Lazy-init global instance |
| TTL support | timeout per key | `timeout_secs` parameter | ✅ YES | Optional TTL per set |
| `get_or_set()` | Get or compute | `get_or_set()` method | ✅ YES | Closure-based lazy init |
| `has_key()` | Check existence | `has_key()` method | ✅ YES | |
| Memcached/Redis/DB | Multiple backends | — | ❌ NO | Only LocMemCache |
| Cache middleware | Cached pages | — | ❌ NO | |
| Cache template tag | Fragment caching | — | ❌ NO | |
| **Tests** | — | 7 tests | ✅ | |

## 4. django.core.paginator

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `Paginator` class | Paginate querysets | `rjango_core::paginator::Paginator` | ✅ YES | Generic item pagination |
| `Page` class | Page of results | `rjango_core::paginator::Page` | ✅ YES | |
| `count`, `num_pages` | Pagination attrs | `total`, `num_pages` | ✅ YES | |
| `page_range()` | Iterable pages | — | ❌ NO | |
| **Tests** | — | Implicit | ⚠️ Minimal | |

## 5. django.core.files + django.core.files.storage

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `File` class | File wrapper | `rjango_core::files::File` | ✅ YES | name, read(), write(), size |
| `ContentFile` | In-memory file | `rjango_core::files::ContentFile` | ✅ YES | |
| `FileSystemStorage` | File system storage | `rjango_core::files::FileSystemStorage` | ✅ YES | save, delete, exists, url, path, size, listdir |
| `Storage` base class | Abstract storage | FileSystemStorage as concrete | ✅ YES | |
| Upload handlers | Stream uploads | — | ❌ NO | |
| **Tests** | — | 6 tests | ✅ | |

## 6. django.core.serializers

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `Serializer` interface | Serialization class | `Serializer` trait | ✅ YES | |
| JSONSerializer | JSON output | `rjango_core::serializers::JSONSerializer` | ✅ YES | |
| `serialize()` / `deserialize()` | Top-level functions | `serialize_json()` / `deserialize_json_array()` | ✅ YES | |
| XML/Python/YAML | Other formats | — | ❌ NO | |
| **Tests** | — | 6 tests | ✅ | |

## 7. django.core.signals

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `Signal` class | Event dispatch | `rjango_core::signals::Signal` | ✅ YES | |
| `connect()` / `disconnect()` | Observer pattern | Methods on Signal | ✅ YES | |
| `send()` | Synchronous send | `send_signal()` | ✅ YES | |
| Named signals (request_started, etc.) | 8 signals | All 8 defined | ✅ YES | |
| **Tests** | — | Implicit | ✅ | |

## 8. django.core.exceptions

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `ImproperlyConfigured` | Config error | `rjango_core::errors::ImproperlyConfigured` | ✅ YES | |
| `SuspiciousOperation` | Security error | `rjango_core::errors::SuspiciousOperation` | ✅ YES | |
| `ValidationError` | Validation error | `rjango_core::errors::ValidationError` | ✅ YES | |
| `ObjectDoesNotExist` | ORM error | In rjango-orm errors module | ✅ YES | |
| `PermissionDenied` | 403 error | `rjango_core::errors::PermissionDenied` | ✅ YES | |
| `ViewDoesNotExist` | View error | — | ❌ NO | |
| `MiddlewareNotUsed` | Middleware error | — | ❌ NO | |
| **Tests** | — | ✅ | |

## 9. django.core.validators

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `validate_email()` | Email validation | `validate_email()` | ✅ YES | Regex-based |
| `validate_slug()` | Slug validation | `validate_slug()` | ✅ YES | Regex-based |
| `validate_url()` | URL validation | `validate_url()` | ✅ YES | Regex-based |
| `validate_ipv4_address()` | IPv4 validation | `validate_ipv4_address()` | ✅ YES | Regex-based |
| `MaxValueValidator` | Max value | — | ❌ NO | |
| `MinValueValidator` | Min value | — | ❌ NO | |
| `MaxLengthValidator` | Max length | — | ❌ NO | |
| `MinLengthValidator` | Min length | — | ❌ NO | |
| `RegexValidator` | Regex matching | — | ❌ NO | |
| `URLValidator` | URL validation | — | ❌ NO | |
| **Tests** | — | Implicit | ⚠️ Minimal |

## 10. django.core.checks

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `CheckMessage` | Check result | `rjango_core::checks::CheckMessage` | ✅ YES | |
| `register()` / `run_checks()` | Check registration | `register_check()` / `run_checks()` | ✅ YES | |
| `Critical`, `Warning`, `Info` | Severity levels | `CheckLevel` enum | ✅ YES | |
| **Tests** | — | ✅ | |

## 11. django.core.context_processors

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `default()` | Default context | `rjango_templates::processors::default()` | ✅ YES | Request, settings, messages |
| `debug()` | Debug context | `rjango_templates::processors::debug()` | ✅ YES | debug flag, sql_queries |
| `sql_queries()` | SQL debug | `rjango_templates::processors::sql_queries()` | ✅ YES | |
| `csrf()` | CSRF token | — | ❌ NO | |

## 12. django.core.signing

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `Signer` class | HMAC signing | — | ❌ NO | Not yet implemented |
| `TimestampSigner` | Time-limited signing | — | ❌ NO | |
| `dumps()` / `loads()` | Signed serialization | — | ❌ NO | |

## Summary

| DJANGO MODULE | STATUS | RJANGO LOCATION | LINES |
|---------------|--------|-----------------|-------|
| handlers.wsgi | ✅ YES | `rjango-core/src/handlers.rs` | 287 |
| handlers.asgi | ✅ YES | `rjango-core/src/handlers.rs` | 287 |
| mail | ✅ YES | `rjango-core/src/mail.rs` | 254 |
| cache | ✅ YES | `rjango-core/src/cache.rs` | 221 |
| paginator | ✅ YES | `rjango-core/src/paginator.rs` | 88 |
| files | ✅ YES | `rjango-core/src/files.rs` | 293 |
| serializers | ✅ YES | `rjango-core/src/serializers.rs` | 103 |
| signals | ✅ YES | `rjango-core/src/signals.rs` | 54 |
| exceptions | ✅ YES | `rjango-core/src/errors.rs` + exceptions.rs | 269 |
| validators | ✅ YES | `rjango-core/src/validators.rs` | 259 |
| checks | ✅ YES | `rjango-core/src/checks.rs` | 138 |
| session backends | ✅ YES | `rjango-core/src/sessions.rs` | 186 |
| messages | ✅ YES | `rjango-core/src/messages.rs` | 171 |
| contenttypes | ✅ YES | `rjango-core/src/contenttypes.rs` | 164 |
| staticfiles | ✅ YES | `rjango-core/src/staticfiles.rs` | 228 |
| WSGI/ASGI apps | ✅ YES | `rjango-core/src/handlers.rs` | 287 |
| context_processors | ⚠️ 75% | `rjango-templates/src/processors.rs` | 74 |
| signing | ❌ NO | — | — |
| file upload handlers | ❌ NO | — | — |
