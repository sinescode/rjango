/// Database expressions and functions — like Django's `django.db.models.expressions` and `django.db.models.functions`.
/// Provides SQL expression types for use in queries.

use std::fmt;

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
}
