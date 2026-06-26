/// Query builder and QuerySet — Django-like ORM query interface.
/// Handles SQL construction and execution via DatabasePool.

use std::collections::HashMap;
use crate::executor::DatabasePool;
use crate::lookups::{FilterCondition, Lookup, parse_filters};

/// A query builder that constructs SQL from method calls.
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    fields: Vec<String>,
    conditions: Vec<(String, String, String)>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    joins: Vec<String>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            fields: vec!["*".into()],
            conditions: vec![],
            order_by: vec![],
            limit: None,
            offset: None,
            joins: vec![],
        }
    }

    pub fn filter(mut self, field: &str, op: &str, value: &str) -> Self {
        self.conditions.push((field.to_string(), op.to_string(), value.to_string()));
        self
    }

    pub fn exclude(mut self, field: &str, op: &str, value: &str) -> Self {
        self.conditions.push((field.to_string(), format!("NOT {}", op), value.to_string()));
        self
    }

    pub fn filter_kwargs(mut self, filters: &HashMap<String, String>) -> Self {
        let conditions = parse_filters(filters);
        for cond in conditions {
            self.add_condition(&cond);
        }
        self
    }

    fn add_condition(&mut self, cond: &FilterCondition) {
        let field = &cond.field;
        let val = &cond.value;
        let escaped = val.replace('\'', "''");
        match cond.lookup {
            Lookup::Exact => self.conditions.push((field.clone(), "=".into(), format!("'{}'", escaped))),
            Lookup::IExact => self.conditions.push((field.clone(), "LIKE".into(), format!("'{}'", escaped.to_lowercase()))),
            Lookup::Contains => self.conditions.push((field.clone(), "LIKE".into(), format!("'%{}%'", escaped))),
            Lookup::IContains => self.conditions.push((field.clone(), "LIKE".into(), format!("'%{}%'", escaped.to_lowercase()))),
            Lookup::In => {
                let items: Vec<String> = val.split(',')
                    .map(|s| format!("'{}'", s.trim().replace('\'', "''")))
                    .collect();
                self.conditions.push((field.clone(), "IN".into(), format!("({})", items.join(", "))));
            }
            Lookup::Gt => self.conditions.push((field.clone(), ">".into(), val.clone())),
            Lookup::Gte => self.conditions.push((field.clone(), ">=".into(), val.clone())),
            Lookup::Lt => self.conditions.push((field.clone(), "<".into(), val.clone())),
            Lookup::Lte => self.conditions.push((field.clone(), "<=".into(), val.clone())),
            Lookup::StartsWith => self.conditions.push((field.clone(), "LIKE".into(), format!("'{}%'", escaped))),
            Lookup::IStartsWith => self.conditions.push((field.clone(), "LIKE".into(), format!("'{}%'", escaped.to_lowercase()))),
            Lookup::EndsWith => self.conditions.push((field.clone(), "LIKE".into(), format!("'%{}'", escaped))),
            Lookup::IEndsWith => self.conditions.push((field.clone(), "LIKE".into(), format!("'%{}'", escaped.to_lowercase()))),
            Lookup::Range => {
                let parts: Vec<&str> = val.splitn(2, ',').collect();
                let low = parts.first().unwrap_or(&"");
                let high = parts.get(1).unwrap_or(&"");
                self.conditions.push((field.clone(), "BETWEEN".into(), format!("'{}' AND '{}'", low, high)));
            }
            Lookup::IsNull => {
                if val == "false" || val == "0" || val == "False" { self.conditions.push((field.clone(), "IS NOT".into(), "NULL".into())); }
                else { self.conditions.push((field.clone(), "IS".into(), "NULL".into())); }
            }
            Lookup::Ne => self.conditions.push((field.clone(), "!=".into(), format!("'{}'", escaped))),
            _ => self.conditions.push((field.clone(), "=".into(), format!("'{}'", escaped))),
        }
    }

    pub fn order_by(mut self, field: &str) -> Self { self.order_by.push(field.to_string()); self }
    pub fn limit(mut self, n: usize) -> Self { self.limit = Some(n); self }
    pub fn offset(mut self, n: usize) -> Self { self.offset = Some(n); self }
    pub fn join(mut self, table: &str, on: &str) -> Self { self.joins.push(format!("JOIN {} ON {}", table, on)); self }

    pub fn build_select(&self) -> String {
        let mut sql = format!("SELECT {} FROM {}", self.fields.join(", "), self.table);
        for join in &self.joins { sql.push_str(&format!(" {}", join)); }
        if !self.conditions.is_empty() {
            let clauses: Vec<String> = self.conditions.iter().map(|(f, op, v)| format!("{} {} {}", f, op, v)).collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }
        if !self.order_by.is_empty() { sql.push_str(&format!(" ORDER BY {}", self.order_by.join(", "))); }
        if let Some(limit) = self.limit { sql.push_str(&format!(" LIMIT {}", limit)); }
        if let Some(offset) = self.offset { sql.push_str(&format!(" OFFSET {}", offset)); }
        sql
    }

    pub fn build_count(&self) -> String {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table);
        for join in &self.joins { sql.push_str(&format!(" {}", join)); }
        if !self.conditions.is_empty() {
            let clauses: Vec<String> = self.conditions.iter().map(|(f, op, v)| format!("{} {} {}", f, op, v)).collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }
        sql
    }

    pub fn build_insert(data: &HashMap<String, String>, table: &str) -> String {
        let fields: Vec<&str> = data.keys().map(|k| k.as_str()).collect();
        let values: Vec<&str> = data.values().map(|v| v.as_str()).collect();
        format!("INSERT INTO {} ({}) VALUES ({})", table, fields.join(", "), values.join(", "))
    }

    pub fn build_update(data: &HashMap<String, String>, table: &str, pk_field: &str, pk_value: &str) -> String {
        let sets: Vec<String> = data.iter().map(|(k, v)| format!("{} = {}", k, v)).collect();
        format!("UPDATE {} SET {} WHERE {} = {}", table, sets.join(", "), pk_field, pk_value)
    }

    pub fn build_delete(table: &str, pk_field: &str, pk_value: &str) -> String {
        format!("DELETE FROM {} WHERE {} = {}", table, pk_field, pk_value)
    }
}

