/// Query builder and QuerySet — Django-like ORM query interface.
/// Handles SQL construction and execution via DatabasePool.

use std::collections::HashMap;
use crate::executor::DatabasePool;
use crate::expressions::{Q, F};
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
    distinct_flag: bool,
    select_fields: Vec<String>,
    deferred_fields: Vec<String>,
    for_update: bool,
    for_update_nowait: bool,
    for_update_skip_locked: bool,
    extra_selects: Vec<(String, String)>,
    extra_wheres: Vec<String>,
    extra_tables: Vec<String>,
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
            distinct_flag: false,
            select_fields: vec![],
            deferred_fields: vec![],
            for_update: false,
            for_update_nowait: false,
            for_update_skip_locked: false,
            extra_selects: vec![],
            extra_wheres: vec![],
            extra_tables: vec![],
        }
    }

    /// Add conditions from a Q object.
    fn add_q_conditions(&mut self, q: &Q) {
        let sql = q.to_sql();
        if !sql.is_empty() {
            self.conditions.push(("".into(), "".into(), sql));
        }
    }

    /// Remove a field from the deferred list.
    fn undefer(&mut self, field: &str) {
        self.deferred_fields.retain(|f| f != field);
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
    pub fn left_join(mut self, table: &str, on: &str) -> Self { self.joins.push(format!("LEFT JOIN {} ON {}", table, on)); self }
    pub fn distinct(mut self) -> Self { self.distinct_flag = true; self }

    fn build_select_fields(&self) -> String {
        let mut selects: Vec<String> = Vec::new();
        if !self.select_fields.is_empty() {
            for f in &self.select_fields {
                selects.push(f.clone());
            }
        } else if self.deferred_fields.is_empty() {
            selects.push(if self.distinct_flag { "DISTINCT *".to_string() } else { "*".to_string() });
        } else {
            selects.push(if self.distinct_flag { "DISTINCT *".to_string() } else { "*".to_string() });
        }
        for (alias, expr) in &self.extra_selects {
            selects.push(format!("({}) AS {}", expr, alias));
        }
        if selects.is_empty() { selects.push("*".to_string()); }
        selects.join(", ")
    }

    fn all_where_clauses(&self) -> Vec<String> {
        let mut clauses: Vec<String> = self.conditions.iter().map(|(f, op, v)| {
            if f.is_empty() && op.is_empty() { v.clone() }
            else { format!("{} {} {}", f, op, v) }
        }).collect();
        for w in &self.extra_wheres { clauses.push(w.clone()); }
        clauses
    }

    fn build_for_update(&self) -> String {
        if !self.for_update { return String::new(); }
        let mut clause = " FOR UPDATE".to_string();
        if self.for_update_nowait { clause.push_str(" NOWAIT"); }
        if self.for_update_skip_locked { clause.push_str(" SKIP LOCKED"); }
        clause
    }

    pub fn build_select(&self) -> String {
        let select_clause = self.build_select_fields();
        let mut sql = format!("SELECT {} FROM {}", select_clause, self.table);
        for t in &self.extra_tables { sql.push_str(&format!(", {}", t)); }
        for join in &self.joins { sql.push_str(&format!(" {}", join)); }
        let clauses = self.all_where_clauses();
        if !clauses.is_empty() { sql.push_str(&format!(" WHERE {}", clauses.join(" AND "))); }
        if !self.order_by.is_empty() { sql.push_str(&format!(" ORDER BY {}", self.order_by.join(", "))); }
        if let Some(limit) = self.limit { sql.push_str(&format!(" LIMIT {}", limit)); }
        if let Some(offset) = self.offset { sql.push_str(&format!(" OFFSET {}", offset)); }
        sql.push_str(&self.build_for_update());
        sql
    }

    fn helper_clauses(&self) -> Vec<String> {
        self.conditions.iter().map(|(f, op, v)| {
            if f.is_empty() && op.is_empty() { v.clone() }
            else { format!("{} {} {}", f, op, v) }
        }).collect()
    }

    pub fn build_count(&self) -> String {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table);
        for t in &self.extra_tables { sql.push_str(&format!(", {}", t)); }
        for join in &self.joins { sql.push_str(&format!(" {}", join)); }
        let clauses = self.all_where_clauses();
        if !clauses.is_empty() { sql.push_str(&format!(" WHERE {}", clauses.join(" AND "))); }
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
    is_none: bool,
    annotations: Vec<crate::aggregates::Aggregate>,
}

