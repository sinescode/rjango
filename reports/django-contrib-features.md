# Django 6.0.6 Contrib Packages — Feature Comparison with Rjango

> **Django modules analyzed:** contrib.admin, contrib.sessions, contrib.messages, contrib.sites, contrib.contenttypes, contrib.staticfiles, contrib.humanize, contrib.sitemaps, contrib.syndication, contrib.flatpages, contrib.redirects, contrib.postgres, contrib.gis

---

## 1. Admin (`django.contrib.admin`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class AdminSite` | `AdminSite` | ✅ | Present |
| `AdminSite.register(model, admin_class)` | `AdminSite::register()` | ✅ | |
| `AdminSite.unregister(model)` | ❌ Missing | |
| `AdminSite.autodiscover()` | ❌ Missing | |
| `AdminSite.each_context(request)` | ❌ Missing | |
| `AdminSite.index(request)` | ✅ | Site index view |
| `AdminSite.app_index(request, app_label)` | ✅ | App index view |
| `AdminSite.login(request)` | ❌ Missing | |
| `AdminSite.logout(request)` | ❌ Missing | |
| `AdminSite.password_change(request)` | ❌ Missing | |
| `AdminSite.password_change_done(request)` | ❌ Missing | |
| `AdminSite.get_urls()` | ❌ Missing | |
| `class ModelAdmin(BaseModelAdmin)` | `ModelAdmin` | ✅ | |
| `ModelAdmin.list_display` | ✅ | |
| `ModelAdmin.list_filter` | ❌ Missing | |
| `ModelAdmin.search_fields` | ❌ Missing | |
| `ModelAdmin.ordering` | ❌ Missing | |
| `ModelAdmin.list_per_page` | ❌ Missing | |
| `ModelAdmin.list_max_show_all` | ❌ Missing | |
| `ModelAdmin.date_hierarchy` | ❌ Missing | |
| `ModelAdmin.fieldsets` | ❌ Missing | |
| `ModelAdmin.fields` | ❌ Missing | |
| `ModelAdmin.exclude` | ❌ Missing | |
| `ModelAdmin.readonly_fields` | ❌ Missing | |
| `ModelAdmin.prepopulated_fields` | ❌ Missing | |
| `ModelAdmin.autocomplete_fields` | ❌ Missing | |
| `ModelAdmin.actions` | ❌ Missing | |
| `ModelAdmin.changelist_view(request)` | ✅ | |
| `ModelAdmin.add_view(request)` | ✅ | |
| `ModelAdmin.change_view(request, obj_id)` | ✅ | |
| `ModelAdmin.delete_view(request, obj_id)` | ✅ | |
| `ModelAdmin.history_view(request, obj_id)` | ❌ Missing | |
| `ModelAdmin.get_queryset(request)` | ❌ Missing | |
| `ModelAdmin.get_search_results(...)` | ❌ Missing | |
| `ModelAdmin.save_model(...)` | ❌ Missing | |
| `ModelAdmin.save_formset(...)` | ❌ Missing | |
| `ModelAdmin.delete_model(...)` | ❌ Missing | |
| `ModelAdmin.delete_queryset(...)` | ❌ Missing | |
| `class InlineModelAdmin(BaseModelAdmin)` | ❌ Missing | |
| `class StackedInline(InlineModelAdmin)` | ❌ Missing | |
| `class TabularInline(InlineModelAdmin)` | ❌ Missing | |
| `class ShowFacets` | ❌ Missing | |
| `IncorrectLookupParameters` | ❌ Missing | |
| `actions` module | ❌ Missing | |
| `filters` module | ❌ Missing | |
| `helpers` module | ❌ Missing | |
| `checks` module | ❌ Missing | |
| `autodiscover()` | ❌ Missing | |
| `site = DefaultAdminSite()` | ✅ | Default admin site singleton |

---

