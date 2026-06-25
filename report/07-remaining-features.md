# Remaining Django Features: Exhaustive Comparison Report

**Date**: 2026-06-25 (Updated)  
**Django Version**: 6.0.6  
**Rjango Version**: 0.1.0  

---

## django.dispatch vs rjango-dispatch

Rjango Location: `rjango-dispatch/src/lib.rs` (105 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| Signal class | Event pub/sub | `Signal` wrapper | ✅ YES | Re-exports from rjango-core |
| connect() | Register handler | Method on Signal | ✅ YES | |
| disconnect() | Unregister | Method on Signal | ✅ YES | |
| send() | Sync dispatch | Method on Signal | ✅ YES | |

---

## django.conf vs rjango-conf

Rjango Location: `rjango-conf/src/lib.rs` (37 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| settings object | Global settings | `settings` accessor | ✅ YES | Thread-safe singleton |
| Lazy settings | Lazy loading | — | ❌ NO | |
| User settings override | Override | — | ❌ NO | |

---

## django.utils vs rjango-utils

Rjango Location: `rjango-utils/src/` (685 lines)

### crypto

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| get_random_string() | Random string | `get_random_secret_key()` | ✅ YES | 50-char hex key |
| constant_time_compare() | Timing safe | — | ❌ NO | Security issue |
| pbkdf2() | Key derivation | — | ❌ NO | |
| md5_hash() | Hash | — | ❌ NO | |

### safestring / html

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| SafeString | Mark safe | `SafeString` struct | ✅ YES | |
| SafeText | Unicode safe | `SafeText` alias | ✅ YES | |
| mark_safe() | Mark safe fn | `mark_safe()` | ✅ YES | |
| conditional_escape() | Escape only if not safe | `conditional_escape()` | ✅ YES | |
| escape() | HTML escape | In SafeString | ✅ YES | |
| format_html() | Safe format | — | ❌ NO | |
| strip_tags() | HTML strip | — | ❌ NO | |

### i18n

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| gettext() | Translation | `gettext()` | ✅ YES | |
| gettext_lazy() | Lazy translation | `gettext_lazy()` | ✅ YES | |
| LazyString | Lazily evaluated | `LazyString` struct | ✅ YES | |
| pgettext() | Contextual translation | — | ❌ NO | |
| ngettext() | Plural forms | — | ❌ NO | |
| activate() | Language switch | — | ❌ NO | |
| deactivate() | Unset active lang | — | ❌ NO | |
| get_language() | Current lang | — | ❌ NO | |
| Locale middleware | Lang detection | — | ❌ NO | |

### text

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| slugify() | Slug creation | `slugify()` | ✅ YES | |
| Truncator | Text truncation | — | ❌ NO | |
| Phone number format | — | — | ❌ NO | |

### functional

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| lazy() | Lazy callable | `lazy()` function | ✅ YES | |
| classproperty | Class prop | — | ❌ NO | |
| keep_lazy() | Lazy decorator | — | ❌ NO | |
| partition() | Split | — | ❌ NO | |
| curry() | Currying | — | ❌ NO | |

### http

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| urlencode() | URL encoding | `urlencode()` | ✅ YES | |
| urlsafe_base64 | Base64 encode | — | ❌ NO | |

---

## django.views vs rjango-views

Rjango Location: `rjango-views/src/lib.rs` (361 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| View (base class) | Base view | `View` trait | ✅ YES | |
| TemplateView | Render template | `TemplateView` struct | ✅ YES | |
| RedirectView | Redirect | `RedirectView` struct | ✅ YES | |
| ListView | Object list | `ListView` struct | ✅ YES | |
| DetailView | Object detail | `DetailView` struct | ✅ YES | |
| FormView | Form handler | `FormView` struct | ✅ YES | |
| CreateView | Object create | `CreateView` struct | ✅ YES | |
| UpdateView | Object update | `UpdateView` struct | ✅ YES | |
| DeleteView | Object delete | `DeleteView` struct | ✅ YES | |
| ArchiveIndexView | Date archive | — | ❌ NO | |
| YearArchiveView | Year archive | — | ❌ NO | |
| MonthArchiveView | Month archive | — | ❌ NO | |
| WeekArchiveView | Week archive | — | ❌ NO | |
| DayArchiveView | Day archive | — | ❌ NO | |
| TodayArchiveView | Today archive | — | ❌ NO | |
| DateDetailView | Date detail | — | ❌ NO | |
| decorators (login_required) | View decorators | — | ❌ NO | |
| csrf_exempt | Exempt CSRF | — | ❌ NO | |
| **Tests** | — | Implicit | ⚠️ | |

---

## django.shortcuts

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| render() | Render template | TemplateEngine::render_to_string | ⚠️ PARTIAL | |
| redirect() | Redirect | Response::redirect() | ✅ YES | |
| get_object_or_404() | DB lookup or 404 | — | ❌ NO | |
| get_list_or_404() | Query or 404 | — | ❌ NO | |
| resolve_url() | URL resolution | — | ❌ NO | |

---

## django.contrib Sessions

Rjango Location: `rjango-core/src/sessions.rs` (186 lines)

| Feature | Status | Notes |
|---------|--------|-------|
| SessionStore (file-based) | ✅ | File persistence in storage_dir |
| load() | ✅ | Load session by key |
| save() | ✅ | Persist session data |
| delete() | ✅ | Delete session file |
| exists() | ✅ | Check if session exists |
| clear_expired() | ✅ | Remove expired sessions |
| generate_key() | ✅ | Random session key generation |
| SessionMiddleware integration | ✅ | In rjango-middleware |
| Database-backed sessions | ❌ | |
| Cache-backed sessions | ❌ | |
| Cookie-backed sessions | ❌ | |
| Signed cookie sessions | ❌ | |

---

## django.contrib Messages

Rjango Location: `rjango-core/src/messages.rs` (171 lines)

| Feature | Status | Notes |
|---------|--------|-------|
| MessageLevel enum | ✅ | DEBUG, INFO, SUCCESS, WARNING, ERROR |
| Message struct | ✅ | level + message |
| add_message() | ✅ | Add to session store |
| get_messages() | ✅ | Retrieve from session |
| info() / success() / warning() / error() / debug() | ✅ | Convenience functions |
| Session-based storage | ✅ | Stored under `_messages` key |
| MessageMiddleware | ✅ | In rjango-middleware |
| FallbackStorage | ❌ | Cookie fallback |
| CookieStorage | ❌ | Cookie-only |

---

## django.contrib ContentTypes

Rjango Location: `rjango-core/src/contenttypes.rs` (164 lines)

| Feature | Status | Notes |
|---------|--------|-------|
| ContentType struct | ✅ | id, app_label, model fields |
| ContentTypeManager | ✅ | Registry-based management |
| register_content_type() | ✅ | Register model type |
| lookup_content_type() | ✅ | Lookup by app_label + model |
| get_for_model() | ✅ | Get by model |
| get_for_id() | ✅ | Get by ID |
| Permissions integration | ❌ | |
| Admin integration | ❌ | |

---

## django.contrib StaticFiles

Rjango Location: `rjango-core/src/staticfiles.rs` (228 lines)

| Feature | Status | Notes |
|---------|--------|-------|
| StaticFinder trait | ✅ | find() method |
| FileSystemFinder | ✅ | Search in directories |
| StaticFilesConfig | ✅ | With defaults |
| find_static_file() | ✅ | Find in all finders |
| Server static serving | ✅ | App::with_static_dir() |
| AppDirectoriesFinder | ❌ | |
| CachedStaticFilesStorage | ❌ | |
| ManifestStaticFilesStorage | ❌ | |

---

## django.contrib Postgres

| Feature | Status |
|---------|--------|
| PostgreSQL-specific fields | ❌ |
| SearchVector/Query/Rank | ❌ |
| ArrayField | ❌ |
| JSONField | ❌ |
| HStoreField | ❌ |
| Range fields | ❌ |
| Trigram similarity | ❌ |
| Unaccent | ❌ |

---

## django.contrib GIS

| Feature | Status |
|---------|--------|
| GeoDjango models | ❌ |
| Spatialite backend | ❌ |
| PostGIS backend | ❌ |
| GeoJSON serialization | ❌ |
| KML generation | ❌ |
| Distance/origin queries | ❌ |
| Map widgets (OpenLayers) | ❌ |
| GDAL/OGR integration | ❌ |
| GEOS geometry | ❌ |

---

## Other Missing contrib Packages

| Feature | Django Module | Status |
|---------|-------------|--------|
| Syndication feeds (RSS/Atom) | django.contrib.syndication | ❌ |
| Sitemaps (SEO) | django.contrib.sitemaps | ❌ |
| Flatpages (static content) | django.contrib.flatpages | ❌ |
| Redirects (URL forwarding) | django.contrib.redirects | ❌ |
| Humanize (data formatting) | django.contrib.humanize | ❌ |
| Sites (multi-site framework) | django.contrib.sites | ❌ |
| Admin documentation | django.contrib.admindocs | ❌ |
| Cross-origin (CORS) | — | ❌ |

---

## Django Core Missing

| Feature | Django Module | Status |
|---------|-------------|--------|
| Signing (Signer, TimestampSigner) | django.core.signing | ❌ |
| File upload handlers | django.core.files.uploadhandler | ❌ |
| Servers (WSGI, ASGI) | django.core.servers | ❌ |
| Cached template loader | django.template.loaders.cached | ❌ |
| Jinja2 template backend | django.template.backends.jinja2 | ❌ |
| Autoreload (dev server) | django.utils.autoreload | ❌ |

---

## Key Security Gaps

| Issue | Severity | Current | Needed |
|-------|----------|---------|--------|
| Password hashing | HIGH | DefaultHasher (insecure) | PBKDF2/Argon2 |
| Constant-time comparison | HIGH | == operator | constant_time_compare() |
| Session signing | MEDIUM | Plain files | HMAC-signed |
| HTTPS redirect | MEDIUM | Not enforced | HSTS + redirect |
| CSP headers | MEDIUM | None | Content-Security-Policy |
| Clickjacking protection | LOW | Via SecurityMiddleware | Full X-Frame-Options |
