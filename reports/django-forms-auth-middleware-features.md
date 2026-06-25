# Django 6.0.6 Forms, Auth & Middleware — Feature Comparison with Rjango

> **Django modules analyzed:** `django.forms`, `django.contrib.auth`, `django.middleware`

---

## 1. Forms (`django.forms`)

### Form Base
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseForm` | `Form` | ⚠️ Partial | Rjango has `Form` struct |
| `BaseForm.is_bound` | ✅ | |
| `BaseForm.is_valid()` | ✅ | |
| `BaseForm.errors` | `Form::errors()` | ✅ | |
| `BaseForm.cleaned_data` | ✅ | |
| `BaseForm.fields` | ✅ | |
| `BaseForm.add_prefix(field_name)` | ❌ Missing | |
| `BaseForm.add_initial_prefix(field_name)` | ❌ Missing | |
| `BaseForm._clean_fields()` | ✅ | |
| `BaseForm._clean_form()` | ❌ Missing | Hook for custom clean |
| `BaseForm.full_clean()` | ✅ | |
| `BaseForm.non_field_errors()` | ❌ Missing | |
| `BaseForm.has_changed()` | ❌ Missing | |
| `BaseForm.changed_data` | ❌ Missing | |
| `class Form(metaclass=DeclarativeFieldsMetaclass)` | ✅ | |
| `Form.as_table()` | `Form::as_table()` | ✅ | |
| `Form.as_p()` | `Form::as_p()` | ✅ | |
| `Form.as_div()` | `Form::as_div()` | ✅ | |
| `Form.as_ul()` | ❌ Missing | |
| `Form.as_databundle()` | ❌ Missing | |

### Form Fields
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Field(required, widget, label, initial, help_text, error_messages, validators, localize, disabled, label_suffix)` | `FormField` | ⚠️ Partial | Basic parameters present |
| `Field.clean(value)` | ✅ | |
| `Field.validate(value)` | ✅ | |
| `Field.run_validators(value)` | ✅ | |
| `Field.widget` | ✅ | |
| `Field.has_changed(initial, data)` | ❌ Missing | |
| `class CharField(Field)` | ✅ | |
| `class IntegerField(Field)` | ✅ | |
| `class FloatField(Field)` | ✅ | |
| `class DecimalField(Field)` | ❌ Missing | |
| `class BooleanField(Field)` | ✅ | |
| `class NullBooleanField(BooleanField)` | ❌ Missing | |
| `class ChoiceField(Field)` | ✅ | |
| `class TypedChoiceField(ChoiceField)` | ❌ Missing | |
| `class MultipleChoiceField(Field)` | ❌ Missing | |
| `class TypedMultipleChoiceField(MultipleChoiceField)` | ❌ Missing | |
| `class DateField(Field)` | ❌ Missing | |
| `class TimeField(Field)` | ❌ Missing | |
| `class DateTimeField(Field)` | ❌ Missing | |
| `class DurationField(Field)` | ❌ Missing | |
| `class SplitDateTimeField(Field)` | ❌ Missing | |
| `class EmailField(CharField)` | ✅ | |
| `class URLField(CharField)` | ✅ | |
| `class FileField(Field)` | ❌ Missing | |
| `class ImageField(FileField)` | ❌ Missing | |
| `class RegexField(CharField)` | ❌ Missing | |
| `class UUIDField(CharField)` | ❌ Missing | |
| `class GenericIPAddressField(Field)` | ❌ Missing | |
| `class SlugField(CharField)` | ❌ Missing | |
| `class BaseTemporalField` | ❌ Missing | |

