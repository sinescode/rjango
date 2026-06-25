# Django vs Rjango: Templates & Forms — Exhaustive Comparison Report

**Date**: 2026-06-25 (Updated)  
**Django Version**: 6.0.6  
**Rjango Version**: 0.1.0  

---

## djano.template vs rjango-templates

Rjango Location: `rjango-templates/src/` (449 lines)

### Engine

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| Template engine | django.template | `TemplateEngine` struct | ✅ YES | render(), render_to_string() |
| FileSystemLoader | Load from dirs | `FileSystemLoader` | ✅ YES | |
| AppDirectoryLoader | Load from apps | `AppDirectoriesLoader` | ✅ YES | |
| Cached loader | Cache templates | — | ❌ NO | |
| DjangoTemplates backend | Backend config | — | ❌ NO | |
| Jinja2 backend | Alternative engine | — | ❌ NO | |

### Context

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| Context (dict-like) | Template vars | `TemplateContext` struct | ✅ YES | HashMap-based |
| Context processors | Auto-added vars | `processors` module | ✅ YES | default(), debug(), sql_queries() |
| RequestContext | Request-aware | — | ❌ NO | |
| csrf context processor | CSRF token | — | ❌ NO | |

### Filters

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `date` filter | Date formatting | — | ❌ NO | |
| `default` filter | Default value | — | ❌ NO | |
| `length` filter | Length | — | ❌ NO | |
| `upper` / `lower` | Case | — | ❌ NO | |
| `safe` filter | Mark safe | — | ❌ NO | |
| `escape` filter | HTML escape | — | ❌ NO | |
| `linebreaks` filter | Line formatting | — | ❌ NO | |
| `pluralize` filter | Plural suffix | — | ❌ NO | |
| Filter infrastructure | Filter registration | Basic filter pipeline | ⚠️ PARTIAL | ~137 lines in filters.rs |

### Tags

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `{% for %}` | Loop | — | ❌ NO | |
| `{% if %}` | Condition | — | ❌ NO | |
| `{% block %}` | Template blocks | — | ❌ NO | |
| `{% extends %}` | Inheritance | — | ❌ NO | |
| `{% include %}` | Include | — | ❌ NO | |
| `{% url %}` | URL reversal | — | ❌ NO | |
| `{% csrf_token %}` | CSRF tag | — | ❌ NO | |
| `{% load %}` | Tag library | — | ❌ NO | |
| `{% comment %}` | Comments | — | ❌ NO | |
| `{% static %}` | Static URL | — | ❌ NO | |
| Tag infrastructure | Tag registration | Basic tag parsing | ⚠️ PARTIAL | ~49 lines in tags.rs |

### Template Inheritance

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `{% extends %}` | Inheritance | — | ❌ NO | |
| `{% block %}` | Block override | — | ❌ NO | |
| `{{ block.super }}` | Parent content | — | ❌ NO | |
| Multiple inheritance | Chain extends | — | ❌ NO | |

### Summary

| AREA | STATUS | LOCATION | LINES |
|------|--------|----------|-------|
| Engine | ✅ YES | engine.rs | 69 |
| Loaders | ✅ YES | loaders.rs | 47 |
| Context | ⚠️ PARTIAL | context.rs | 59 |
| Processors | ✅ YES | processors.rs | 74 |
| Filters | ⚠️ PARTIAL | filters.rs | 137 |
| Tags | ⚠️ PARTIAL | tags.rs | 49 |
| Inheritance | ❌ NO | — | — |

---

## django.forms vs rjango-forms

Rjango Location: `rjango-forms/src/` (671 lines)

### Fields

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| CharField | Text input | `FieldType::CharField` | ✅ YES | |
| IntegerField | Number input | `FieldType::IntegerField` | ✅ YES | |
| BooleanField | Checkbox | `FieldType::BooleanField` | ✅ YES | |
| EmailField | Email input | `FieldType::EmailField` | ✅ YES | |
| URLField | URL input | `FieldType::URLField` | ✅ YES | |
| DateField | Date picker | `FieldType::DateField` | ✅ YES | |
| DateTimeField | DateTime | `FieldType::DateTimeField` | ✅ YES | |
| ChoiceField | Select | `FieldType::ChoiceField` | ✅ YES | |
| FileField | File upload | — | ❌ NO | |
| ImageField | Image upload | — | ❌ NO | |
| DecimalField | Decimal | `FieldType::DecimalField` | ✅ YES | |
| FloatField | Float | `FieldType::FloatField` | ✅ YES | |
| JSONField | JSON | — | ❌ NO | |
| RegexField | Pattern match | — | ❌ NO | |
| TypedChoiceField | Typed select | — | ❌ NO | |
| MultipleChoiceField | Multi select | — | ❌ NO | |
| SplitDateTimeField | Date + time | — | ❌ NO | |
| DurationField | Duration | — | ❌ NO | |
| GenericIPAddressField | IP address | — | ❌ NO | |
| SlugField | Slug | — | ❌ NO | |
| UUIDField | UUID | — | ❌ NO | |
| NullBooleanField | Nullable bool | — | ❌ NO | |

### Widgets

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| TextInput | Text input | Widget enum | ✅ YES | |
| Textarea | Text area | Widget enum | ✅ YES | |
| Select | Dropdown | Widget enum | ✅ YES | |
| CheckboxInput | Checkbox | Widget enum | ✅ YES | |
| RadioSelect | Radio buttons | — | ❌ NO | |
| SelectMultiple | Multi select | — | ❌ NO | |
| FileInput | File upload | — | ❌ NO | |
| DateInput | Date picket | — | ❌ NO | |
| DateTimeInput | DateTime picker | — | ❌ NO | |
| HiddenInput | Hidden field | — | ❌ NO | |
| EmailInput | Email input | — | ❌ NO | |
| URLInput | URL input | — | ❌ NO | |
| NumberInput | Number input | — | ❌ NO | |
| PasswordInput | Password | — | ❌ NO | |

### Form Class

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| Form class | Base form | `Form` struct | ✅ YES | |
| `is_valid()` | Validation | `is_valid()` | ✅ YES | |
| `cleaned_data` | Cleaned values | `cleaned_data()` | ✅ YES | |
| `render()` | HTML output | `render()` | ✅ YES | |
| `as_table()` / `as_p()` / `as_ul()` | Format variants | — | ❌ NO | |
| `errors` dict | Error messages | `FormErrors` struct | ✅ YES | |
| Field validation | Per-field | Validation pipeline | ✅ YES | |
| ModelForm | Form from model | — | ❌ NO | |
| FormSets | Multiple forms | — | ❌ NO | |
| Media (CSS/JS) | Form assets | — | ❌ NO | |

### Validation

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| Required validation | is_required | `required` flag | ✅ YES | |
| Min/max length | Length check | — | ❌ NO | |
| Regex validation | Pattern | — | ❌ NO | |
| Custom validators | Callable list | Validator trait | ✅ YES | |
| Clean methods | Per-field + form | — | ❌ NO | |
| Error messages | Custom msgs | Basic string | ✅ YES | |

### Summary

| AREA | STATUS | LOCATION | LINES |
|------|--------|----------|-------|
| Form struct + render | ✅ YES | lib.rs | 258 |
| Fields | ⚠️ PARTIAL | fields.rs | 140 |
| Validation | ✅ YES | validation.rs | 167 |
| Widgets | ⚠️ PARTIAL | widgets.rs | 36 |
| Rendering | ✅ YES | rendering.rs | 70 |
| ModelForm | ❌ NO | — | — |
| FormSets | ❌ NO | — | — |
