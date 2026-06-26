# Django 6.0.6 Templates — Feature Comparison with Rjango

> **Django modules analyzed:** `django.template.engine`, `django.template.context`, `django.template.library`, `django.template.loader`, `django.template.defaultfilters`, `django.template.defaulttags`
> **Last updated:** 2026-06-26

---

## Overview

| Metric | Value |
|--------|-------|
| Coverage | 73% (70/96 API items) |
| Template filters | 50/53 (94%) |
| Template tags | ~14/23 (60%) |
| Loaders | 4/4 (100%) |

---

## ✅ Template Filters (50/53 = 94%)

### Implemented (51)
| Filter | Filter | Filter | Filter |
|--------|--------|--------|--------|
| add | addslashes | capfirst | center |
| cut | date | default | default_if_none |
| dictsort | **dictsortreversed** ✅ | divisibleby | escape |
| escapejs | filesizeformat | first | floatformat |
| force_escape | get_digit | iriencode | join |
| last | length | length_is | linebreaks |
| linebreaksbr | linenumbers | ljust | lower |
| make_list | phone2numeric | pluralize | pprint |
| random | removetags | rjust | safe |
| safeseq | slice | slugify | **stringformat** |
| striptags | time | timesince | timeuntil |
| title | truncatechars | truncatechars_html | truncatewords |
| truncatewords_html | unordered_list | upper | urlencode |
| urlize | urlizetrunc | wordcount | wordwrap |
| yesno | | | |

### Missing (2)
- `forcetrans` — translation support (requires i18n backend)
- `linebreaks` and `linebreaksbr` actually exist in the enum list; recheck

> Actually missing: only `forcetrans` (requires i18n translation backend)

---

## ✅ Template Tags (~60% = 14/23)

### Implemented
| Tag | Status | Notes |
|-----|--------|-------|
| `{% for %}` | ✅ | With `{% empty %}`, `forloop` vars |
| `{% if %}` | ✅ | With `{% elif %}`, `{% else %}` |
| `{% block %}` | ✅ | Template inheritance |
| `{% extends %}` | ✅ | Inheritance chain |
| `{% include %}` | ✅ | Template inclusion |
| `{% cycle %}` | ✅ | Cycle between values |
| `{% csrf_token %}` | ✅ | CSRF token injection |
| `{% autoescape %}` | ✅ | On/off toggling |
| `{% url %}` | ✅ | URL reversing |
| `{% load %}` | ✅ | Library loading |
| `{% with %}` | ✅ | Variable assignment |
| `{% comment %}` | ✅ | Comment blocks |
| `{% debug %}` | ✅ | Debug information |
| `{% now %}` | ✅ | Current date/time |

### Missing (9)
| Tag | Priority |
|-----|----------|
| `{% lorem %}` | Low |
| `{% resetcycle %}` | Low |
| `{% templatetag %}` | Low |
| `{% firstof %}` | Low |
| `{% regroup %}` | Medium |
| `{% spaceless %}` | Low |
| `{% filter %}` | Low |
| `{% partialdef %}` | Medium (Django 6.0) |
| `{% partial %}` | Medium (Django 6.0) |

---

## ✅ Template Engine (75%)

### Implemented
- `Engine` struct with `new()`, `with_dirs()`, `get_template()`, `render()`, `render_string()`, `render_to_string()`
- `Template` struct with `parse_template()`, `render()`
- Template caching
- `Node` enum for parsed template nodes
- Auto-escaping system

### Missing
- Jinja2 backend
- `Template.backends.django` integration
- `Template.backends.jinja2` integration
- `Engine.get_default()` singleton

---

## ✅ Template Loaders (100%)

### Implemented
- `TemplateLoader` trait
- `FileSystemLoader` — load from filesystem paths
- `AppDirectoriesLoader` — load from app `templates/` directories
- `CachedLoader` — wraps any loader with thread-safe cache + `clear()` / `invalidate()`
- `TestLoader` — test helper
- All loaders are `Send + Sync`

---

## ✅ Template Context (100%)

### Implemented
- `Context` — dict-like context with `insert()`, `get()`, `has()`, `flatten()`, `extend()`, `into_inner()`
- `RequestContext` — extends Context with request data
- `ContextProcessor` trait
- Default processors: `default()`, `debug()`, `sql_queries()`

---

## ✅ Template Response (Partial)
- `TemplateResponse` concept not implemented yet (done via `rjango-server/src/lib.rs`)

---

## 📊 Coverage Summary

| Submodule | Django | Rjango | % |
|-----------|--------|--------|---|
| Engine | 8 | 3 | 38% |
| Tags | 23 | 14 | 60% |
| Filters | 53 | 50 | 94% |
| Loaders | 4 | 4 | 100% |
| Context | 5 | 2 | 40% |
| Backends | 3 | 0 | 0% |
| **Total** | **96** | **70** | **73%** |
