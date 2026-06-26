# Django 6.0.6 ORM — Feature Comparison with Rjango

> **Django modules analyzed:** `django.db.models`, `django.db.models.fields`, `django.db.models.lookups`, `django.db.models.expressions`, `django.db.models.aggregates`, `django.db.backends`, `django.db.migrations`
> **Last updated:** 2026-06-26

---

## Overview

| Metric | Value |
|--------|-------|
| **Django ORM source files** | ~130 across models, backends, migrations |
| **Rjango ORM Rust files** | 12 in `rjango-orm/src/` |
| **ORM coverage** | 24% (25/105 API items) |
| **Tests** | ~120 across rjango-orm |

---

## ✅ Implemented

### Model System
- `Model` trait with `table_name()`, `field_definitions()`, `from_row()`, primary key methods
- `ModelMetadata` with options, field access, creator functions
- `Options`, `OptionsBuilder` for model configuration
- `ModelBuilder` for builder-pattern model creation
- Composite primary key support (struct exists)

### Field Types (10 of 25+ Django field types)
| Rjango | Django Equivalent |
|--------|-------------------|
| `SimpleField` (Integer, Char, Text, Boolean, Float, Date, DateTime) | IntegerField, CharField, TextField, BooleanField, FloatField, DateField, DateTimeField |
| `DecimalField` ✅ | DecimalField |
| `UUIDField` ✅ | UUIDField |
| `EmailField` ✅ | EmailField |
| `URLField` ✅ | URLField |
| `SlugField` ✅ | SlugField |
| `GenericIPAddressField` ✅ | GenericIPAddressField |
| `TimeField` ✅ | TimeField |

### Field Options
- `null`, `unique`, `default`, `max_length`, `db_index`, `db_column`
- `primary_key`, `choices`, `help_text`, `verbose_name`

### Relationships (Partial)
- `ForeignKey` with `on_delete`, `related_name`, `to_field`
- `OneToOneField`, `OneToOne`
- `ManyToMany` with table name
- **Missing:** `ManyToMany.through`, `limit_choices_to`, `related_query_name`, `db_constraint`

### QuerySet (Partial)
- `QuerySet` struct with builder methods: `filter()`, `exclude()`, `order_by()`, `limit()`, `offset()`, `join()`, `select_related()`, `prefetch_related()`, `distinct()`, `values()`, `values_list()`
- `QuerySetIter`, `PrefetchQuerySet`, `QuerySetChunkedIter`
- `QuerySetManager` for per-model QuerySets
- **Missing:** `get()`, `create()`, `update()`, `delete()`, `exists()`, `count()`, `first()`, `last()` with SQL execution

### Query Builder
- `QueryBuilder` with SELECT/FROM/WHERE/JOIN SQL generation
- Parameterized query support
- **Missing:** Full lookup-to-WHERE translation, aggregation SQL, subquery SQL

### Lookups (Basic)
- `Lookup` enum with variants: exact, iexact, contains, icontains, in, gt, gte, lt, lte, startswith, istartswith, endswith, iendswith, range, date, year, month, day, week_day, hour, minute, second, isnull, regex, iregex
- `FilterCondition` with `parse()`, `sql_operator()`, `format_value()`, `sql_snippet()`, `build_where_clause()`
- **Missing:** Wiring lookups → QuerySet.filter() → actual SQL WHERE

### Expressions
- `Q` objects with AND/OR/NOT, lookup chaining, `to_sql()`
- `F()` expression, `Value()`
- `Coalesce`, `When`, `Subquery`, `Exists`
- Expression trait, `SqlExpression`, `Conditional` enums

### Aggregates
- `AggType` enum: Count, Sum, Avg, Min, Max
- `Aggregate` struct with `distinct()`, `to_sql()`, alias support
- `Aggregation` (multiple aggregates)

