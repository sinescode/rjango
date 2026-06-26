# Django 6.0.6 URLs, Views, Test, CLI — Feature Comparison with Rjango

> **Django modules analyzed:** `django.urls`, `django.views.generic`, `django.views.decorators`, `django.test`, `django.core.management`
> **Last updated:** 2026-06-26

---

## Overview

| Category | Coverage | Status |
|----------|----------|--------|
| **URL Dispatcher** | 9/12 = **75%** | 🟢 Strong |
| **Views (Class-based)** | 7/48 = **15%** | 🔴 Weak |
| **View Decorators** | 4/8 = **50%** | 🟡 Partial |
| **Test Utilities** | 6/14 = **43%** | 🟡 Partial |
| **CLI Commands** | 13/27 = **48%** | 🟡 Partial |

---

## ✅ URL Dispatcher — 75%

### Implemented
- `resolve()` — URL pattern matching
- `reverse()` — URL reversing with args/kwargs
- `include()` — Include sub-URL configurations
- `set_urlconf()` / `get_urlconf()` — Thread-local URL config
- `clear_url_caches()` — Reset URL cache
- `set_script_prefix()` / `get_script_prefix()` — Script name support
- `LazyString` — Deferred string evaluation
- URLPattern, ResolverMatch, URLResolver with namespace/app_name
- Path converters: `IntConverter`, `StrConverter`, `SlugConverter`, `UUIDConverter`, `AnyPathConverter`
- `path()` — Django-style path routing
- `re_path()` — Regex-based routing
- `default_converters()` — Built-in converter registry

### Missing
- `RegexPattern` as standalone pattern class
- `URLPattern.reverse()` for named patterns
- `register_converter()` for custom converters

---

## 🟡 Generic Views — 15%

### ✅ Implemented (7 APIs)

#### Base Views
| View | Status |
|------|--------|
| `View` trait | ✅ |
| `TemplateView` | ✅ |
| `RedirectView` | ✅ |
| `ContextMixin` | ✅ |
| `TemplateResponseMixin` | ✅ |

#### Mixins
| Mixin | Status |
|-------|--------|
| `MultipleObjectMixin` | ✅ |
| `SingleObjectMixin` | ✅ |
| `FormMixin` | ✅ |

#### Detail Views
| View | Status |
|------|--------|
| `DetailView` | ✅ |

### ❌ Not Implemented (41 APIs)

#### List Views
| View | Priority |
|------|----------|
| `ListView` with full paginated context | 🔴 High |
| `MultipleObjectTemplateResponseMixin` | 🟡 Medium |

#### Edit Views (critical)
| View | Priority |
|------|----------|
| `FormView` | 🔴 High |
| `CreateView` | 🔴 High |
| `UpdateView` | 🔴 High |
| `DeleteView` | 🔴 High |
| `DeletionMixin` | 🟡 Medium |
| `ModelFormMixin` (form → model save) | 🔴 High |
| `BaseFormView`, `BaseCreateView`, `BaseUpdateView`, `BaseDeleteView` | 🔴 High |
| `ProcessFormView` | 🟡 Medium |

#### Date-Based Archive Views
| View | Priority |
|------|----------|
| `ArchiveIndexView` | 🔴 High |
| `YearArchiveView` | 🔴 High |
| `MonthArchiveView` | 🔴 High |
| `WeekArchiveView` | 🟡 Medium |
| `DayArchiveView` | 🟡 Medium |
| `TodayArchiveView` | 🟡 Medium |
| `DateDetailView` | 🟡 Medium |
| `DateMixin`, `YearMixin`, `MonthMixin`, `WeekMixin`, `DayMixin` | 🟡 Medium |
| `BaseDateListView`, `BaseArchiveIndexView`, etc. (6 base classes) | 🟡 Medium |

#### Default Views
| View | Priority |
|------|----------|
| `bad_request` (400) | 🟢 Low |
| `permission_denied` (403) | 🟢 Low |
| `page_not_found` (404) | 🟢 Low |
| `server_error` (500) | 🟢 Low |

---

## 🟡 View Decorators — 50%

### ✅ Implemented
| Decorator | Status |
|-----------|--------|
| `never_cache` | ✅ |
| `require_POST` | ✅ |
| `require_GET` | ✅ |
| `require_http_methods` | ✅ |

