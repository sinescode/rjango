/// Database expressions and functions — like Django's `django.db.models.expressions` and `django.db.models.functions`.
/// Provides SQL expression types for use in queries.

use std::fmt;

/// Q() object — complex query conditions (like Django's `django.db.models.Q`).
/// Supports AND (`&`), OR (`|`), NOT (`~`) operations.
#[derive(Debug, Clone)]
pub struct Q {
    pub children: Vec<QNode>,
    pub connector: QConnector,
    pub negated: bool,
}

#[derive(Debug, Clone)]
pub enum QNode {
    Condition(String, String, Option<String>), // field, lookup, value
    Subquery(Box<Q>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum QConnector {
    And,
    Or,
}

impl Q {
    /// Create a new Q object with a single condition.
    pub fn new(field: &str, value: &str) -> Self {
        Self {
            children: vec![QNode::Condition(field.to_string(), "exact".into(), Some(value.to_string()))],
            connector: QConnector::And,
            negated: false,
        }
    }

    /// Create a Q object with a custom lookup.
    pub fn with_lookup(field: &str, lookup: &str, value: &str) -> Self {
        Self {
            children: vec![QNode::Condition(field.to_string(), lookup.to_string(), Some(value.to_string()))],
            connector: QConnector::And,
            negated: false,
        }
    }

    /// Negate this Q object (`~Q()`).
    pub fn negate(mut self) -> Self {
        self.negated = !self.negated;
        self
    }

    /// Add a child Q object.
    fn add_child(&mut self, child: Q) {
        self.children.push(QNode::Subquery(Box::new(child)));
    }

    /// Combine with another Q using AND (`&`).
    pub fn and(mut self, other: Q) -> Self {
        if self.connector == QConnector::And {
            self.children.push(QNode::Subquery(Box::new(other)));
            self
        } else {
            Self {
                children: vec![
                    QNode::Subquery(Box::new(self)),
                    QNode::Subquery(Box::new(other)),
                ],
                connector: QConnector::And,
                negated: false,
            }
        }
    }

    /// Combine with another Q using OR (`|`).
    pub fn or(mut self, other: Q) -> Self {
        if self.connector == QConnector::Or {
            self.children.push(QNode::Subquery(Box::new(other)));
            self
        } else {
            Self {
                children: vec![
                    QNode::Subquery(Box::new(self)),
                    QNode::Subquery(Box::new(other)),
                ],
                connector: QConnector::Or,
                negated: false,
            }
        }
    }

    /// Render this Q object to a SQL WHERE clause.
    pub fn to_sql(&self) -> String {
        let parts: Vec<String> = self.children.iter().map(|child| {
            match child {
                QNode::Condition(field, lookup, value) => {
                    match lookup.as_str() {
                        "exact" => format!("{} = '{}'", quote_name(field), value.as_deref().unwrap_or("")),
                        "iexact" => format!("LOWER({}) = LOWER('{}')", quote_name(field), value.as_deref().unwrap_or("")),
                        "contains" => format!("{} LIKE '%{}%'", quote_name(field), value.as_deref().unwrap_or("")),
                        "icontains" => format!("LOWER({}) LIKE LOWER('%{}%')", quote_name(field), value.as_deref().unwrap_or("")),
                        "startswith" => format!("{} LIKE '{}%'", quote_name(field), value.as_deref().unwrap_or("")),
                        "endswith" => format!("{} LIKE '%{}'", quote_name(field), value.as_deref().unwrap_or("")),
                        "gt" => format!("{} > '{}'", quote_name(field), value.as_deref().unwrap_or("")),
                        "gte" => format!("{} >= '{}'", quote_name(field), value.as_deref().unwrap_or("")),
                        "lt" => format!("{} < '{}'", quote_name(field), value.as_deref().unwrap_or("")),
                        "lte" => format!("{} <= '{}'", quote_name(field), value.as_deref().unwrap_or("")),
                        "in" => format!("{} IN ({})", quote_name(field), value.as_deref().unwrap_or("")),
                        "isnull" => {
                            if value.as_deref() == Some("true") {
                                format!("{} IS NULL", quote_name(field))
                            } else {
                                format!("{} IS NOT NULL", quote_name(field))
                            }
                        }
                        _ => format!("{} {} '{}'", quote_name(field), lookup, value.as_deref().unwrap_or("")),
                    }
                }
                QNode::Subquery(q) => {
                    let sql = q.to_sql();
                    if q.negated { format!("NOT ({})", sql) } else { sql }
                }
            }
        }).collect();

        let joined = match self.connector {
            QConnector::And => parts.join(" AND "),
            QConnector::Or => parts.join(" OR "),
        };

        if self.negated {
            format!("NOT ({})", joined)
        } else if parts.len() > 1 {
            format!("({})", joined)
        } else {
            joined
        }
    }
}

fn quote_name(name: &str) -> String {
    format!("\"{}\"", name)
}

/// Create a Q object.
pub fn q(field: &str, value: &str) -> Q {
    Q::new(field, value)
}

/// Aggregate expression — like Django's `django.db.models.aggregates`.
#[derive(Debug, Clone)]
pub enum Aggregate {
    Sum(Expression),
    Count(Expression),
    Avg(Expression),
    Min(Expression),
    Max(Expression),
}

impl Aggregate {
    pub fn to_sql(&self) -> String {
        match self {
            Aggregate::Sum(e) => format!("SUM({})", e.to_sql()),
            Aggregate::Count(e) => format!("COUNT({})", e.to_sql()),
            Aggregate::Avg(e) => format!("AVG({})", e.to_sql()),
            Aggregate::Min(e) => format!("MIN({})", e.to_sql()),
            Aggregate::Max(e) => format!("MAX({})", e.to_sql()),
        }
    }
}

pub fn sum(expr: Expression) -> Aggregate { Aggregate::Sum(expr) }
pub fn count(expr: Expression) -> Aggregate { Aggregate::Count(expr) }
pub fn avg(expr: Expression) -> Aggregate { Aggregate::Avg(expr) }
pub fn min(expr: Expression) -> Aggregate { Aggregate::Min(expr) }
pub fn max(expr: Expression) -> Aggregate { Aggregate::Max(expr) }

/// A database expression that can be rendered as SQL.
#[derive(Debug, Clone)]
pub enum Expression {
    /// Column reference: `"table"."column"`
    Column { table: Option<String>, name: String },
    /// Value literal
    Value(String),
    /// SQL function call
    Function { name: String, args: Vec<Expression>, alias: Option<String> },
    /// Raw SQL expression
    Raw(String),
    /// Combined with AND
    And(Box<Expression>, Box<Expression>),
    /// Combined with OR
    Or(Box<Expression>, Box<Expression>),
    /// Negation
    Not(Box<Expression>),
}

impl Expression {
    /// Render the expression as SQL.
    pub fn to_sql(&self) -> String {
        match self {
            Expression::Column { table, name } => {
                if let Some(t) = table {
                    format!("\"{}\".\"{}\"", t, name)
                } else {
                    format!("\"{}\"", name)
                }
            }
            Expression::Value(v) => {
                if v == "NULL" {
                    "NULL".into()
                } else {
                    format!("'{}'", v.replace('\'', "''"))
                }
            }
            Expression::Function { name, args, alias } => {
                let args_sql: Vec<String> = args.iter().map(|a| a.to_sql()).collect();
                let mut sql = format!("{}({})", name.to_uppercase(), args_sql.join(", "));
                if let Some(a) = alias {
                    sql.push_str(&format!(" AS \"{}\"", a));
                }
                sql
            }
            Expression::Raw(s) => s.clone(),
            Expression::And(a, b) => format!("({} AND {})", a.to_sql(), b.to_sql()),
            Expression::Or(a, b) => format!("({} OR {})", a.to_sql(), b.to_sql()),
            Expression::Not(e) => format!("NOT ({})", e.to_sql()),
        }
    }
}

// ── Database Functions (like Django's django.db.models.functions) ──

/// `NOW()` — current timestamp (like Django's `Now`).
pub fn now() -> Expression {
    Expression::Function {
        name: "NOW".into(),
        args: vec![],
        alias: None,
    }
}

/// `CAST(expr AS type)` — type cast (like Django's `Cast`).
pub fn cast(expr: Expression, sql_type: &str) -> Expression {
    Expression::Raw(format!("CAST({} AS {})", expr.to_sql(), sql_type))
}

/// `COALESCE(expr1, expr2, ...)` — first non-null (like Django's `Coalesce`).
pub fn coalesce(args: Vec<Expression>) -> Expression {
    Expression::Function {
        name: "COALESCE".into(),
        args,
        alias: None,
    }
}

/// `LENGTH(expr)` — string length (like Django's `Length`).
pub fn length(expr: Expression) -> Expression {
    Expression::Function {
        name: "LENGTH".into(),
        args: vec![expr],
        alias: None,
    }
}

/// `LOWER(expr)` — lowercase (like Django's `Lower`).
pub fn lower(expr: Expression) -> Expression {
    Expression::Function {
        name: "LOWER".into(),
        args: vec![expr],
        alias: None,
    }
}

/// `UPPER(expr)` — uppercase (like Django's `Upper`).
pub fn upper(expr: Expression) -> Expression {
    Expression::Function {
        name: "UPPER".into(),
        args: vec![expr],
        alias: None,
    }
}

/// `SUBSTR(expr, start, length)` — substring (like Django's `Substr`).
pub fn substr(expr: Expression, start: i32, length: Option<i32>) -> Expression {
    let mut args = vec![expr, Expression::Value(start.to_string())];
    if let Some(len) = length {
        args.push(Expression::Value(len.to_string()));
    }
    Expression::Function {
        name: "SUBSTR".into(),
        args,
        alias: None,
    }
}

/// `TRIM(expr)` — trim whitespace (like Django's `Trim`).
pub fn trim(expr: Expression) -> Expression {
    Expression::Function {
        name: "TRIM".into(),
        args: vec![expr],
        alias: None,
    }
}

/// `CONCAT(expr1, expr2, ...)` — string concatenation (like Django's `Concat`).
pub fn concat(args: Vec<Expression>) -> Expression {
    Expression::Function {
        name: "CONCAT".into(),
        args,
        alias: None,
    }
}

/// Column reference helper.
pub fn col(name: &str) -> Expression {
    Expression::Column { table: None, name: name.into() }
}

/// Column reference with table.
pub fn col_table(table: &str, name: &str) -> Expression {
    Expression::Column { table: Some(table.into()), name: name.into() }
}

/// Raw SQL expression.
pub fn raw(sql: &str) -> Expression {
    Expression::Raw(sql.into())
}

/// F() expression — references a model field (like Django's `F()`).
pub struct F {
    pub name: String,
}

impl F {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }
}

impl fmt::Display for F {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.name)
    }
}

