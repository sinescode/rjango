# Rjango vs Django 6.0.6 — Complete Feature Comparison

> **Generated:** 2026-06-25  
> **Django 6.0.6 analyzed:** 679 source files across all modules  
> **Django API surface analyzed:** urls, views, templates, ORM, forms, auth, admin, middleware, contrib (all), test, core, dispatch, signals, settings, cache, files, serializers, validators, signing

---

## 📊 Master Scoreboard

| Category | Django APIs | Rjango Coverage | Gap % |
|---|---|---|---|
| **URL Dispatcher** | 22 APIs | 70% | 30% |
| **Views (generic)** | 68 APIs | 15% | 85% |
| **Templates (tags)** | 23 tags | 52% | 48% |
| **Templates (filters)** | 53 filters | 83% | 17% |
| **Template Engine** | 12 APIs | 75% | 25% |
| **Core (paginator, exceptions, signals)** | 55 APIs | 45% | 55% |
| **ORM (models, fields, QuerySets, relations)** | 150+ APIs | 25% | 75% |
| **Forms** | 65 APIs | 50% | 50% |
| **Auth** | 60 APIs | 40% | 60% |
| **Middleware** | 16 APIs | 40% | 60% |
| **Admin** | 55 APIs | 25% | 75% |
| **Staticfiles** | 10 APIs | 10% | 90% |
| **Messages** | 22 APIs | 25% | 75% |
| **Sessions** | 28 APIs | 15% | 85% |
| **Cache Framework** | 15 APIs | 0% | 100% |
| **Migrations** | 30 APIs | 20% | 80% |
| **CLI Commands** | 27 commands | 48% | 52% |
| **Contrib (postgres, gis, sites, sitemaps, etc.)** | 100+ APIs | 0% | 100% |
| **Test Utilities** | 30 APIs | 30% | 70% |
| **Utils (crypto, text, i18n, etc.)** | 80+ APIs | 25% | 75% |
| **Signals & Dispatch** | 15 APIs | 80% | 20% |
| **i18n / L10n** | 20 APIs | 10% | 90% |

> **Overall (estimated): ~25-30% of Django 6.0.6's total API surface** is covered by Rjango.

---

## ✅ Fully Implemented Features

| Feature | Rjango | Django |
|---|---|---|
| URL path converters (int, str, slug, uuid) | ✅ | ✅ |
| URL resolution / reversal | ✅ | ✅ |
| `include()` for URL configs | ✅ | ✅ |
| Template engine with tokenizer/parser | ✅ | ✅ |
| Template variable resolution | ✅ | ✅ |
| Auto-escaping system | ✅ | ✅ |
| Template inheritance (extends/block) | ✅ | ✅ |
| Template loaders (FS, App, Test) | ✅ | ✅ |
| **44 of 53 template filters** | ✅ (83%) | ✅ |
| Signal connect/send/disconnect | ✅ | ✅ |
| Basic Settings | ✅ | ✅ |
| Request / Response | ✅ | ✅ |
| QueryDict (basic) | ✅ | ✅ |
| SafeString / mark_safe | ✅ | ✅ |
| slugify, escape, phone2numeric | ✅ | ✅ |
| Shortcuts: render, redirect, get_object_or_404 | ✅ | ✅ |
| Auth decorators (4) | ✅ | ✅ |
| Auth middleware (basic) | ✅ | ✅ |
| Login/Logout views | ✅ | ✅ |
| User model with permissions | ✅ | ✅ |
| Password hashing (PBKDF2, BCrypt) | ✅ | ✅ |
| CSRF middleware | ✅ | ✅ |
| Security middleware | ✅ | ✅ |
| Session middleware | ✅ | ✅ |
| Message middleware | ✅ | ✅ |
| Basic CLI commands (13 of 27) | ✅ | ✅ |
| Basic migration operations (6 of 20) | ✅ | ✅ |
| Basic field types (10 of 25) | ✅ | ✅ |
| Test client (GET/POST) | ✅ | ✅ |
| Test runner | ✅ | ✅ |
| Generic views (basic structs) | ✅ | ✅ |

---

## ⚠️ Partially Implemented Features