### Form Widgets
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Widget(attrs)` | `Widget` | ✅ | |
| `Widget.render(name, value, attrs)` | ✅ | |
| `Widget.get_context(name, value, attrs)` | ❌ Missing | |
| `Widget.id_for_label(id_)` | ❌ Missing | |
| `Widget.value_from_datadict(data, files, name)` | ❌ Missing | |
| `Widget.format_value(value)` | ❌ Missing | |
| `class Input(Widget)` | ✅ | |
| `class TextInput(Input)` | ✅ | |
| `class NumberInput(Input)` | ✅ | |
| `class EmailInput(Input)` | ✅ | |
| `class URLInput(Input)` | ❌ Missing | |
| `class PasswordInput(Input)` | ✅ | |
| `class HiddenInput(Input)` | ✅ | |
| `class DateInput(Input)` | ✅ | |
| `class DateTimeInput(Input)` | ✅ | |
| `class TimeInput(Input)` | ❌ Missing | |
| `class Textarea(Widget)` | ✅ | |
| `class CheckboxInput(Widget)` | ✅ | |
| `class Select(Widget)` | ✅ | |
| `class NullBooleanSelect(Select)` | ❌ Missing | |
| `class SelectMultiple(Select)` | ❌ Missing | |
| `class RadioSelect(Select)` | ✅ | |
| `class CheckboxSelectMultiple(Select)` | ❌ Missing | |
| `class FileInput(Input)` | ❌ Missing | |
| `class ClearableFileInput(FileInput)` | ❌ Missing | |
| `class MultipleHiddenInput(HiddenInput)` | ❌ Missing | |
| `class SplitDateTimeWidget(Widget)` | ❌ Missing | |
| `class SplitHiddenDateTimeWidget` | ❌ Missing | |
| `class ColorInput(Input)` | ❌ Missing | |
| `class SearchInput(Input)` | ❌ Missing | |
| `class TelInput(Input)` | ❌ Missing | |
| `class Media` | ❌ Missing | CSS/JS media framework |

### Formsets
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseFormSet` | ❌ Missing | |
| `BaseFormSet.forms` | ❌ Missing | |
| `BaseFormSet.management_form` | ❌ Missing | |
| `BaseFormSet.total_form_count()` | ❌ Missing | |
| `BaseFormSet.initial_form_count()` | ❌ Missing | |
| `BaseFormSet.media` | ❌ Missing | |
| `BaseFormSet.is_valid()` | ❌ Missing | |
| `BaseFormSet.cleaned_data` | ❌ Missing | |
| `BaseFormSet.errors` | ❌ Missing | |
| `BaseFormSet.can_order` | ❌ Missing | |
| `BaseFormSet.can_delete` | ❌ Missing | |
| `formset_factory(form, formset)` | ❌ Missing | |
| `modelformset_factory(model)` | ❌ Missing | |
| `inlineformset_factory(parent, child)` | ❌ Missing | |

### Model Forms
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class ModelForm(BaseForm)` | ❌ Missing | |
| `class BaseModelForm(BaseForm)` | ❌ Missing | |
| `ModelForm.save(commit=True)` | ❌ Missing | |
| `ModelForm.instance` | ❌ Missing | |
| `modelform_factory(model)` | ❌ Missing | |
| `model_to_dict(instance)` | ❌ Missing | |
| `fields_for_model(model)` | ❌ Missing | |
| `construct_instance(form, instance)` | ❌ Missing | |
| `class ModelChoiceField` | ❌ Missing | |
| `class ModelMultipleChoiceField` | ❌ Missing | |

### Form Renderers
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseRenderer` | ❌ Missing | |
| `class DjangoTemplates(BaseRenderer)` | ❌ Missing | |
| `class Jinja2(BaseRenderer)` | ❌ Missing | |
| `class TemplatesSetting(BaseRenderer)` | ❌ Missing | |
| `get_default_renderer()` | ❌ Missing | |

---

## 2. Authentication (`django.contrib.auth`)

### User Model
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class User(AbstractUser)` | `User` | ✅ | Present |
| `class AbstractUser(AbstractBaseUser, PermissionsMixin)` | ❌ Missing | |
| `class AnonymousUser` | `AnonymousUser` | ✅ | |
| `class PermissionsMixin` | ✅ | Permissions handling |
| `class PermissionManager` | ❌ Missing | |
| `class Permission` | ❌ Missing | |
| `class GroupManager` | ❌ Missing | |
| `class Group` | ❌ Missing | |
| `class UserManager(BaseUserManager)` | ❌ Missing | |
| `User.is_authenticated` | ✅ | |
| `User.is_anonymous` | ✅ | |
| `User.is_active` | ✅ | |
| `User.is_staff` | ✅ | |
| `User.is_superuser` | ✅ | |
| `User.has_perm(perm, obj)` | ✅ | |
| `User.has_perms(perm_list, obj)` | ✅ | |
| `User.has_module_perms(app_label)` | ✅ | |
| `User.get_username()` | ❌ Missing | |
| `User.get_full_name()` | ❌ Missing | |
| `User.get_short_name()` | ❌ Missing | |
| `User.set_password(raw)` | ❌ Missing | |
| `User.check_password(raw)` | ❌ Missing | |
| `User.set_unusable_password()` | ❌ Missing | |
| `User.has_usable_password()` | ❌ Missing | |
| `User.get_group_permissions(obj)` | ❌ Missing | |
| `User.get_user_permissions(obj)` | ❌ Missing | |
| `User.email_user(subject, ...)` | ❌ Missing | |

### Auth Functions
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `authenticate(request, **credentials)` | ✅ | |
| `login(request, user)` | ✅ | |
| `logout(request)` | ✅ | |
| `get_user(request)` | ✅ | |
| `get_user_model()` | ✅ | |
| `update_session_auth_hash(request, user)` | ❌ Missing | |
| `load_backend(path)` | ❌ Missing | |
| `get_backends()` | ❌ Missing | |
| `_get_user_session_key(request)` | ❌ Missing | |

### Auth Backends
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseBackend` | ❌ Missing | |
| `class ModelBackend(BaseBackend)` | ❌ Missing | |
| `class AllowAllUsersModelBackend(ModelBackend)` | ❌ Missing | |
| `class RemoteUserBackend(ModelBackend)` | ❌ Missing | |
| `class AllowAllUsersRemoteUserBackend(RemoteUserBackend)` | ❌ Missing | |

