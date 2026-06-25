# Django 6.0.6 Core — Feature Comparison with Rjango

> **Django modules analyzed:** `django.core`, `django.apps`, `django.conf`, `django.dispatch`, `django.http`, `django.shortcuts`, `django.utils`

---

## 1. `django.core.paginator`

### Django Source
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BasePaginator` | `rjango_core::Paginator` | ⚠️ Partial | Rjango has only one Paginator class |
| `Paginator(object_list, per_page, orphans=0, allow_empty_first_page=True)` | `Paginator::new(items, per_page)` | ❌ Partial | Missing `orphans`, `allow_empty_first_page`, `error_messages` params |
| `Paginator.count` | `Paginator::count()` | ✅ | Present |
| `Paginator.num_pages` | `Paginator::num_pages()` | ✅ | Present |
| `Paginator.page_range` | `Paginator::page_range()` | ✅ | Present |
| `Paginator.page(number)` | `Paginator::page(number)` | ❌ Missing | Not implemented in current Rjango |
| `Paginator.get_page(number)` | ❌ Missing | Returns valid page even when out of range |
| `Paginator.get_elided_page_range()` | ❌ Missing | `[1, 2, …, 40, 41, …, 49, 50]` |
| `Paginator.validate_number(number)` | ❌ Missing | Validation logic |
| `class Page(collections.abc.Sequence)` | ❌ Missing | Page object with `has_next()`, `has_previous()`, `next_page_number()`, etc. |
| `Page.has_next()` | ❌ Missing | |
| `Page.has_previous()` | ❌ Missing | |
| `Page.has_other_pages()` | ❌ Missing | |
| `Page.next_page_number()` | ❌ Missing | |
| `Page.previous_page_number()` | ❌ Missing | |
| `Page.start_index()` | ❌ Missing | 1-based index of first item |
| `Page.end_index()` | ❌ Missing | 1-based index of last item |
| `Page.object_list` | ❌ Missing | The items on this page |
| `Page.paginator` | ❌ Missing | Reference back to paginator |
| `class AsyncPaginator` | ❌ Missing | Async version |
| `class AsyncPage` | ❌ Missing | Async page object |
| `class InvalidPage` | ❌ Missing | Exception |
| `class PageNotAnInteger(InvalidPage)` | ❌ Missing | |
| `class EmptyPage(InvalidPage)` | ❌ Missing | |
| `class UnorderedObjectListWarning` | ❌ Missing | Warning for unordered QuerySet |

---

## 2. `django.core.exceptions`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class FieldDoesNotExist` | ❌ Missing | |
| `class AppRegistryNotReady` | ❌ Missing | |
| `class ObjectDoesNotExist` | ❌ Missing | |
| `class ObjectNotUpdated` | ❌ Missing | |
| `class MultipleObjectsReturned` | ❌ Missing | |
| `class SuspiciousOperation` | `SuspiciousOperation` | ✅ | Present in `rjango_core::exceptions` |
| `class SuspiciousMultipartForm` | ❌ Missing | |
| `class SuspiciousFileOperation` | ❌ Missing | |
| `class DisallowedHost` | ❌ Missing | |
| `class DisallowedRedirect` | ❌ Missing | |
| `class TooManyFieldsSent` | ❌ Missing | |
| `class TooManyFilesSent` | ❌ Missing | |
| `class RequestDataTooBig` | ❌ Missing | |
| `class RequestAborted` | ❌ Missing | |
| `class BadRequest` | ❌ Missing | |
| `class PermissionDenied` | `PermissionDenied` | ✅ | Present |
| `class ViewDoesNotExist` | ❌ Missing | |
| `class MiddlewareNotUsed` | ❌ Missing | |
| `class ImproperlyConfigured` | ❌ Missing | |
| `class FieldError` | ❌ Missing | |
| `class ValidationError(message, code, params)` | `ValidationError` | ⚠️ Partial | Rjango has `new()` but missing `code`, `params`, `error_dict`, `message_dict`, `messages` |
| `ValidationError.error_dict` | ❌ Missing | Dict of field errors |
| `ValidationError.error_list` | ⚠️ Partial | Rjango has `field_errors` but not the full API |
| `ValidationError.messages` | ❌ Missing | |
| `ValidationError.message_dict` | ❌ Missing | |
| `NON_FIELD_ERRORS = "__all__"` | ❌ Missing | |
| `class EmptyResultSet` | ❌ Missing | |
| `class FullResultSet` | ❌ Missing | |
| `class SynchronousOnlyOperation` | ❌ Missing | |

