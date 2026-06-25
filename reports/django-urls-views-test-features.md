# Django 6.0.6 URLs, Views, Test & Management — Feature Comparison with Rjango

> **Django modules analyzed:** `django.urls`, `django.views`, `django.test`, `django.core.management`, `django.dispatch`

---

## 1. URL Routing (`django.urls`)

### Functions
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `path(route, view, kwargs, name)` | `register_route()` | ✅ | Present |
| `re_path(route, view, kwargs, name)` | ❌ Missing | Regex-based paths |
| `include(arg, namespace)` | `include()` | ✅ | Present |
| `reverse(viewname, args, kwargs)` | `reverse()` | ❌ Missing | Not implemented |
| `reverse_lazy(viewname, args, kwargs)` | ❌ Missing | Lazy version |
| `resolve(path, urlconf)` | `resolve_url()` | ✅ | Present |
| `clear_url_caches()` | ❌ Missing | |
| `set_script_prefix(prefix)` | ❌ Missing | |
| `get_script_prefix()` | ❌ Missing | |
| `is_valid_path(path)` | ❌ Missing | |
| `translate_url(url, lang_code)` | ❌ Missing | |

### URL Patterns & Resolvers
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class URLPattern(pattern, view, name)` | `Route` / `UrlPattern` | ✅ | Present |
| `class URLResolver(pattern, view, namespace)` | `URLResolver` | ✅ | Present |
| `class RoutePattern(route)` | ❌ Missing | Modern path-based pattern |
| `class RegexPattern(regex)` | ❌ Missing | Old regex-based pattern |
| `class ResolverMatch(func, args, kwargs, ...)` | ❌ Missing | Result of resolve() |
| `class LocalePrefixPattern` | ❌ Missing | |
| `get_resolver(urlconf)` | ❌ Missing | |

### URL Converters
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class IntConverter` | `IntConverter` | ✅ | |
| `class StringConverter` | `StringConverter` | ✅ | |
| `class UUIDConverter` | `UUIDConverter` | ✅ | |
| `class SlugConverter(StringConverter)` | `SlugConverter` | ✅ | |
| `class PathConverter(StringConverter)` | ❌ Missing | Matches any path including / |
| `register_converter(converter, type_name)` | `register_converter()` | ✅ | |
| `get_converters()` | ❌ Missing | |

### URL Conf
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `module-level urlpatterns` | ❌ Missing | Django uses module-level lists |
| `app_name in urlconf` | ❌ Missing | URL namespacing by app |

---

## 2. Generic Views (`django.views.generic`)

### Base Views
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class View` | `View` | ✅ | |
| `View.as_view()` | ❌ Missing | Class-based view entry point |
| `View.dispatch(request, *args, **kwargs)` | ✅ | |
| `View.setup(request, *args, **kwargs)` | ❌ Missing | |
| `View.http_method_not_allowed(request)` | ❌ Missing | |
| `View.options(request)` | ❌ Missing | |
| `class ContextMixin` | ❌ Missing | |
| `ContextMixin.get_context_data(**kwargs)` | ❌ Missing | |
| `class TemplateResponseMixin` | `TemplateView` | ❌ Missing | Not a separate mixin |
| `TemplateResponseMixin.template_name` | ✅ | |
| `TemplateResponseMixin.render_to_response(context)` | ❌ Missing | |
| `class RedirectView(View)` | ❌ Missing | |

### Template View
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class TemplateView(TemplateResponseMixin, View)` | `TemplateView` | ⚠️ Partial | Rjango has TemplateView but simpler |
| `TemplateView.get(request, *args, **kwargs)` | ✅ | |
| `TemplateView.get_context_data(**kwargs)` | ❌ Missing | |

### List View
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class ListView` | `ListView` | ⚠️ Partial | |
| `class MultipleObjectMixin(ContextMixin)` | ❌ Missing | |
| `MultipleObjectMixin.model` | ✅ | |
| `MultipleObjectMixin.queryset` | ❌ Missing | |
| `MultipleObjectMixin.paginate_by` | ❌ Missing | |
| `MultipleObjectMixin.get_queryset()` | ❌ Missing | |
| `MultipleObjectMixin.get_context_data(**kwargs)` | ❌ Missing | |
| `class MultipleObjectTemplateResponseMixin` | ❌ Missing | |

### Detail View
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class DetailView` | `DetailView` | ⚠️ Partial | |
| `class SingleObjectMixin(ContextMixin)` | ❌ Missing | |
| `SingleObjectMixin.get_object(queryset)` | ❌ Missing | |
| `SingleObjectMixin.get_context_data(**kwargs)` | ❌ Missing | |
| `class SingleObjectTemplateResponseMixin` | ❌ Missing | |

