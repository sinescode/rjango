# Django 6.0.6 ORM — Feature Comparison with Rjango

> **Django modules analyzed:** `django.db.models`, `django.db.models.fields`, `django.db.models.lookups`, `django.db.models.expressions`, `django.db.models.aggregates`, `django.db.backends`, `django.db.migrations`

---

## 1. Model System (`django.db.models`)

### Model Base
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Model` | `Model` | ⚠️ Partial | Rjango has `ModelMeta` + `ModelBuilder` pattern |
| `Model.save()` | ❌ Missing | No DB persistence |
| `Model.save(force_insert, force_update, using)` | ❌ Missing | |
| `Model.delete()` | ❌ Missing | |
| `Model.pk` | `ModelBuilder::auto_field()` | ⚠️ Partial | Basic primary key support |
| `Model.objects` | `ModelBuilder::manager()` | ⚠️ Partial | Manager pattern exists |
| `Model.DoesNotExist` | ❌ Missing | Exception class per model |
| `Model.MultipleObjectsReturned` | ❌ Missing | |
| `Model.get_absolute_url()` | ❌ Missing | |
| `from_db(db, field_names, values)` | ❌ Missing | |
| `Model.clean()` | ❌ Missing | |
| `Model.clean_fields(exclude)` | ❌ Missing | |
| `Model.full_clean(exclude, validate_unique)` | ❌ Missing | |
| `Model.validate_unique(exclude)` | ❌ Missing | |
| `Model.date_error_message(lookup)` | ❌ Missing | |
| `Model.natural_key()` | ❌ Missing | |
| `class ModelBuilder` | ✅ Unique | Rjango's builder pattern for constructing model definitions |

### Model Meta
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Options` | `ModelMeta` | ✅ | Rjango has `ModelMeta` |
| `Options.db_table` | `ModelMeta::db_table()` | ✅ | |
| `Options.verbose_name` | `ModelMeta::verbose_name()` | ✅ | |
| `Options.verbose_name_plural` | ❌ Missing | |
| `Options.ordering` | `ModelMeta::ordering()` | ✅ | |
| `Options.unique_together` | ❌ Missing | |
| `Options.indexes` | ❌ Missing | |
| `Options.constraints` | ❌ Missing | |
| `Options.get_latest_by` | ❌ Missing | |
| `Options.permissions` | ❌ Missing | |
| `Options.default_permissions` | ❌ Missing | |
| `Options.managed` | ❌ Missing | |
| `Options.proxy` | ❌ Missing | |
| `Options.abstract` | ❌ Missing | |
| `Options.app_label` | ❌ Missing | |
| `Options.label` | ✅ | |
| `Options.label_lower` | ❌ Missing | |
| `Options.object_name` | ✅ | |
| `Options.module_name` | ❌ Missing | |

---