---

## 3. `django.core.signals`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `request_started = Signal()` | `RequestStarted` signal | ✅ | Present in `rjango_core::signals` |
| `request_finished = Signal()` | `RequestFinished` signal | ✅ | Present |
| `got_request_exception = Signal()` | `GotRequestException` signal | ✅ | Present |
| `setting_changed = Signal()` | ❌ Missing | |

---

## 4. `django.core.signing`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Signer(key, sep, salt, algorithm, fallback_keys)` | ❌ Missing | |
| `Signer.sign(value)` | ❌ Missing | |
| `Signer.unsign(signed_value)` | ❌ Missing | |
| `Signer.sign_object(obj)` | ❌ Missing | |
| `Signer.unsign_object(signed_obj)` | ❌ Missing | |
| `class TimestampSigner(Signer)` | ❌ Missing | |
| `TimestampSigner.sign(value)` | ❌ Missing | |
| `TimestampSigner.unsign(value, max_age)` | ❌ Missing | |
| `class JSONSerializer` | ❌ Missing | |
| `function dumps(obj, key, salt, serializer, compress)` | ❌ Missing | |
| `function loads(s, key, salt, serializer, max_age)` | ❌ Missing | |
| `function b62_encode(s)` | ❌ Missing | Base62 encoding |
| `function b62_decode(s)` | ❌ Missing | |
| `function b64_encode(s)` | ❌ Missing | |
| `function b64_decode(s)` | ❌ Missing | |
| `function base64_hmac(salt, value, key, algorithm)` | ❌ Missing | |
| `class BadSignature` | ❌ Missing | |
| `class SignatureExpired(BadSignature)` | ❌ Missing | |

---

## 5. `django.core.cache`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseCache` | ❌ Missing | Entire cache framework |
| `class CacheHandler` | ❌ Missing | |
| `function get_cache(key)` | ❌ Missing | |
| `BaseCache.set(key, value, timeout)` | ❌ Missing | |
| `BaseCache.get(key, default)` | ❌ Missing | |
| `BaseCache.add(key, value, timeout)` | ❌ Missing | |
| `BaseCache.delete(key)` | ❌ Missing | |
| `BaseCache.clear()` | ❌ Missing | |
| `BaseCache.get_many(keys)` | ❌ Missing | |
| `BaseCache.set_many(data, timeout)` | ❌ Missing | |
| `BaseCache.delete_many(keys)` | ❌ Missing | |
| `BaseCache.has_key(key)` | ❌ Missing | |
| `BaseCache.ttl(key)` | ❌ Missing | |
| `BaseCache.incr(key, delta)` | ❌ Missing | |
| `BaseCache.decr(key, delta)` | ❌ Missing | |
| `BaseCache.touch(key, timeout)` | ❌ Missing | |
| `backends: locmem, db, filebased, memcached, redis` | ❌ Missing | |
| `class InvalidCacheBackendError` | ❌ Missing | |
| `class CacheKeyWarning` | ❌ Missing | |

---

