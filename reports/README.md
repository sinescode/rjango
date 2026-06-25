# 📁 Rjango Reports Index

This directory contains detailed Django 6.0.6 feature comparisons for the Rjango Rust port.

## Reports

| # | Report | Lines | Description |
|---|--------|-------|-------------|
| 1 | `gap-analysis.md` | ~200 | **Master gap analysis** — all categories, priority matrix, quick wins |
| 2 | `django-core-features.md` | ~400 | Core: paginator, exceptions, validators, cache, files, handlers, serializers, apps, conf, dispatch, shortcuts, utils |
| 3 | `django-orm-features.md` | ~350 | ORM: model system, fields, QuerySet, lookups, aggregates, expressions, backends, migrations |
| 4 | `django-templates-features.md` | ~200 | Templates: engine, context, loaders, 60+ filters, 28+ tags, library |
| 5 | `django-forms-auth-middleware-features.md` | ~350 | Forms: fields, widgets, formsets; Auth: users, permissions, backends; Middleware: CSRF, session, security |
| 6 | `django-urls-views-test-features.md` | ~250 | URLs: routing, converters, reverse; Views: generic class-based; Test: client, runner, assertions |
| 7 | `django-contrib-features.md` | ~250 | Contrib: admin, contenttypes, postgres, staticfiles, humanize, sitemaps, etc. |

## Legend

| Status | Meaning |
|--------|---------|
| ✅ | Complete — matches Django API |
| ⚠️ | Partial — basic version exists, missing features |
| ❌ | Missing — not implemented |
| N/A | Not applicable |

## Quick Stats

- **Django 6.0.6**: ~679 Python source files, ~30 packages
- **Rjango**: 98 Rust source files, 16 crates, 17,414 lines
- **API coverage**: ~22% of Django's public API surface
- **Template features**: ~75% coverage (best category)
- **ORM features**: ~15% coverage (worst critical category)
- **Zero coverage**: Caching, Mail, Signing, GIS, PostgreSQL contrib