## 2. Sessions (`django.contrib.sessions`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class SessionMiddleware` | `SessionMiddleware` | ✅ | |
| `class SessionBase` | ❌ Missing | |
| `SessionBase.cycle_key()` | ❌ Missing | |
| `SessionBase.flush()` | ❌ Missing | |
| `SessionBase.set_expiry(value)` | ❌ Missing | |
| `SessionBase.get_expiry_age()` | ❌ Missing | |
| `SessionBase.get_expiry_date()` | ❌ Missing | |
| `SessionBase.get_expire_at_browser_close()` | ❌ Missing | |
| `db backend` | ❌ Missing | Database-backed sessions |
| `cache backend` | ❌ Missing | |
| `file backend` | ❌ Missing | |
| `signed_cookies backend` | ❌ Missing | |
| `class CreateError` | ❌ Missing | |
| `class UpdateError` | ❌ Missing | |
| `SESSION_ENGINE` setting | ❌ Missing | |
| `SESSION_COOKIE_NAME` setting | ❌ Missing | |
| `SESSION_COOKIE_AGE` setting | ❌ Missing | |

---

## 3. Messages (`django.contrib.messages`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class MessageMiddleware` | `MessageMiddleware` | ✅ | |
| `class Message(level, message, extra_tags)` | `Message` | ✅ | |
| `Message.level` | ✅ | |
| `Message.message` | ✅ | |
| `Message.tags` | ✅ | |
| `class BaseStorage` | `MessageStorage` | ⚠️ Partial | |
| `class SessionStorage(BaseStorage)` | ✅ | |
| `class CookieStorage(BaseStorage)` | ❌ Missing | |
| `class FallbackStorage(Session, Cookie)` | ❌ Missing | |
| `add_message(request, level, message)` | ✅ | |
| `get_messages(request)` | ✅ | |
| `get_level(request)` | ❌ Missing | |
| `set_level(request, level)` | ❌ Missing | |
| `debug(request, message)` | ✅ | |
| `info(request, message)` | ✅ | |
| `success(request, message)` | ✅ | |
| `warning(request, message)` | ✅ | |
| `error(request, message)` | ✅ | |
| `constants: DEBUG, INFO, SUCCESS, WARNING, ERROR` | ✅ | |
| `default_tags` | ❌ Missing | |
| `class MessageFailure` | ❌ Missing | |
| `context_processors` | ❌ Missing | |

---

## 4. Sites (`django.contrib.sites`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Site(models.Model)` | ❌ Missing | |
| `class SiteManager(models.Manager)` | ❌ Missing | |
| `class CurrentSiteMiddleware` | ❌ Missing | |
| `get_current_site(request)` | ❌ Missing | |
| `shortcut` functions | ❌ Missing | |
| `SITE_ID` setting | ❌ Missing | |

---

## 5. Contenttypes (`django.contrib.contenttypes`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class ContentType(models.Model)` | ❌ Missing | |
| `class GenericForeignKey` | ❌ Missing | |
| `class GenericRelation` | ❌ Missing | |
| `class GenericRel` | ❌ Missing | |
| `create_generic_related_manager()` | ❌ Missing | |
| `class BaseGenericInlineFormSet` | ❌ Missing | |
| `generic_inlineformset_factory()` | ❌ Missing | |

---

## 6. Staticfiles (`django.contrib.staticfiles`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class BaseFinder` | ❌ Missing | |
| `class FileSystemFinder(BaseFinder)` | ❌ Missing | |
| `class AppDirectoriesFinder(BaseFinder)` | ❌ Missing | |
| `class BaseStorageFinder(BaseFinder)` | ❌ Missing | |
| `class DefaultStorageFinder(BaseFinder)` | ❌ Missing | |
| `find(path)` | ❌ Missing | |
| `get_finders()` | ❌ Missing | |
| `class StaticFilesHandler` | ❌ Missing | |
| `class StaticFilesView` | ❌ Missing | |
| `collectstatic` management command | `collectstatic` command | ✅ | Present in CLI |
| `static()` template tag | ❌ Missing | |

---

## 7. Humanize (`django.contrib.humanize`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `intcomma(value)` | ❌ Missing | |
| `intword(value)` | ❌ Missing | |
| `apnumber(value)` | ❌ Missing | |
| `naturalday(value)` | ❌ Missing | |
| `naturaltime(value)` | ❌ Missing | |
| `ordinal(value)` | ❌ Missing | |

---