## 6. `django.core.files`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class File(FileProxyMixin)` | `File` in `rjango_core::files` | ✅ | Present |
| `File.name` | `File::name` | ✅ | |
| `File.size` | `File::size` | ✅ | |
| `File.read(n)` | `File::read()` | ✅ | |
| `File.write(content)` | ❌ Missing | |
| `File.close()` | ❌ Missing | |
| `File.open(mode)` | ❌ Missing | |
| `File.chunks(chunk_size)` | ❌ Missing | |
| `File.multiple_chunks()` | ❌ Missing | |
| `class ContentFile(File)` | `ContentFile` | ✅ | Present |
| `class FileSystemStorage` | ❌ Missing | |
| `class BaseStorage` | ❌ Missing | |
| `class UploadedFile(File)` | ❌ Missing | |
| `class TemporaryUploadedFile` | ❌ Missing | |
| `class InMemoryUploadedFile` | ❌ Missing | |
| `class ImageFile(File)` | ❌ Missing | |

---

## 7. `django.core.validators`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class RegexValidator(regex, message, code)` | `RegexValidator` | ✅ | Present |
| `class DomainNameValidator(RegexValidator)` | ❌ Missing | |
| `class URLValidator(RegexValidator)` | `URLValidator` | ✅ | Present |
| `class EmailValidator` | `EmailValidator` | ✅ | Present |
| `class MinLengthValidator(limit_value)` | `MinLengthValidator` | ✅ | Present |
| `class MaxLengthValidator(limit_value)` | `MaxLengthValidator` | ✅ | Present |
| `class MinValueValidator(limit_value)` | ❌ Missing | |
| `class MaxValueValidator(limit_value)` | ❌ Missing | |
| `class StepValueValidator(limit_value)` | ❌ Missing | |
| `class DecimalValidator(max_digits, decimal_places)` | ❌ Missing | |
| `class FileExtensionValidator(extensions)` | ❌ Missing | |
| `class ProhibitNullCharactersValidator` | ❌ Missing | |
| `function validate_integer(value)` | ❌ Missing | |
| `function validate_ipv4_address(value)` | ❌ Missing | |
| `function validate_ipv6_address(value)` | ❌ Missing | |
| `function validate_ipv46_address(value)` | ❌ Missing | |
| `function int_list_validator(sep)` | ❌ Missing | |
| `function validate_image_file_extension(value)` | ❌ Missing | |

---

## 8. `django.core.serializers`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Serializer` | ⚠️ Partial | Rjango has basic serializers |
| `class Deserializer` | ❌ Missing | |
| `class BadSerializer` | ❌ Missing | |
| `format: json` | `rjango_core::serializers` | ✅ | Basic JSON |
| `format: xml` | ❌ Missing | |
| `format: yaml` | ❌ Missing | |
| `format: python` | ❌ Missing | |
| `format: jsonl` | ❌ Missing | |

---

## 9. `django.core.handlers`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseHandler` | ❌ Missing | |
| `BaseHandler.load_middleware()` | ❌ Missing | |
| `class WSGIHandler(BaseHandler)` | ❌ Missing | |
| `class ASGIHandler(BaseHandler)` | ❌ Missing | |
| `class ExceptionHandler` | ❌ Missing | |

---

## 10. `django.core.mail`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `send_mail()` | ❌ Missing | Entire mail system |
| `send_mass_mail()` | ❌ Missing | |
| `mail_admins()` | ❌ Missing | |
| `mail_managers()` | ❌ Missing | |
| `EmailMessage` | ❌ Missing | |
| `EmailMultiAlternatives` | ❌ Missing | |
| `EmailBackend` | ❌ Missing | SMTP/file/console backends |

---

