//! Lookup types for QuerySet filters.
//! Mirrors Django's `django.db.models.lookups` — 20+ lookup types.

use std::collections::HashMap;

/// A single filter condition.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterCondition {
    pub field: String,
    pub lookup: Lookup,
    pub value: String,
}

/// All supported lookup types (mirrors Django's defaults).
#[derive(Debug, Clone, PartialEq)]
pub enum Lookup {
    /// Exact match (default)
    Exact,
    /// Case-insensitive exact match
    IExact,
    /// Field contains value
    Contains,
    /// Case-insensitive contains
    IContains,
    /// Value is in a list
    In,
    /// Greater than
    Gt,
    /// Greater than or equal
    Gte,
    /// Less than
    Lt,
    /// Less than or equal
    Lte,
    /// Starts with
    StartsWith,
    /// Case-insensitive starts with
    IStartsWith,
    /// Ends with
    EndsWith,
    /// Case-insensitive ends with
    IEndsWith,
    /// Between two values
    Range,
    /// Date field: year match
    Year,
    /// Date field: month match
    Month,
    /// Date field: day match
    Day,
    /// Date field: week day match
    WeekDay,
    /// Date field: quarter match
    Quarter,
    /// Date field: hour match
    Hour,
    /// Date field: minute match
    Minute,
    /// Date field: second match
    Second,
    /// Is null / is not null
    IsNull,
    /// Regex match
    Regex,
    /// Case-insensitive regex
    IRegex,
    /// Not equal
    Ne,
}

impl Lookup {
    /// Parse a Django-style lookup string into a Lookup variant.
    /// `field__lookup` → (field, lookup)
    /// `field` → (field, Exact)
    pub fn parse(full_field: &str) -> (String, Lookup) {
        if let Some(sep) = full_field.rfind("__") {
            let field = &full_field[..sep];
            let lookup_str = &full_field[sep + 2..];
            let lookup = match lookup_str {
                "exact" => Lookup::Exact,
                "iexact" => Lookup::IExact,
                "contains" => Lookup::Contains,
                "icontains" => Lookup::IContains,
                "in" => Lookup::In,
                "gt" => Lookup::Gt,
                "gte" => Lookup::Gte,
                "lt" => Lookup::Lt,
                "lte" => Lookup::Lte,
                "startswith" => Lookup::StartsWith,
                "istartswith" => Lookup::IStartsWith,
                "endswith" => Lookup::EndsWith,
                "iendswith" => Lookup::IEndsWith,
                "range" => Lookup::Range,
                "year" => Lookup::Year,
                "month" => Lookup::Month,
                "day" => Lookup::Day,
                "week_day" => Lookup::WeekDay,
                "quarter" => Lookup::Quarter,
                "hour" => Lookup::Hour,
                "minute" => Lookup::Minute,
                "second" => Lookup::Second,
                "isnull" => Lookup::IsNull,
                "regex" => Lookup::Regex,
                "iregex" => Lookup::IRegex,
                "ne" => Lookup::Ne,
                _ => Lookup::Exact, // fallback
            };
            (field.to_string(), lookup)
        } else {
            (full_field.to_string(), Lookup::Exact)
        }
    }