### ❌ Missing
| Decorator | Priority |
|-----------|----------|
| `require_safe` | 🟢 Low |
| `cache_page` | 🟡 Medium |
| `cache_control` | 🟡 Medium |
| `csrf_exempt` | ✅ (exists in auth) |
| `gzip_page` | 🟢 Low |
| `xframe_options_deny` | 🟢 Low |
| `xframe_options_sameorigin` | 🟢 Low |
| `xframe_options_exempt` | 🟢 Low |
| `vary_on_headers` / `vary_on_cookie` | 🟢 Low |
| `sensitive_variables` / `sensitive_post_parameters` | 🟢 Low |
| `@method_decorator` | 🟡 Medium |

---

## 🟡 Test Utilities — 43%

### ✅ Implemented

#### Test Client
- `Client` struct with `GET()`, `POST()`, `PUT()`, `PATCH()`, `DELETE()` methods
- Header and cookie support
- `ClientResponse` with body, status, headers

#### Test Runner
- `TestRunner` with `run()` returning `TestResult`
- Test discovery and execution
- Result summary

#### TestCase
- `TestCase` struct with `setup()`, `teardown()`, `run_test()`
- `assert_equal()`, `assert_true()`, `assert_false()`

#### Assert Helpers
- `assert_contains()`, `assert_not_contains()`
- `assert_template_used()`, `assert_redirects()`
- `assert_status()`, `override_settings()`

### ❌ Missing

| Feature | Priority |
|---------|----------|
| `SimpleTestCase`, `TransactionTestCase` | 🟡 Medium |
| `RequestFactory` (build Request objects) | 🔴 High |
| `Client.login()` / `Client.force_login()` | 🔴 High |
| `modify_settings` decorator | 🟢 Low |
| `tag_test` decorator | 🟢 Low |
| `Client.session()` manipulation | 🟡 Medium |
| Test database setup/teardown | 🟡 Medium |

---

## 🟡 CLI Commands — 48%

### ✅ Implemented (13 of 27)

| Command | Status | Notes |
|---------|--------|-------|
| `runserver` | ✅ | Basic dev server |
| `migrate` | ✅ | With `MigrationRunner.apply()` |
| `makemigrations` | ✅ | With `SchemaDetector` |
| `test` | ✅ | Test runner integration |
| `startapp` | ✅ | App scaffold |
| `startproject` | ✅ | Project scaffold |
| `shell` | ✅ | With auto-import options |
| `dbshell` | ✅ | DB shell |
| `showmigrations` | ✅ | Migration listing |
| `diffsettings` | ✅ | Settings diff |
| `check` | ✅ | System checks |
| `sqlmigrate` | ✅ | SQL output |
| `sqlflush` | ✅ | SQL flush output |

### ❌ Missing (14 commands)

| Command | Priority | Notes |
|---------|----------|-------|
| `dumpdata` | 🔴 High | Serialize DB to JSON/XML |
| `loaddata` | 🔴 High | Load fixtures |
| `inspectdb` | 🟡 Medium | Reverse-engineer DB → models |
| `flush` | 🟡 Medium | Clear DB data |
| `compilemessages` | 🟡 Medium | .po → .mo compilation |
| `makemessages` | 🟡 Medium | Extract translatable strings |
| `squashmigrations` | 🟢 Low | Reduce migration count |
| `optimizemigration` | 🟢 Low | Optimize a migration |
| `sendtestemail` | 🟢 Low | Test email config |
| `testserver` | 🟢 Low | Test data server |
| `createcachetable` | 🟢 Low | Create cache table |
| `dumpscript` | 🟢 Low | Dump as Python script |
| `clear_cache` | 🟢 Low | Clear cache |
| `remove_stale_contenttypes` | 🟢 Low | Clean content types |

---

## 📊 Coverage Summary

| Submodule | Django | Rjango | % | Priority for Improvement |
|-----------|--------|--------|---|-------------------------|
| URL Dispatcher | 12 | 9 | **75%** | Low |
| Views (generic) | 48 | 7 | **15%** | **High** |
| View Decorators | 8 | 4 | **50%** | Low |
| Test Utilities | 14 | 6 | **43%** | Medium |
| CLI Commands | 27 | 13 | **48%** | Medium |
| **Total** | **109** | **39** | **36%** | |
