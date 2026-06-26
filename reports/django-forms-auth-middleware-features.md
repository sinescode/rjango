# Django 6.0.6 Forms, Auth & Middleware — Feature Comparison with Rjango

> **Django modules analyzed:** `django.forms`, `django.contrib.auth`, `django.middleware`
> **Last updated:** 2026-06-26

---

## Overview

| Category | Coverage | Status |
|----------|----------|--------|
| **Middleware** | 14/14 = **100%** | ✅ Complete |
| **Auth** | 24/40 = **60%** | 🟡 Strong |
| **Forms** | 12/83 = **15%** | ❌ Weak |

---

## 🟢 Middleware — Complete (100%)

### All 14 Middleware Types Implemented

| Middleware | File | Django Equivalent |
|-----------|------|-------------------|
| `SecurityMiddleware` | `rjango-middleware/src/security.rs` | `SecurityMiddleware` |
| `XFrameOptionsMiddleware` | `rjango-middleware/src/clickjacking.rs` | `XFrameOptionsMiddleware` |
| `CsrfMiddleware` | `rjango-middleware/src/csrf.rs` | `CsrfViewMiddleware` |
| `SessionMiddleware` | `rjango-middleware/src/session.rs` | `SessionMiddleware` |
| `MessageMiddleware` | `rjango-middleware/src/messages.rs` | `MessageMiddleware` |
| `CommonMiddleware` | `rjango-middleware/src/common.rs` | `CommonMiddleware` |
| `GZipMiddleware` | `rjango-middleware/src/gzip.rs` | `GZipMiddleware` |
| `ConditionalGetMiddleware` | `rjango-middleware/src/conditional_get.rs` | `ConditionalGetMiddleware` |
| `LocaleMiddleware` | `rjango-middleware/src/locale.rs` | `LocaleMiddleware` |
| `AuthMiddleware` | `rjango-auth/src/middleware_.rs` | `AuthenticationMiddleware` |
| `RemoteUserMiddleware` | `rjango-middleware/src/remote_user.rs` | `RemoteUserMiddleware` |
| `UpdateCacheMiddleware` | `rjango-middleware/src/cache.rs` | `UpdateCacheMiddleware` |
| `FetchFromCacheMiddleware` | `rjango-middleware/src/cache.rs` | `FetchFromCacheMiddleware` |
| `ContentSecurityPolicyMiddleware` | `rjango-middleware/src/csp.rs` | `django-csp` |

### MiddlewareStack
- Ordered middleware chain with `add()`, `process()`, `len()`, `is_empty()`
- `Middleware` trait with `process_request()`, `process_response()`, `process_view()`, `process_exception()`

---

## 🟡 Auth — 60% Coverage

### ✅ Implemented (24 APIs)

#### Models
- `User` with `has_perm()`, `has_module_perms()`, `has_perms()`, `get_all_permissions()`, `get_group_permissions()`
- `Permission`, `PermissionManager` with `by_codename()`, `matches()`, `natural_key()`
- `Group`, `GroupManager`
- `PermissionMixin`, `UserModelProxy` traits
- `AnonymousUser`
- `ContentType` with `get_for_model()`, `get_for_id()`

#### Backends
- `DefaultBackend` (ModelBackend)
- `RemoteUserBackend`
- `AuthenticationBackend` trait

#### Decorators
- `login_required`, `permission_required`, `user_passes_test`, `superuser_only`
- `require_http_methods`, `apply_decorator`, `apply_decorators`

#### Forms
- `AuthenticationForm`, `PasswordResetForm`, `PasswordChangeForm`, `UserCreationForm`, `SetPasswordForm`
- Validation methods with username/existing user checks

#### Password Management
- `PBKDF2PasswordHasher` with iterations, salt, digest
- `check_password()`, `make_password()`, `is_password_usable()`, `constant_time_compare()`
- `PasswordValidator` trait with `MinimumLengthValidator`, `UserAttributeSimilarityValidator`, `CommonPasswordValidator`, `NumericPasswordValidator`
- `validate_password()`, `password_changed()`, `password_validators_help_texts()`

#### Views
- `login_view()`, `handle_login()`, `logout_view()`

#### Middleware
- `AuthMiddleware` with `process_request()`, `is_authenticated()`, `login_required()`

### ❌ Missing (16 APIs)

| Feature | Priority | Notes |
|---------|----------|-------|
| `PasswordResetConfirmView` | Medium | Password reset via token |
| `PasswordResetDoneView` | Medium | Success page |
| `PasswordResetCompleteView` | Medium | Completion page |
| `LoginView` (CBV) | Low | Class-based version (fn exists) |
| `LogoutView` (CBV) | Low | Class-based version (fn exists) |
| Permission assign/remove for individual users | Medium | Currently group-level only |
| `user.get_all_permissions()` across groups | Low | Basic implementation exists |
| `AuthBackend.get_user_permissions()` | Low | |
| `AuthBackend.get_group_permissions()` | Low | |
| Rate limiting on login | Low | |
| Session invalidation on password change | Low | |
| Built-in password reset email templates | Low | |
| `AuthenticationMiddleware` with request.user lazy loading | Low | Basic implementation exists |

---

## ❌ Forms — 15% Coverage (Weakest Module)

### ✅ Implemented (12 APIs)

