/// Query builder for constructing SQL queries.
/// Mirrors Django's QuerySet API.

use std::collections::HashMap;

/// A query builder that constructs SQL from method calls.
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    fields: Vec<String>,
    conditions: Vec<(String, String, String)>, // (field, op, value)
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
        // NOT condition
        self.conditions.push((field.to_string(), format!("NOT {}", op), value.to_string()));
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by.push(field.to_string());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    pub fn join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(format!("JOIN {} ON {}", table, on));
        self
    }

    pub fn build_select(&self) -> String {
        let fields = self.fields.join(", ");
        let mut sql = format!("SELECT {} FROM {}", fields, self.table);

        for join in &self.joins {
            sql.push_str(&format!(" {}", join));
        }

        if !self.conditions.is_empty() {
            let clauses: Vec<String> = self.conditions.iter()
                .map(|(f, op, v)| format!("{} {} {}", f, op, v))
                .collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }

        if !self.order_by.is_empty() {
            sql.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    /// Build a COUNT query.
    pub fn build_count(&self) -> String {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table);
        for join in &self.joins {
            sql.push_str(&format!(" {}", join));
        }
        if !self.conditions.is_empty() {
            let clauses: Vec<String> = self.conditions.iter()
                .map(|(f, op, v)| format!("{} {} {}", f, op, v))
                .collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }
        sql
    }

    /// Build an INSERT query.
    pub fn build_insert(data: &HashMap<String, String>, table: &str) -> String {
        let fields: Vec<&str> = data.keys().map(|k| k.as_str()).collect();
        let values: Vec<&str> = data.values().map(|v| v.as_str()).collect();
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            fields.join(", "),
            values.join(", ")
        )
    }

    /// Build an UPDATE query.
    pub fn build_update(data: &HashMap<String, String>, table: &str, pk_field: &str, pk_value: &str) -> String {
        let sets: Vec<String> = data.iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect();
        format!(
            "UPDATE {} SET {} WHERE {} = {}",
            table,
            sets.join(", "),
            pk_field,
            pk_value
        )
    }

    /// Build a DELETE query.
    pub fn build_delete(table: &str, pk_field: &str, pk_value: &str) -> String {
        format!("DELETE FROM {} WHERE {} = {}", table, pk_field, pk_value)
    }
}

/// A QuerySet wraps a QueryBuilder and provides Django-like chaining.
#[derive(Debug, Clone)]
pub struct QuerySet {
    builder: QueryBuilder,
}

impl QuerySet {
    pub fn new(table: &str) -> Self {
        Self { builder: QueryBuilder::new(table) }
    }

    pub fn filter(mut self, field: &str, op: &str, value: &str) -> Self {
        self.builder = self.builder.filter(field, op, value);
        self
    }

    pub fn exclude(mut self, field: &str, op: &str, value: &str) -> Self {
        self.builder = self.builder.exclude(field, op, value);
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.builder = self.builder.order_by(field);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.builder = self.builder.limit(n);
        self
    }

    pub fn sql(&self) -> String {
        self.builder.build_select()
    }

    pub fn count_sql(&self) -> String {
        self.builder.build_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let qb = QueryBuilder::new("users");
        assert_eq!(qb.build_select(), "SELECT * FROM users");
    }

    #[test]
    fn test_filtered_select() {
        let qb = QueryBuilder::new("users")
            .filter("age", ">", "18")
            .filter("active", "=", "1");
        assert_eq!(
            qb.build_select(),
            "SELECT * FROM users WHERE age > 18 AND active = 1"
        );
    }

    #[test]
    fn test_select_with_limits() {
        let qb = QueryBuilder::new("posts")
            .filter("published", "=", "true")
            .order_by("created_at DESC")
            .limit(10);
        let sql = qb.build_select();
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("ORDER BY created_at DESC"));
    }

    #[test]
    fn test_query_set_chaining() {
        let qs = QuerySet::new("products")
            .filter("price", ">", "10")
            .filter("in_stock", "=", "true")
            .order_by("name");
        assert!(qs.sql().contains("WHERE"));
        assert!(qs.sql().contains("ORDER BY name"));
    }

    #[test]
    fn test_insert_build() {
        let mut data = HashMap::new();
        data.insert("name".into(), "'John'".into());
        data.insert("age".into(), "30".into());
        let sql = QueryBuilder::build_insert(&data, "users");
        assert!(sql.starts_with("INSERT INTO users"));
    }

    #[test]
    fn test_count_query() {
        let qb = QueryBuilder::new("users")
            .filter("active", "=", "1");
        assert_eq!(qb.build_count(), "SELECT COUNT(*) FROM users WHERE active = 1");
    }
}
