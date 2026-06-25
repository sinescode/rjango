# Django vs Rjango: Auth & Admin Exhaustive Comparison Report

**Date**: 2026-06-25 (Updated)  
**Django Version**: 6.0.6  
**Rjango Version**: 0.1.0  

---

## django.contrib.auth vs rjango-auth

Rjango Location: `rjango-auth/src/` (365 lines)

### Models

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| User model | AbstractUser | `User` struct | ✅ YES | id, username, email, password, is_active, is_staff, is_superuser |
| Group model | Group permissions | — | ❌ NO | |
| Permission model | Permission matrix | — | ❌ NO | |
| AnonymousUser | Unauthenticated | `AnonymousUser` struct | ✅ YES | |
| last_login / date_joined | Timestamps | Included | ✅ YES | |

### Backends

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| ModelBackend | Auth by model | `ModelBackend` struct | ✅ YES | |
| RemoteUserBackend | External auth | — | ❌ NO | |
| Backend registry | AUTHENTICATION_BACKENDS | `get_backends()` function | ✅ YES | |
| authenticate() | Auth call | In backend trait | ✅ YES | |
| get_user() | User lookup | In backend trait | ✅ YES | |

### Password Hashing

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| PBKDF2PasswordHasher | Standard | — | ❌ NO | Uses DefaultHasher (not secure) |
| Argon2PasswordHasher | Recommended | — | ❌ NO | |
| BCryptSHA256PasswordHasher | Compat | — | ❌ NO | |
| check_password() | Verify hash | — | ❌ NO | |
| set_password() | Hash storage | — | ❌ NO | |
| is_password_usable() | Check hash | — | ❌ NO | |
| make_password() | Create hash | — | ❌ NO | |

### Middleware

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| AuthenticationMiddleware | Attach user | `AuthMiddleware` | ✅ YES | Adds user to request |
| RemoteUserMiddleware | External user | — | ❌ NO | |
| LoginRequiredMiddleware | — | — | ❌ NO | |

### Views

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| LoginView | Login form | `login_view()` | ✅ YES | Styled HTML, CSRF, form handling |
| LogoutView | Logout | `logout_view()` | ✅ YES | Clears session |
| PasswordChangeView | Change PW | — | ❌ NO | |
| PasswordResetView | Reset PW | — | ❌ NO | |
| LoginRequired mixin | Require login | — | ❌ NO | |
| PermissionRequiredMixin | Require perm | — | ❌ NO | |

### Permissions

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| Default permissions (add/change/delete/view) | Auto per model | — | ❌ NO | |
| Permission checks | has_perm() | — || ❌ NO | |
| Content types permissions | permission per model | — | ❌ NO | |
| Permission admin | Admin UI for perms | — | ❌ NO | |

### Signals

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| user_logged_in | Login signal | In core signals | ✅ YES | |
| user_logged_out | Logout signal | In core signals | ✅ YES | |
| user_login_failed | Failed login | In core signals | ✅ YES | |

### Summary

| AREA | STATUS | LOCATION | LINES |
|------|--------|----------|-------|
| User/AnonymousUser models | ✅ YES | models.rs | 50 |
| Auth backends | ✅ YES | backends.rs | 36 |
| Auth middleware | ✅ YES | middleware_.rs | 46 |
| Login/Logout views | ✅ YES | views.rs | 174 |
| Password hashing | ❌ NO | — | — |
| Permissions | ❌ NO | — | — |
| Groups | ❌ NO | — | — |

---

## django.contrib.admin vs rjango-admin

Rjango Location: `rjango-admin/src/` (459 lines)

### AdminSite

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| AdminSite class | Admin container | `AdminSite` struct | ✅ YES | |
| register() | Register models | `register()` method | ✅ YES | |
| unregister() | Unregister | — | ❌ NO | |
| urls() | URL patterns | `urls()` method | ✅ YES | Returns URLResolver |
| index() | Dashboard | `index_view()` | ✅ YES | Styled HTML |

### Model CRUD Views

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| changelist_view() | List models | `changelist_view()` | ✅ YES | Styled HTML table |
| add_view() | Create model | `add_view()` | ✅ YES | Styled form |
| change_view() | Edit model | `change_view()` | ✅ YES | Styled form |
| delete_view() | Delete model | `delete_view()` | ✅ YES | Confirm page |
| history_view() | Audit log | — | ❌ NO | |

### Admin Features

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| List display | Column config | — | ❌ NO | |
| List filter | Filter sidebar | — | ❌ NO | |
| Search | Search box | — | ❌ NO | |
| Inline models | Related models | — | ❌ NO | |
| Actions | Bulk actions | — | ❌ NO | |
| Autocomplete | Select2 widget | — | ❌ NO | |
| Admin ordering | Default sort | — | ❌ NO | |
| Fieldsets | Form layout | — | ❌ NO | |
| Read-only fields | Display only | — | ❌ NO | |
| Admin widgets | Rich inputs | — | ❌ NO | |
| Permission checks | Per-user admin | — | ❌ NO | |
| Admin templates | Extensible | — | ❌ NO | |
| Log entry model | Audit entries | — | ❌ NO | |

### Summary

| AREA | STATUS | LOCATION | LINES |
|------|--------|----------|-------|
| AdminSite + registration | ✅ YES | sites.rs | 395 |
| Index view | ✅ YES | sites.rs | — |
| Changelist view | ✅ YES | sites.rs | — |
| Add/Change views | ✅ YES | sites.rs | — |
| Delete view | ✅ YES | sites.rs | — |
| Model connection to admin | ❌ NO | — | — |
| Admin templates | ❌ NO | — | — |
