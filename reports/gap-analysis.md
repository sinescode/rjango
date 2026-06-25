# 🦀 Rjango — Django 6.0.6 Feature Gap Analysis

> **Generated:** 2026-06-25  
> **Django version analyzed:** 6.0.6 (679 source files across ~30 packages)  
> **Rjango version:** Rust port, 16 crates, 98 source files, 17,414 lines  
> **Current tests:** 763 passing

---

## 📊 Overall Feature Coverage

| Category | Django APIs | Rjango ✅ | Rjango ⚠️ | Rjango ❌ | Coverage % |
|----------|------------|-----------|-----------|-----------|-----------|
| **URL Routing** | 25 | 12 | 2 | 11 | **48%** |
| **View System** | 45 | 3 | 8 | 34 | **6%** |
| **Template Engine** | 50 | 40 | 4 | 6 | **80%** |
| **Template Filters** | 60 | 44 | 0 | 16 | **73%** |
| **Template Tags** | 28 | 20 | 0 | 8 | **71%** |
| **Core (Exceptions, Paginator, etc.)** | 80 | 20 | 10 | 50 | **25%** |
| **HTTP** | 25 | 8 | 4 | 13 | **32%** |
| **Shortcuts** | 10 | 3 | 4 | 3 | **30%** |
| **Settings** | 25 | 2 | 5 | 18 | **8%** |
| **Signals/Dispatch** | 8 | 3 | 3 | 2 | **37%** |
| **Utils** | 60 | 8 | 5 | 47 | **13%** |
| **Validators** | 16 | 4 | 0 | 12 | **25%** |
| **Auth** | 60 | 12 | 14 | 34 | **20%** |
| **Forms** | 75 | 10 | 12 | 53 | **13%** |
| **Widgets** | 30 | 12 | 4 | 14 | **40%** |
| **Middleware** | 20 | 6 | 4 | 10 | **30%** |
| **Sessions** | 15 | 2 | 2 | 11 | **13%** |
| **Messages** | 18 | 6 | 6 | 6 | **33%** |
| **ORM Models** | 45 | 6 | 8 | 31 | **13%** |
| **ORM Fields** | 50 | 12 | 4 | 34 | **24%** |
| **ORM QuerySet** | 40 | 2 | 2 | 36 | **5%** |
| **ORM Lookups** | 22 | 0 | 1 | 21 | **0%** |
| **ORM Aggregates** | 12 | 5 | 2 | 5 | **42%** |
| **ORM Expressions** | 15 | 1 | 1 | 13 | **7%** |
| **DB Backends** | 15 | 3 | 0 | 12 | **20%** |
| **Migrations** | 30 | 6 | 4 | 20 | **20%** |
| **Admin** | 40 | 6 | 0 | 34 | **15%** |
| **Test Framework** | 30 | 3 | 3 | 24 | **10%** |
| **Management Commands** | 30 | 13 | 0 | 17 | **43%** |
| **Contrib (Postgres, GIS, etc.)** | 80 | 0 | 0 | 80 | **0%** |
| **Caching** | 20 | 0 | 0 | 20 | **0%** |
| **Mail** | 15 | 0 | 0 | 15 | **0%** |
| **Files/Storage** | 15 | 2 | 1 | 12 | **13%** |
| **i18n/L10n** | 20 | 1 | 0 | 19 | **5%** |
| **Serializers** | 10 | 1 | 1 | 8 | **10%** |
| **Signing** | 18 | 0 | 0 | 18 | **0%** |
| **Total (approx.)** | **~1,100+** | **~250** | **~115** | **~735** | **~22%** |

---

## 🔥 Top Priority Gaps (Must Fix)

### 1. `reverse()` URL Resolution
**Critical for templates, views, admin, redirects, tests** — without it, `redirect()` can't resolve named URLs, templates can't use `{% url %}`, admin can't link internally.

**Files affected:** `rjango-urls/src/base.rs`
```
Django:  reverse(viewname, args=None, kwargs=None) → str
Rjango:  ❌ Missing entirely
```
**Effort:** Medium. Implement existing `UrlPattern` lookup by name + args/kwargs serialization.

### 2. Real Database CRUD Operations
**QuerySet can construct queries but can't execute them.** No `get()`, `create()`, `update()`, `delete()`, `save()`, `exists()`.

**Files affected:** `rjango-orm/src/models.rs`, `rjango-orm/src/backend.rs`
```
Django:  queryset.get(pk=1)  →  Model instance
Django:  queryset.create(**kwargs)  →  Model instance
Django:  queryset.filter(...).update(**kwargs)  →  int
Django:  model.save()  →  void
Rjango:  ❌ None of these execute SQL
```
**Effort:** High. Need `sqlx` integration with real query execution, result mapping, transaction support, connection pooling.

### 3. Full Lookups (gt, gte, lt, lte, in, contains, etc.)
**Only basic `filter()` with exact match exists.** Django has 22+ lookup types.

**Files affected:** `rjango-orm/src/lib.rs`
```
Django:  MyModel.objects.filter(age__gt=18, name__contains="foo")
Django:  MyModel.objects.filter(id__in=[1,2,3])
Rjango:  ❌ Only exact equality
```
**Effort:** Medium. Lookups are well-defined; need SQL generation for each.

### 4. Missing Field Types
**15+ field types not implemented**, including: Decimal, UUID, Duration, Email, URL, Slug, IPAddress, FilePath, Binary, Big/Small/Positive integers, Time.

**Files affected:** `rjango-orm/src/fields.rs`
```
Django has 35 field types. Rjango has ~15 core types.
```
**Effort:** Low. Each is a new enum variant + validation + SQL mapping.

