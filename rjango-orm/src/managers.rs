use crate::Result;

/// Manager trait — provides database operations for models.
#[async_trait::async_trait]
pub trait Manager: Send + Sync {
    type Model: crate::Model;
    type QuerySet: Send;

    async fn all(&self, pool: &crate::Pool) -> Result<Vec<Self::Model>>;
    async fn get(&self, pool: &crate::Pool, pk: i64) -> Result<Option<Self::Model>>;
    async fn create(&self, pool: &crate::Pool, values: std::collections::HashMap<String, serde_json::Value>) -> Result<Self::Model>;
    async fn filter(&self, pool: &crate::Pool, field: &str, op: &str, value: &str) -> Result<Vec<Self::Model>>;
    async fn count(&self, pool: &crate::Pool) -> Result<i64>;
}

/// Simple model manager implementation (like Django's `objects`).
pub struct ModelManager {
    #[allow(dead_code)]
    table: String,
}

impl ModelManager {
    pub fn new(table: &str) -> Self {
        Self { table: table.to_string() }
    }
}
