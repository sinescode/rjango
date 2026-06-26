# 🦀 Rjango — Django 6.0.6 Feature Gap Analysis

> **Generated:** 2026-06-26  
> **Current test count:** 1,784 passing — 0 failing  
> **Overall coverage:** 41.1%  

---

## 🔴 Critical Gaps (blocking real-world use)

### 1. Real ORM CRUD — get()/create()/update()/delete()
- **Files:** `rjango-orm/src/query.rs`, `rjango-orm/src/executor.rs`, `rjango-orm/src/models.rs`
- **Status:** QuerySet struct exists with filter/exclude/order_by builder methods but `get()`, `create()`, `update()`, `delete()` return `Option<QueryResult>` without actually executing SQL
- **Fix needed:** Wire QuerySet methods → SQL generation → executor.execute()
- **Depends on:** Lookup enum → SQL WHERE clause generation

### 2. Full ORM Lookup Support
- **Files:** `rjango-orm/src/lookups.rs`
- **Status:** `Lookup` enum exists with variants (exact, iexact, contains, etc.) and `FilterCondition` with `parse()`, `sql_operator()`, `format_value()`, `sql_snippet()` — but not wired into QuerySet SQL generation
- **Fix needed:** Connect FilterCondition → QuerySet WHERE clause building → executor SQL

### 3. Model.save() / delete()
- **Files:** `rjango-orm/src/models.rs`
- **Status:** `Model` trait exists with `table_name()`, `field_definitions()`, `from_row()` — but no `save()`, `delete()`, `clean()`, `full_clean()` methods
- **Fix needed:** Add INSERT/UPDATE/DELETE SQL methods to Model trait

### 4. Cache Backend Trait
- **Files:** `rjango-core/src/cache.rs`
- **Status:** Free functions `cache_set()`, `cache_get()`, `cache_delete()`, `cache_clear()` exist — but no `CacheBackend` trait, no backend implementations beyond locmem, no key prefixing/versioning
- **Fix needed:** `CacheBackend` trait + LocMemCache/FileBasedCache/DatabaseCache/DummyCache implementations

### 5. DB-backed Sessions
- **Files:** `rjango-core/src/sessions.rs`
- **Status:** `SessionBackend` trait + `InMemorySessionBackend` exist — missing DatabaseSessionBackend, CacheSessionBackend, FileSessionBackend, CookieSessionBackend
- **Fix needed:** At minimum DatabaseSessionBackend using the ORM/existing DB connection

---

## 🟡 High-Priority Gaps

### 1. Admin list_filter / search / actions
- **Files:** `rjango-contrib/src/admin.rs`, `rjango-admin/src/lib.rs`
- **Status:** `ModelAdmin` exists with `list_display`, `search_fields`, `list_filter` properties — but missing filter rendering, search query execution, action framework
- **Fix needed:** Wire search_fields → ORM query, list_filter → filter UI, actions → POST handling

### 2. Admin Inlines save flow
- **Files:** `rjango-contrib/src/admin.rs`
- **Status:** `InlineModelAdmin` struct exists with `TabularInline`/`StackedInline` — but inline forms don't save related objects
- **Fix needed:** Inline save → FK assignment → parent/child save flow

### 3. Form per-field clean hooks
- **Files:** `rjango-forms/src/fields.rs`, `rjango-forms/src/lib.rs`
- **Status:** `Form` has `full_clean()` which collects errors — but `clean_<fieldname>()` convention not implemented
- **Fix needed:** `Form::_clean_fields()` that calls field.clean(), then `Form::_clean_form()`, then `Form::_post_clean()`

### 4. CreateView / UpdateView / DeleteView
- **Files:** `rjango-views/src/lib.rs`
- **Status:** `View` trait exists, `TemplateView`, `RedirectView`, `DetailView` exist — but no CreateView, UpdateView, DeleteView
- **Fix needed:** ModelFormMixin → form.save() → redirect; DeleteView → object.delete() → redirect

### 5. Date-based archive views (14 classes)
- **Files:** `rjango-views/src/lib.rs`
- **Status:** Not implemented at all
- **Fix needed:** YearMixin, MonthMixin, WeekMixin, DayMixin → ArchiveIndexView, YearArchiveView, MonthArchiveView, WeekArchiveView, DayArchiveView, TodayArchiveView, DateDetailView

### 6. ORM remaining field types
- **Files:** `rjango-orm/src/fields.rs`
- **Status:** 10 types implemented. Missing: DurationField, BigIntegerField, PositiveBigIntegerField, SmallIntegerField, AutoField, BigAutoField, SmallAutoField, BinaryField, JSONField, FileField, ImageField
- **Fix needed:** Add each as SimpleField subclass or new struct