/// A QuerySet wraps SQL building + a database pool for real execution.
#[derive(Debug, Clone)]
pub struct QuerySet {
    builder: QueryBuilder,
    pool: Option<DatabasePool>,
}

impl QuerySet {
    pub fn new(table: &str) -> Self { Self { builder: QueryBuilder::new(table), pool: None } }
    pub fn using(mut self, pool: DatabasePool) -> Self { self.pool = Some(pool); self }
    pub fn filter(mut self, field: &str, op: &str, value: &str) -> Self { self.builder = self.builder.filter(field, op, value); self }
    pub fn filter_kwargs(mut self, filters: &HashMap<String, String>) -> Self { self.builder = self.builder.filter_kwargs(filters); self }
    pub fn exclude(mut self, field: &str, op: &str, value: &str) -> Self { self.builder = self.builder.exclude(field, op, value); self }
    pub fn order_by(mut self, field: &str) -> Self { self.builder = self.builder.order_by(field); self }
    pub fn limit(mut self, n: usize) -> Self { self.builder = self.builder.limit(n); self }
    pub fn offset(mut self, n: usize) -> Self { self.builder = self.builder.offset(n); self }
    pub fn sql(&self) -> String { self.builder.build_select() }
    pub fn count_sql(&self) -> String { self.builder.build_count() }

