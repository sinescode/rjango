# Django 6.0.6 Core — Feature Comparison with Rjango

> **Django modules analyzed:** `django.core`, `django.apps`, `django.conf`, `django.dispatch`, `django.http`, `django.shortcuts`, `django.utils`
> **Last updated:** 2026-06-26

---

## Overview

| Metric | Value |
|--------|-------|
| Django source files | 835 total (all modules) |
| Rjango core Rust files | 22 in `rjango-core/src/`, 9 in `rjango-utils/src/` |
| Core features covered | 37/94 (39.4%) |
| Tests passing | 272 (rjango-core only) |

---

## ✅ Implemented

### HTTP Layer
- **Request** — `HttpMethod`, headers, query params, `Request::is_ajax()`, `Request::is_secure()`, `Request::get_host()`, `Request::build_absolute_uri()`
- **Response** — `Response` struct with status, headers, cookies, body
- **StreamingHttpResponse** — Chunked streaming with `header()`/`set_header()`
- **FileResponse** — Attachment and inline file downloads
- **QueryDict** — MultiValueDict with urlencode
- **StatusCode** — Wrapper with `from(u16)`, `as_u16()`

### Cache (Partial)
- Free functions: `cache_set()`, `cache_get()`, `cache_delete()`, `cache_clear()`, `cache_get_or_set()`, `cache_add()`, `cache_incr()`, `cache_decr()`
- **Missing:** `CacheBackend` trait, LocMemCache/FileBasedCache/DatabaseCache/DummyCache as proper backends, key prefixing/versioning, template fragment caching

### Mail (Partial)
- `EmailMessage`, `EmailMultiAlternatives` with attachments
- `send_mail()`, `send_mass_mail()`, `mail_admins()`, `mail_managers()`
- `EmailMessage::attach()`, `attach_alternative()`, `content_subtype()`
- `EmailMultiAlternatives::attach_alternative()`
- **Missing:** SMTP backend, Console backend, File-based backend, `attach_file()`

### Paginator (Partial)
- `Paginator` struct with `page()`, `count`, `num_pages`, `per_page`, `page_range`
- `Page` struct with basic API
- **Missing:** `elided_page_range()`, `orphans`, `allow_empty_first_page`, `AsyncPaginator`, proper error handling

### Signals & Dispatch
- `Signal` struct with `connect()`, `send()`, `send_robust()`, `disconnect()`, `receiver_count()`
- Built-in signals in `rjango-dispatch`: `request_started`, `request_finished`
- **Missing:** `got_request_exception`, `setting_changed` signals

### Signing
- `Signer`, `TimestampSigner` with HMAC signing
- `b62_encode()`, `b62_decode()`, `b64_encode()`, `b64_decode()`
- Algorithm enum (SHA256, SHA1)
- **Missing:** `dumps()`/`loads()` convenience, `JSONSerializer` as standalone

### Exceptions
- 21 exception types: `SuspiciousOperation`, `PermissionDenied`, `ViewDoesNotExist`, `MiddlewareNotUsed`, `ImproperlyConfigured`, `FieldDoesNotExist`, `MultipleObjectsReturned`, `ObjectDoesNotExist`, `EmptyResultSet`, `SuspiciousFileOperation`, `DisallowedHost`, `BadRequest`, `ValidationError`, etc.

### Validators (Partial)
- `Validator` struct with `required_validator()`, `email_validator()`, `url_validator()`, `slug_validator()`, `integer_validator()`, `ipv4_validator()`, `ipv6_validator()`, `ipv46_validator()`, `max_length_validator()`, `min_length_validator()`, `regex_validator()`, `max_value_validator()`, `min_value_validator()`
- **Missing:** `DecimalValidator`, `FileExtensionValidator`, `ProhibitNullCharactersValidator`, `StepValueValidator`, `validate_integer()`, `validate_unicode_slug()`

### Shortcuts
- `render()`, `redirect()`, `get_object_or_404()`, `get_list_or_404()`, `resolve_url()`