### Auth Decorators
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `login_required(function)` | `login_required()` | ✅ | |
| `login_required(login_url, redirect_field_name)` | ✅ | |
| `login_not_required(view_func)` | ❌ Missing | |
| `permission_required(perm)` | `permission_required()` | ✅ | |
| `permission_required(login_url, raise_exception)` | ✅ | |
| `user_passes_test(test_func)` | `user_passes_test()` | ✅ | |
| `user_passes_test(login_url)` | ✅ | |

### Auth Middleware
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class AuthenticationMiddleware` | `AuthMiddleware` | ✅ | |
| `class LoginRequiredMiddleware` | ❌ Missing | |
| `class RemoteUserMiddleware` | ❌ Missing | |
| `class PersistentRemoteUserMiddleware` | ❌ Missing | |

### Auth Views
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class LoginView` | `LoginView` | ✅ | |
| `class LogoutView` | `LogoutView` | ✅ | |
| `logout_then_login(request)` | ❌ Missing | |
| `redirect_to_login(next, login_url)` | ❌ Missing | |
| `class PasswordResetView` | ❌ Missing | |
| `class PasswordResetDoneView` | ❌ Missing | |
| `class PasswordResetConfirmView` | ❌ Missing | |
| `class PasswordResetCompleteView` | ❌ Missing | |
| `class PasswordChangeView` | ❌ Missing | |
| `class PasswordChangeDoneView` | ❌ Missing | |

### Password Hashing
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `make_password(password)` | ✅ | |
| `check_password(password, encoded)` | ✅ | |
| `is_password_usable(encoded)` | ❌ Missing | |
| `verify_password(password, encoded)` | ❌ Missing | |
| `get_hashers()` | ❌ Missing | |
| `identify_hasher(encoded)` | ❌ Missing | |
| `class BasePasswordHasher` | ❌ Missing | |
| `class PBKDF2PasswordHasher` | ❌ Missing | |
| `class PBKDF2SHA1PasswordHasher` | ❌ Missing | |
| `class Argon2PasswordHasher` | ❌ Missing | |
| `class BCryptSHA256PasswordHasher` | ❌ Missing | |
| `class BCryptPasswordHasher` | ❌ Missing | |
| `class ScryptPasswordHasher` | ❌ Missing | |
| `class MD5PasswordHasher` | ❌ Missing | |

### Password Validation
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `validate_password(password)` | ❌ Missing | |
| `password_changed(password)` | ❌ Missing | |
| `password_validators_help_texts()` | ❌ Missing | |
| `class MinimumLengthValidator` | ❌ Missing | |
| `class UserAttributeSimilarityValidator` | ❌ Missing | |
| `class CommonPasswordValidator` | ❌ Missing | |
| `class NumericPasswordValidator` | ❌ Missing | |
| `get_default_password_validators()` | ❌ Missing | |

