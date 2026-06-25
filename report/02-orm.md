# Django vs Rjango ORM: Comprehensive Comparison Report

**Date**: 2026-06-25 (Updated)  
**Django Version**: 6.0.6  
**Rjango Version**: 0.1.0  
**Rjango Location**: `rjango-orm/src/` (1,273 lines)  

---

## 1. Models

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `Model` base class | All models inherit | `Model` struct + `ModelTrait` | ✅ YES | |
| `Meta` class | Model metadata | ModelMeta struct | ✅ YES | table_name, ordering |
| Auto-generated `id` pk | AutoField | Auto-increment PK | ✅ YES | |
| String representation | `__str__()` | Derive from struct | ⚠️ PARTIAL | Manual via Debug |
| Custom managers | `objects = Manager()` | `Manager` struct + `ManagerTrait` | ✅ YES | |
| Signals (pre_save, post_save) | ORM lifecycle | In rjango-core signals | ✅ YES | |
| Constraints | Unique, Check | — | ❌ NO | |
| Index composition | Meta.indexes | — | ❌ NO | |

## 2. Fields

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `CharField` | String field | `FieldTypes::CharField` | ✅ YES | |
| `TextField` | Text field | `FieldTypes::TextField` | ✅ YES | |
| `IntegerField` | Integer field | `FieldTypes::IntegerField` | ✅ YES | |
| `BooleanField` | Bool field | `FieldTypes::BooleanField` | ✅ YES | |
| `DateTimeField` | DateTime | `FieldTypes::DateTimeField` | ✅ YES | |
| `DateField` | Date only | `FieldTypes::DateField` | ✅ YES | |
| `FloatField` | Float | `FieldTypes::FloatField` | ✅ YES | |
| `DecimalField` | Decimal | `FieldTypes::DecimalField` | ✅ YES | |
| `EmailField` | Email | `FieldTypes::EmailField` | ✅ YES | |
| `URLField` | URL | `FieldTypes::URLField` | ✅ YES | |
| `SlugField` | Slug | `FieldTypes::SlugField` | ✅ YES | |
| `UUIDField` | UUID | `FieldTypes::UUIDField` | ✅ YES | |
| `JSONField` | JSON | — | ❌ NO | |
| `FileField` | File upload | — | ❌ NO | |
| `ImageField` | Image upload | — | ❌ NO | |
| `ForeignKey` | FK relationship | `Relationship::ForeignKey` | ✅ YES | |
| `OneToOneField` | 1-1 relationship | `Relationship::OneToOne` | ✅ YES | |
| `ManyToManyField` | M2M relationship | `Relationship::ManyToMany` | ✅ YES | |
| Null/blank/default/choices | Field options | `Field` trait | ⚠️ PARTIAL | Basic options only |
| SQL type mapping | Per-backend types | `sql_type()` via backend | ✅ YES | |

## 3. QuerySet

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `filter()` | WHERE clause | `QuerySet::filter()` | ✅ YES | |
| `exclude()` | NOT WHERE | `QuerySet::exclude()` | ✅ YES | |
| `order_by()` | ORDER BY | `QuerySet::order_by()` | ✅ YES | |
| `values()` | Specific columns | — | ❌ NO | |
| `annotate()` | Aggregation | — | ❌ NO | |
| `aggregate()` | Aggregation | — | ❌ NO | |
| `count()` | COUNT | `QuerySet::count()` | ✅ YES | |
| `first()` / `last()` | First/last | — | ❌ NO | |
| `exists()` | EXISTS | `QuerySet::exists()` | ✅ YES | |
| `distinct()` | SELECT DISTINCT | — | ❌ NO | |
| `select_related()` | FK joins | — | ❌ NO | |
| `prefetch_related()` | Optimized joins | — | ❌ NO | |
| `Q` objects | Complex queries | — | ❌ NO | |
| `F()` expressions | Field refs | `rjango_orm::expressions::F` | ✅ YES | |
| `defer()` / `only()` | Column subset | — | ❌ NO | |
| **Tests** | — | 30 tests | ✅ | |

