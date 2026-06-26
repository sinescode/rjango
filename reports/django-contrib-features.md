# Django 6.0.6 Contrib Apps — Feature Comparison with Rjango

> **Django modules analyzed:** `django.contrib.admin`, `django.contrib.auth`, `django.contrib.messages`, `django.contrib.sessions`, `django.contrib.contenttypes`, `django.contrib.sites`, `django.contrib.sitemaps`, `django.contrib.flatpages`, `django.contrib.humanize`, `django.contrib.staticfiles`, `django.contrib.redirects`, `django.contrib.syndication`, `django.contrib.postgres`, `django.contrib.gis`
> **Last updated:** 2026-06-26

---

## Overview

| Metric | Value |
|--------|-------|
| Contrib features covered | 19/60 (32%) |
| 3rd party replacements | Tasks (new crate) |
| New since last update | CPS middleware, RemoteUser, better admin |

---

## ✅ Admin Site — 35%

### Implemented
- `AdminSite` with `register()`, `unregister()`, `is_registered()`, `get_registered_apps()`, `get_models()`, `dispatch()`, `index()`, `login()`, `logout()`, `password_change()`
- `ModelAdmin` with `list_display`, `list_filter`, `search_fields`, `list_editable`, `list_per_page`, `list_max_show_all`
- `InlineModelAdmin` with `TabularInline`, `StackedInline`, `InlineType` enum, `extra`, `max_num`, `min_num`, `can_delete`, `show_change_link`, `fields`, `readonly_fields`
- `AdminSite.register()` with model admin class override
- Admin URL registration per model (add, change, delete, history)

### Missing (critical)
| Feature | Priority |
|---------|----------|
| `list_filter` UI rendering (DateFieldListFilter, etc.) | 🔴 High |
| `search_fields` → ORM query | 🔴 High |
| Admin actions (delete_selected) | 🔴 High |
| Admin inlines save → FK assignment | 🔴 High |
| `LogEntry` audit trail | 🟡 Medium |
| Permission checks per model/action | 🟡 Medium |
| Admin templates (change_list, change_form, etc.) | 🟡 Medium |
| `date_hierarchy` drill-down | 🟡 Medium |
| `list_editable` form processing | 🟡 Medium |
| Admin CSS/JS media | 🟢 Low |
| Admin dashboard / index app list styling | 🟢 Low |
| Admin filtering by date | 🟢 Low |
| Admin pagination controls | 🟡 Medium |
| `get_queryset()` override in admin | 🟢 Low |
| `get_readonly_fields()` dynamic | 🟢 Low |
| `formfield_for_dbfield()` / `formfield_for_choice_field()` | 🟢 Low |
| `get_list_display()` dynamic | 🟢 Low |
| `get_list_filter()` dynamic | 🟢 Low |
| `get_search_results()` override | 🟢 Low |
| `get_actions()` custom actions | 🟡 Medium |

---

## ✅ Humanize — 100%

### Implemented (5/5)
| Function | Status |
|----------|--------|
| `ordinal` | ✅ |
| `intcomma` | ✅ |
| `intword` | ✅ |
| `naturaltime` | ✅ |
| `naturalday` | ✅ |

---

## ✅ Sitemaps — 60%

### Implemented
- `SitemapEntry` struct with `new()`, `lastmod()`, `changefreq()`, `priority()`
- `Sitemap` collection with `add()`, `render_xml()`, `render_index()`
- URL resolution for sitemaps

### Missing
- Sitemap class-based view (Django `django.contrib.sitemaps.views.index`, `.sitemap`)
- `Sitemap.__get_urls__()` method
- `ping_google()` for search engine notification
- Sitemap items queryset integration

---

## ✅ Flatpages — 60%

### Implemented
- `FlatPage` struct with content, url, template_name, title
- `register_flatpage()`, `get_flatpage()`, `list_flatpages()`, `clear_flatpages()`
- `render()` with template rendering

### Missing
- Flatpage middleware (fallback lookup)
- Flatpage admin integration
- Sites framework binding (SITE_ID)

---

## ✅ Syndication (RSS/Atom) — 60%

### Implemented
- `FeedItem` struct with title, link, description, content, author, pub_date, categories, unique_id
- `Feed` collection with `add()`, `render_rss()` (RSS 2.0 XML), `render_atom()` (Atom XML)
- Well-formed RSS/Atom XML generation

### Missing
- Feed class-based view (Django `django.contrib.syndication.views.Feed`)
- `Feed.item_link()`, `Feed.item_title()`, etc. methods
- `Feed.get_object()` with URL parameters
- Feed content negotiation

---

## ✅ Redirects — 40%

### Implemented
- `Redirect` struct with `old_path`, `new_path`, `site_id`
- `register_redirect()`, `resolve_redirect()`, `list_redirects()`, `clear_redirects()`

### Missing
- Redirect middleware (301/302 automatic redirects)
- Redirect admin integration
- Pattern matching support

---

## ✅ Staticfiles — 60%

### Implemented
- `StaticFilesFinder` trait, `StaticFilesConfig`
- `find()`, `list()`, `resolve_url()`, `url_prefix()`