### Form Views
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class FormView` | `FormView` | ⚠️ Partial | |
| `class FormMixin(ContextMixin)` | ❌ Missing | |
| `FormMixin.get_form(form_class)` | ❌ Missing | |
| `FormMixin.get_success_url()` | ❌ Missing | |
| `FormMixin.form_valid(form)` | ❌ Missing | |
| `FormMixin.form_invalid(form)` | ❌ Missing | |
| `class ProcessFormView(View)` | ❌ Missing | |

### Create/Update/Delete Views
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class CreateView` | `CreateView` | ⚠️ Partial | |
| `class BaseCreateView(ModelFormMixin, ProcessFormView)` | ❌ Missing | |
| `class UpdateView` | `UpdateView` | ⚠️ Partial | |
| `class BaseUpdateView(ModelFormMixin, ProcessFormView)` | ❌ Missing | |
| `class DeleteView` | `DeleteView` | ⚠️ Partial | |
| `class DeletionMixin` | ❌ Missing | |
| `class ModelFormMixin(FormMixin, SingleObjectMixin)` | ❌ Missing | |

### Date-Based Views
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class ArchiveIndexView` | ❌ Missing | |
| `class YearArchiveView` | ❌ Missing | |
| `class MonthArchiveView` | ❌ Missing | |
| `class WeekArchiveView` | ❌ Missing | |
| `class DayArchiveView` | ❌ Missing | |
| `class TodayArchiveView` | ❌ Missing | |
| `class DateDetailView` | ❌ Missing | |
| All date mixins (YearMixin, MonthMixin, DayMixin, WeekMixin, DateMixin) | ❌ Missing | |

### View Decorators
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `require_http_methods(methods)` | `require_http_methods()` | ✅ | |
| `require_GET()` | `require_get()` | ✅ | |
| `require_POST()` | `require_post()` | ✅ | |
| `require_safe()` | ❌ Missing | |
| `gzip_page()` | ❌ Missing | |
| `never_cache()` | ❌ Missing | |
| `csrf_exempt()` | ❌ Missing | |
| `csrf_protect()` | ❌ Missing | |
| `csrf_input()` | ❌ Missing | |

---

## 3. Test Framework (`django.test`)

### Test Client
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Client` | `TestClient` | ⚠️ Partial | |
| `Client.get(path, data)` | ✅ | |
| `Client.post(path, data)` | ✅ | |
| `Client.head(path, data)` | ❌ Missing | |
| `Client.options(path, data)` | ❌ Missing | |
| `Client.put(path, data, content_type)` | ❌ Missing | |
| `Client.patch(path, data, content_type)` | ❌ Missing | |
| `Client.delete(path, data)` | ❌ Missing | |
| `Client.trace(path)` | ❌ Missing | |
| `Client.login(username, password)` | ❌ Missing | |
| `Client.logout()` | ❌ Missing | |
| `Client.force_login(user)` | ❌ Missing | |
| `Client.session` | ❌ Missing | |
| `Client.handler` | ❌ Missing | |
| `class AsyncClient` | ❌ Missing | |
| `class RequestFactory` | ❌ Missing | |
| `class AsyncRequestFactory` | ❌ Missing | |

### Test Cases
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class SimpleTestCase(unittest.TestCase)` | `RjangoTestCase` | ⚠️ Partial | |
| `SimpleTestCase.assertContains(response, text)` | ❌ Missing | |
| `SimpleTestCase.assertNotContains(response, text)` | ❌ Missing | |
| `SimpleTestCase.assertTemplateUsed(response)` | ❌ Missing | |
| `SimpleTestCase.assertTemplateNotUsed(response)` | ❌ Missing | |
| `SimpleTestCase.assertRaisesMessage()` | ❌ Missing | |
| `SimpleTestCase.assertFieldOutput()` | ❌ Missing | |
| `SimpleTestCase.assertHTMLEqual()` | ❌ Missing | |
| `SimpleTestCase.assertHTMLNotEqual()` | ❌ Missing | |
| `SimpleTestCase.assertInHTML()` | ❌ Missing | |
| `SimpleTestCase.assertJSONEqual()` | ❌ Missing | |
| `SimpleTestCase.assertJSONNotEqual()` | ❌ Missing | |
| `SimpleTestCase.assertURLEqual()` | ❌ Missing | |
| `class TransactionTestCase(SimpleTestCase)` | ❌ Missing | |
| `class TestCase(TransactionTestCase)` | ❌ Missing | |
| `TestCase.setUpTestData()` | ❌ Missing | |
| `TestCase.fixtures` | ❌ Missing | |
| `class LiveServerTestCase` | ❌ Missing | |
| `skipIfDBFeature(*features)` | ❌ Missing | |
| `skipUnlessDBFeature(*features)` | ❌ Missing | |

### Test Runner
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class DiscoverRunner` | `TestRunner` | ⚠️ Partial | |
| `DiscoverRunner.setup_test_environment()` | ❌ Missing | |
| `DiscoverRunner.build_suite()` | ❌ Missing | |
| `DiscoverRunner.run_tests(test_labels)` | ✅ | |
| `DiscoverRunner.setup_databases()` | ❌ Missing | |
| `DiscoverRunner.teardown_databases()` | ❌ Missing | |
| `DiscoverRunner.run_checks()` | ❌ Missing | |
| `DiscoverRunner.suite_result()` | ❌ Missing | |
| `ParallelTestSuite` | ❌ Missing | |
| `Shuffler` | ❌ Missing | |
| `DebugSQLTextTestResult` | ❌ Missing | |
| `PDBDebugResult` | ❌ Missing | |