    pub fn all(&self) -> crate::Result<Vec<HashMap<String, String>>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        pool.query(&self.builder.build_select())
    }
    pub fn values(&self) -> crate::Result<Vec<HashMap<String, String>>> { self.all() }
    pub fn first(&self) -> crate::Result<Option<HashMap<String, String>>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let rows = pool.query(&self.clone().limit(1).builder.build_select())?;
        Ok(rows.into_iter().next())
    }
    pub fn last(&self) -> crate::Result<Option<HashMap<String, String>>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let rows = pool.query(&self.clone().order_by("id DESC").limit(1).builder.build_select())?;
        Ok(rows.into_iter().next())
    }
    pub fn count(&self) -> crate::Result<i64> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let rows = pool.query(&self.builder.build_count())?;
        Ok(rows.first().and_then(|r| r.values().next()).and_then(|v| v.parse().ok()).unwrap_or(0))
    }
    pub fn exists(&self) -> crate::Result<bool> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        Ok(!pool.query(&self.clone().limit(1).builder.build_select())?.is_empty())
    }
    pub fn get_by_pk(&self, pk: i64) -> crate::Result<HashMap<String, String>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let mut rows = pool.query(&format!("SELECT * FROM {} WHERE id = {}", self.builder.table, pk))?;
        rows.pop().ok_or_else(|| format!("{} pk={} does not exist", self.builder.table, pk).into())
    }
    pub fn create(&self, data: HashMap<String, String>) -> crate::Result<i64> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        pool.insert(&QueryBuilder::build_insert(&data, &self.builder.table))
    }
    pub fn update(&self, data: HashMap<String, String>) -> crate::Result<u64> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        pool.execute(&self.build_update_sql(&data))
    }
    pub fn delete(&self) -> crate::Result<u64> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        pool.execute(&self.build_delete_sql())
    }
    pub fn bulk_create(&self, rows: Vec<HashMap<String, String>>) -> crate::Result<u64> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let mut total = 0u64;
        for data in rows { pool.execute(&QueryBuilder::build_insert(&data, &self.builder.table))?; total += 1; }
        Ok(total)
    }
    pub fn in_bulk(&self, ids: &[i64]) -> crate::Result<Vec<HashMap<String, String>>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let id_list: Vec<String> = ids.iter().map(|i| i.to_string()).collect();
        pool.query(&format!("SELECT * FROM {} WHERE id IN ({})", self.builder.table, id_list.join(", ")))
    }

    /// Get a single object matching the given filter. Returns an error if 0 or >1 match.
    pub fn get(&self, filters: &HashMap<String, String>) -> crate::Result<HashMap<String, String>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let mut qs = self.clone();
        for (k, v) in filters { qs = qs.filter(k, "exact", v); }
        let rows = pool.query(&qs.builder.build_select())?;
        if rows.is_empty() { return Err(format!("{} matching query does not exist", self.builder.table).into()); }
        if rows.len() > 1 { return Err(format!("{} matching query returned more than 1 result", self.builder.table).into()); }
        Ok(rows.into_iter().next().unwrap())
    }

    /// Get or create — tries to find, creates if not found.
    pub fn get_or_create(&self, filters: &HashMap<String, String>, defaults: &HashMap<String, String>) -> crate::Result<(HashMap<String, String>, bool)> {
        match self.get(filters) {
            Ok(obj) => Ok((obj, false)),
            Err(_) => {
                let mut data = defaults.clone();
                for (k, v) in filters { data.entry(k.clone()).or_insert_with(|| v.clone()); }
                let id = self.create(data)?;
                Ok((self.get_by_pk(id as i64)?, true))
            }
        }
    }

    /// Update or create — tries to find, updates if found, creates if not.
    pub fn update_or_create(&self, filters: &HashMap<String, String>, defaults: &HashMap<String, String>) -> crate::Result<(HashMap<String, String>, bool)> {
        match self.get(filters) {
            Ok(mut obj) => {
                let mut data = defaults.clone();
                for (k, v) in &obj { data.entry(k.clone()).or_insert_with(|| v.clone()); }
                let pk = obj.get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                self.clone().filter("id", "exact", &pk.to_string()).update(data)?;
                obj.extend(defaults.clone());
                Ok((obj, false))
            }
            Err(_) => {
                let mut data = defaults.clone();
                for (k, v) in filters { data.entry(k.clone()).or_insert_with(|| v.clone()); }
                let id = self.create(data)?;
                Ok((self.get_by_pk(id as i64)?, true))
            }
        }
    }

    /// Get the latest object ordered by a field.
    pub fn latest(&self, field: &str) -> crate::Result<HashMap<String, String>> {
        self.clone().order_by(format!("{} DESC", field).as_str()).first()
            .and_then(|r| r.ok_or_else(|| format!("{} has no objects", self.builder.table).into()))
    }

    /// Get the earliest object ordered by a field.
    pub fn earliest(&self, field: &str) -> crate::Result<HashMap<String, String>> {
        self.clone().order_by(format!("{} ASC", field).as_str()).first()
            .and_then(|r| r.ok_or_else(|| format!("{} has no objects", self.builder.table).into()))
    }

    fn build_update_sql(&self, data: &HashMap<String, String>) -> String {
        let sets: Vec<String> = data.iter().map(|(k, v)| format!("{} = '{}'", k, v.replace('\'', "''"))).collect();
        let mut sql = format!("UPDATE {} SET {}", self.builder.table, sets.join(", "));
        if !self.builder.conditions.is_empty() {
            let clauses: Vec<String> = self.builder.conditions.iter().map(|(f, op, v)| format!("{} {} {}", f, op, v)).collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }
        sql
    }
    fn build_delete_sql(&self) -> String {
        let mut sql = format!("DELETE FROM {}", self.builder.table);
        if !self.builder.conditions.is_empty() {
            let clauses: Vec<String> = self.builder.conditions.iter().map(|(f, op, v)| format!("{} {} {}", f, op, v)).collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }
        sql
    }
}

