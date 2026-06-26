//! Database functions — mirrors Django's `django.db.models.functions`.
//! SQL functions that can be used in queries and annotations.

/// Supported SQL function types.
#[derive(Debug, Clone, PartialEq)]
pub enum DbFunction {
    Coalesce(Vec<String>),
    Concat(Vec<String>),
    Length(String),
    Lower(String),
    Upper(String),
    Trim(String),
    LTrim(String),
    RTrim(String),
    Substr(String, isize, Option<isize>),
    Replace(String, String, String),
    Reverse(String),
    Md5(String),
    Sha1(String),
    Sha256(String),
    Sha512(String),
    Crc32(String),
    Cast(String, String),
    Abs(String),
    Ceil(String),
    Floor(String),
    Round(String, Option<u32>),
    Power(String, f64),
    Sqrt(String),
    Now,
    ExtractYear(String),
    ExtractMonth(String),
    ExtractDay(String),
    ExtractHour(String),
    ExtractMinute(String),
    ExtractSecond(String),
    StrIndex(String, String),
    Greatest(Vec<String>),
    Least(Vec<String>),
}

impl DbFunction {
    /// Generate SQL for this function.
    pub fn to_sql(&self, param_idx: &mut usize) -> String {
        match self {
            DbFunction::Coalesce(fields) => {
                let mut args = Vec::with_capacity(fields.len());
                for _ in fields {
                    args.push(format!("${}", param_idx));
                    *param_idx += 1;
                }
                format!("COALESCE({})", args.join(", "))
            }
            DbFunction::Concat(fields) => {
                let args: Vec<String> = fields.iter().map(|_f| {
                    let idx = *param_idx;
                    *param_idx += 1;
                    format!("${}", idx)
                }).collect();
                format!("CONCAT({})", args.join(", "))
            }
            DbFunction::Length(field) => format!("LENGTH({})", field),
            DbFunction::Lower(field) => format!("LOWER({})", field),
            DbFunction::Upper(field) => format!("UPPER({})", field),
            DbFunction::Trim(field) => format!("TRIM({})", field),
            DbFunction::LTrim(field) => format!("LTRIM({})", field),
            DbFunction::RTrim(field) => format!("RTRIM({})", field),
            DbFunction::Substr(field, start, len) => {
                if let Some(l) = len {
                    format!("SUBSTR({}, {}, {})", field, start, l)
                } else {
                    format!("SUBSTR({}, {})", field, start)
                }
            }
            DbFunction::Replace(field, _from, _to) => {
                let f_idx = *param_idx; *param_idx += 1;
                let t_idx = *param_idx; *param_idx += 1;
                format!("REPLACE({}, ${}, ${})", field, f_idx, t_idx)
            }
            DbFunction::Reverse(field) => format!("REVERSE({})", field),
            DbFunction::Md5(field) => format!("MD5({})", field),
            DbFunction::Sha1(field) => format!("SHA1({})", field),
            DbFunction::Sha256(field) => format!("SHA256({})", field),
            DbFunction::Sha512(field) => format!("SHA512({})", field),
            DbFunction::Crc32(field) => format!("CRC32({})", field),
            DbFunction::Cast(field, as_type) => format!("CAST({} AS {})", field, as_type),
            DbFunction::Abs(field) => format!("ABS({})", field),
            DbFunction::Ceil(field) => format!("CEIL({})", field),
            DbFunction::Floor(field) => format!("FLOOR({})", field),
            DbFunction::Round(field, decimals) => {
                if let Some(d) = decimals {
                    format!("ROUND({}, {})", field, d)
                } else {
                    format!("ROUND({})", field)
                }
            }
            DbFunction::Power(field, exp) => format!("POWER({}, {})", field, exp),
            DbFunction::Sqrt(field) => format!("SQRT({})", field),
            DbFunction::Now => "NOW()".to_string(),
            DbFunction::ExtractYear(field) => format!("CAST(strftime('%Y', {}) AS INTEGER)", field),
            DbFunction::ExtractMonth(field) => format!("CAST(strftime('%m', {}) AS INTEGER)", field),
            DbFunction::ExtractDay(field) => format!("CAST(strftime('%d', {}) AS INTEGER)", field),
            DbFunction::ExtractHour(field) => format!("CAST(strftime('%H', {}) AS INTEGER)", field),
            DbFunction::ExtractMinute(field) => format!("CAST(strftime('%M', {}) AS INTEGER)", field),
            DbFunction::ExtractSecond(field) => format!("CAST(strftime('%S', {}) AS INTEGER)", field),
            DbFunction::StrIndex(field, _substr) => {
                let idx = *param_idx; *param_idx += 1;
                format!("INSTR({}, ${})", field, idx)
            }
            DbFunction::Greatest(fields) => {
                format!("GREATEST({})", fields.join(", "))
            }
            DbFunction::Least(fields) => {
                format!("LEAST({})", fields.join(", "))
            }
        }
    }
}