    /// Get the SQL operator for this lookup.
    pub fn sql_operator(&self) -> &'static str {
        match self {
            Lookup::Exact | Lookup::IExact => "=",
            Lookup::Contains => "LIKE",
            Lookup::IContains => "LIKE",
            Lookup::In => "IN",
            Lookup::Gt => ">",
            Lookup::Gte => ">=",
            Lookup::Lt => "<",
            Lookup::Lte => "<=",
            Lookup::StartsWith | Lookup::IStartsWith => "LIKE",
            Lookup::EndsWith | Lookup::IEndsWith => "LIKE",
            Lookup::Range => "BETWEEN",
            Lookup::Year => "=",
            Lookup::Month => "=",
            Lookup::Day => "=",
            Lookup::WeekDay => "=",
            Lookup::Quarter => "=",
            Lookup::Hour => "=",
            Lookup::Minute => "=",
            Lookup::Second => "=",
            Lookup::IsNull => "IS NULL",
            Lookup::Regex | Lookup::IRegex => "REGEXP",
            Lookup::Ne => "!=",
        }
    }

    /// Format the value for SQL (parameters, not values).
    pub fn format_value(&self, value: &str) -> Vec<String> {
        match self {
            Lookup::Contains => vec![format!("%{}%", value)],
            Lookup::IContains => vec![format!("%{}%", value.to_lowercase())],
            Lookup::StartsWith => vec![format!("{}%", value)],
            Lookup::IStartsWith => vec![format!("{}%", value.to_lowercase())],
            Lookup::EndsWith => vec![format!("%{}", value)],
            Lookup::IEndsWith => vec![format!("%{}", value.to_lowercase())],
            Lookup::IExact => vec![value.to_lowercase()],
            Lookup::Range => {
                let parts: Vec<&str> = value.splitn(2, ',').collect();
                vec![parts.first().unwrap_or(&"").to_string(), parts.get(1).unwrap_or(&"").to_string()]
            }
            Lookup::In => {
                value.split(',').map(|s| s.trim().to_string()).collect()
            }
            Lookup::IsNull => vec![],
            Lookup::Year => vec![value.to_string()],
            Lookup::Month => vec![value.to_string()],
            Lookup::Day => vec![value.to_string()],
            Lookup::WeekDay => vec![value.to_string()],
            Lookup::Quarter => vec![value.to_string()],
            Lookup::Hour => vec![value.to_string()],
            Lookup::Minute => vec![value.to_string()],
            Lookup::Second => vec![value.to_string()],
            _ => vec![value.to_string()],
        }
    }

    /// Generate SQL snippet for the field+lookup combination.
    pub fn sql_snippet(&self, field: &str, param_idx: usize) -> String {
        match self {
            Lookup::Exact => format!("{} = ${}", field, param_idx),
            Lookup::IExact => format!("LOWER({}) = LOWER(${})", field, param_idx),
            Lookup::Contains => format!("{} LIKE ${}", field, param_idx),
            Lookup::IContains => format!("LOWER({}) LIKE LOWER(${})", field, param_idx),
            Lookup::In => {
                // Multiple params: need to build $1, $2, etc.
                format!("{} IN (${})", field, param_idx)
            }
            Lookup::Gt => format!("{} > ${}", field, param_idx),
            Lookup::Gte => format!("{} >= ${}", field, param_idx),
            Lookup::Lt => format!("{} < ${}", field, param_idx),
            Lookup::Lte => format!("{} <= ${}", field, param_idx),
            Lookup::StartsWith => format!("{} LIKE ${}", field, param_idx),
            Lookup::IStartsWith => format!("LOWER({}) LIKE LOWER(${})", field, param_idx),
            Lookup::EndsWith => format!("{} LIKE ${}", field, param_idx),
            Lookup::IEndsWith => format!("LOWER({}) LIKE LOWER(${})", field, param_idx),
            Lookup::Range => format!("{} BETWEEN ${} AND ${}", field, param_idx, param_idx + 1),
            Lookup::Year => format!("CAST(strftime('%Y', {}) AS INTEGER) = ${}", field, param_idx),
            Lookup::Month => format!("CAST(strftime('%m', {}) AS INTEGER) = ${}", field, param_idx),
            Lookup::Day => format!("CAST(strftime('%d', {}) AS INTEGER) = ${}", field, param_idx),
            Lookup::WeekDay => format!("CAST(strftime('%w', {}) AS INTEGER) + 1 = ${}", field, param_idx),
            Lookup::Quarter => format!("CAST((CAST(strftime('%m', {}) AS INTEGER) + 2) / 3 AS INTEGER) = ${}", field, param_idx),
            Lookup::Hour => format!("CAST(strftime('%H', {}) AS INTEGER) = ${}", field, param_idx),
            Lookup::Minute => format!("CAST(strftime('%M', {}) AS INTEGER) = ${}", field, param_idx),
            Lookup::Second => format!("CAST(strftime('%S', {}) AS INTEGER) = ${}", field, param_idx),
            Lookup::IsNull => format!("{} IS NULL", field),
            Lookup::Regex => format!("{} REGEXP ${}", field, param_idx),
            Lookup::IRegex => format!("{} REGEXP ${}", field, param_idx),
            Lookup::Ne => format!("{} != ${}", field, param_idx),
        }
    }
}

/// Parse multiple filter kwargs into FilterConditions.
pub fn parse_filters(filters: &HashMap<String, String>) -> Vec<FilterCondition> {
    let mut conditions = Vec::with_capacity(filters.len());
    for (full_field, value) in filters {
        let (field, lookup) = Lookup::parse(full_field);
        conditions.push(FilterCondition {
            field,
            lookup,
            value: value.clone(),
        });
    }
    conditions
}