/// QuerySetManager — lightweight manager for model operations.
pub struct QuerySetManager {
    pub table: String,
    pub pool: DatabasePool,
}

impl QuerySetManager {
    pub fn new(table: &str, pool: DatabasePool) -> Self { Self { table: table.to_string(), pool } }
    pub fn qs(&self) -> QuerySet { QuerySet::new(&self.table).using(self.pool.clone()) }
    pub fn all(&self) -> QuerySet { self.qs() }
    pub fn filter(&self, field: &str, op: &str, value: &str) -> QuerySet { self.qs().filter(field, op, value) }
    pub fn exclude(&self, field: &str, op: &str, value: &str) -> QuerySet { self.qs().exclude(field, op, value) }
    pub fn create(&self, data: HashMap<String, String>) -> crate::Result<i64> { self.qs().create(data) }
    pub fn get_by_pk(&self, pk: i64) -> crate::Result<HashMap<String, String>> { self.qs().get_by_pk(pk) }
    pub fn count(&self) -> crate::Result<i64> { self.qs().count() }
    pub fn exists(&self) -> crate::Result<bool> { self.qs().exists() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        assert_eq!(QueryBuilder::new("users").build_select(), "SELECT * FROM users");
    }

    #[test]
    fn test_filtered_select() {
        let sql = QueryBuilder::new("users").filter("age", ">", "18").filter("active", "=", "1").build_select();
        assert!(sql.contains("age > 18") && sql.contains("active = 1"));
    }

    #[test]
    fn test_select_with_limits() {
        let sql = QueryBuilder::new("posts").filter("published", "=", "true").order_by("created_at DESC").limit(10).build_select();
        assert!(sql.contains("LIMIT 10") && sql.contains("ORDER BY created_at DESC"));
    }

    #[test]
    fn test_query_set_chaining() {
        let sql = QuerySet::new("products").filter("price", ">", "10").filter("in_stock", "=", "true").order_by("name").sql();
        assert!(sql.contains("WHERE") && sql.contains("ORDER BY name"));
    }

    #[test]
    fn test_insert_build() {
        let mut data = HashMap::new();
        data.insert("name".into(), "'John'".into());
        data.insert("age".into(), "30".into());
        assert!(QueryBuilder::build_insert(&data, "users").starts_with("INSERT INTO users"));
    }

    #[test]
    fn test_count_query() {
        assert_eq!(QueryBuilder::new("users").filter("active", "=", "1").build_count(), "SELECT COUNT(*) FROM users WHERE active = 1");
    }

    #[test]
    fn test_query_set_no_pool() {
        let qs = QuerySet::new("items");
        assert!(qs.first().is_err() && qs.count().is_err() && qs.delete().is_err());
    }

    #[test]
    fn test_query_set_limit_offset() {
        let sql = QuerySet::new("posts").filter("published", "=", "1").order_by("-created_at").limit(10).offset(20).sql();
        assert!(sql.contains("LIMIT 10") && sql.contains("OFFSET 20"));
    }

    #[test]
    fn test_build_update_sql() {
        let mut data = HashMap::new();
        data.insert("name".into(), "Bob".into());
        let sql = QuerySet::new("users").filter("id", "=", "1").build_update_sql(&data);
        assert!(sql.contains("UPDATE users SET") && sql.contains("name = 'Bob'") && sql.contains("WHERE id = 1"));
    }