### Serializers (Partial)
- `Serializer` trait, `JSONSerializer`, `serialize_json()`, `deserialize_json()`, `serialize_xml()`
- **Missing:** Python serializer, YAML serializer, deserialization error handling

### Files / Storage (Partial)
- `File`, `ContentFile`, `FileSystemStorage` with `save()`, `delete()`, `exists()`
- `Storage` trait with `InMemoryStorage`, `FileSystemStorage`
- **Missing:** Upload handlers (`MemoryFileUploadHandler`, `TemporaryFileUploadHandler`, `FileUploadHandler`), `ImageFile`, `TemporaryUploadedFile`

### Handlers (Partial)
- `WSGIHandler` and `ASGIHandler` structs
- **Missing:** `BaseHandler` with `get_response()` chain, exception handling, `LimitedStream`, `adapt_method_mode`

### Settings
- `Settings` struct with `get()`, `set()`, `from_toml()` / from environment
- `DatabaseConfig` with `url()` connection string builder
- `rjango-conf` module with `get_settings()`, `set_settings()`, `debug()`, `secret_key()`, `allowed_hosts()`, `installed_apps()`, `root_urlconf()`

### Apps / AppConfig
- `AppConfig`, `Registry`, `ApplicationBuilder`
- App registration, model registration, app listing
- **Missing:** `populate()`, `get_model()`, `get_models()`, `AppConfig.ready()` hook

### Staticfiles (Partial)
- `StaticFinder` trait, `FileSystemFinder`, `StaticFilesConfig`
- **Missing:** `AppDirectoriesFinder`, `CachedStaticFilesStorage`, static file serving view

### ContentTypes (Partial)
- `ContentType`, `ContentTypeManager`
- `get_for_model()`, `get_for_id()`
- **Missing:** `GenericForeignKey`, `GenericRelation`, `get_object_for_this_type()`

### Messages (Partial)
- `Message` struct with levels (DEBUG, INFO, SUCCESS, WARNING, ERROR)
- `add_message()`, `get_messages()`, `info()`, `success()`, `warning()`, `error()`, `debug()`
- **Missing:** Storage backends (Cookie, Session, Fallback), template tags

### Sessions (Partial)
- `SessionBackend` trait, `SessionStore`, `InMemorySessionBackend`
- **Missing:** DatabaseSessionBackend, CacheSessionBackend, FileSessionBackend, CookieSessionBackend

### Checks Framework (Partial)
- `CheckMessage`, `CheckLevel` enum, `ChecksRegistry`
- `register()`, `run_all()`, `run_tagged()`

---

## ❌ Not Implemented

| Module | Django APIs | Notes |
|--------|------------|-------|
| `django.core.checks.security` | 60+ checks | HSTS, CSRF, session security checks |
| `django.core.checks.compatibility` | 5+ checks | Django 4.0+ compatibility |
| `django.core.checks.model_checks` | 25+ checks | Field name checks, index checks |
| `django.core.checks.urls` | 20+ checks | URL configuration validation |
| `django.core.servers.basehttp` | 30+ items | Built-in dev server (basic exists) |
| `django.core.files.uploadhandler` | 18 items | File upload processing |
| `django.core.files.locks` | 15 items | File locking utilities |
| `django.core.files.temp` | 5 items | Temp file utilities |

---

## 📊 API Surface by Submodule

| Submodule | Django APIs | Rjango | Coverage |
|-----------|------------|--------|----------|
| HTTP (request/response) | 12 | 7 | 58% |
| Signals | 4 | 3 | 75% |
| Exceptions | 10 | 8 | 80% |
| Validators | 14 | 8 | 57% |
| Paginator | 3 | 2 | 67% |
| Signing | 2 | 1 | 50% |
| Mail | 5 | 3 | 60% |
| Cache | 7 | 2 | 29% |
| Files/Storage | 6 | 3 | 50% |
| Serializers | 3 | 2 | 67% |
| Management/CLI | 27 | 13 | 48% |
| Checks | 10 | 4 | 40% |
| **Total Core** | **94** | **37** | **39%** |
| **Total Utils** | **33** | **14** | **42%** |