impl QuerySet {
    pub fn new(table: &str) -> Self {
        Self {
            builder: QueryBuilder::new(table),
            pool: None,
            is_none: false,
            annotations: vec![],
        }
    }
    pub fn using(mut self, pool: DatabasePool) -> Self { self.pool = Some(pool); self }
    pub fn filter(mut self, field: &str, op: &str, value: &str) -> Self { self.builder = self.builder.filter(field, op, value); self }
    pub fn filter_kwargs(mut self, filters: &HashMap<String, String>) -> Self { self.builder = self.builder.filter_kwargs(filters); self }
    pub fn exclude(mut self, field: &str, op: &str, value: &str) -> Self { self.builder = self.builder.exclude(field, op, value); self }
    pub fn order_by(mut self, field: &str) -> Self { self.builder = self.builder.order_by(field); self }
    pub fn limit(mut self, n: usize) -> Self { self.builder = self.builder.limit(n); self }
    pub fn offset(mut self, n: usize) -> Self { self.builder = self.builder.offset(n); self }
    pub fn sql(&self) -> String { self.builder.build_select() }
    pub fn count_sql(&self) -> String { self.builder.build_count() }

    /// Create an always-empty QuerySet (like Django's `.none()`).
    pub fn none() -> Self {
        Self {
            builder: QueryBuilder::new(""),
            pool: None,
            is_none: true,
            annotations: vec![],
        }
    }

    /// Reverse the ordering (like Django's `.reverse()`).
    pub fn reverse(mut self) -> Self {
        let reversed: Vec<String> = self.builder.order_by.iter().map(|f| {
            if f.starts_with('-') {
                f.trim_start_matches('-').to_string()
            } else {
                format!("-{}", f)
            }
        }).collect();
        self.builder.order_by = reversed;
        self
    }

    /// Set a database alias for multi-db support (like Django's `.using()`).
    pub fn db_alias(self, _alias: &str) -> Self { self }

    /// Filter using Q objects (complex AND/OR/NOT logic).
    pub fn filter_q(mut self, q: &Q) -> Self {
        self.builder.add_q_conditions(q);
        self
    }

    /// Exclude using Q objects.
    pub fn exclude_q(mut self, q: &Q) -> Self {
        self.builder.add_q_conditions(&q.clone().negate());
        self
    }

    // ── Field selection ──