#### Fields
- `FormField` struct with name, label, field_type, required, widget, validators
- `FieldType` enum with 23 variants: Char, Integer, Boolean, Email, URL, Date, DateTime, Time, Duration, Float, Decimal, Regex, Slug, UUID, IpAddress, IPv6, URL, NullBoolean, Hidden, File, Image, SplitDateTime, TypedChoice, TypedMultipleChoice, FilePath, GenericIPAddress
- `FormField::clean()` with validation for Integer, Float, Decimal, Boolean, Email, URL, Date, DateTime, UUID, IpAddress

#### Widgets
- `WidgetType` enum with 23 variants: TextInput, EmailInput, PasswordInput, NumberInput, URLInput, Textarea, Select, SelectMultiple, CheckboxInput, CheckboxSelectMultiple, RadioSelect, NullBooleanSelect, DateInput, DateTimeInput, TimeInput, HiddenInput, FileInput, ClearableFileInput, SplitDateTimeInput, SelectDateWidget, MultipleHiddenInput
- `render_widget()` function for HTML generation
- Widget renderers for all 23 widget types

#### Form
- `Form` struct with `new()`, `bind()`, `full_clean()`, `is_valid()`, `cleaned_data()`, `errors()`, `render()`, `as_p()`, `as_table()`, `as_div()`
- `ModelFormOptions`, `modelform_factory()` (basic)
- `FormState` with valid/invalid states
- CSRF input rendering

#### Formsets
- `BaseFormSet` with `new()`, `is_valid()`, `total_form_count()`, `initial_form_count()`, `cleaned_data()`, `render_all()`
- `ManagementForm`, `FormsetForm`
- `formset_factory()` function
- `modelformset_factory()` ✅ (new)
- `inlineformset_factory()` ✅ (new)
- `name_fieldset()` helper

### ❌ Missing (71 APIs)

#### Fields (critical)
| Missing Field | Status |
|--------------|--------|
| Per-field `clean_<name>()` hook | ❌ Core missing feature |
| `TypedChoiceField` | Partial (enum exists, no clean logic) |
| `TypedMultipleChoiceField` | Partial (enum exists, no clean logic) |
| `SplitDateTimeField` | Partial (enum exists, no clean logic) |
| ComboField | ❌ |
| `MultiValueField` | ❌ |
| `RegexField` | Partial (enum exists) |
| `ChoiceField` / `MultipleChoiceField` with proper choices | Partial |
| `JSONField` | ❌ |
| `GenericIPAddressField` clean logic | ❌ |
| Field `has_changed()` | ❌ |
| Field `prepare_value()` for rendering | ❌ |
| `validate()` / `run_validators()` | ❌ |

#### Forms (critical)
| Missing API | Status |
|-------------|--------|
| `hidden_fields()` / `visible_fields()` | ❌ |
| `non_field_errors()` | ❌ |
| `changed_data` | ❌ |
| `bound_field_class` integration | ❌ |
| Renderer system (template_name_table/ul/p/div) | ❌ |
| `Form.media` property | ❌ |
| `Form.is_multipart()` | ❌ |
| `add_prefix()` / `add_initial_prefix()` | ❌ |
| ErrorDict / ErrorList as structured output | Partial |

#### Formsets (critical)
| Missing API | Status |
|-------------|--------|
| `deleted_forms()` | ❌ |
| `ordered_forms()` | ❌ |
| `extra_forms()` | ❌ |
| `empty_form` | ❌ |
| `has_changed()` | ❌ |
| `non_form_errors()` | ❌ |
| `total_error_count()` | ❌ |
| `full_clean()` / `is_valid()` chain | Partial |
| `get_form_kwargs()` | ❌ |
| Deletion/ordering widgets | ❌ |

#### ModelForms (critical)
| Missing API | Status |
|-------------|--------|
| `ModelForm` (full) with `save()` | ❌ |
| `form.save(commit=False)` → save_m2m() | ❌ |
| `construct_instance()` | ❌ |
| `model_to_dict()`, `fields_for_model()` | ❌ |
| `ModelChoiceField`, `ModelMultipleChoiceField` | ❌ |
| `BaseModelFormSet`, `BaseInlineFormSet` (full) | ❌ |
| `inlineformset_factory()` (full) | Partial |
| `modelformset_factory()` (full) | Partial |
| Limit choices to model instances | ❌ |

#### Widgets
| Missing Widget | Status |
|----------------|--------|
| `MultiWidget` composition | ❌ |
| `SplitHiddenDateTimeWidget` | ❌ |
| `ColorInput`, `SearchInput`, `TelInput` | ❌ |
| Widget `attrs` handling | ❌ |
| Widget `Media` (CSS/JS) | ❌ |
| `build_attrs()` | ❌ |
| `id_for_label()` | ❌ |
| `use_required_attribute()` | ❌ |
| `value_from_datadict()` | ❌ |
| `value_omitted_from_data()` | ❌ |
| `format_value()` / `decompress()` | ❌ |
| `get_context()` | ❌ |

---

## 📊 Coverage Summary

| Submodule | Django | Rjango | % |
|-----------|--------|--------|---|
| Middleware | 14 | 14 | **100%** ✅ |
| Auth | 40 | 24 | **60%** |
| Forms | 83 | 12 | **15%** |
| **Total** | **137** | **50** | **36%** |