## 11. `django.http`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class HttpRequest` | `Request` | ✅ | Present |
| `HttpRequest.method` | `Request::method` | ✅ | |
| `HttpRequest.path` | `Request::path` | ✅ | |
| `HttpRequest.GET` | `Request::query_params()` | ⚠️ Partial | Rjango doesn't have a full QueryDict |
| `HttpRequest.POST` | `Request::body()` | ⚠️ Partial | |
| `HttpRequest.COOKIES` | `Request::cookies` | ✅ | |
| `HttpRequest.META` | ❌ Missing | |
| `HttpRequest.headers` | ❌ Missing | |
| `HttpRequest.user` | ❌ Missing | From middleware |
| `HttpRequest.session` | ❌ Missing | From middleware |
| `HttpRequest.is_ajax()` | ❌ Missing | |
| `HttpRequest.is_secure()` | ❌ Missing | |
| `HttpRequest.get_host()` | ❌ Missing | |
| `HttpRequest.get_port()` | ❌ Missing | |
| `HttpRequest.build_absolute_uri()` | ❌ Missing | |
| `class QueryDict(MultiValueDict)` | `QueryDict` | ⚠️ Partial | Rjango has QueryDict but missing `MultiValueDict` parent |
| `QueryDict.getlist(key)` | `QueryDict::get_list()` | ✅ | Present |
| `QueryDict.dict()` | ❌ Missing | |
| `QueryDict.urlencode()` | ❌ Missing | |
| `QueryDict.lists()` | ❌ Missing | |
| `class HttpHeaders` | ❌ Missing | |
| `class MediaType` | ❌ Missing | |
| `class HttpResponse` | `Response` | ✅ | Present |
| `HttpResponse.status_code` | `Response::status_code()` | ✅ | |
| `HttpResponse.content` | `Response::body()` | ✅ | |
| `HttpResponse.headers` | `Response::headers()` | ✅ | |
| `HttpResponse.__setitem__(header, value)` | `Response::set_header()` | ✅ | |
| `HttpResponse.__getitem__(header)` | `Response::header()` | ✅ | |
| `HttpResponse.cookies` | ❌ Missing | |
| `HttpResponse.has_header()` | ❌ Missing | |
| `class HttpResponseRedirect` | `Response::redirect()` | ⚠️ Partial | Rjango has `Response::redirect(302)` for temporary |
| `class HttpResponsePermanentRedirect` | `Response::redirect(301)` | ⚠️ Partial | |
| `class HttpResponseNotModified` | ❌ Missing | |
| `class HttpResponseBadRequest` | ❌ Missing | |
| `class HttpResponseNotFound` | ❌ Missing | |
| `class HttpResponseForbidden` | ❌ Missing | |
| `class HttpResponseNotAllowed` | ❌ Missing | |
| `class HttpResponseGone` | ❌ Missing | |
| `class HttpResponseServerError` | ❌ Missing | |
| `class JsonResponse(data)` | ❌ Missing | |
| `class StreamingHttpResponse` | ❌ Missing | |
| `class FileResponse` | ❌ Missing | |
| `class Http404(Exception)` | ❌ Missing | |

---

## 12. `django.shortcuts`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `render(request, template_name, context, content_type, status, using)` | `render()` | ⚠️ Partial | Rjango version takes different args |
| `redirect(to, ...)` | `redirect()` | ⚠️ Partial | Rjango version takes only URL, Django resolves URLs/models |
| `get_object_or_404(klass, ...)` | `get_object_or_404()` | ⚠️ Partial | Rjango lacks database integration |
| `aget_object_or_404(klass, ...)` | ❌ Missing | Async version |
| `get_list_or_404(klass, ...)` | `get_list_or_404()` | ⚠️ Partial | |
| `aget_list_or_404(klass, ...)` | ❌ Missing | Async version |
| `resolve_url(to, ...)` | `resolve_url()` | ⚠️ Partial | Lacks `reverse()` integration |

---