## 2. Fields (`django.db.models.fields`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Field` | `FieldType` enum / `Field` trait | ✅ | Present |
| `Field.null` | ✅ | |
| `Field.blank` | ✅ | |
| `Field.default` | `Field::default` | ✅ | |
| `Field.unique` | ❌ Missing | |
| `Field.db_index` | ❌ Missing | |
| `Field.db_column` | ❌ Missing | |
| `Field.primary_key` | ✅ | |
| `Field.choices` | ❌ Missing | |
| `Field.help_text` | ✅ | |
| `Field.verbose_name` | ✅ | |
| `Field.validators` | ✅ | |
| `Field.get_choices(include_blank)` | ❌ Missing | |
| `Field.value_from_object(obj)` | ❌ Missing | |
| `Field.value_to_string(obj)` | ❌ Missing | |
| `class AutoField(AutoFieldMixin, IntegerField)` | ✅ | Present |
| `class BigAutoField(AutoFieldMixin, BigIntegerField)` | ✅ | Present |
| `class SmallAutoField(AutoFieldMixin, SmallIntegerField)` | ❌ Missing | |
| `class CharField(Field)` | ✅ | Present |
| `CharField.max_length` | ✅ | |
| `CharField.widget_attrs` | ❌ Missing | |
| `class TextField(Field)` | ✅ | Present |
| `class IntegerField(Field)` | ✅ | Present |
| `class BigIntegerField(IntegerField)` | ❌ Missing | |
| `class SmallIntegerField(IntegerField)` | ❌ Missing | |
| `class PositiveIntegerField` | ❌ Missing | |
| `class PositiveBigIntegerField` | ❌ Missing | |
| `class PositiveSmallIntegerField` | ❌ Missing | |
| `class FloatField(Field)` | ✅ | Present |
| `class DecimalField(Field)` | ❌ Missing | |
| `class BooleanField(Field)` | ✅ | Present |
| `class NullBooleanField(BooleanField)` | ❌ Missing | |
| `class DateField(DateTimeCheckMixin, Field)` | ✅ | Present |
| `class DateTimeField(DateField)` | ✅ | Present |
| `class TimeField(DateTimeCheckMixin, Field)` | ❌ Missing | |
| `class DurationField(Field)` | ❌ Missing | |
| `class EmailField(CharField)` | ❌ Missing | |
| `class URLField(CharField)` | ❌ Missing | |
| `class SlugField(CharField)` | ❌ Missing | |
| `class IPAddressField(Field)` | ❌ Missing | |
| `class GenericIPAddressField(Field)` | ❌ Missing | |
| `class BinaryField(Field)` | ❌ Missing | |
| `class UUIDField(Field)` | ❌ Missing | |
| `class FilePathField(Field)` | ❌ Missing | |
| `class ImageField(FileField)` | ❌ Missing | |
| `class FileField(Field)` | ❌ Missing | |
| `class CommaSeparatedIntegerField(CharField)` | ❌ Missing | |
| `AutoFieldMixin` | ✅ | Present |

### Relationship Fields
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class ForeignKey(to, on_delete)` | `ForeignKey` | ✅ | Present |
| `ForeignKey.related_name` | ✅ | |
| `ForeignKey.related_query_name` | ❌ Missing | |
| `ForeignKey.to_field` | ❌ Missing | |
| `ForeignKey.db_constraint` | ❌ Missing | |
| `ForeignKey.on_delete` | ✅ | |
| `class OneToOneField(ForeignKey)` | `OneToOneField` | ✅ | Present |
| `class ManyToManyField(to)` | `ManyToManyField` | ✅ | Present |
| `ManyToManyField.through` | ❌ Missing | |
| `ManyToManyField.db_table` | ❌ Missing | |
| `ManyToManyField.symmetrical` | ❌ Missing | |
| `ManyToManyField.related_name` | ✅ | |
| `ManyToManyField.through_fields` | ❌ Missing | |

---

## 3. QuerySet (`django.db.models.query`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class QuerySet(model=None)` | `QuerySet` | ⚠️ Partial | Rjango has basic query capabilities |
| `QuerySet.filter(**kwargs)` | `QuerySet::filter()` | ✅ | |
| `QuerySet.exclude(**kwargs)` | ❌ Missing | |
| `QuerySet.annotate(**kwargs)` | ❌ Missing | |
| `QuerySet.order_by(*fields)` | `QuerySet::order_by()` | ✅ | |
| `QuerySet.reverse()` | ❌ Missing | |
| `QuerySet.distinct(*fields)` | ❌ Missing | |
| `QuerySet.values(*fields)` | ❌ Missing | |
| `QuerySet.values_list(*fields, flat, named)` | ❌ Missing | |
| `QuerySet.dates(field, kind, order)` | ❌ Missing | |
| `QuerySet.datetimes(field, kind, order, tzinfo)` | ❌ Missing | |
| `QuerySet.none()` | ❌ Missing | |
| `QuerySet.select_related(*fields)` | ❌ Missing | |
| `QuerySet.prefetch_related(*lookups)` | ❌ Missing | |
| `QuerySet.defer(*fields)` | ❌ Missing | |
| `QuerySet.only(*fields)` | ❌ Missing | |
| `QuerySet.using(alias)` | ❌ Missing | |
| `QuerySet.get(**kwargs)` | ❌ Missing | |
| `QuerySet.create(**kwargs)` | ❌ Missing | |
| `QuerySet.get_or_create(...)` | ❌ Missing | |
| `QuerySet.update_or_create(...)` | ❌ Missing | |
| `QuerySet.count()` | ❌ Missing | |
| `QuerySet.in_bulk(id_list)` | ❌ Missing | |
| `QuerySet.iterator(chunk_size)` | ❌ Missing | |
| `QuerySet.latest(*fields)` | ❌ Missing | |
| `QuerySet.earliest(*fields)` | ❌ Missing | |
| `QuerySet.first()` | ❌ Missing | |
| `QuerySet.last()` | ❌ Missing | |
| `QuerySet.aggregate(**kwargs)` | ❌ Missing | |
| `QuerySet.exists()` | ❌ Missing | |
| `QuerySet.update(**kwargs)` | ❌ Missing | |
| `QuerySet.delete()` | ❌ Missing | |
| `QuerySet.as_manager()` | ❌ Missing | |
| `QuerySet.bulk_create(objs)` | ❌ Missing | |
| `QuerySet.bulk_update(objs, fields)` | ❌ Missing | |
| `QuerySet.union(*other_qs)` | ❌ Missing | |
| `QuerySet.intersection(*other_qs)` | ❌ Missing | |
| `QuerySet.difference(*other_qs)` | ❌ Missing | |
| `class Prefetch(lookup, queryset, to_attr)` | ❌ Missing | |
| `class RawQuerySet` | ❌ Missing | |
| `class EmptyQuerySet` | ❌ Missing | |