### Auth Forms
| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class AuthenticationForm` | ❌ Missing | |
| `class PasswordResetForm` | ❌ Missing | |
| `class SetPasswordForm` | ❌ Missing | |
| `class PasswordChangeForm` | ❌ Missing | |
| `class AdminPasswordChangeForm` | ❌ Missing | |
| `class UserCreationForm` | ❌ Missing | |
| `class UserChangeForm` | ❌ Missing | |

---

## 3. Middleware (`django.middleware`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class SecurityMiddleware` | `SecurityMiddleware` | ✅ | Security headers |
| `SecurityMiddleware.SECURE_SSL_REDIRECT` | ❌ Missing | |
| `SecurityMiddleware.SECURE_HSTS_SECONDS` | ❌ Missing | |
| `SecurityMiddleware.SECURE_CONTENT_TYPE_NOSNIFF` | ❌ Missing | |
| `SecurityMiddleware.SECURE_BROWSER_XSS_FILTER` | ❌ Missing | |
| `class CSRFMiddleware` | `CsrfMiddleware` | ✅ | |
| `class SessionMiddleware` | `SessionMiddleware` | ✅ | |
| `class MessageMiddleware` | `MessageMiddleware` | ✅ | |
| `class XFrameOptionsMiddleware` | ❌ Missing | |
| `class CommonMiddleware` | ❌ Missing | URL normalizing, APPEND_SLASH |
| `class GZipMiddleware` | ❌ Missing | |
| `class ConditionalGetMiddleware` | ❌ Missing | ETags, Last-Modified |
| `class LocaleMiddleware` | ❌ Missing | |
| `class BrokenLinkEmailsMiddleware` | ❌ Missing | |
| `class UpdateCacheMiddleware` | ❌ Missing | |
| `class FetchFromCacheMiddleware` | ❌ Missing | |
| `class CacheMiddleware(Update, Fetch)` | ❌ Missing | |
| `class ContentSecurityPolicyMiddleware` | ✅ | Rjango has CSP |
| `MiddlewareMixin` | ❌ Missing | Django base middleware class |

---

## 4. Sessions (contrib.sessions)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class SessionBase` | `SessionStore` | ⚠️ Partial | |
| `SessionBase.get(key)` | ✅ | |
| `SessionBase.setdefault(key, value)` | ❌ Missing | |
| `SessionBase.update(dict)` | ❌ Missing | |
| `SessionBase.pop(key)` | ❌ Missing | |
| `SessionBase.keys()` | ❌ Missing | |
| `SessionBase.values()` | ❌ Missing | |
| `SessionBase.items()` | ❌ Missing | |
| `SessionBase.clear()` | ❌ Missing | |
| `SessionBase.set_expiry(value)` | ❌ Missing | |
| `SessionBase.get_expiry_age()` | ❌ Missing | |
| `SessionBase.get_expiry_date()` | ❌ Missing | |
| `SessionBase.get_expire_at_browser_close()` | ❌ Missing | |
| `SessionBase.flush()` | ❌ Missing | |
| `SessionBase.cycle_key()` | ❌ Missing | |
| `db backend (SessionStore)` | ❌ Missing | |
| `cache backend` | ❌ Missing | |
| `file backend` | ❌ Missing | |
| `signed_cookies backend` | ❌ Missing | |

---

## Summary

### Rjango Forms/Auth/Middleware Features (✅ Complete)
- ✅ Form field types (Char, Integer, Float, Boolean, Choice, Email, URL)
- ✅ Form validation + error handling
- ✅ Widgets (Text, Number, Email, Password, Hidden, Textarea, Checkbox, Select, Radio, Date, DateTime)
- ✅ Form rendering (as_table, as_p, as_div)
- ✅ User model (is_active, is_staff, is_superuser, permissions)
- ✅ Auth decorators (login_required, permission_required, user_passes_test, superuser_only)
- ✅ Login/Logout views
- ✅ Auth middleware
- ✅ CSRF middleware
- ✅ Session middleware
- ✅ Messages middleware
- ✅ Security middleware
- ✅ CSP middleware

### Missing Features (❌)
- ❌ Formsets
- ❌ ModelForms (save to DB)
- ❌ ModelChoiceField
- ❌ Form renderers
- ❌ Full widget set (ColorInput, SearchInput, TelInput, etc.)
- ❌ Media framework for CSS/JS
- ❌ Auth forms (AuthenticationForm, PasswordResetForm, etc.)
- ❌ Password validation system
- ❌ Full password hasher framework
- ❌ Password reset view chain
- ❌ Groups & Permissions models
- ❌ RemoteUserBackend
- ❌ LoginRequiredMiddleware
- ❌ XFrameOptionsMiddleware
- ❌ CommonMiddleware
- ❌ GZipMiddleware
- ❌ ConditionalGetMiddleware
- ❌ LocaleMiddleware
- ❌ Cache middleware
- ❌ Session expiry management
- ❌ Multiple session backends