## 13. `django.apps`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class AppConfig` | `AppConfig` | ✅ | Present in `rjango_core::app` |
| `AppConfig.name` | `AppConfig::name` | ✅ | |
| `AppConfig.label` | `AppConfig::label` | ✅ | |
| `AppConfig.verbose_name` | ⚠️ Partial | Rjango has `label` but not `verbose_name` |
| `AppConfig.path` | ❌ Missing | |
| `AppConfig.models_module` | ❌ Missing | |
| `AppConfig.get_models()` | ❌ Missing | |
| `AppConfig.get_model(model_name)` | ❌ Missing | |
| `AppConfig.import_models()` | ❌ Missing | |
| `AppConfig.ready()` | ❌ Missing | Hook for app startup |
| `class Apps` | `Registry` | ⚠️ Partial | Rjango has basic register/get |
| `Apps.get_app_configs()` | `Registry::get_apps()` | ✅ | |
| `Apps.get_app_config(app_label)` | `Registry::get_app()` | ✅ | |
| `Apps.is_installed(app_name)` | `Registry::is_app_installed()` | ✅ | |
| `Apps.get_model(app_label, model_name)` | ❌ Missing | |
| `Apps.register_model(app_label, model)` | `Registry::register_model()` | ✅ | |
| `Apps.get_models()` | ❌ Missing | |
| `Apps.clear_cache()` | ❌ Missing | |
| `populate(installed_apps)` | ❌ Missing | |

---

## 14. `django.conf`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `settings` module-level proxy | `Settings` | ✅ | Present |
| `settings.DEBUG` | `Settings.debug` | ✅ | |
| `settings.SECRET_KEY` | `Settings.secret_key` | ✅ | |
| `settings.ALLOWED_HOSTS` | `Settings.allowed_hosts` | ✅ | |
| `settings.INSTALLED_APPS` | `Settings.installed_apps` | ✅ | |
| `settings.ROOT_URLCONF` | `Settings.root_urlconf` | ✅ | |
| `settings.DATABASES` | `Settings.databases` | ✅ | |
| `settings.TEMPLATES` | ❌ Missing | Template config |
| `settings.MIDDLEWARE` | ❌ Missing | |
| `settings.STATIC_URL` | ❌ Missing | |
| `settings.MEDIA_URL` | ❌ Missing | |
| `settings.LANGUAGE_CODE` | ❌ Missing | |
| `settings.TIME_ZONE` | ❌ Missing | |
| `settings.USE_I18N` | ❌ Missing | |
| `settings.STATIC_ROOT` | ❌ Missing | |
| `settings.MEDIA_ROOT` | ❌ Missing | |
| `settings.SECRET_KEY_FALLBACKS` | ❌ Missing | |
| `settings.DEFAULT_AUTO_FIELD` | ❌ Missing | |
| `settings.AUTH_USER_MODEL` | ❌ Missing | |
| `settings.LOGIN_URL` | ❌ Missing | |
| `settings.LOGIN_REDIRECT_URL` | ❌ Missing | |
| `settings.LOGOUT_REDIRECT_URL` | ❌ Missing | |
| `settings.CSRF_COOKIE_NAME` | ❌ Missing | |
| `settings.SESSION_COOKIE_NAME` | ❌ Missing | |
| `settings.LANGUAGE_COOKIE_NAME` | ❌ Missing | |
| `settings.TEMPLATE_DIRS` | ❌ Missing | |
| `user_settings` | ❌ Missing | |

---

## 15. `django.dispatch`

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Signal` | `Signal` | ✅ | Present |
| `Signal.connect(receiver, sender, weak, dispatch_uid)` | `Signal::connect()` | ✅ | |
| `Signal.disconnect(receiver, sender, dispatch_uid)` | `Signal::disconnect()` | ✅ | |
| `Signal.send(sender, **kwargs)` | `Signal::send()` | ✅ | |
| `Signal.send_robust(sender, **kwargs)` | ❌ Missing | Doesn't stop on exceptions |
| `function receiver(signal, **kwargs)` | ❌ Missing | Decorator for connecting |

---

## 16. `django.utils`

### Utils functional
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `cached_property` | `CachedProperty` | ✅ | Present |
| `lazy(func)` | ❌ Missing | |
| `Promise` | ❌ Missing | Lazy evaluation |
| `keep_lazy(func)` | ❌ Missing | |
| `class LazyObject` | ❌ Missing | Rjango has basic version |
| `class LazySettings` | ❌ Missing | |

### Utils safestring
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class SafeString` | `SafeString` | ✅ | Present |
| `class SafeData` | ❌ Missing | |
| `mark_safe(s)` | `mark_safe()` | ✅ | |