### Lookups / Q Expressions
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Q` | `Q` | ✅ | AND, OR, NOT |
| `class F` | ❌ Missing | Field references |
| `class Subquery` | ❌ Missing | |
| `class Exists` | ❌ Missing | |
| `lookups: exact, iexact` | ❌ Missing | |
| `lookups: contains, icontains` | ❌ Missing | |
| `lookups: in` | ❌ Missing | |
| `lookups: gt, gte, lt, lte` | ❌ Missing | |
| `lookups: startswith, istartswith` | ❌ Missing | |
| `lookups: endswith, iendswith` | ❌ Missing | |
| `lookups: range` | ❌ Missing | |
| `lookups: date, year, month, day` | ❌ Missing | |
| `lookups: week, week_day, quarter` | ❌ Missing | |
| `lookups: isnull` | ❌ Missing | |
| `lookups: regex, iregex` | ❌ Missing | |
| `lookups: search` | ❌ Missing | |
| `class Lookup` | ❌ Missing | Base lookup class |
| `class Transform` | ❌ Missing | |

---

## 4. Aggregates (`django.db.models.aggregates`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Aggregate(Func)` | `Aggregate` enum | ⚠️ Partial | Rjango has enum, Django has full SQL function |
| `class Avg` | ✅ | |
| `class Count` | ✅ | |
| `class Sum` | ✅ | |
| `class Min` | ✅ | |
| `class Max` | ✅ | |
| `class StdDev` | ❌ Missing | |
| `class Variance` | ❌ Missing | |
| `class AnyValue` | ❌ Missing | |
| `class StringAgg` | ❌ Missing | |

---

## 5. Expressions (`django.db.models.expressions`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Expression` | `Expression` | ⚠️ Partial | Rjango has basic expression |
| `class F(Expression)` | ❌ Missing | |
| `class Value(Expression)` | ❌ Missing | |
| `class Func(Expression)` | ❌ Missing | |
| `class OrderBy(Expression)` | ❌ Missing | |
| `class Window(Expression)` | ❌ Missing | |
| `class Subquery(Expression)` | ❌ Missing | |
| `class Exists(Subquery)` | ❌ Missing | |
| `class OuterRef(ResolvedOuterRef)` | ❌ Missing | |
| `class CombinedExpression` | ❌ Missing | |
| `class Case(Expression)` | ❌ Missing | |
| `class When(Expression)` | ❌ Missing | |
| `class SubquerySum` | ❌ Missing | |
| `class Random(Expression)` | ❌ Missing | |