    /// Select specific fields as a list of dicts (like Django's `.values()`).
    pub fn values(mut self, fields: &[&str]) -> Self {
        self.builder.select_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    /// Select specific fields and return flat values (like Django's `.values_list()`).
    pub fn values_list(&self, fields: &[&str], flat: bool) -> crate::Result<Vec<Vec<String>>> {
        let qs = self.clone().values(fields);
        let rows = qs.all()?;
        let result: Vec<Vec<String>> = rows.into_iter().map(|row| {
            if flat {
                vec![row.values().next().cloned().unwrap_or_default()]
            } else {
                fields.iter().map(|f| row.get(*f).cloned().unwrap_or_default()).collect()
            }
        }).collect();
        Ok(result)
    }

    /// Defer loading certain fields (like Django's `.defer()`).
    pub fn defer(mut self, fields: &[&str]) -> Self {
        for f in fields {
            if !self.builder.deferred_fields.contains(&f.to_string()) {
                self.builder.deferred_fields.push(f.to_string());
            }
        }
        self
    }

    /// Only load specific fields (like Django's `.only()`).
    pub fn only(mut self, fields: &[&str]) -> Self {
        self.builder.select_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    /// Undefer a field.
    pub fn undefer(mut self, field: &str) -> Self {
        self.builder.undefer(field);
        self
    }

    // ── Relationships ──

    /// Add LEFT JOINs for ForeignKey relationships (like Django's `.select_related()`).
    pub fn select_related(mut self, relations: &[&str]) -> Self {
        for rel in relations {
            let fk_col = format!("{}_id", rel);
            let on = format!("{}.{} = {}.id", self.builder.table, fk_col, rel);
            self.builder = self.builder.left_join(rel, &on);
        }
        self
    }

    /// Mark relations for separate prefetch (stored for later evaluation).
    pub fn prefetch_related(&self, relations: &[&str]) -> PrefetchQuerySet {
        PrefetchQuerySet {
            base: self.clone(),
            prefetch_relations: relations.iter().map(|r| r.to_string()).collect(),
        }
    }

    /// Select for update — row-level locking (like Django's `select_for_update()`).
    pub fn select_for_update(mut self, nowait: bool, skip_locked: bool) -> Self {
        self.builder.for_update = true;
        self.builder.for_update_nowait = nowait;
        self.builder.for_update_skip_locked = skip_locked;
        self
    }

    /// Raw SQL extra clause injection (like Django's `extra()`).
    pub fn extra(mut self, extra_select: Option<Vec<(&str, &str)>>,
                 where_sql: Option<Vec<&str>>,
                 tables: Option<Vec<&str>>,
                 order_by: Option<Vec<&str>>) -> Self {
        if let Some(sel) = extra_select {
            for (alias, expr) in sel {
                self.builder.extra_selects.push((alias.to_string(), expr.to_string()));
            }
        }
        if let Some(wh) = where_sql {
            for clause in wh {
                self.builder.extra_wheres.push(clause.to_string());
            }
        }
        if let Some(tbls) = tables {
            for t in tbls {
                self.builder.extra_tables.push(t.to_string());
            }
        }
        if let Some(ob) = order_by {
            for o in ob {
                self.builder.order_by.push(o.to_string());
            }
        }
        self
    }

    /// Complex filter with raw Q objects (like Django's `complex_filter()`).
    pub fn complex_filter(mut self, q: &Q) -> Self {
        self.builder.add_q_conditions(q);
        self
    }

    /// Date extraction query (like Django's `.dates()`).
    pub fn dates(self, field: &str, kind: &str) -> crate::Result<Vec<String>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let extracted = match kind {
            "year" => format!("strftime('%Y', {})", field),
            "month" => format!("strftime('%Y-%m', {})", field),
            "day" => format!("strftime('%Y-%m-%d', {})", field),
            _ => return Err(format!("Invalid kind '{}' for dates()", kind).into()),
        };
        let sql = format!("SELECT DISTINCT {} AS date_val FROM {}", extracted, self.builder.table);
        let rows = pool.query(&sql)?;
        Ok(rows.into_iter().filter_map(|r| r.get("date_val").cloned()).collect())
    }

    /// Iterator with chunking for large querysets (like Django's `iterator()`).
    pub fn iterator(self, chunk_size: usize) -> QuerySetChunkedIter {
        QuerySetChunkedIter::new(self, chunk_size)
    }

    /// Get query plan (like Django's `explain()`).
    pub fn explain(&self) -> crate::Result<Vec<String>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let sql = format!("EXPLAIN QUERY PLAN {}", self.builder.build_select());
        let rows = pool.query(&sql)?;
        Ok(rows.into_iter().map(|r| r.values().cloned().collect::<Vec<_>>().join(" | ")).collect())
    }