### Selenium Tests
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class SeleniumTestCase` | ❌ Missing | |
| `StaticLiveServerTestCase` | ❌ Missing | |

---

## 4. Management Commands (`django.core.management`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `call_command(name, *args, **options)` | ❌ Missing | |
| `class BaseCommand` | ❌ Missing | |
| `BaseCommand.handle(*args, **options)` | ❌ Missing | |
| `BaseCommand.add_arguments(parser)` | ❌ Missing | |
| `BaseCommand.style` | ❌ Missing | |
| `Command.handle()` | ❌ Missing | |
| `command: runserver` | ✅ | |
| `command: startproject` | ✅ | |
| `command: startapp` | ✅ | |
| `command: migrate` | ✅ | |
| `command: makemigrations` | ✅ | |
| `command: showmigrations` | ✅ | |
| `command: shell` | ✅ | |
| `command: dbshell` | ✅ | |
| `command: createsuperuser` | ✅ | |
| `command: collectstatic` | ✅ | |
| `command: test` | ✅ | |
| `command: check` | ✅ | |
| `command: validate` | ✅ | |
| `command: dumpdata` | ❌ Missing | |
| `command: loaddata` | ❌ Missing | |
| `command: inspectdb` | ❌ Missing | |
| `command: sqlmigrate` | ❌ Missing | |
| `command: sqlflush` | ❌ Missing | |
| `command: sqlsequencereset` | ❌ Missing | |
| `command: squashmigrations` | ❌ Missing | |
| `command: optimizemigration` | ❌ Missing | |
| `command: flush` | ❌ Missing | |
| `command: diffsettings` | ❌ Missing | |
| `command: sendtestemail` | ❌ Missing | |
| `command: compilemessages` | ❌ Missing | i18n compilation |
| `command: makemessages` | ❌ Missing | i18n string extraction |
| `command: testserver` | ❌ Missing | |
| `command: createcachetable` | ❌ Missing | |

---

## 5. Server (`django.core.servers`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `basehttp.WSGIServer` | ❌ Missing | |
| `basehttp.ServerHandler` | ❌ Missing | |
| `basehttp.WSGIRequestHandler` | ❌ Missing | |
| `class Server(HTTPServer)` | `Server` | ⚠️ Partial | Rjango has basic HTTP server |

---

## Summary

### Rjango URL/Views/Test Features (✅ Complete)
- ✅ Path converters (int, str, slug, uuid)
- ✅ URL routing with include()
- ✅ Basic View/class-based views
- ✅ TemplateView, ListView, DetailView, FormView, CreateView, UpdateView, DeleteView
- ✅ HTTP method decorators (require_http_methods, require_GET, require_POST)
- ✅ TestClient with GET/POST
- ✅ TestCase base class
- ✅ TestRunner
- ✅ 13 management commands

### Missing Features (❌)
- ❌ `reverse()` URL resolver — **critical gap**
- ❌ Regex-based URL patterns
- ❌ `re_path()` support
- ❌ Date-based views (archive index, year/month/day/week archives)
- ❌ RedirectView
- ❌ Full ModelFormMixin (no DB-backed view operations)
- ❌ DeletionMixin
- ❌ Full assertion helpers (assertContains, assertTemplateUsed, etc.)
- ❌ Async client
- ❌ Transaction/live server test cases
- ❌ Parallel test execution
- ❌ Selenium tests
- ❌ BaseCommand framework with argument parsing
- ❌ dumpdata/loaddata (serialization)
- ❌ inspectdb (database introspection)
- ❌ sqlmigrate/sqlflush/sqlsequencereset
- ❌ squashmigrations
- ❌ i18n commands (compilemessages, makemessages)