/// Helper constructors matching Django's `django.db.models.functions`.
pub fn coalesce(fields: Vec<String>) -> DbFunction { DbFunction::Coalesce(fields) }
pub fn concat(fields: Vec<String>) -> DbFunction { DbFunction::Concat(fields) }
pub fn length(field: &str) -> DbFunction { DbFunction::Length(field.to_string()) }
pub fn lower(field: &str) -> DbFunction { DbFunction::Lower(field.to_string()) }
pub fn upper(field: &str) -> DbFunction { DbFunction::Upper(field.to_string()) }
pub fn trim(field: &str) -> DbFunction { DbFunction::Trim(field.to_string()) }
pub fn ltrim(field: &str) -> DbFunction { DbFunction::LTrim(field.to_string()) }
pub fn rtrim(field: &str) -> DbFunction { DbFunction::RTrim(field.to_string()) }
pub fn substr(field: &str, start: isize, len: Option<isize>) -> DbFunction {
    DbFunction::Substr(field.to_string(), start, len)
}
pub fn replace(field: &str, from: &str, to: &str) -> DbFunction {
    DbFunction::Replace(field.to_string(), from.to_string(), to.to_string())
}
pub fn reverse(field: &str) -> DbFunction { DbFunction::Reverse(field.to_string()) }
pub fn md5(field: &str) -> DbFunction { DbFunction::Md5(field.to_string()) }
pub fn now() -> DbFunction { DbFunction::Now }
pub fn abs(field: &str) -> DbFunction { DbFunction::Abs(field.to_string()) }
pub fn ceil(field: &str) -> DbFunction { DbFunction::Ceil(field.to_string()) }
pub fn floor(field: &str) -> DbFunction { DbFunction::Floor(field.to_string()) }
pub fn round(field: &str, decimals: Option<u32>) -> DbFunction {
    DbFunction::Round(field.to_string(), decimals)
}
pub fn sqrt(field: &str) -> DbFunction { DbFunction::Sqrt(field.to_string()) }
pub fn greatest(fields: Vec<String>) -> DbFunction { DbFunction::Greatest(fields) }
pub fn least(fields: Vec<String>) -> DbFunction { DbFunction::Least(fields) }
pub fn cast(field: &str, as_type: &str) -> DbFunction {
    DbFunction::Cast(field.to_string(), as_type.to_string())
}
pub fn extract_year(field: &str) -> DbFunction { DbFunction::ExtractYear(field.to_string()) }
pub fn extract_month(field: &str) -> DbFunction { DbFunction::ExtractMonth(field.to_string()) }
pub fn extract_day(field: &str) -> DbFunction { DbFunction::ExtractDay(field.to_string()) }
pub fn extract_hour(field: &str) -> DbFunction { DbFunction::ExtractHour(field.to_string()) }
pub fn extract_minute(field: &str) -> DbFunction { DbFunction::ExtractMinute(field.to_string()) }
pub fn extract_second(field: &str) -> DbFunction { DbFunction::ExtractSecond(field.to_string()) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length() {
        let f = length("name");
        assert_eq!(f.to_sql(&mut 1), "LENGTH(name)");
    }

    #[test]
    fn test_lower() {
        let f = lower("name");
        assert_eq!(f.to_sql(&mut 1), "LOWER(name)");
    }

    #[test]
    fn test_upper() {
        let f = upper("name");
        assert_eq!(f.to_sql(&mut 1), "UPPER(name)");
    }

    #[test]
    fn test_trim() {
        let f = trim("name");
        assert_eq!(f.to_sql(&mut 1), "TRIM(name)");
    }

    #[test]
    fn test_abs() {
        let f = abs("score");
        assert_eq!(f.to_sql(&mut 1), "ABS(score)");
    }

    #[test]
    fn test_now() {
        let f = now();
        assert_eq!(f.to_sql(&mut 1), "NOW()");
    }

    #[test]
    fn test_coalesce() {
        let f = coalesce(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(f.to_sql(&mut 1), "COALESCE($1, $2)");
    }

    #[test]
    fn test_substr() {
        let f = substr("bio", 1, Some(10));
        assert_eq!(f.to_sql(&mut 1), "SUBSTR(bio, 1, 10)");
    }

    #[test]
    fn test_substr_no_len() {
        let f = substr("bio", 3, None);
        assert_eq!(f.to_sql(&mut 1), "SUBSTR(bio, 3)");
    }

    #[test]
    fn test_replace() {
        let f = replace("name", "old", "new");
        let sql = f.to_sql(&mut 1);
        assert!(sql.starts_with("REPLACE(name,"));
    }

    #[test]
    fn test_round() {
        let f = round("price", Some(2));
        assert_eq!(f.to_sql(&mut 1), "ROUND(price, 2)");
    }

    #[test]
    fn test_round_default() {
        let f = round("price", None);
        assert_eq!(f.to_sql(&mut 1), "ROUND(price)");
    }

    #[test]
    fn test_cast() {
        let f = cast("id", "TEXT");
        assert_eq!(f.to_sql(&mut 1), "CAST(id AS TEXT)");
    }

    #[test]
    fn test_greatest() {
        let f = greatest(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(f.to_sql(&mut 1), "GREATEST(a, b)");
    }

    #[test]
    fn test_least() {
        let f = least(vec!["x".to_string(), "y".to_string()]);
        assert_eq!(f.to_sql(&mut 1), "LEAST(x, y)");
    }

    #[test]
    fn test_extract_year() {
        let f = extract_year("created_at");
        assert!(f.to_sql(&mut 1).contains("strftime"));
    }

    #[test]
    fn test_extract_month() {
        let f = extract_month("created_at");
        assert!(f.to_sql(&mut 1).contains("'%m'"));
    }

    #[test]
    fn test_sqrt() {
        let f = sqrt("distance");
        assert_eq!(f.to_sql(&mut 1), "SQRT(distance)");
    }

    #[test]
    fn test_reverse() {
        let f = reverse("name");
        assert_eq!(f.to_sql(&mut 1), "REVERSE(name)");
    }

    #[test]
    fn test_db_function_clone() {
        let f = length("name");
        let c = f.clone();
        assert_eq!(f, c);
    }

    #[test]
    fn test_db_function_debug() {
        let f = lower("email");
        let d = format!("{:?}", f);
        assert!(d.contains("Lower"));
    }
}
