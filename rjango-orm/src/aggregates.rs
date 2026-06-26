//! Aggregation support for ORM queries.
//! Mirrors Django's `django.db.models.aggregates`.

use std::fmt;

/// Supported aggregation types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggType {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    StdDev,
    Variance,
}

impl fmt::Display for AggType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggType::Count => write!(f, "COUNT"),
            AggType::Sum => write!(f, "SUM"),
            AggType::Avg => write!(f, "AVG"),
            AggType::Min => write!(f, "MIN"),
            AggType::Max => write!(f, "MAX"),
            AggType::StdDev => write!(f, "STDDEV"),
            AggType::Variance => write!(f, "VARIANCE"),
        }
    }
}

/// A single aggregation expression: `AggType(field)` with optional alias.
#[derive(Debug, Clone)]
pub struct Aggregate {
    pub agg_type: AggType,
    pub field: String,
    pub alias: String,
    pub distinct: bool,
}

impl Aggregate {
    /// Create a new aggregate.
    pub fn new(agg_type: AggType, field: &str) -> Self {
        let alias = format!("{}_{}", agg_type.to_string().to_lowercase(), field);
        Self {
            agg_type,
            field: field.to_string(),
            alias,
            distinct: false,
        }
    }

    /// Create with a custom alias.
    pub fn with_alias(agg_type: AggType, field: &str, alias: &str) -> Self {
        Self {
            agg_type,
            field: field.to_string(),
            alias: alias.to_string(),
            distinct: false,
        }
    }

    /// Set the DISTINCT flag.
    pub fn distinct(mut self, value: bool) -> Self {
        self.distinct = value;
        self
    }

    /// Generate SQL for this aggregate.
    pub fn to_sql(&self) -> String {
        let distinct = if self.distinct { "DISTINCT " } else { "" };
        format!("{}({}{}) AS {}", self.agg_type, distinct, self.field, self.alias)
    }
}

/// A collection of aggregations for a query.
#[derive(Debug, Clone, Default)]
pub struct Aggregation {
    pub aggregates: Vec<Aggregate>,
}

impl Aggregation {
    /// Create an empty aggregation set.
    pub fn new() -> Self {
        Self { aggregates: Vec::new() }
    }

    /// Add an aggregate.
    pub fn add(&mut self, agg: Aggregate) -> &mut Self {
        self.aggregates.push(agg);
        self
    }

    /// Add Count.
    pub fn count(field: &str) -> Self {
        Self {
            aggregates: vec![Aggregate::new(AggType::Count, field)],
        }
    }

    /// Add Sum.
    pub fn sum(field: &str) -> Self {
        Self {
            aggregates: vec![Aggregate::new(AggType::Sum, field)],
        }
    }

    /// Add Avg.
    pub fn avg(field: &str) -> Self {
        Self {
            aggregates: vec![Aggregate::new(AggType::Avg, field)],
        }
    }

    /// Add Min.
    pub fn min(field: &str) -> Self {
        Self {
            aggregates: vec![Aggregate::new(AggType::Min, field)],
        }
    }

    /// Add Max.
    pub fn max(field: &str) -> Self {
        Self {
            aggregates: vec![Aggregate::new(AggType::Max, field)],
        }
    }

    /// Build SELECT clause.
    pub fn to_sql(&self) -> Option<String> {
        if self.aggregates.is_empty() {
            return None;
        }
        let parts: Vec<String> = self.aggregates.iter().map(|a| a.to_sql()).collect();
        Some(parts.join(", "))
    }

    /// Number of aggregates.
    pub fn len(&self) -> usize {
        self.aggregates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.aggregates.is_empty()
    }
}