### 7. Full Mail Backends
- **Files:** `rjango-core/src/mail.rs`
- **Status:** `EmailMessage`, `EmailMultiAlternatives`, `send_mail()`, `send_mass_mail()`, `mail_admins()`, `mail_managers()` exist — but only MemoryBackend. Missing SMTP, Console, FileBased
- **Fix needed:** `SmtpEmailBackend`, `ConsoleEmailBackend`, `FileBasedEmailBackend`

---

## 🟠 Medium-Priority Gaps

### 1. Migration executor (real SQL)
- `AddIndex`, `RemoveIndex`, `RunSQL`, `RunPython` operations
- Migration dependency graph resolution
- Migration state tracking / recorder table

### 2. Full Paginator API
- `Page::elided_page_range()` with ELLIPSIS markers
- `orphans` parameter
- `allow_empty_first_page` handling
- `AsyncPaginator`

### 3. Full Validators API
- Missing: `DecimalValidator`, `FileExtensionValidator`, `ProhibitNullCharactersValidator`, `StepValueValidator`, `MaxValueValidator`, `MinValueValidator`
- `validate_ipv4_address()`, `validate_ipv6_address()`, `validate_ipv46_address()`
- `validate_integer()`, `validate_unicode_slug()`

### 4. i18n .po/.mo parsing
- `MessageCatalog` with actual file parsing
- `{% trans %}`, `{% blocktrans %}`, `{% plural %}` template tags
- Plural form handling
- Locale path discovery

### 5. RequestFactory (test)
- Build `Request` objects for testing
- `Client.login()` / `Client.force_login()`
- `override_settings` / `modify_settings` decorators

### 6. View decorators
- `require_POST`, `require_GET`, `require_http_methods` (exist but untested)
- `never_cache` (exists)
- `csrf_exempt` (exists in auth)
- `cache_page`, `cache_control`

### 7. Template tags
- `{% lorem %}`, `{% templatetag %}` (existed previously?)
- `{% resetcycle %}`
- `{% partialdef %}`, `{% partial %}` (Django 6.0 feature)

### 8. Full `reverse()` with namespaces
- `reverse()` works for simple views but app_name/namespace resolution needs testing
- `reverse_lazy()` for deferred URL resolution

### 9. CLI missing commands
- `dumpdata`, `loaddata`, `inspectdb`, `sqlmigrate`, `flush`, `compilemessages`, `makemessages`, `squashmigrations`, `optimizemigration`, `sendtestemail`, `testserver`, `createcachetable`, `diffsettings`

---

## 🔵 Low-Priority Gaps

### 1. Postgres-specific fields
- `ArrayField`, `HStoreField`, `RangeField`, `IntegerRangeField`, `DateRangeField`, `DateTimeRangeField`
- Requires `rjango-contrib/postgres` crate

### 2. GIS module
- Geometry types (Point, LineString, Polygon, Multi*)
- Spatial lookups (dwithin, intersects, crosses, etc.)
- Requires `rjango-contrib/gis` crate

### 3. Sites framework
- `Site` model, `get_current()`, `SITE_ID` setting

### 4. Sitemaps framework
- Only basic Sitemap/Entry structs exist — missing `Sitemap` class-based view, ping_google, sitemap index

### 5. Syndication (RSS/Atom)
- Only basic Feed/FeedItem structs exist — missing `Feed` class-based view, `syndication.add_items()`

### 6. Cookie/secure cookie handling
- `set_signed_cookie()`, `get_signed_cookie()`
- Cookie signing utilities

### 7. Multipart parsing
- Form data with file upload parsing

### 8. ASGI/WSGI handler full implementation
- `BaseHandler` with `get_response()` chain
- Exception handling middleware
- `LimitedStream`

---

## 📊 Gap Count by Priority

| Priority | Count | Est. Work |
|----------|-------|-----------|
| 🔴 Critical | 5 | ~80h |
| 🟡 High | 8 | ~100h |
| 🟠 Medium | 8 | ~60h |
| 🔵 Low | 8 | ~40h |
| **Total** | **29** | **~280h** |

---

## ✅ No Longer Gaps (implemented since last report)

- Middleware: All 14 types ✅
- HTTP: StreamingHttpResponse, FileResponse ✅
- Templates: CachedLoader ✅
- Templates: dictsortreversed filter ✅
- Tasks: Full Tasks crate ✅
- Forms: FilePathField, GenericIPAddressField ✅
- Forms: MultipleHiddenInput widget ✅
- Forms: modelformset_factory, inlineformset_factory ✅
- HTTP: Request.is_ajax(), is_secure(), get_host(), build_absolute_uri() ✅
- Auth: PermissionMixin, ContentType ✅
- Middleware: Cache middleware (UpdateCache, FetchFromCache) ✅
- Middleware: RemoteUserMiddleware, RemoteUserBackend ✅
- Middleware: CSP (ContentSecurityPolicyMiddleware) ✅