| Feature | What's Missing |
|---|---|
| **Paginator** | Missing `Page` object, `orphans` param, `allow_empty_first_page`, `get_elided_page_range()`, `AsyncPaginator`, error handling |
| **ValidationError** | Missing `code`, `params`, `error_dict`, `message_dict`, `messages` properties |
| **Validators** | Missing: MinValueValidator, MaxValueValidator, StepValueValidator, DecimalValidator, FileExtensionValidator, ProhibitNullCharactersValidator, validate_integer, validate_ipv4/6_address |
| **Shortcuts** | Missing `redirect(to)` resolving URLs/models, missing `aget_object_or_404`, `aget_list_or_404` |
| **App registry** | Missing `populate()`, `get_model()`, `get_models()`, `clear_cache()`, `AppConfig.ready()` hook |
| **Settings** | Missing 50+ Django settings (MIDDLEWARE, TEMPLATES, STATIC_URL, AUTH_USER_MODEL, etc.) |
| **Forms** | Missing: DecimalField, DateField, TimeField, DateTimeField, RegexField, MultipleChoiceField, TypedChoiceField, FormSets, ModelForms, per-field clean hooks |
| **Auth** | Missing: Permission/Group models, UserManager, password validation, password reset views, auth forms (AuthenticationForm, UserCreationForm), RemoteUserBackend |
| **Middleware** | Missing: CommonMiddleware, GZipMiddleware, CacheMiddleware, LocaleMiddleware, CSPMiddleware, ConditionalGetMiddleware |
| **Template tags** | Missing: csrf_token, url, load, with, autoescape, filter, lorem, ifchanged, for...empty |
| **Template filters** | Missing 9 of 53: escapejs, force_escape, truncatechars_html, truncatewords_html, urlizetrunc, wordwrap, get_digit, iriencode, timeuntil |
| **ORM fields** | Missing: DecimalField, SlugField, UUIDField, BinaryField, DurationField, TimeField, IPAddressField, JSONField, FileField, ImageField |
| **ORM relationships** | Missing: ManyToMany.through, ForeignKey.limit_choices_to, related_name/related_query_name, GenericForeignKey, GenericRelation |
| **Admin** | Missing: list_filter, search_fields, date_hierarchy, Inlines, actions, autocomplete, filters, admin decorators, history view |
| **Views** | Missing: Full class-based lifecycle (dispatch, setup, as_view), Mixin integration, date-based archive views, RedirectView |
| **Test client** | Missing: PUT/PATCH/DELETE/HEAD/OPTIONS, RequestFactory, login/logout/force_login, session/cookie access |
| **Test runner** | Missing: discover/build_suite/setup_databases, parallel test execution, shuffler |
| **Migrations** | Missing: 14+ operations (AddIndex, RunSQL, RunPython), Migration loader/executor/recorder/writer, state management |
| **CLI** | Missing: dumpdata, loaddata, inspectdb, sqlmigrate, flush, makemessages, squashmigrations, etc. |

---

## ❌ Entirely Missing Modules

| Django Module | Rjango | Priority |
|---|---|---|
| `django.core.cache` (7 backends) | ❌ | **High** |
| `django.core.mail` (SMTP, console, file) | ❌ | Medium |
| `django.core.signing` (Signer, TimestampSigner) | ❌ | Medium |
| `django.core.serializers` (JSON, XML, YAML, Python) | ❌ | Medium |
| `django.core.handlers` (WSGI, ASGI) | ❌ | Medium |
| `django.core.asgi` / `django.core.wsgi` | ❌ | Medium |
| `django.core.checks` (system checks framework) | ❌ | Low |
| `django.core.paginator` (Page API) | ❌ (partial) | Medium |
| `django.utils.timesince` / `django.utils.timezone` | ❌ (partial) | Medium |
| `django.utils.dates` / `django.utils.dateformat` | ❌ | Low |
| `django.utils.decorators` | ❌ | Low |
| `django.utils.encoding` | ❌ | Medium |
| `django.utils.translation` (full) | ❌ | Low |
| `django.utils.functional` (lazy, Promise) | ❌ | Medium |
| `django.utils.http` (urlencode, int_to_base36) | ❌ | Low |
| `django.utils.module_loading` | ❌ | Medium |
| `django.utils.deconstruct` | ❌ | Low |
| `django.utils.datastructures` (MultiValueDict) | ❌ | Medium |
| `django.contrib.postgres` | ❌ | Low |
| `django.contrib.gis` | ❌ | Low |
| `django.contrib.sites` | ❌ | Medium |
| `django.contrib.sitemaps` | ❌ | Low |
| `django.contrib.flatpages` | ❌ | Low |
| `django.contrib.redirects` | ❌ | Low |
| `django.contrib.humanize` | ❌ | Low |
| `django.contrib.contenttypes` | ❌ | **High** (needed by admin) |
| `django.contrib.syndication` (feeds) | ❌ | Low |
| `django.contrib.admindocs` | ❌ | Low |
| `django.contrib.staticfiles` (finders, storage) | ❌ (CLI only) | Medium |
| `django.db.models.constraints` | ❌ | Low |
| `django.db.models.indexes` | ❌ | Low |
| `django.db.models.functions` | ❌ | Low |
| `django.db.models.lookups` (23+ lookup types) | ❌ | **High** |
| `django.db.models.query` (300+ QuerySet API) | ❌ (partial) | **High** |
| `django.db.models.deletion` (CASCADE/etc) | ❌ | Medium |
| `django.db.models.signals` | ❌ | Medium |
| `django.db.backends` (real SQL execution) | ❌ (basic only) | **Critical** |
| Oracle DB backend | ❌ | Low |
| `django.views.generic.dates` (14 view classes) | ❌ | Low |
| `django.http` (JsonResponse, FileResponse, StreamingHttpResponse) | ❌ | Medium |
| `django.http.cookie` | ❌ | Medium |
| `django.test.selenium` | ❌ | Low |
| `django.test.runner.ParallelTestSuite` | ❌ | Low |
| `django.core.management` (14 missing commands) | ❌ | Medium |

