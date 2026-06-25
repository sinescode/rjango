# Django 6.0.6 Templates — Feature Comparison with Rjango

> **Django modules analyzed:** `django.template.engine`, `django.template.context`, `django.template.library`, `django.template.loader`, `django.template.defaultfilters`, `django.template.defaulttags`

---

## 1. Template Engine

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `Engine(dirs, app_dirs, context_processors, debug, loaders, string_if_invalid, file_charset, libraries, builtins, autoescape)` | `TemplateEngine` | ⚠️ Partial | Rjango has `TemplateEngine` but fewer options |
| `Engine.from_string(template_code)` | ✅ | |
| `Engine.get_template(template_name)` | ❌ Missing | |
| `Engine.render_to_string(template_name, context)` | ❌ Missing | |
| `Engine.dirs` | ❌ Missing | |
| `Engine.app_dirs` | ❌ Missing | |
| `Engine.debug` | ❌ Missing | |
| `Engine.loaders` | ❌ Missing | |
| `Engine.builtins` | ❌ Missing | |
| `class DjangoTemplates(BaseEngine)` | ❌ Missing | Backend class |
| `backends.django.DjangoTemplates` | ❌ Missing | Template backend integration |

### Backends
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseEngine` | ❌ Missing | |
| `class DjangoTemplates(BaseEngine)` | ❌ Missing | |
| `class Jinja2(BaseEngine)` | ❌ Missing | |

---

## 2. Template Context

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Context(dict)` | `Context` / `RenderContext` | ⚠️ Partial | Rjango has basic context |
| `Context.push()` / `Context.pop()` | ❌ Missing | Context stack |
| `Context.update(other_dict)` | ❌ Missing | |
| `Context.flatten()` | ❌ Missing | |
| `class RequestContext(Context)` | ❌ Missing | Automatically includes request + context processors |
| `RequestContext.request` | ❌ Missing | |
| `RequestContext.processors` | ❌ Missing | |
| `class RenderContext(BaseContext)` | ❌ Missing | |
| `make_context(context, request)` | ❌ Missing | |
| `class ContextPopException` | ❌ Missing | |

---

## 3. Template Loaders & Loader

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `get_template(template_name)` | ✅ | Present |
| `select_template(template_name_list)` | ✅ | Present |
| `render_to_string(template_name, context, request, using)` | ✅ | Present |
| `class filesystem.Loader` | `FileSystemLoader` | ✅ | Present |
| `class app_directories.Loader` | `AppDirectoriesLoader` | ✅ | Present |
| `class cached.Loader` | ❌ Missing | |
| `class locmem.Loader` | ❌ Missing | |

---

## 4. Template Tags (Default Tags)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `{% autoescape on/off %}` | `AutoEscapeControlNode` | ✅ | Present |
| `{% comment %}` | `CommentNode` | ✅ | Present |
| `{% csrf_token %}` | `CsrfTokenNode` | ❌ Missing | |
| `{% cycle %}` | `CycleNode` | ✅ | Present |
| `{% debug %}` | ❌ Missing | |
| `{% filter %}` | `FilterNode` | ✅ | Present |
| `{% firstof %}` | `FirstOfNode` | ✅ | Present |
| `{% for %}` | `ForNode` | ✅ | Present |
| `{% for ... empty %}` | ForNode with empty | ✅ | Present |
| `{% if %}` / `{% elif %}` / `{% else %}` | `IfNode` | ✅ | Present |
| `{% ifchanged %}` | ❌ Missing | |
| `{% lorem %}` | ❌ Missing | |
| `{% now %}` | `NowNode` | ✅ | Present |
| `{% load %}` | `LoadNode` | ✅ | Present |
| `{% regroup %}` | `RegroupNode` | ✅ | Present |
| `{% resetcycle %}` | ❌ Missing | |
| `{% spaceless %}` | `SpacelessNode` | ✅ | Present |
| `{% templatetag %}` | ❌ Missing | |
| `{% url %}` | ❌ Missing | URL reversing in templates |
| `{% verbatim %}` | `VerbatimNode` | ✅ | Present |
| `{% widthratio %}` | `WidthRatioNode` | ✅ | Present |
| `{% with %}` | `WithNode` | ✅ | Present |
| `{% block %}` | `BlockNode` | ✅ | Present (in engine parser) |
| `{% extends %}` | `ExtendsNode` | ✅ | Present |
| `{% include %}` | ❌ Missing | |
| `{% ssi %}` | ❌ Missing | Deprecated in Django |
| `{% partialdef %}` / `{% partial %}` | ❌ Missing | Django 5.0+ feature |