## 8. Sitemaps (`django.contrib.sitemaps`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Sitemap` | ❌ Missing | |
| `class GenericSitemap` | ❌ Missing | |
| `class FlatPageSitemap` | ❌ Missing | |
| `sitemap view` | ❌ Missing | |
| `index view` | ❌ Missing | |

---

## 9. Syndication (`django.contrib.syndication`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Feed` | ❌ Missing | RSS/Atom feed framework |
| `Feed.title` | ❌ Missing | |
| `Feed.link` | ❌ Missing | |
| `Feed.description` | ❌ Missing | |
| `Feed.items()` | ❌ Missing | |
| `Feed.item_title(item)` | ❌ Missing | |
| `Feed.item_description(item)` | ❌ Missing | |
| `feed view` | ❌ Missing | |

---

## 10. Flatpages (`django.contrib.flatpages`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class FlatPage(models.Model)` | ❌ Missing | |
| `class FlatPageSitemap` | ❌ Missing | |
| `flatpage view` | ❌ Missing | |
| `FlatpageFallbackMiddleware` | ❌ Missing | |

---

## 11. Redirects (`django.contrib.redirects`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `class Redirect(models.Model)` | ❌ Missing | |
| `RedirectFallbackMiddleware` | ❌ Missing | |

---

## 12. PostgreSQL (`django.contrib.postgres`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `ArrayField` | ❌ Missing | |
| `HStoreField` | ❌ Missing | |
| `JSONField` | ❌ Missing | |
| `RangeFields` (IntegerRange, etc.) | ❌ Missing | |
| `SearchVector, SearchQuery, SearchRank` | ❌ Missing | Full-text search |
| `TrigramSimilarity` | ❌ Missing | |
| `Unaccent` | ❌ Missing | |
| `HStoreExtension` | ❌ Missing | |
| `BtreeGinExtension` | ❌ Missing | |
| `aggregates: ArrayAgg, StringAgg` | ❌ Missing | |
| `indexes: GinIndex, BrinIndex` | ❌ Missing | |
| `constraints: ExclusionConstraint` | ❌ Missing | |
| `Lookups: TrigramSimilar, Unaccent` | ❌ Missing | |

---

## 13. GIS (`django.contrib.gis`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| Entire `django.contrib.gis` module | ❌ Missing | |
| `GeoDjango` model fields | ❌ Missing | |
| `PointField, LineStringField, PolygonField` | ❌ Missing | |
| `GeoQuerySet` | ❌ Missing | |
| `GDAL` / `GEOS` / `PROJ` bindings | ❌ Missing | |
| `GeoIP` | ❌ Missing | |
| `KML` / `GML` output | ❌ Missing | |

---

## 14. Admindocs (`django.contrib.admindocs`)

| Django API | Rjango Equivalent | Status | Notes |
|---|---|---|---|
| `admindocs` module | ❌ Missing | Auto-generated admin docs |

---

## Summary

### Rjango Contrib Features (✅ Complete)
- ✅ AdminSite with register, index, app_index, CRUD views
- ✅ ModelAdmin with changelist/add/change/delete views
- ✅ Message middleware & storage (session-based)
- ✅ Message shortcuts (debug, info, success, warning, error)
- ✅ Session middleware (basic)
- ✅ collectstatic command

### Missing Contrib Features (❌)
| Package | Status | Priority |
|---|---|---|
| **PostgreSQL contrib** | ❌ Entirely missing | High |
| **ContentTypes** | ❌ Entirely missing | High |
| **Sites** | ❌ Entirely missing | Medium |
| **Staticfiles** | ❌ Missing finders/handlers/static tag | Medium |
| **Flatpages** | ❌ Entirely missing | Low |
| **Redirects** | ❌ Entirely missing | Low |
| **Sitemaps** | ❌ Entirely missing | Low |
| **Syndication (RSS/Atom)** | ❌ Entirely missing | Low |
| **Humanize** | ❌ Entirely missing | Low |
| **GIS** | ❌ Entirely missing | Low |
| **Admindocs** | ❌ Entirely missing | Low |
| **Admin filters** | ❌ Missing | Medium |
| **Admin inlines** | ❌ Missing | Medium |
| **Admin actions** | ❌ Missing | Medium |
| **Advanced session backends** | ❌ Missing | Medium |