impl From<F> for Expression {
    fn from(f: F) -> Self {
        Expression::Column { table: None, name: f.name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_function() {
        let sql = now().to_sql();
        assert_eq!(sql, "NOW()");
    }

    #[test]
    fn test_cast_function() {
        let sql = cast(col("age"), "TEXT").to_sql();
        assert_eq!(sql, r#"CAST("age" AS TEXT)"#);
    }

    #[test]
    fn test_coalesce_function() {
        let sql = coalesce(vec![col("title"), Expression::Value("Untitled".into())]).to_sql();
        assert_eq!(sql, r#"COALESCE("title", 'Untitled')"#);
    }

    #[test]
    fn test_length_function() {
        let sql = length(col("name")).to_sql();
        assert_eq!(sql, r#"LENGTH("name")"#);
    }

    #[test]
    fn test_lower_function() {
        let sql = lower(col("email")).to_sql();
        assert_eq!(sql, r#"LOWER("email")"#);
    }

    #[test]
    fn test_upper_function() {
        let sql = upper(col("name")).to_sql();
        assert_eq!(sql, r#"UPPER("name")"#);
    }

    #[test]
    fn test_substr_function() {
        let sql = substr(col("name"), 1, Some(10)).to_sql();
        assert_eq!(sql, r#"SUBSTR("name", '1', '10')"#);
    }

    #[test]
    fn test_trim_function() {
        let sql = trim(col("name")).to_sql();
        assert_eq!(sql, r#"TRIM("name")"#);
    }

    #[test]
    fn test_concat_function() {
        let sql = concat(vec![col("first_name"), Expression::Value(" ".into()), col("last_name")]).to_sql();
        assert_eq!(sql, r#"CONCAT("first_name", ' ', "last_name")"#);
    }

    #[test]
    fn test_column_expression() {
        let sql = col("id").to_sql();
        assert_eq!(sql, r#""id""#);
    }

    #[test]
    fn test_column_with_table() {
        let sql = col_table("users", "email").to_sql();
        assert_eq!(sql, r#""users"."email""#);
    }

    #[test]
    fn test_value_expression() {
        let sql = Expression::Value("hello".into()).to_sql();
        assert_eq!(sql, "'hello'");
    }

    #[test]
    fn test_null_value() {
        let sql = Expression::Value("NULL".into()).to_sql();
        assert_eq!(sql, "NULL");
    }

    #[test]
    fn test_raw_expression() {
        let sql = raw("COUNT(*)").to_sql();
        assert_eq!(sql, "COUNT(*)");
    }

    #[test]
    fn test_and_expression() {
        let sql = Expression::And(
            Box::new(Expression::Raw("age > 18".into())),
            Box::new(Expression::Raw("active = 1".into())),
        ).to_sql();
        assert_eq!(sql, "(age > 18 AND active = 1)");
    }

    #[test]
    fn test_or_expression() {
        let sql = Expression::Or(
            Box::new(Expression::Raw("status = 'draft'".into())),
            Box::new(Expression::Raw("status = 'pending'".into())),
        ).to_sql();
        assert_eq!(sql, "(status = 'draft' OR status = 'pending')");
    }

    #[test]
    fn test_not_expression() {
        let sql = Expression::Not(Box::new(Expression::Raw("deleted".into()))).to_sql();
        assert_eq!(sql, "NOT (deleted)");
    }

    #[test]
    fn test_f_expression() {
        let f = F::new("price");
        let sql: Expression = f.into();
        assert_eq!(sql.to_sql(), r#""price""#);
    }

    #[test]
    fn test_expression_display_f() {
        let f = F::new("quantity");
        assert_eq!(format!("{}", f), r#""quantity""#);
    }

    #[test]
    fn test_nested_functions() {
        let expr = lower(trim(col("email")));
        let sql = expr.to_sql();
        assert_eq!(sql, r#"LOWER(TRIM("email"))"#);
    }

    #[test]
    fn test_q_simple() {
        let q = Q::new("name", "Alice");
        assert_eq!(q.to_sql(), r#""name" = 'Alice'"#);
    }

    #[test]
    fn test_q_contains() {
        let q = Q::with_lookup("title", "contains", "rust");
        assert_eq!(q.to_sql(), r#""title" LIKE '%rust%'"#);
    }

    #[test]
    fn test_q_and() {
        let q1 = Q::new("status", "active");
        let q2 = Q::new("age", "18");
        let combined = q1.and(q2);
        assert_eq!(combined.to_sql(), r#"("status" = 'active' AND "age" = '18')"#);
    }

    #[test]
    fn test_q_or() {
        let q1 = Q::new("role", "admin");
        let q2 = Q::new("role", "moderator");
        let combined = q1.or(q2);
        assert_eq!(combined.to_sql(), r#"("role" = 'admin' OR "role" = 'moderator')"#);
    }

    #[test]
    fn test_q_negated() {
        let q = Q::new("deleted", "true").negate();
        assert_eq!(q.to_sql(), r#"NOT ("deleted" = 'true')"#);
    }

    #[test]
    fn test_q_complex() {
        // (status='active' AND (role='admin' OR role='moderator'))
        let role_q = Q::new("role", "admin").or(Q::new("role", "moderator"));
        let combined = Q::new("status", "active").and(role_q);
        assert_eq!(combined.to_sql(),
            r#"("status" = 'active' AND ("role" = 'admin' OR "role" = 'moderator'))"#);
    }

    #[test]
    fn test_q_lookup_gt() {
        let q = Q::with_lookup("age", "gt", "21");
        assert_eq!(q.to_sql(), r#""age" > '21'"#);
    }

    #[test]
    fn test_q_lookup_isnull() {
        let q = Q::with_lookup("email", "isnull", "true");
        assert_eq!(q.to_sql(), r#""email" IS NULL"#);
    }

    #[test]
    fn test_q_function() {
        let q = q("name", "Bob");
        assert_eq!(q.to_sql(), r#""name" = 'Bob'"#);
    }

    #[test]
    fn test_aggregate_sum() {
        let agg = sum(col("price"));
        assert_eq!(agg.to_sql(), r#"SUM("price")"#);
    }
    #[test]
    fn test_aggregate_count() {
        let agg = count(col("id"));
        assert_eq!(agg.to_sql(), r#"COUNT("id")"#);
    }
    #[test]
    fn test_aggregate_avg() {
        let agg = avg(col("rating"));
        assert_eq!(agg.to_sql(), r#"AVG("rating")"#);
    }
    #[test]
    fn test_aggregate_min() {
        let agg = min(col("age"));
        assert_eq!(agg.to_sql(), r#"MIN("age")"#);
    }
    #[test]
    fn test_aggregate_max() {
        let agg = max(col("score"));
        assert_eq!(agg.to_sql(), r#"MAX("score")"#);
    }
}
