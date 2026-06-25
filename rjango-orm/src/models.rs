use std::collections::HashMap;
use crate::fields::Field;

/// Metadata about a model (like Django's _meta).
#[derive(Debug)]
pub struct ModelMetadata {
    pub app_label: String,
    pub model_name: String,
    pub table_name: String,
    pub fields: Vec<Box<dyn Field>>,
    pub pk_field: String,
    pub ordering: Option<Vec<String>>,
    pub verbose_name: String,
    pub verbose_name_plural: String,
}

impl Clone for ModelMetadata {
    fn clone(&self) -> Self {
        Self {
            app_label: self.app_label.clone(),
            model_name: self.model_name.clone(),
            table_name: self.table_name.clone(),
            fields: Vec::new(), // Fields aren't cloneable; skip
            pk_field: self.pk_field.clone(),
            ordering: self.ordering.clone(),
            verbose_name: self.verbose_name.clone(),
            verbose_name_plural: self.verbose_name_plural.clone(),
        }
    }
}

/// Base model trait — all models implement this.
#[async_trait::async_trait]
pub trait Model: Send + Sync + 'static {
    fn meta(&self) -> &ModelMetadata;

    fn pk_value(&self) -> Option<serde_json::Value>;

    fn set_pk(&mut self, value: serde_json::Value);

    fn field_values(&self) -> HashMap<String, serde_json::Value>;

    fn set_field(&mut self, name: &str, value: serde_json::Value);

    /// Save the model to the database.
    async fn save(&self, _pool: &crate::Pool) -> crate::Result<()> {
        Ok(())
    }

    /// Delete the model from the database.
    async fn delete(&self, _pool: &crate::Pool) -> crate::Result<()> {
        Ok(())
    }
}

/// Macro-like helper to define a model struct.
/// User calls ModelBuilder::new(...).field(...).field(...).build()
pub struct ModelBuilder {
    app_label: String,
    model_name: String,
    fields: Vec<Box<dyn Field>>,
}

impl ModelBuilder {
    pub fn new(app_label: &str, model_name: &str) -> Self {
        Self {
            app_label: app_label.to_string(),
            model_name: model_name.to_string(),
            fields: Vec::new(),
        }
    }

    pub fn field(mut self, f: Box<dyn Field>) -> Self {
        self.fields.push(f);
        self
    }

    pub fn build(self) -> ModelMetadata {
        let app_label = self.app_label.clone();
        let model_name = self.model_name.clone();
        let table_name = format!("{}_{}", &app_label, &model_name.to_lowercase()).to_lowercase();
        let pk = self.fields.iter()
            .find(|f| f.is_pk())
            .map(|f| f.name())
            .unwrap_or("id")
            .to_string();
        let verbose = model_name.replace('_', " ");
        ModelMetadata {
            app_label: self.app_label,
            model_name: self.model_name,
            table_name,
            fields: self.fields,
            pk_field: pk,
            ordering: None,
            verbose_name: verbose.clone(),
            verbose_name_plural: format!("{}s", verbose),
        }
    }
}
