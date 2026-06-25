# CLI, Migrations & Testing: Django vs Rjango — Exhaustive Comparison

**Date**: 2026-06-25 (Updated)  
**Django Version**: 6.0.6  
**Rjango Version**: 0.1.0  

---

## django.core.management vs rjango-cli

Rjango Location: `rjango-cli/src/main.rs` + `commands/mod.rs` (230 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| `manage.py` | CLI entry | `rjango` binary | ✅ YES | |
| Command framework | BaseCommand | Hand-written dispatch | ⚠️ PARTIAL | Not extensible yet |

### Commands

| Command | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| runserver | Dev server | ✅ | ✅ YES | Starts rjango TCP server |
| startproject | Create project | ✅ | ✅ YES | Creates project structure |
| startapp | Create app | ✅ | ✅ YES | Creates app stub |
| migrate | Run migrations | ✅ | ✅ YES | Generates SQL, creates tables |
| makemigrations | Detect changes | ✅ | ✅ YES | Basic detection |
| showmigrations | List migrations | ✅ | ✅ YES | Lists apply status |
| createsuperuser | Create admin | ✅ | ✅ YES | Creates user |
| shell | REPL | ✅ | ✅ YES | Interactive Rust expression loop |
| check | Diagnose issues | ✅ | ✅ YES | Runs system checks |
| test | Run tests | ✅ | ✅ YES | Runs tests |
| validate | Validate config | ✅ | ✅ YES | Same as check |
| collectstatic | Gather static | ✅ | ✅ YES | Copy static files |
| console | REPL alias | ✅ | ✅ YES | |
| dbshell | DB CLI | ✅ | ✅ YES | Opens sqlite3 CLI |

### Missing Commands

| Command | Purpose | Severity |
|---------|---------|----------|
| diffsettings | Show settings diff | MEDIUM |
| dumpdata | Export data | MEDIUM |
| loaddata | Import data | MEDIUM |
| inspectdb | Introspect DB | HIGH |
| sqlmigrate | Show migration SQL | MEDIUM |
| squashmigrations | Combine migrations | LOW |
| makemessages | i18n extraction | LOW |
| compilemessages | i18n compilation | LOW |
| sendtestemail | Test email | LOW |
| flush | Clear DB | LOW |
| sqlflush | Show flush SQL | LOW |
| sqlsequencereset | Reset sequences | LOW |
| changepassword | Change user PW | LOW |
| remove_stale_contenttypes | Cleanup | LOW |
| testserver | Test with fixtures | LOW |
| optimizemigration | Migration optimization | LOW |
| createcachetable | Create cache table | LOW |

### Summary

| AREA | STATUS | LINES |
|------|--------|-------|
| CLI framework | ⚠️ PARTIAL | 179 (main.rs) |
| Command implementations | ✅ YES (14 commands) | directory-based dispatch |
| Command extensibility | ❌ NO | Can't add custom commands |
| Output formatting | ⚠️ PARTIAL | Plain println! |

---

## django.db.migrations vs rjango-migrations

Rjango Location: `rjango-migrations/src/` (359 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| MigrationOperation enum | All operations | `MigrationOperation` enum | ✅ YES | CreateModel, DeleteModel, AddField, RemoveField, AlterField, AlterModelTable, RunSQL, RunPython |
| Operations to SQL | SQL per operation | `to_sql()` | ✅ YES | Generates real SQL |
| Graph | Dependency graph | — | ❌ NO | |
| Autodetector | Compare models | `detector.rs` | ⚠️ PARTIAL | Basic detection |
| Executor | Run migrations | `runner.rs` | ✅ YES | Creates/drops tables via execute() |
| Migration file writer | Generate files | — | ❌ NO | |
| Recorder | Track applied | `recorder` | ⚠️ PARTIAL | Basic track in runner |
| Reversibility | Rollback | — | ❌ NO | |
| Optimizer | Minimize ops | — | ❌ NO | |
| Serializer | Python→JSON | — | ❌ NO | |
| Questioner | Ask questions | — | ❌ NO | |
| **Tests** | — | 5 tests | ✅ | |

### Operations Detail

| Operation | Status | SQL Generated |
|-----------|--------|-------------|
| CreateModel | ✅ | CREATE TABLE |
| DeleteModel | ✅ | DROP TABLE |
| AddField | ✅ | ALTER TABLE ADD |
| RemoveField | ✅ | ALTER TABLE DROP |
| AlterField | ✅ | ALTER TABLE MODIFY |
| AlterModelTable | ✅ | ALTER TABLE RENAME TO |
| RunSQL | ✅ | Executes raw SQL |
| RunPython | ⚠️ | No-op (not applicable to Rust) |
| AlterUniqueTogether | ❌ | |
| AlterIndexTogether | ❌ | |
| RenameModel | ❌ | |
| AlterOrderWithRespectTo | ❌ | |
| SeparateDatabaseAndState | ❌ | |

---

## django.test vs rjango-test

Rjango Location: `rjango-test/src/` (161 lines)

| Feature | Django | Rjango | Status | Notes |
|---------|--------|--------|--------|-------|
| TestCase | Base test class | `TestCase` struct | ✅ YES | |
| Client | Request testing | `TestClient` | ✅ YES | get/post with Request→Response |
| RequestFactory | Build requests | In TestClient | ✅ YES | |
| Runner | Run tests | `TestRunner` struct | ✅ YES | Runs test files |
| Discovery | Auto-find tests | — | ❌ NO | |
| TransactionTestCase | DB isolation | — | ❌ NO | |
| LiveServerTestCase | Live server | — | ❌ NO | |
| Async tests | Async test case | — | ❌ NO | |
| Tagging | Test tags | — | ❌ NO | |
| Fixtures | Test data | — | ❌ NO | |
| Assertions | Rich assertions | Basic `assert!` | ❌ NO | |
| `modify_settings` | Context manager | — | ❌ NO | |
| **Tests** | — | 11 integration tests | ✅ | |