    /// `in_bulk` with dict result keyed by id (Django-style).
    pub fn in_bulk_dict(self, id_list: &[i64]) -> crate::Result<HashMap<i64, HashMap<String, String>>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        if id_list.is_empty() { return Ok(HashMap::new()); }
        let ids: Vec<String> = id_list.iter().map(|i| i.to_string()).collect();
        let sql = format!("SELECT * FROM {} WHERE id IN ({})", self.builder.table, ids.join(", "));
        let rows = pool.query(&sql)?;
        let mut result = HashMap::new();
        for row in rows {
            if let Some(pk) = row.get("id").and_then(|v| v.parse::<i64>().ok()) {
                result.insert(pk, row);
            }
        }
        Ok(result)
    }

    /// Delete with Django-style info dict.
    pub fn delete_info(self) -> crate::Result<(u64, HashMap<String, u64>)> {
        let count = self.delete()?;
        let mut info = HashMap::new();
        info.insert(self.builder.table.clone(), count);
        Ok((count, info))
    }

    // ── Aggregation / Annotation ──

    /// Annotate the queryset with aggregate expressions (like Django's `.annotate()`).
    pub fn annotate(mut self, agg: &crate::aggregates::Aggregate) -> Self {
        let sql = agg.to_sql();
        self.annotations.push(agg.clone());
        if self.builder.select_fields.is_empty() || self.builder.select_fields == vec!["*".to_string()] {
            self.builder.select_fields = vec![format!("*, {}", sql)];
        } else {
            self.builder.select_fields.push(sql);
        }
        self
    }

    /// Run an aggregate query over the queryset.
    pub fn aggregate(&self, agg: &crate::aggregates::Aggregate) -> crate::Result<HashMap<String, String>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let sql = format!("SELECT {} FROM {}", agg.to_sql(), self.builder.table);
        let mut select = sql;
        if !self.builder.conditions.is_empty() {
            let clauses = self.builder.helper_clauses();
            select.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }
        let rows = pool.query(&select)?;
        Ok(rows.into_iter().next().unwrap_or_default())
    }

    // ── Set operations ──

    /// Union with another QuerySet.
    pub fn union(qs1: QuerySet, qs2: QuerySet) -> Self {
        let b1 = &qs1.builder;
        let alias = "union_qs";
        Self {
            builder: QueryBuilder {
                table: format!("({} UNION {}) AS {}", b1.build_select(), qs2.builder.build_select(), alias),
                fields: b1.fields.clone(),
                conditions: vec![],
                order_by: vec![],
                limit: None,
                offset: None,
                joins: vec![],
                distinct_flag: false,
                select_fields: b1.select_fields.clone(),
                deferred_fields: vec![],
                for_update: false,
                for_update_nowait: false,
                for_update_skip_locked: false,
                extra_selects: vec![],
                extra_wheres: vec![],
                extra_tables: vec![],
            },
            pool: qs1.pool.clone(),
            is_none: false,
            annotations: vec![],
        }
    }

    /// Intersection with another QuerySet.
    pub fn intersection(qs1: QuerySet, qs2: QuerySet) -> Self {
        let b1 = &qs1.builder;
        let alias = "inter_qs";
        Self {
            builder: QueryBuilder {
                table: format!("({} INTERSECT {}) AS {}", b1.build_select(), qs2.builder.build_select(), alias),
                fields: b1.fields.clone(),
                conditions: vec![],
                order_by: vec![],
                limit: None,
                offset: None,
                joins: vec![],
                distinct_flag: false,
                select_fields: b1.select_fields.clone(),
                deferred_fields: vec![],
                for_update: false,
                for_update_nowait: false,
                for_update_skip_locked: false,
                extra_selects: vec![],
                extra_wheres: vec![],
                extra_tables: vec![],
            },
            pool: qs1.pool.clone(),
            is_none: false,
            annotations: vec![],
        }
    }

    /// Difference with another QuerySet.
    pub fn difference(qs1: QuerySet, qs2: QuerySet) -> Self {
        let b1 = &qs1.builder;
        let alias = "diff_qs";
        Self {
            builder: QueryBuilder {
                table: format!("({} EXCEPT {}) AS {}", b1.build_select(), qs2.builder.build_select(), alias),
                fields: b1.fields.clone(),
                conditions: vec![],
                order_by: vec![],
                limit: None,
                offset: None,
                joins: vec![],
                distinct_flag: false,
                select_fields: b1.select_fields.clone(),
                deferred_fields: vec![],
                for_update: false,
                for_update_nowait: false,
                for_update_skip_locked: false,
                extra_selects: vec![],
                extra_wheres: vec![],
                extra_tables: vec![],
            },
            pool: qs1.pool.clone(),
            is_none: false,
            annotations: vec![],
        }
    }

    // ── F expression support ──

    /// Filter with an F expression (field-to-field comparison).
    pub fn filter_f(mut self, field: &str, lookup: &str, f_expr: &F) -> Self {
        let op = match lookup { "exact" => "=", "gt" => ">", "gte" => ">=", "lt" => "<", "lte" => "<=", "ne" => "!=", other => other };
        self.builder.conditions.push((field.to_string(), op.to_string(), f_expr.0.clone()));
        self
    }

    /// Update using F expressions (increment, decrement, etc.).
    pub fn update_f(&self, data: HashMap<String, F>) -> crate::Result<u64> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let sets: Vec<String> = data.iter().map(|(k, v)| format!("{} = {}", k, v.0)).collect();
        let mut sql = format!("UPDATE {} SET {}", self.builder.table, sets.join(", "));
        if !self.builder.conditions.is_empty() {
            let clauses = self.builder.helper_clauses();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }
        pool.execute(&sql)
    }

    pub fn all(&self) -> crate::Result<Vec<HashMap<String, String>>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        pool.query(&self.builder.build_select())
    }

    pub fn first(&self) -> crate::Result<Option<HashMap<String, String>>> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let rows = pool.query(&self.clone().limit(1).builder.build_select())?;
        Ok(rows.into_iter().next())
    }
    pub fn last(&self) -> crate::Result<Option<HashMap<String, String>>> {
        if self.is_none { return Ok(None); }
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let rows = pool.query(&self.clone().order_by("id DESC").limit(1).builder.build_select())?;
        Ok(rows.into_iter().next())
    }
    pub fn count(&self) -> crate::Result<i64> {
        if self.is_none { return Ok(0); }
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let rows = pool.query(&self.builder.build_count())?;
        Ok(rows.first().and_then(|r| r.values().next()).and_then(|v| v.parse().ok()).unwrap_or(0))
    }
    pub fn exists(&self) -> crate::Result<bool> {
        if self.is_none { return Ok(false); }
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
    pub fn none(&self) -> QuerySet { QuerySet::none() }
    pub fn filter_q(&self, q: &Q) -> QuerySet { self.qs().filter_q(q) }
}