### Missing
- `AppDirectoriesFinder` (auto-find in each app)
- `CachedStaticFilesStorage`, `ManifestStaticFilesStorage`
- Static file view for development
- `static()` template tag
- File hashing/versioning

---

## ✅ ContentTypes — Partial

### Implemented
- `ContentType` struct with app_label, model, name
- `ContentTypeManager` with `register()`, `get_for_model()`, `get_for_id()`

### Missing
- `ContentType.get_object_for_this_type()` (query by content type + pk)
- `GenericForeignKey` field type
- `GenericRelation` reverse relation
- Content types management command

---

## ✅ Sessions — Partial

### Implemented
- `SessionBackend` trait with `load()`, `save()`, `delete()`, `exists()`, `clear_expired()`
- `SessionStore` with session key generation and data management
- `InMemorySessionBackend`

### Missing
- `DatabaseSessionBackend` — store sessions in DB
- `CacheSessionBackend` — store sessions in cache
- `FileSessionBackend` — store sessions in files
- `CookieSessionBackend` — store sessions in signed cookies
- Session serialization formats (JSON vs Pickle)
- Session middleware with cookie-based key

---

## ✅ Messages — 60%

### Implemented
- `Message` struct with level, message, extra_tags
- `MessageLevel` enum: DEBUG, INFO, SUCCESS, WARNING, ERROR
- `add_message()`, `get_messages()`, individual level helpers

### Missing
- Message storage backends:
  - `CookieStorage` (messages in cookies)
  - `SessionStorage` (messages in session)
  - `FallbackStorage` (cookie with session fallback)
- Message template tags (`{% if messages %}`, etc.)
- Message level overrides per request

---

## ✅ Postgres Fields — 0%

### Not Implemented
| Field | Django Counterpart | Priority |
|-------|-------------------|----------|
| ArrayField | `django.contrib.postgres.fields.ArrayField` | 🟡 Medium |
| HStoreField | `django.contrib.postgres.fields.HStoreField` | 🟢 Low |
| RangeField | `django.contrib.postgres.fields.RangeField` | 🟢 Low |
| IntegerRangeField | `django.contrib.postgres.fields.IntegerRangeField` | 🟢 Low |
| DateRangeField | `django.contrib.postgres.fields.DateRangeField` | 🟢 Low |
| DateTimeRangeField | `django.contrib.postgres.fields.DateTimeRangeField` | 🟢 Low |

---

## ❌ Sites — 0%

### Not Implemented
| Feature | Django Counterpart | Notes |
|---------|-------------------|-------|
| Site model | `django.contrib.sites.models.Site` | Domain + name |
| `get_current()` | `django.contrib.sites.shortcuts.get_current_site` | Request-based site lookup |
| `SITE_ID` setting | Setting to identify current site | |

---

## ❌ GIS — 0%

### Not Implemented
| Feature | Django Counterpart | Notes |
|---------|-------------------|-------|
| Geometry types | `Point`, `LineString`, `Polygon`, `MultiPoint`, `MultiLineString`, `MultiPolygon`, `GeometryCollection` | Full GIS crate needed |
| Spatial fields | `GeometryField`, `PointField`, `LineStringField`, `PolygonField`, `MultiPolygonField` | |
| Spatial lookups | `dwithin`, `intersects`, `crosses`, `contains`, `within`, `overlaps` | |
| Spatial functions | `Distance`, `Length`, `Area`, `Union`, `Intersection` | |
| SRS support | SRID, coordinate system transforms | |

---

## 📦 Tasks Crate — New (not in Django core contrib)

Rjango has a new `rjango-tasks` crate that provides Django 6.0's task framework:

| Feature | Status |
|---------|--------|
| Task struct with id/name/args/status/result | ✅ |
| TaskStatus enum (Queued/Running/Completed/Failed/Cancelled) | ✅ |
| TaskResult (success/failure with output & error) | ✅ |
| TaskDef with name, function, description | ✅ |
| TaskRegistry with register/get/contains/all | ✅ |
| `global_registry()` singleton | ✅ |
| `enqueue()` convenience function | ✅ |
| `register_task!` macro | ✅ |
| `TaskHandle` for status polling & cancellation | ✅ |
| `TaskQueue` trait | ✅ |
| `MemoryTaskQueue` backend | ✅ |
| `Worker` with `process_one()`, `run()`, `stop()` | ✅ |
| 23 tests | ✅ |

---

## 📊 Coverage Summary

| Subapp | Django APIs | Rjango | % | Priority |
|--------|------------|--------|---|----------|
| Admin | 20 | 3 | **15%** | Critical |
| Humanize | 5 | 5 | **100%** | Complete |
| Sitemaps | 3 | 2 | **67%** | Medium |
| Flatpages | 3 | 1 | **33%** | Low |
| Syndication | 3 | 2 | **67%** | Low |
| Messages | 5 | 3 | **60%** | Medium |
| Sessions | 5 | 2 | **40%** | Medium |
| Staticfiles | 3 | 2 | **67%** | Low |
| Redirects | 3 | 1 | **33%** | Low |
| ContentTypes | 3 | 2 | **67%** | High |
| Postgres Fields | 4 | 0 | **0%** | Low |
| GIS | 3 | 0 | **0%** | Low |
| **Total Contrib** | **60** | **19** | **32%** | |