### Database Functions
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `django.db.models.functions` module | ❌ Missing | |
| `Coalesce` | ❌ Missing | |
| `Concat` | ❌ Missing | |
| `ConcatPair` | ❌ Missing | |
| `Greatest` | ❌ Missing | |
| `Least` | ❌ Missing | |
| `Length` | ❌ Missing | |
| `Lower` | ❌ Missing | |
| `Upper` | ❌ Missing | |
| `Trim` | ❌ Missing | |
| `LTrim` / `RTrim` | ❌ Missing | |
| `Substr` | ❌ Missing | |
| `Replace` | ❌ Missing | |
| `Reverse` | ❌ Missing | |
| `StrIndex` | ❌ Missing | |
| `Now` | ❌ Missing | |
| `Extract` | ❌ Missing | |
| `ExtractYear` / `ExtractMonth` / etc. | ❌ Missing | |
| `Trunc` family | ❌ Missing | |
| `JSONObject` | ❌ Missing | |
| `JSONBAgg` | ❌ Missing | |
| `Md5` | ❌ Missing | |
| `Sha1` / `Sha256` / `Sha512` | ❌ Missing | |
| `CRC32` | ❌ Missing | |
| `Cast` | ❌ Missing | |
| `Collate` | ❌ Missing | |
| `Sign` / `Abs` / `Round` / `Ceil` / `Floor` | ❌ Missing | |
| `Power` / `Sqrt` | ❌ Missing | |
| `Ln` / `Log` / `Mod` | ❌ Missing | |
| `Pi` / `Exp` / `Sin` / `Cos` / `Tan` | ❌ Missing | |
| `Acos` / `Asin` / `Atan` / `Atan2` | ❌ Missing | |

---

## 6. Managers (`django.db.models.manager`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Manager` | `Manager` trait | ✅ | Present |
| `Manager.get_queryset()` | ❌ Missing | |
| `Manager.from_queryset(QuerySet)` | ❌ Missing | |
| `Manager.queryset_only` | ❌ Missing | |
| `class BaseManager.from_queryset(QuerySet)` | ❌ Missing | |

---

## 7. Constraints & Indexes

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Index(fields, name, db_tablespace)` | ❌ Missing | |
| `Index.opclasses` | ❌ Missing | |
| `Index.condition` | ❌ Missing | |
| `Index.include` | ❌ Missing | |
| `class CheckConstraint(condition, name)` | ❌ Missing | |
| `class UniqueConstraint(fields, name)` | ❌ Missing | |
| `UniqueConstraint.condition` | ❌ Missing | |
| `UniqueConstraint.deferrable` | ❌ Missing | |
| `UniqueConstraint.include` | ❌ Missing | |
| `UniqueConstraint.opclasses` | ❌ Missing | |
| `UniqueConstraint.nulls_distinct` | ❌ Missing | |
| `class Deferrable` | ❌ Missing | |

---

## 8. Database Backends (`django.db.backends`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseDatabaseWrapper` | `Backend` trait | ✅ | Present |
| `BaseDatabaseWrapper.cursor()` | ❌ Missing | |
| `BaseDatabaseWrapper.execute(sql, params)` | ❌ Missing | |
| `BaseDatabaseWrapper.create_cursor()` | ❌ Missing | |
| `SQLite backend` | `SqliteBackend` | ✅ | Present |
| `PostgreSQL backend` | `PostgresBackend` | ✅ | Present |
| `MySQL backend` | `MySqlBackend` | ✅ | Present |
| `Oracle backend` | ❌ Missing | |
| `BaseDatabaseIntrospection` | ❌ Missing | |
| `BaseDatabaseOperations` | ❌ Missing | |
| `BaseDatabaseSchemaEditor` | ❌ Missing | |
| `BaseDatabaseClient` | ❌ Missing | |
| `BaseDatabaseCreation` | ❌ Missing | |
| `BaseDatabaseFeatures` | ❌ Missing | |

---