### Database Functions
- `DbFunction` enum: Coalesce, Concat, Length, Lower, Upper, Trim, LTrim, RTrim, Substr, Replace, Abs, Ceil, Floor, Round, Now, Extract
- 15+ function implementations

### Managers
- `Manager` trait, `ModelManager`

### Database Backend
- `DatabaseBackend` enum: Sqlite, Postgres, MySQL
- `placeholder()` (SQL param placeholder), `quote_ident()` (identifier quoting)
- `DatabaseConfig` with SQLite/Postgres URLs
- `DatabasePool` with SQLx-based pooling, `execute()`, `query()`, `table_exists()`, `create_table()`

### Migrations (Partial)
- `Migration` struct with operations, dependencies, full_name
- `MigrationOperation` enum: CreateModel, DeleteModel, AddField, RemoveField, AlterField, AlterModelTable, AddIndex, RemoveIndex
- `MigrationField`, `ModelOptions`
- `SchemaDetector` with `detect_field_changes()`, `create_model_op()`
- `MigrationRunner` with `apply()`, `plan()`, `to_sql()`
- **Missing:** Real SQL execution of migration operations, migration recorder table, dependency resolution, `RunSQL`, `RunPython`

---

## ❌ Missing Field Types

| Django Field | Rjango | Priority |
|-------------|--------|----------|
| AutoField | ❌ | High |
| BigAutoField | ❌ | High |
| SmallAutoField | ❌ | High |
| BigIntegerField | ❌ | High |
| PositiveBigIntegerField | ❌ | Medium |
| SmallIntegerField | ❌ | Medium |
| DurationField | ❌ | Medium |
| BinaryField | ❌ | Low |
| JSONField | ❌ | Medium |
| FileField | ❌ | High |
| ImageField | ❌ | Medium |

## ❌ Missing QuerySet Methods

| Django Method | Rjango | Notes |
|--------------|--------|-------|
| `get(**kwargs)` | ❌ | Core CRUD |
| `create(**kwargs)` | ❌ | INSERT |
| `update(**kwargs)` | ❌ | UPDATE |
| `delete()` | ❌ | DELETE |
| `exists()` | ❌ | COUNT |
| `count()` | ❌ | COUNT |
| `first()` / `last()` | ❌ | ORDER BY + LIMIT |
| `latest()` / `earliest()` | ❌ | ORDER BY date |
| `get_or_create()` | ❌ | SELECT + INSERT |
| `update_or_create()` | ❌ | SELECT + UPDATE |
| `bulk_create()` | ❌ | Multi-INSERT |
| `in_bulk()` | ❌ | pk → dict |
| `only()` / `defer()` | ❌ | Column selection |
| `select_for_update()` | ❌ | Row locking |
| `iterator()` | ❌ | Chunked iteration |

## ❌ Missing Model Features

| Feature | Rjango | Priority |
|---------|--------|----------|
| `Model.save()` (upsert) | ❌ | Critical |
| `Model.delete()` | ❌ | Critical |
| `Model.clean()` / `full_clean()` | ❌ | High |
| `Model.validate_unique()` | ❌ | Medium |
| `get_absolute_url()` | ❌ | Medium |
| Model inheritance | ❌ | Medium |
| `Model.objects` as Manager entry point | ❌ | High |
| `Model._meta` introspection | Partial | Medium |
| `ModelMeta` class | ✅ | Complete |

---

## 📊 Coverage Summary

| Submodule | Django | Rjango | % |
|-----------|--------|--------|---|
| Models (base) | 15 | 3 | 20% |
| QuerySet | 15 | 2 | 13% |
| Fields | 25 | 10 | 40% |
| Lookups | 15 | 1 | 7% |
| Expressions | 10 | 3 | 30% |
| Migrations | 8 | 2 | 25% |
| Backends | 6 | 2 | 33% |
| Relationships | 6 | 3 | 50% |
| Aggregates | 5 | 2 | 40% |
| **Total ORM** | **105** | **25** | **24%** |