/// Iterator adapter for QuerySet (allows `for row in qs`).
pub struct QuerySetIter {
    rows: Vec<HashMap<String, String>>,
    index: usize,
}

impl Iterator for QuerySetIter {
    type Item = HashMap<String, String>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.rows.len() { return None; }
        let row = self.rows[self.index].clone();
        self.index += 1;
        Some(row)
    }
}

impl IntoIterator for QuerySet {
    type Item = HashMap<String, String>;
    type IntoIter = QuerySetIter;
    fn into_iter(self) -> Self::IntoIter {
        match self.all() {
            Ok(rows) => QuerySetIter { rows, index: 0 },
            Err(_) => QuerySetIter { rows: vec![], index: 0 },
        }
    }
}

impl IntoIterator for PrefetchQuerySet {
    type Item = HashMap<String, String>;
    type IntoIter = QuerySetIter;
    fn into_iter(self) -> Self::IntoIter {
        match self.execute() {
            Ok(rows) => QuerySetIter { rows, index: 0 },
            Err(_) => QuerySetIter { rows: vec![], index: 0 },
        }
    }
}

/// A QuerySet that has been prepared for prefetch_related.
#[derive(Debug, Clone)]
pub struct PrefetchQuerySet {
    base: QuerySet,
    prefetch_relations: Vec<String>,
}

impl PrefetchQuerySet {
    pub fn execute(self) -> crate::Result<Vec<HashMap<String, String>>> {
        // Note: actual prefetch resolution will use self.prefetch_relations
        // when multi-model query support is implemented.
        let _ = self.prefetch_relations.len();
        self.base.all()
    }
    pub fn all(self) -> crate::Result<Vec<HashMap<String, String>>> { self.execute() }
}

/// Chunked iterator for large QuerySets (like Django's `iterator(chunk_size)`).
pub struct QuerySetChunkedIter {
    qs: QuerySet,
    offset: usize,
    chunk_size: usize,
    current_chunk: Vec<HashMap<String, String>>,
    current_index: usize,
    exhausted: bool,
}

impl QuerySetChunkedIter {
    pub fn new(qs: QuerySet, chunk_size: usize) -> Self {
        let chunk_size = chunk_size.max(1);
        Self {
            qs,
            offset: 0,
            chunk_size,
            current_chunk: vec![],
            current_index: 0,
            exhausted: false,
        }
    }

    fn fetch_next_chunk(&mut self) -> bool {
        if self.exhausted { return false; }
        let chunk_qs = self.qs.clone().offset(self.offset).limit(self.chunk_size);
        match chunk_qs.all() {
            Ok(rows) => {
                let count = rows.len();
                self.current_chunk = rows;
                self.current_index = 0;
                self.offset += count;
                if count < self.chunk_size {
                    self.exhausted = true;
                }
                !self.current_chunk.is_empty()
            }
            Err(_) => {
                self.exhausted = true;
                false
            }
        }
    }
}