/// Build a WHERE clause from filter conditions.
pub fn build_where_clause(conditions: &[FilterCondition]) -> (String, Vec<String>) {
    if conditions.is_empty() {
        return (String::new(), vec![]);
    }

    let mut clauses = Vec::with_capacity(conditions.len());
    let mut params = Vec::new();
    let mut param_idx = 1;

    for cond in conditions {
        let mut values = cond.lookup.format_value(&cond.value);
        let snippet = cond.lookup.sql_snippet(&cond.field, param_idx);
        clauses.push(snippet);

        // Adjust param_idx based on how many params this lookup uses
        let count = values.len();
        param_idx += count;
        params.append(&mut values);
    }

    (format!("WHERE {}", clauses.join(" AND ")), params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_parse_exact() {
        let (field, lookup) = Lookup::parse("name");
        assert_eq!(field, "name");
        assert_eq!(lookup, Lookup::Exact);
    }

    #[test]
    fn test_lookup_parse_with_lookup() {
        let (field, lookup) = Lookup::parse("age__gt");
        assert_eq!(field, "age");
        assert_eq!(lookup, Lookup::Gt);
    }

    #[test]
    fn test_lookup_parse_all_variants() {
        let cases = vec![
            ("name__exact", Lookup::Exact),
            ("name__iexact", Lookup::IExact),
            ("name__contains", Lookup::Contains),
            ("name__icontains", Lookup::IContains),
            ("id__in", Lookup::In),
            ("age__gt", Lookup::Gt),
            ("age__gte", Lookup::Gte),
            ("age__lt", Lookup::Lt),
            ("age__lte", Lookup::Lte),
            ("name__startswith", Lookup::StartsWith),
            ("name__istartswith", Lookup::IStartsWith),
            ("name__endswith", Lookup::EndsWith),
            ("name__iendswith", Lookup::IEndsWith),
            ("date__range", Lookup::Range),
            ("date__year", Lookup::Year),
            ("date__month", Lookup::Month),
            ("date__day", Lookup::Day),
            ("date__week_day", Lookup::WeekDay),
            ("date__quarter", Lookup::Quarter),
            ("date__hour", Lookup::Hour),
            ("date__minute", Lookup::Minute),
            ("date__second", Lookup::Second),
            ("name__isnull", Lookup::IsNull),
            ("name__regex", Lookup::Regex),
            ("name__iregex", Lookup::IRegex),
            ("name__ne", Lookup::Ne),
        ];
        for (input, expected) in cases {
            let (field, lookup) = Lookup::parse(input);
            assert_eq!(field, input.split("__").next().unwrap());
            assert_eq!(lookup, expected, "Failed for {}", input);
        }
    }

    #[test]
    fn test_lookup_sql_operators() {
        assert_eq!(Lookup::Exact.sql_operator(), "=");
        assert_eq!(Lookup::Contains.sql_operator(), "LIKE");
        assert_eq!(Lookup::In.sql_operator(), "IN");
        assert_eq!(Lookup::Gt.sql_operator(), ">");
        assert_eq!(Lookup::Gte.sql_operator(), ">=");
        assert_eq!(Lookup::Lt.sql_operator(), "<");
        assert_eq!(Lookup::Lte.sql_operator(), "<=");
        assert_eq!(Lookup::Range.sql_operator(), "BETWEEN");
        assert_eq!(Lookup::IsNull.sql_operator(), "IS NULL");
        assert_eq!(Lookup::Ne.sql_operator(), "!=");
    }

    #[test]
    fn test_format_value_contains() {
        let vals = Lookup::Contains.format_value("hello");
        assert_eq!(vals, vec!["%hello%"]);
    }

    #[test]
    fn test_format_value_range() {
        let vals = Lookup::Range.format_value("10,20");
        assert_eq!(vals, vec!["10", "20"]);
    }

    #[test]
    fn test_format_value_in() {
        let vals = Lookup::In.format_value("1,2,3");
        assert_eq!(vals, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_format_value_isnull() {
        let vals = Lookup::IsNull.format_value("true");
        assert_eq!(vals.len(), 0);
    }

    #[test]
    fn test_parse_filters() {
        let mut filters = HashMap::new();
        filters.insert("name__contains".to_string(), "rust".to_string());
        filters.insert("age__gt".to_string(), "18".to_string());

        let conditions = parse_filters(&filters);
        assert_eq!(conditions.len(), 2);

        let contains_cond = conditions.iter().find(|c| c.lookup == Lookup::Contains).unwrap();
        assert_eq!(contains_cond.field, "name");
        assert_eq!(contains_cond.value, "rust");
    }

    #[test]
    fn test_build_where_clause_empty() {
        let (sql, params) = build_where_clause(&[]);
        assert_eq!(sql, "");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_where_clause_single() {
        let conds = vec![FilterCondition {
            field: "name".to_string(),
            lookup: Lookup::Exact,
            value: "test".to_string(),
        }];
        let (sql, params) = build_where_clause(&conds);
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("name"));
        assert!(sql.contains("="));
        assert_eq!(params, vec!["test"]);
    }

    #[test]
    fn test_build_where_clause_multiple() {
        let conds = vec![
            FilterCondition { field: "name".to_string(), lookup: Lookup::Contains, value: "rust".to_string() },
            FilterCondition { field: "age".to_string(), lookup: Lookup::Gte, value: "21".to_string() },
        ];
        let (sql, params) = build_where_clause(&conds);
        assert!(sql.contains("AND"));
        assert!(sql.contains("LIKE"));
        assert!(sql.contains(">="));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_range() {
        let conds = vec![FilterCondition {
            field: "price".to_string(),
            lookup: Lookup::Range,
            value: "10,100".to_string(),
        }];
        let (sql, params) = build_where_clause(&conds);
        assert!(sql.contains("BETWEEN"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_isnull() {
        let conds = vec![FilterCondition {
            field: "deleted_at".to_string(),
            lookup: Lookup::IsNull,
            value: "true".to_string(),
        }];
        let (sql, params) = build_where_clause(&conds);
        assert!(sql.contains("IS NULL"));
        assert!(params.is_empty());
    }

    #[test]
    fn test_lookup_parse_fallback() {
        let (field, lookup) = Lookup::parse("name__unknown");
        assert_eq!(field, "name");
        assert_eq!(lookup, Lookup::Exact); // fallback
    }

    #[test]
    fn test_lookup_sql_snippet_contains() {
        let snippet = Lookup::Contains.sql_snippet("title", 1);
        assert_eq!(snippet, "title LIKE $1");
    }

    #[test]
    fn test_lookup_sql_snippet_range() {
        let snippet = Lookup::Range.sql_snippet("price", 1);
        assert_eq!(snippet, "price BETWEEN $1 AND $2");
    }

    #[test]
    fn test_lookup_sql_snippet_year() {
        let snippet = Lookup::Year.sql_snippet("created_at", 1);
        assert!(snippet.contains("strftime"));
        assert!(snippet.contains("created_at"));
    }

    #[test]
    fn test_lookup_sql_snippet_isnull() {
        let snippet = Lookup::IsNull.sql_snippet("deleted_at", 1);
        assert_eq!(snippet, "deleted_at IS NULL");
    }

    #[test]
    fn test_filter_condition_debug() {
        let fc = FilterCondition {
            field: "age".to_string(),
            lookup: Lookup::Gt,
            value: "25".to_string(),
        };
        let debug = format!("{:?}", fc);
        assert!(debug.contains("age"));
        assert!(debug.contains("Gt"));
    }

    #[test]
    fn test_lookup_parse_deep_nested_field() {
        let (field, lookup) = Lookup::parse("user__profile__bio__contains");
        assert_eq!(field, "user__profile__bio");
        assert_eq!(lookup, Lookup::Contains);
    }

    #[test]
    fn test_lookup_clone() {
        let l = Lookup::IStartsWith;
        let cloned = l.clone();
        assert_eq!(l, cloned);
    }

    #[test]
    fn test_lookup_format_value_starts_with() {
        let vals = Lookup::StartsWith.format_value("rust");
        assert_eq!(vals, vec!["rust%"]);
    }

    #[test]
    fn test_lookup_format_value_i_contains() {
        let vals = Lookup::IContains.format_value("HELLO");
        assert_eq!(vals, vec!["%hello%"]);
    }

    #[test]
    fn test_lookup_sql_snippet_istartswith() {
        let snippet = Lookup::IStartsWith.sql_snippet("name", 1);
        assert!(snippet.starts_with("LOWER(name)"));
    }
}
