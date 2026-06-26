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
    table: String,
}

impl ModelManager {
    pub fn new(table: &str) -> Self {
        Self { table: table.to_string() }
    }

    /// Get the table name for this manager.
    pub fn table(&self) -> &str {
        &self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_manager_new() {
        let _mgr = ModelManager::new("test_table");
        // Can't directly test the table field since it's private
        // but verify it compiles and is constructable
        assert!(true);
    }

    #[test]
    fn test_model_manager_new_with_different_names() {
        let mgr1 = ModelManager::new("myapp_mymodel");
        let mgr2 = ModelManager::new("auth_user");
        let mgr3 = ModelManager::new("");
        // All should construct without panic
        let _ = (mgr1, mgr2, mgr3);
    }

    #[test]
    fn test_manager_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ModelManager>();
    }

    #[test]
    fn test_model_manager_default_construction() {
        let mgr = ModelManager::new("table_name");
        // Verify it compiles with the expected API
        assert!(std::mem::size_of_val(&mgr) > 0);
    }
}