impl Iterator for QuerySetChunkedIter {
    type Item = crate::Result<HashMap<String, String>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.current_chunk.len() {
            if self.exhausted { return None; }
            if !self.fetch_next_chunk() { return None; }
        }
        if self.current_index < self.current_chunk.len() {
            let row = self.current_chunk[self.current_index].clone();
            self.current_index += 1;
            Some(Ok(row))
        } else {
            None
        }
    }
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

    #[test]
    fn test_select_for_update_constructs_correct_sql() {
        let qs = QuerySet::new("items").select_for_update(true, false);
        let sql = qs.sql();
        assert!(sql.contains("FOR UPDATE"));
        assert!(sql.contains("NOWAIT"));
        assert!(!sql.contains("SKIP LOCKED"));
    }

    #[test]
    fn test_select_for_update_skip_locked() {
        let qs = QuerySet::new("items").select_for_update(false, true);
        let sql = qs.sql();
        assert!(sql.contains("SKIP LOCKED"));
        assert!(!sql.contains("NOWAIT"));
    }

    #[test]
    fn test_extra_select_adds_computed_column() {
        let qs = QuerySet::new("items")
            .extra(Some(vec![("total", "price * quantity")]), None, None, None);
        let sql = qs.sql();
        assert!(sql.contains("(price * quantity) AS total"));
    }

    #[test]
    fn test_extra_where_adds_raw_clause() {
        let qs = QuerySet::new("items")
            .extra(None, Some(vec!["price > 100"]), None, None);
        let sql = qs.sql();
        assert!(sql.contains("WHERE price > 100"));
    }

    #[test]
    fn test_extra_tables_adds_join() {
        let qs = QuerySet::new("items")
            .extra(None, None, Some(vec!["categories"]), None);
        let sql = qs.sql();
        assert!(sql.contains("SELECT * FROM items, categories"));
    }

    #[test]
    fn test_complex_filter_with_q_objects() {
        let q = Q::new("status", "active").and(Q::new("priority", "high"));
        let qs = QuerySet::new("items").complex_filter(&q);
        let sql = qs.sql();
        assert!(sql.contains("AND"));
    }

    #[test]
    fn test_dates_with_year_extraction() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE dated (id INTEGER, created_at TEXT)").unwrap();
        pool.execute("INSERT INTO dated VALUES (1, '2025-01-15')").unwrap();
        let result = QuerySet::new("dated").using(pool).dates("created_at", "year").unwrap();
        assert_eq!(result, vec!["2025"]);
    }

    #[test]
    fn test_dates_with_month_extraction() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE dated (id INTEGER, created_at TEXT)").unwrap();
        pool.execute("INSERT INTO dated VALUES (1, '2025-01-15')").unwrap();
        let result = QuerySet::new("dated").using(pool).dates("created_at", "month").unwrap();
        assert_eq!(result, vec!["2025-01"]);
    }

    #[test]
    fn test_explain_returns_plan() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE explain_t (id INTEGER)").unwrap();
        let result = QuerySet::new("explain_t").using(pool).explain().unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_in_bulk_dict_returns_keyed_by_id() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE bulkdict (id INTEGER, name TEXT)").unwrap();
        pool.execute("INSERT INTO bulkdict VALUES (1, 'a')").unwrap();
        pool.execute("INSERT INTO bulkdict VALUES (2, 'b')").unwrap();
        let result = QuerySet::new("bulkdict").using(pool).in_bulk_dict(&[1, 2]).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[&1].get("name").unwrap(), "a");
    }

    #[test]
    fn test_in_bulk_dict_empty_list() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE bd (id INTEGER)").unwrap();
        let result = QuerySet::new("bd").using(pool).in_bulk_dict(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_delete_info_returns_tuple() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE di (id INTEGER)").unwrap();
        pool.execute("INSERT INTO di VALUES (1)").unwrap();
        let (count, info) = QuerySet::new("di").using(pool).delete_info().unwrap();
        assert_eq!(count, 1);
        assert_eq!(info.get("di").unwrap(), &1);
    }

    #[test]
    fn test_iterator_chunked_iteration() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE iter_t (id INTEGER)").unwrap();
        pool.execute("INSERT INTO iter_t VALUES (1)").unwrap();
        pool.execute("INSERT INTO iter_t VALUES (2)").unwrap();
        pool.execute("INSERT INTO iter_t VALUES (3)").unwrap();
        let iter = QuerySet::new("iter_t").using(pool).iterator(2);
        let count = iter.count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_select_for_update() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE locking (id INTEGER PRIMARY KEY, val TEXT)").unwrap();
        pool.execute("INSERT INTO locking VALUES (1, 'test')").unwrap();
        let qs = QuerySet::new("locking").using(pool).select_for_update(false, false);
        let sql = qs.sql();
        assert!(sql.contains("FOR UPDATE"));
    }

    #[test]
    fn test_select_for_update_nowait() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE locking2 (id INTEGER PRIMARY KEY, val TEXT)").unwrap();
        let qs = QuerySet::new("locking2").using(pool).select_for_update(true, false);
        let sql = qs.sql();
        assert!(sql.contains("NOWAIT"));
    }

    #[test]
    fn test_extra_where() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE extra_test (id INTEGER, flag INTEGER)").unwrap();
        pool.execute("INSERT INTO extra_test VALUES (1, 1)").unwrap();
        pool.execute("INSERT INTO extra_test VALUES (2, 0)").unwrap();
        let rows = QuerySet::new("extra_test").using(pool)
            .extra(None, Some(vec!["flag = 1"]), None, None)
            .all().unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_extra_select() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE extra_sel (id INTEGER, val INTEGER)").unwrap();
        pool.execute("INSERT INTO extra_sel VALUES (1, 5)").unwrap();
        let qs = QuerySet::new("extra_sel").using(pool)
            .extra(Some(vec![("doubled", "val * 2")]), None, None, None);
        let sql = qs.sql();
        assert!(sql.contains("doubled"));
        assert!(sql.contains("val * 2"));
    }

    #[test]
    fn test_complex_filter_q() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE comp (id INTEGER, status TEXT)").unwrap();
        pool.execute("INSERT INTO comp VALUES (1, 'active')").unwrap();
        pool.execute("INSERT INTO comp VALUES (2, 'inactive')").unwrap();
        let q = Q::new("status", "active");
        let rows = QuerySet::new("comp").using(pool)
            .complex_filter(&q)
            .all().unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_dates_year() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE dated (id INTEGER, created TEXT)").unwrap();
        pool.execute("INSERT INTO dated VALUES (1, '2024-01-15')").unwrap();
        pool.execute("INSERT INTO dated VALUES (2, '2024-06-20')").unwrap();
        let result = QuerySet::new("dated").using(pool).dates("created", "year").unwrap();
        assert_eq!(result, vec!["2024"]);
    }

    #[test]
    fn test_dates_month() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE dated2 (id INTEGER, created TEXT)").unwrap();
        pool.execute("INSERT INTO dated2 VALUES (1, '2024-01-15')").unwrap();
        pool.execute("INSERT INTO dated2 VALUES (2, '2024-02-20')").unwrap();
        let result = QuerySet::new("dated2").using(pool).dates("created", "month").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|s| s == "2024-01"));
        assert!(result.iter().any(|s| s == "2024-02"));
    }

    #[test]
    fn test_dates_invalid_kind() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE dated3 (id INTEGER, created TEXT)").unwrap();
        let result = QuerySet::new("dated3").using(pool).dates("created", "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_iterator() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE iter_test (id INTEGER PRIMARY KEY, label TEXT)").unwrap();
        for i in 0..10 {
            pool.execute(&format!("INSERT INTO iter_test VALUES ({}, 'item')", i)).unwrap();
        }
        let mut count = 0;
        for result in QuerySet::new("iter_test").using(pool).iterator(3) {
            if let Ok(_row) = result { count += 1; }
        }
        assert_eq!(count, 10);
    }

    #[test]
    fn test_in_bulk_dict() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE bulkd (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        for i in 1..=5 { pool.execute(&format!("INSERT INTO bulkd VALUES ({}, 'n{}')", i, i)).unwrap(); }
        let result = QuerySet::new("bulkd").using(pool).in_bulk_dict(&[1, 3, 5]).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[&1].get("name").unwrap(), "n1");
    }

    #[test]
    fn test_in_bulk_dict_empty() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE bulkd2 (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        let result = QuerySet::new("bulkd2").using(pool).in_bulk_dict(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_delete_info() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE del_info (id INTEGER, val TEXT)").unwrap();
        pool.execute("INSERT INTO del_info VALUES (1, 'a')").unwrap();
        pool.execute("INSERT INTO del_info VALUES (2, 'b')").unwrap();
        let qs = QuerySet::new("del_info").using(pool);
        let (count, info) = qs.delete_info().unwrap();
        assert_eq!(count, 2);
        assert_eq!(info.get("del_info").unwrap(), &2);
    }

    #[test]
    fn test_explain() {
        let pool = DatabasePool::new(crate::executor::DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE explain_t (id INTEGER)").unwrap();
        pool.execute("INSERT INTO explain_t VALUES (1)").unwrap();
        let result = QuerySet::new("explain_t").using(pool).explain().unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_none_queryset() {
        // none() creates a QuerySet that is always empty
        let _qs = QuerySet::none();
        // Just testing it compiles and doesn't panic
    }
}