    #[test]
    fn test_query_set_using() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        let qs = QuerySet::new("items").using(pool);
        assert!(qs.first().is_ok() && qs.all().is_ok() && qs.count().unwrap() == 0 && !qs.exists().unwrap());
    }

    #[test]
    fn test_query_set_create_and_fetch() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE test_model (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, age INTEGER)").unwrap();
        let qs = QuerySet::new("test_model").using(pool.clone());
        let mut data = HashMap::new();
        data.insert("name".into(), "'Alice'".into());
        data.insert("age".into(), "30".into());
        assert_eq!(qs.create(data).unwrap(), 1);
        assert_eq!(qs.get_by_pk(1).unwrap().get("name").map(|s| s.as_str()), Some("Alice"));
        assert_eq!(qs.count().unwrap(), 1);
        assert!(qs.exists().unwrap());
    }

    #[test]
    fn test_query_set_update() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE test_items (id INTEGER PRIMARY KEY, name TEXT, status TEXT)").unwrap();
        pool.execute("INSERT INTO test_items VALUES (1, 'first', 'active')").unwrap();
        pool.execute("INSERT INTO test_items VALUES (2, 'second', 'inactive')").unwrap();
        let mut data = HashMap::new();
        data.insert("status".into(), "'inactive'".into());
        assert_eq!(QuerySet::new("test_items").using(pool).filter("status", "=", "'active'").update(data).unwrap(), 1);
    }

    #[test]
    fn test_query_set_delete() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE test_data (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        pool.execute("INSERT INTO test_data VALUES (1, 'keep')").unwrap();
        pool.execute("INSERT INTO test_data VALUES (2, 'delete_me')").unwrap();
        assert_eq!(QuerySet::new("test_data").using(pool.clone()).filter("name", "=", "'delete_me'").delete().unwrap(), 1);
    }

    #[test]
    fn test_filter_kwargs_contains() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE kw_test (id INTEGER, title TEXT)").unwrap();
        pool.execute("INSERT INTO kw_test VALUES (1, 'Hello World')").unwrap();
        pool.execute("INSERT INTO kw_test VALUES (2, 'Goodbye World')").unwrap();
        let mut filters = HashMap::new();
        filters.insert("title__contains".into(), "Hello".into());
        let qs = QuerySet::new("kw_test").using(pool).filter_kwargs(&filters);
        assert!(qs.sql().contains("LIKE") && qs.count().unwrap() == 1);
    }

    #[test]
    fn test_filter_kwargs_gt() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE scores (id INTEGER, score INTEGER)").unwrap();
        pool.execute("INSERT INTO scores VALUES (1, 50)").unwrap();
        pool.execute("INSERT INTO scores VALUES (2, 75)").unwrap();
        pool.execute("INSERT INTO scores VALUES (3, 100)").unwrap();
        let mut filters = HashMap::new();
        filters.insert("score__gte".into(), "75".into());
        assert_eq!(QuerySet::new("scores").using(pool).filter_kwargs(&filters).count().unwrap(), 2);
    }

    #[test]
    fn test_filter_kwargs_isnull() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE nullable (id INTEGER, val TEXT)").unwrap();
        pool.execute("INSERT INTO nullable VALUES (1, 'has')").unwrap();
        pool.execute("INSERT INTO nullable VALUES (2, NULL)").unwrap();
        let mut filters = HashMap::new();
        filters.insert("val__isnull".into(), "true".into());
        let qs = QuerySet::new("nullable").using(pool).filter_kwargs(&filters);
        assert!(qs.sql().contains("IS NULL") && qs.count().unwrap() == 1);
    }

    #[test]
    fn test_filter_kwargs_in() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE tags (id INTEGER, name TEXT)").unwrap();
        pool.execute("INSERT INTO tags VALUES (1, 'rust')").unwrap();
        pool.execute("INSERT INTO tags VALUES (2, 'python')").unwrap();
        pool.execute("INSERT INTO tags VALUES (3, 'go')").unwrap();
        let mut filters = HashMap::new();
        filters.insert("name__in".into(), "rust,python".into());
        assert_eq!(QuerySet::new("tags").using(pool).filter_kwargs(&filters).count().unwrap(), 2);
    }

    #[test]
    fn test_filter_kwargs_range() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE prices (id INTEGER, amount INTEGER)").unwrap();
        pool.execute("INSERT INTO prices VALUES (1, 10)").unwrap();
        pool.execute("INSERT INTO prices VALUES (2, 50)").unwrap();
        pool.execute("INSERT INTO prices VALUES (3, 100)").unwrap();
        let mut filters = HashMap::new();
        filters.insert("amount__range".into(), "20,80".into());
        assert_eq!(QuerySet::new("prices").using(pool).filter_kwargs(&filters).count().unwrap(), 1);
    }

    #[test]
    fn test_filter_kwargs_starts_ends_with() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE names (id INTEGER, name TEXT)").unwrap();
        pool.execute("INSERT INTO names VALUES (1, 'apple')").unwrap();
        pool.execute("INSERT INTO names VALUES (2, 'apricot')").unwrap();
        pool.execute("INSERT INTO names VALUES (3, 'cherry')").unwrap();
        let mut f1 = HashMap::new();
        f1.insert("name__startswith".into(), "ap".into());
        assert_eq!(QuerySet::new("names").using(pool.clone()).filter_kwargs(&f1).count().unwrap(), 2);
        let mut f2 = HashMap::new();
        f2.insert("name__endswith".into(), "ry".into());
        assert_eq!(QuerySet::new("names").using(pool).filter_kwargs(&f2).count().unwrap(), 1);
    }

    #[test]
    fn test_filter_kwargs_ne() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE statuses (id INTEGER, status TEXT)").unwrap();
        pool.execute("INSERT INTO statuses VALUES (1, 'active')").unwrap();
        pool.execute("INSERT INTO statuses VALUES (2, 'inactive')").unwrap();
        let mut filters = HashMap::new();
        filters.insert("status__ne".into(), "inactive".into());
        assert_eq!(QuerySet::new("statuses").using(pool).filter_kwargs(&filters).count().unwrap(), 1);
    }

    #[test]
    fn test_bulk_create() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE bulk_items (id INTEGER PRIMARY KEY AUTOINCREMENT, label TEXT)").unwrap();
        let items = vec![
            { let mut m = HashMap::new(); m.insert("label".into(), "'A'".into()); m },
            { let mut m = HashMap::new(); m.insert("label".into(), "'B'".into()); m },
            { let mut m = HashMap::new(); m.insert("label".into(), "'C'".into()); m },
        ];
        assert_eq!(QuerySet::new("bulk_items").using(pool.clone()).bulk_create(items).unwrap(), 3);
        assert_eq!(QuerySet::new("bulk_items").using(pool).count().unwrap(), 3);
    }

    #[test]
    fn test_in_bulk() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE bulk_get (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        for i in 1..=5 { pool.execute(&format!("INSERT INTO bulk_get VALUES ({}, 'item{}')", i, i)).unwrap(); }
        assert_eq!(QuerySet::new("bulk_get").using(pool).in_bulk(&[1, 3, 5]).unwrap().len(), 3);
    }

    #[test]
    fn test_filter_kwargs_contains_sqlite() {
        // SQLite LIKE is case-insensitive by default (same as Django's behavior on SQLite)
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE text_data (id INTEGER, label TEXT)").unwrap();
        pool.execute("INSERT INTO text_data VALUES (1, 'RustLang')").unwrap();
        pool.execute("INSERT INTO text_data VALUES (2, 'RUSTLANG')").unwrap();
        pool.execute("INSERT INTO text_data VALUES (3, 'Python')").unwrap();
        let mut filters = HashMap::new();
        filters.insert("label__contains".into(), "rust".into());
        // On SQLite, LIKE is case-insensitive => matches both RustLang and RUSTLANG
        assert_eq!(QuerySet::new("text_data").using(pool).filter_kwargs(&filters).count().unwrap(), 2);
    }

    #[test]
    fn test_first_last() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE ordered (id INTEGER PRIMARY KEY, val TEXT)").unwrap();
        pool.execute("INSERT INTO ordered VALUES (1, 'a')").unwrap();
        pool.execute("INSERT INTO ordered VALUES (2, 'b')").unwrap();
        let qs = QuerySet::new("ordered").using(pool);
        assert_eq!(qs.first().unwrap().unwrap().get("id").unwrap(), "1");
    }
}