---

## 📈 Gap Summary by Priority

### Critical (blocking production use)
| Feature | Why |
|---|---|
| **Real ORM CRUD** | Can't read/write to database |
| **QuerySet lookup support** | No filtering beyond basic Q |
| **Model save/delete** | No data persistence |
| **DB-backed sessions** | Sessions can't persist |
| **Database migration executor** | Migrations are detected but not run |

### High (needed for real-world apps)
| Feature | Why |
|---|---|
| **Cache framework** | Performance-critical |
| **ContentTypes** | Required by admin |
| **GenericForeignKey** | Required by admin/content |
| **Admin search/filter** | Admin is barely functional |
| **FormSets / ModelForms** | Needed for real forms |
| **CommonMiddleware** | APPEND_SLASH, URL cleanup |
| **dumpdata / loaddata** | Data export/import |
| **Full settings coverage** | Many settings undeclared |

### Medium
| Feature | Why |
|---|---|
| Email sending | Notifications |
| URL `re_path()` | Regex URL patterns |
| More CLI commands | dumpdata, loaddata, inspectdb |
| Password reset flow | UX |
| Translation utilities | Multi-language |
| AutoField models | ID generation |
| `{% url %}` template tag | URL reversing |

### Low
| Feature | Why |
|---|---|
| Date-based archive views | Niche use |
| GIS module | Niche |
| Postgres-specific features | Niche |
| Sitemaps | SEO |
| RSS/Atom feeds | Niche |
| Oracle support | Niche |
| Selenium tests | Niche |

---

## 💡 Recommended Priority Roadmap

### Phase 1: Database-Backed ORM (Critical)
1. Implement `Model::save()` (INSERT/UPDATE)
2. Implement `Model::delete()` (DELETE)
3. Implement QuerySet lookups (exact, contains, gt, lt, in, range, etc.)
4. Implement `QuerySet.filter()` chaining with real SQL
5. Implement migration executor (run operations against DB)
6. Implement DB-backed session store

### Phase 2: Production Admin (High)
1. Implement ContentTypes + GenericForeignKey
2. Implement admin search (+ `search_fields`)
3. Implement admin filters (`list_filter`)
4. Implement admin actions framework
5. Implement admin Inlines (StackedInline, TabularInline)

### Phase 3: Feature Completion (Medium)
1. Cache framework (locmem + redis backends)
2. CommonMiddleware + more middleware
3. Email sending
4. Form validation hooks (per-field clean)
5. FormSets
6. Password validation / reset flow
7. `{% url %}` tag + `re_path()`
8. dumpdata / loaddata commands

### Phase 4: Polish (Low)
1. Date archive views
2. Sitemaps
3. Humanize template tags
4. Full settings coverage
5. Translation utilities
6. Oracle backend

---

## 📊 Detailed Reports

Each Django module has a dedicated comparison report:

| Report | File |
|--------|------|
| 📄 Core (exceptions, signals, signing, cache, validators, etc.) | [django-core-features.md](django-core-features.md) |
| 📄 ORM (models, fields, QuerySets, lookups, migrations) | [django-orm-features.md](django-orm-features.md) |
| 📄 Templates (engine, tags, filters, loaders) | [django-templates-features.md](django-templates-features.md) |
| 📄 Forms, Auth, Middleware, Sessions, Messages | [django-forms-auth-middleware-features.md](django-forms-auth-middleware-features.md) |
| 📄 URLs, Views, Test, CLI, Migrations | [django-urls-views-test-features.md](django-urls-views-test-features.md) |
| 📄 Contrib (admin, postgres, gis, sites, sitemaps, etc.) | [django-contrib-features.md](django-contrib-features.md) |

---

> **Bottom line:** Rjango covers ~25-30% of Django 6.0.6's total API surface. The URL routing, template engine (especially filters), and signal systems are strongest. The ORM (database CRUD), admin, and contrib modules are weakest and need the most work.