## 9. Migration Operations (`django.db.migrations`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class CreateModel(name, fields)` | CreateModel | ✅ | Present |
| `class DeleteModel(name)` | DeleteModel | ✅ | Present |
| `class RenameModel(old, new)` | RenameModel | ✅ | Present |
| `class AlterModelTable(name, table)` | ❌ Missing | |
| `class AlterModelTableComment` | ❌ Missing | |
| `class AlterUniqueTogether` | ❌ Missing | |
| `class AlterIndexTogether` | ❌ Missing | |
| `class AlterOrderWithRespectTo` | ❌ Missing | |
| `class AlterModelOptions` | ❌ Missing | |
| `class AlterModelManagers` | ❌ Missing | |
| `class AddField(model, name, field)` | AddField | ✅ | Present |
| `class RemoveField(model, name)` | RemoveField | ✅ | Present |
| `class AlterField(model, name, field)` | AlterField | ✅ | Present |
| `class RenameField(model, old, new)` | RenameField | ✅ | Present |
| `class AddIndex(model, index)` | ❌ Missing | |
| `class RemoveIndex(model, name)` | ❌ Missing | |
| `class RenameIndex(model, old, new)` | ❌ Missing | |
| `class AddConstraint(model, constraint)` | ❌ Missing | |
| `class RemoveConstraint(model, name)` | ❌ Missing | |
| `class AlterConstraint(model, name, constraint)` | ❌ Missing | |
| `class RunSQL(sql, reverse_sql)` | ❌ Missing | |
| `class RunPython(code, reverse_code)` | ❌ Missing | |
| `class SeparateDatabaseAndState` | ❌ Missing | |
| `Migration.operations` | ✅ | |
| `Migration.dependencies` | ✅ | |
| `MigrationAtomic` | ❌ Missing | |

### Migration Framework
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Migration` | `Migration` | ✅ | Present |
| `class MigrationAutodetector` | `MigrationDetector` | ✅ | Present |
| `MigrationAutodetector.changes()` | `MigrationDetector::detect_changes()` | ✅ | |
| `class MigrationExecutor` | ❌ Missing | |
| `class MigrationLoader` | ❌ Missing | |
| `class MigrationGraph` | ❌ Missing | |
| `class MigrationRecorder` | ❌ Missing | |
| `class MigrationWriter` | ❌ Missing | |
| `class MigrationQuestioner` | ❌ Missing | |
| `class MigrationSerializer` | ❌ Missing | |
| `class State` | ❌ Missing | |

---

## 10. Enums & Choices

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class TextChoices(str, enum.Enum)` | ❌ Missing | |
| `class IntegerChoices(int, enum.Enum)` | ❌ Missing | |

---

## 11. Deletion

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `CASCADE` | ✅ | Present |
| `PROTECT` | ✅ | |
| `RESTRICT` | ❌ Missing | |
| `SET_NULL` | ✅ | |
| `SET_DEFAULT` | ✅ | |
| `SET(value)` | ❌ Missing | |
| `DO_NOTHING` | ✅ | |
| `class Collector` | ❌ Missing | |

---

## Summary

### Rjango ORM Features (✅ Complete)
- ✅ `Q` expression builder (AND, OR, NOT)
- ✅ Basic aggregates (Sum, Count, Avg, Min, Max)
- ✅ Basic field types (Auto, Char, Text, Integer, Float, Boolean, DateTime, Date, ForeignKey, OneToOne, ManyToMany)
- ✅ Backend trait with SQLite/PostgreSQL/MySQL implementations
- ✅ Migration detection + 6 basic operations
- ✅ Manager trait
- ✅ `Field` trait with validation basics

### Rjango ORM (⚠️ Partial)
- ⚠️ Model registration (builder pattern vs Django metaclass)
- ⚠️ QuerySet (has filter/order_by, missing get/create/delete/update/exists)
- ⚠️ Expressions (basic, missing F, Value, Subquery, Case/When)

### Missing Django ORM Features (❌)
- ❌ Full `Model.save()/delete()` — no real DB persistence
- ❌ 15+ field types missing (Decimal, Duration, UUID, IP, File, Image, etc.)
- ❌ All lookup types (contains, gt/gte, lt/lte, in, range, regex, isnull, etc.)
- ❌ `F` expressions (field references in queries)
- ❌ `Prefetch` and `select_related` / `prefetch_related`
- ❌ `values()` / `values_list()` query results
- ❌ `bulk_create()` / `bulk_update()`
- ❌ `select_for_update()`
- ❌ `defer()` / `only()`
- ❌ `annotate()` with expressions
- ❌ Database functions module (30+ functions)
- ❌ Window functions
- ❌ Constraints and Indexes
- ❌ `unique_together` in Meta
- ❌ `proxy` and `abstract` model inheritance
- ❌ `choices` on fields
- ❌ Custom model validators
- ❌ Full migration framework (executor, loader, writer, recorder)
- ❌ Migration operations: RunSQL, RunPython, Index/Constraint operations
- ❌ Oracle database backend
- ❌ Database schema editor
- ❌ Enum/Choice classes (TextChoices, IntegerChoices)