## 4. Expressions / Database Functions

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `Now()` | Current timestamp | `now()` | ✅ YES | |
| `Cast()` | Type cast | `cast()` | ✅ YES | |
| `Coalesce()` | First non-null | `coalesce()` | ✅ YES | |
| `Length()` | String length | `length()` | ✅ YES | |
| `Lower()` | Lowercase | `lower()` | ✅ YES | |
| `Upper()` | Uppercase | `upper()` | ✅ YES | |
| `Substr()` | Substring | `substr()` | ✅ YES | Start + optional length |
| `Trim()` | Whitespace trim | `trim()` | ✅ YES | |
| `Concat()` | String concat | `concat()` | ✅ YES | |
| `F()` | Field reference | `F` struct + `col()` | ✅ YES | |
| `RawSQL()` | Raw SQL | `raw()` | ✅ YES | |
| `Greatest()` / `Least()` | Compare | — | ❌ NO | |
| Window functions | OVER clause | — | ❌ NO | |
| `Extract()` | Date parts | — | ❌ NO | |
| **Tests** | — | 20 tests | ✅ | |

## 5. Executor (Real DB Operations)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| SQLite execution | via sqlite3 | `sqlx::SqlitePool` | ✅ YES | Real in-memory + file SQLite |
| PostgreSQL execution | via psycopg2 | `sqlx::PgPool` (configured) | ⚠️ PARTIAL | Needs PG server |
| MySQL execution | via mysqlclient | `sqlx::MySqlPool` (configured) | ⚠️ PARTIAL | Needs MySQL server |
| `execute()` | Raw SQL | `DatabasePool::execute()` | ✅ YES | Returns rows affected |
| `query()` | Raw SQL | `DatabasePool::query()` | ✅ YES | Returns HashMap rows |
| `create_table()` | DDL | `create_table()` + `create_table_sql()` | ✅ YES | |
| `drop_table()` | DDL | `drop_table()` | ✅ YES | |
| `table_exists()` | Check | `table_exists()` | ✅ YES | |
| `insert()` + last_rowid | Insert | `insert()` | ✅ YES | Returns last_insert_rowid |
| **Tests** | — | 9 tests (real SQLite) | ✅ | |

## 6. Relationships

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| ForeignKey | Relates models | `Relationship::ForeignKey` | ✅ YES | |
| OneToOneField | 1-1 relation | `Relationship::OneToOne` | ✅ YES | |
| ManyToManyField | M2M relation | `Relationship::ManyToMany` | ✅ YES | |
| RelatedManager (FOO_set) | Reverse query | — | ❌ NO | |
| Prefetch objects | Optimized loading | — | ❌ NO | |
| **Tests** | — | Implicit | ⚠️ | |

## 7. Aggregates

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `Sum` | SUM | — | ❌ NO | |
| `Count` | COUNT | — | ❌ NO | |
| `Avg` | AVG | — | ❌ NO | |
| `Min` | MIN | — | ❌ NO | |
| `Max` | MAX | — | ❌ NO | |

## Summary

| SUB-MODULE | STATUS | LOCATION | LINES | TESTS |
|------------|--------|----------|-------|-------|
| Models | ✅ YES | `models.rs` | 99 | ✅ |
| Fields | ✅ YES | `fields.rs` | 165 | ✅ |
| QuerySet | ⚠️ PARTIAL | `query.rs` | 236 | ✅ 30 |
| Expressions | ✅ YES | `expressions.rs` | 311 | ✅ 20 |
| Executor | ✅ YES | `executor.rs` | 308 | ✅ 9 (real SQLite) |
| Relationships | ✅ YES | `relationships.rs` | 71 | ✅ |
| Managers | ✅ YES | `managers.rs` | 26 | ✅ |
| Backend config | ✅ YES | `backend.rs` | 33 | ✅ |