### Utils text
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `slugify(value)` | `slugify()` | ✅ | Present |
| `Truncator(value)` | ❌ Missing | Handles truncation with HTML |
| `Truncator.chars(num)` | ❌ Missing | |
| `Truncator.words(num)` | ❌ Missing | |
| `Truncator.html_chars(num)` | ❌ Missing | |
| `Truncator.html_words(num)` | ❌ Missing | |
| `phone2numeric(phone)` | `phone2numeric()` | ✅ | |
| `wrap(word, length)` | ❌ Missing | |
| `normalize_newlines(text)` | ❌ Missing | |
| `escape(text)` | `escape()` | ✅ | |
| `get_valid_filename(s)` | ❌ Missing | |

### Utils crypto
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `random.sample()` | ❌ Missing | |
| `salted_hmac(salt, value, secret)` | ❌ Missing | |
| `get_random_string(length, chars)` | ❌ Missing | |
| `constant_time_compare(a, b)` | ❌ Missing | |
| `pbkdf2(password, salt, iterations, dklen)` | ❌ Missing | |

### Utils html
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `escape(text)` | `html::escape()` | ✅ | Present |
| `conditional_escape(text)` | ❌ Missing | |
| `format_html(format, ...)` | ❌ Missing | |
| `format_html_join(sep, format, ...)` | ❌ Missing | |
| `json_script(value, element_id)` | ❌ Missing | |
| `strip_tags(value)` | ❌ Missing | |
| `urlize(value)` | ❌ Missing | |
| `urlizetrunc(value, limit)` | ❌ Missing | |

### Utils translation / i18n
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `gettext(message)` | `i18n::gettext()` | ✅ | Present |
| `ngettext(singular, plural, n)` | ❌ Missing | |
| `pgettext(context, message)` | ❌ Missing | |
| `activate(language)` | ❌ Missing | |
| `deactivate()` | ❌ Missing | |
| `get_language()` | ❌ Missing | |
| `get_language_bidi()` | ❌ Missing | |
| `get_language_from_request(request)` | ❌ Missing | |
| `override(language)` | ❌ Missing | Context manager |
| `ugettext() / ungettext()` | ❌ Missing | |
| `LANGUAGE_SESSION_KEY` | ❌ Missing | |
| `LocaleMiddleware` | ❌ Missing | |

### Utils http
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `urlencode(query, doseq)` | ❌ Missing | |
| `parse_cookie(cookie)` | `http::parse_cookies()` | ✅ | Present |

### Utils timesince
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `timesince(d, now)` | `timesince()` | ✅ | Present |
| `timeuntil(d, now)` | ❌ Missing | |

### Utils dateformat / dateparse / timezone
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `DateFormatter(format)` | ❌ Missing | |
| `TimeFormatter(format)` | ❌ Missing | |
| `format(value, format_str)` | ❌ Missing | |
| `parse_date(value)` | ❌ Missing | |
| `parse_time(value)` | ❌ Missing | |
| `parse_datetime(value)` | ❌ Missing | |
| `get_fixed_timezone(offset)` | ❌ Missing | |
| `make_aware(value, timezone)` | ❌ Missing | |
| `make_naive(value, timezone)` | ❌ Missing | |
| `now()` | ❌ Missing | Timezone-aware now |
| `is_aware(value)` | ❌ Missing | |
| `is_naive(value)` | ❌ Missing | |
| `activate(timezone)` | ❌ Missing | |
| `deactivate()` | ❌ Missing | |
| `override(timezone)` | ❌ Missing | |
| `get_current_timezone()` | ❌ Missing | |
| `class FixedOffset` | ❌ Missing | |