### 5. Caching Framework
**Zero cache support.** Django has 7 backends (locmem, db, file, memcached, redis, dummy, redis).

**Files affected:** Missing module (`rjango-core/src/cache.rs` suggestion)
**Effort:** Medium. Implement `BaseCache` trait + `LocMemCache` (Rust HashMap) + `RedisCache`.

### 6. Setting Coverage
**Of Django's 50+ settings, Rjango has only ~10.**

**Files affected:** `rjango-conf/src/lib.rs`, `rjango-core/src/settings.rs`
```
Critical missing: TEMPLATES, MIDDLEWARE, AUTH_USER_MODEL, STATIC_URL/MEDIA_URL,
LANGUAGE_CODE, TIME_ZONE, USE_I18N, SESSION_*, CSRF_*, LOGIN_URL, etc.
```
**Effort:** Low-Medium. Add field by field.

---

## ⚡ Medium Priority Gaps

### 7. `{% url %}` Template Tag
Dependency on `reverse()`. Can't link between pages in templates.
```
{% url 'profile' user.id %}
```
**Effort:** Low (once reverse() exists).

### 8. `{% include %}` Template Tag
Include other templates. Rjango can parse blocks/extends but not include.
```
{% include "header.html" %}
```
**Effort:** Low.

### 9. `ModelForm` + `ModelFormMixin`
Forms don't save to DB. CreateView/UpdateView can't persist data.
**Effort:** Medium. Requires DB CRUD first.

### 10. Password Validation System
Django validates passwords against 4 rules (min length, similarity to user, common passwords, numeric-only). Rjango: ❌.
**Effort:** Low. Straightforward validation functions.

### 11. Admin Inlines + Filters + Actions
Admin only has basic CRUD. Missing StackedInline, TabularInline, list_filter, search_fields, actions.
**Effort:** Medium.

### 12. `assertContains` / `assertTemplateUsed` / Test Assertions
Test client is basic. Missing Django's rich assertion helpers.
**Effort:** Low.

### 13. Formsets
Multiple forms on same page (e.g., multiple model instances). Django's formset_factory.
**Effort:** Medium.

---

## 🐢 Low Priority Gaps

### 14. Date-Based Generic Views
ArchiveIndexView, YearArchiveView, MonthArchiveView, etc. — 12 views in Django, 0 in Rjango.

### 15. Contrib: Humanize, Sitemaps, Syndication, Flatpages, Redirects
Each is small and self-contained but not core.

### 16. i18n/L10n
Translation utilities: `ngettext`, `pgettext`, `activate`, `override`, locale middleware, translation template tags.

### 17. Signing Framework
`django.core.signing` — Signer, TimestampSigner, dumps/loads. Used by signed cookies.

### 18. WSGI/ASGI Handler Integration
Django's BaseHandler, WSGIHandler, ASGIHandler. Rjango has basic server but doesn't follow the handler contract.

### 19. PostgreSQL Contrib
ArrayField, HStoreField, SearchVector, TrigramSimilarity — significant effort but not critical for MVP.

### 20. GIS
Entire GeoDjango module — very large, very niche.

---

## 📈 Priority Matrix

```
                    High Value
                    ┌─────────────────────────────────────┐
                    │  reverse()        DB CRUD           │
                    │  Lookups          Missing fields    │
                    │  Settings         Caching           │
                    │                                     │
  Low Effort ◄─────┼─────────────────────────────────────┤─────► High Effort
                    │  Password valid.  Formsets          │
                    │  {% url %} tag    ModelForm         │
                    │  {% include %}    Admin inlines     │
                    │  Assertions       Postgres contrib  │
                    │                   GIS               │
                    └─────────────────────────────────────┘
                    Low Value
```

---

## 🎯 Quick Wins (Low Effort, High Value)

| # | Feature | Est. Files | Est. Lines | Value |
|---|---------|-----------|-----------|-------|
| 1 | Add DecimalField | 2 | 50 | High |
| 2 | Add UUIDField | 2 | 40 | High |
| 3 | Add EmailField/URLField/SlugField | 3 | 100 | High |
| 4 | Add TimeField | 2 | 40 | Medium |
| 5 | Password validation | 1 | 100 | Medium |
| 6 | assertContains/assertTemplateUsed | 2 | 80 | Medium |
| 7 | `{% url %}` template tag | 2 | 60 | High |
| 8 | `{% include %}` template tag | 1 | 30 | High |
| 9 | Add missing settings (MIDDLEWARE, TEMPLATES, etc.) | 1 | 100 | High |
| 10 | MultiValueDict | 1 | 50 | Medium |

---

## 📁 Reports Directory

| Report | Path | Content |
|--------|------|---------|
| **Core & Utils** | `reports/django-core-features.md` | Paginator, exceptions, validators, signing, cache, files, handlers, serializers, apps, conf, dispatch, utils, shortcuts |
| **ORM** | `reports/django-orm-features.md` | Model system, fields, QuerySet, lookups, aggregates, expressions, functions, constraints, managers, backends, migrations |
| **Templates** | `reports/django-templates-features.md` | Engine, context, loaders, 60+ filters, 28+ tags, library registration |
| **Forms, Auth, Middleware** | `reports/django-forms-auth-middleware-features.md` | Form fields/widgets/validation, auth system, middleware stack, sessions |
| **URLs, Views, Test** | `reports/django-urls-views-test-features.md` | URL routing, generic views, test client/runner, management commands |
| **Contrib** | `reports/django-contrib-features.md` | Admin, contenttypes, postgres, staticfiles, humanize, sitemaps, flatpages, redirects |
| **Master Report** | `REPORT.md` | Overall project stats, per-crate breakdown, feature summary |