---

## 5. Template Filters (Default Filters)

**Total Django filters: 60+** | **Rjango: 44 filters**

### String & Text Filters
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `addslashes` | ✅ | |
| `capfirst` | ✅ | |
| `center` | ✅ | |
| `cut` | ✅ | |
| `escape` | ✅ | |
| `escapejs` | ❌ Missing | |
| `first` | ✅ | |
| `join` | ✅ | |
| `last` | ✅ | |
| `length` | ✅ | |
| `linenumbers` | ✅ | |
| `linebreaks` | ✅ | |
| `linebreaksbr` | ✅ | |
| `ljust` | ✅ | |
| `rjust` | ✅ | |
| `lower` | ✅ | |
| `make_list` | ✅ | |
| `slugify` | ✅ | |
| `stringformat` | ✅ | |
| `striptags` | ✅ | |
| `title` | ✅ | |
| `truncatechars` | ✅ | |
| `truncatechars_html` | ❌ Missing | |
| `truncatewords` | ✅ | |
| `truncatewords_html` | ❌ Missing | |
| `upper` | ✅ | |
| `wordcount` | ✅ | |
| `wordwrap` | ❌ Missing | |
| `yesno` | ✅ | |

### HTML & URL Filters
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `iriencode` | ❌ Missing | |
| `json_script` | ❌ Missing | |
| `safe` | ✅ | |
| `safeseq` | ❌ Missing | |
| `urlencode` | ✅ | |
| `urlize` | ✅ | |
| `urlizetrunc` | ❌ Missing | |

### Number & Math Filters
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `add` | ✅ | |
| `divisibleby` | ✅ | |
| `filesizeformat` | ✅ | |
| `floatformat` | ✅ | |
| `get_digit` | ❌ Missing | |
| `phone2numeric` | ✅ | |
| `pluralize` | ✅ | |
| `random` | ✅ | |

### Date & Time Filters
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `date` | ✅ | |
| `time` | ✅ | |
| `timesince` | ✅ | |
| `timeuntil` | ✅ | |

### List & Collection Filters
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `dictsort` | ✅ | |
| `dictsortreversed` | ❌ Missing | |
| `slice` | ✅ | |
| `unordered_list` | ✅ | |

### Other Filters
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `default` | ✅ | |
| `default_if_none` | ❌ Missing | |
| `escapejs` | ❌ Missing | |
| `escapeseq` | ❌ Missing | |
| `force_escape` | ❌ Missing | |
| `pprint` | ✅ | |
| `urlize` | ✅ | |
| `urlizetrunc` | ❌ Missing | |

---

## 6. Template Library & Registration

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Library` | ✅ | Present in `tags.rs` with register_trait/filter/tag |
| `Library.filter(name, func)` | ✅ | |
| `Library.tag(name, compile_function)` | ✅ | |
| `Library.simple_tag(func)` | ✅ | |
| `Library.inclusion_tag(func)` | ❌ Missing | |
| `import_library(name)` | ❌ Missing | |
| `class TagHelperNode` | ❌ Missing | |
| `class SimpleNode` | ❌ Missing | |
| `class InclusionNode` | ❌ Missing | |
| `parse_bits(parser, bits, params, ...)` | ❌ Missing | |

---

## Summary

### Rjango Templates Features (✅ Complete)
- ✅ Engine with template parsing
- ✅ 44 of 60+ Django filters (73%)
- ✅ 20 of 28 template tags (71%)
- ✅ Context, loaders, tag/filter registration
- ✅ Block/Extends template inheritance
- ✅ Auto-escaping support

### Missing Template Features (❌)
- ❌ 16 filters: escapejs, truncatechars_html, truncatewords_html, wordwrap, safeseq, urlizetrunc, dictsortreversed, json_script, iriencode, escapeseq, force_escape, default_if_none, get_digit
- ❌ 8 tags: csrf_token, debug, ifchanged, lorem, resetcycle, templatetag, url, partialdef/partial
- ❌ RequestContext with automatic context processors
- ❌ Jinja2 backend
- ❌ smartif expression engine
- ❌ Inclusion tags
- ❌ `{% include %}` tag