### Utils others
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `module_loading.import_string(path)` | ❌ Missing | |
| `module_loading.module_has_submodule(pkg, name)` | ❌ Missing | |
| `deconstruct` | ❌ Missing | For field deconstruction |
| `inspect.method_has_no_args(meth)` | ❌ Missing | |
| `autoreload` | ❌ Missing | |
| `encoding.smart_str(s)` | ❌ Missing | |
| `encoding.force_str(s)` | ❌ Missing | |
| `encoding.smart_bytes(s)` | ❌ Missing | |
| `encoding.force_bytes(s)` | ❌ Missing | |
| `deprecation.RemovedInDjango70Warning` | ❌ Missing | |
| `datastructures.MultiValueDict` | ❌ Missing | |
| `datastructures.ImmutableList` | ❌ Missing | |
| `datastructures.OrderedSet` | ❌ Missing | |
| `datastructures.CaseInsensitiveMapping` | ❌ Missing | |
| `numberformat.format(n, decimal_sep, ...)` | ❌ Missing | |
| `timezone.now()` | `chrono::Utc::now()` | ⚠️ Partial | Using raw chrono |
| `tree.Node` | ❌ Missing | |
| `version.get_version()` | ❌ Missing | |
| `log.AdminEmailHandler` | ❌ Missing | |
| `feedgenerator` | ❌ Missing | RSS/Atom feeds |
| `lorem_ipsum.paragraphs(n)` | ❌ Missing | |
| `lorem_ipsum.words(n)` | ❌ Missing | |
| `lorem_ipsum.sentence()` | ❌ Missing | |
| `regex_helper._lazy_re_compile(regex)` | ❌ Missing | |
| `cache` | ❌ Missing | Cache utilities |
| `connection` | ❌ Missing | DB connection utilities |

---

## Summary

### Django Core Features in Rjango (✅ Complete)
- ✅ `Paginator` (basic, missing Page object + many methods)
- ✅ `SuspiciousOperation` exception
- ✅ `PermissionDenied` exception
- ✅ `ValidationError` (partial)
- ✅ `File` / `ContentFile`
- ✅ RegexValidator / URLValidator / EmailValidator / MinLengthValidator / MaxLengthValidator
- ✅ Signal with connect/send/disconnect
- ✅ AppConfig / Registry
- ✅ Settings
- ✅ Request / Response
- ✅ QueryDict
- ✅ SafeString / mark_safe
- ✅ slugify / phone2numeric / escape
- ✅ Shortcuts: render, redirect, get_object_or_404, get_list_or_404, resolve_url
- ✅ timesince
- ✅ i18n gettext
- ✅ CachedProperty

### Missing Django Core Features (❌)
- ❌ Page object with full navigation API
- ❌ AsyncPaginators
- ❌ All signed-cookie utilities (Signer, TimestampSigner, dumps/loads)
- ❌ Cache framework (7 backends)
- ❌ File storage backend system
- ❌ UploadedFile classes
- ❌ Many validators (MinValue, MaxValue, StepValue, Decimal, Extension, IP, integer)
- ❌ Serializer framework (JSON/XML/YAML/Python)
- ❌ WSGI/ASGI handlers
- ❌ Mail system
- ❌ Many HTTP response types (JsonResponse, FileResponse, streaming, status codes)
- ❌ Many settings (50+ settings missing)
- ❌ Translation utilities (pgettext, ngettext, activate, override)
- ❌ Date/time formatting/parsing utilities
- ❌ Timezone support (make_aware, make_naive, activate, override)
- ❌ MultiValueDict
- ❌ Most utils (encoding, hashable, module_loading, deconstruct, inspect, decorators, etc.)