// Shorthand constructors (Django-style)
pub fn count(field: &str) -> Aggregate {
    Aggregate::new(AggType::Count, field)
}
pub fn sum(field: &str) -> Aggregate {
    Aggregate::new(AggType::Sum, field)
}
pub fn avg(field: &str) -> Aggregate {
    Aggregate::new(AggType::Avg, field)
}
pub fn min(field: &str) -> Aggregate {
    Aggregate::new(AggType::Min, field)
}
pub fn max(field: &str) -> Aggregate {
    Aggregate::new(AggType::Max, field)
}
pub fn stddev(field: &str) -> Aggregate {
    Aggregate::new(AggType::StdDev, field)
}
pub fn variance(field: &str) -> Aggregate {
    Aggregate::new(AggType::Variance, field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agg_type_display() {
        assert_eq!(AggType::Count.to_string(), "COUNT");
        assert_eq!(AggType::Sum.to_string(), "SUM");
        assert_eq!(AggType::Avg.to_string(), "AVG");
        assert_eq!(AggType::Min.to_string(), "MIN");
        assert_eq!(AggType::Max.to_string(), "MAX");
        assert_eq!(AggType::StdDev.to_string(), "STDDEV");
        assert_eq!(AggType::Variance.to_string(), "VARIANCE");
    }

    #[test]
    fn test_aggregate_new() {
        let agg = Aggregate::new(AggType::Count, "id");
        assert_eq!(agg.agg_type, AggType::Count);
        assert_eq!(agg.field, "id");
        assert_eq!(agg.alias, "count_id");
        assert!(!agg.distinct);
    }

    #[test]
    fn test_aggregate_with_alias() {
        let agg = Aggregate::with_alias(AggType::Sum, "price", "total_price");
        assert_eq!(agg.alias, "total_price");
    }

    #[test]
    fn test_aggregate_distinct() {
        let agg = Aggregate::new(AggType::Count, "name").distinct(true);
        assert!(agg.distinct);
        let sql = agg.to_sql();
        assert!(sql.contains("DISTINCT"));
    }

    #[test]
    fn test_aggregate_to_sql() {
        let agg = Aggregate::new(AggType::Count, "id");
        let sql = agg.to_sql();
        assert_eq!(sql, "COUNT(id) AS count_id");
    }

    #[test]
    fn test_aggregate_sum_to_sql() {
        let agg = Aggregate::with_alias(AggType::Sum, "price", "total");
        let sql = agg.to_sql();
        assert_eq!(sql, "SUM(price) AS total");
    }

    #[test]
    fn test_aggregation_new() {
        let agg = Aggregation::new();
        assert!(agg.is_empty());
    }

    #[test]
    fn test_aggregation_add() {
        let mut agg = Aggregation::new();
        agg.add(Aggregate::new(AggType::Count, "id"));
        agg.add(Aggregate::new(AggType::Sum, "price"));
        assert_eq!(agg.len(), 2);
    }

    #[test]
    fn test_aggregation_count_helper() {
        let agg = Aggregation::count("id");
        assert_eq!(agg.len(), 1);
    }

    #[test]
    fn test_aggregation_sum_helper() {
        let agg = Aggregation::sum("amount");
        assert_eq!(agg.len(), 1);
    }

    #[test]
    fn test_aggregation_avg_helper() {
        let agg = Aggregation::avg("rating");
        assert_eq!(agg.len(), 1);
    }

    #[test]
    fn test_aggregation_min_helper() {
        let agg = Aggregation::min("price");
        assert_eq!(agg.len(), 1);
    }

    #[test]
    fn test_aggregation_max_helper() {
        let agg = Aggregation::max("price");
        assert_eq!(agg.len(), 1);
    }

    #[test]
    fn test_aggregation_to_sql_empty() {
        let agg = Aggregation::new();
        assert_eq!(agg.to_sql(), None);
    }

    #[test]
    fn test_aggregation_to_sql_multiple() {
        let mut agg = Aggregation::new();
        agg.add(Aggregate::new(AggType::Count, "id"));
        agg.add(Aggregate::with_alias(AggType::Sum, "price", "total"));
        let sql = agg.to_sql().unwrap();
        assert!(sql.contains("COUNT(id)"));
        assert!(sql.contains("SUM(price) AS total"));
    }

    #[test]
    fn test_shorthand_functions() {
        assert_eq!(count("id").agg_type, AggType::Count);
        assert_eq!(sum("p").agg_type, AggType::Sum);
        assert_eq!(avg("p").agg_type, AggType::Avg);
        assert_eq!(min("p").agg_type, AggType::Min);
        assert_eq!(max("p").agg_type, AggType::Max);
        assert_eq!(stddev("p").agg_type, AggType::StdDev);
        assert_eq!(variance("p").agg_type, AggType::Variance);
    }

    #[test]
    fn test_aggregate_clone() {
        let agg = Aggregate::new(AggType::Count, "id");
        let cloned = agg.clone();
        assert_eq!(agg.agg_type, cloned.agg_type);
        assert_eq!(agg.alias, cloned.alias);
    }
}
