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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::{SimpleField, FieldTypes};

    #[test]
    fn test_model_metadata_new() {
        let meta = ModelMetadata {
            app_label: "blog".to_string(),
            model_name: "Article".to_string(),
            table_name: "blog_article".to_string(),
            fields: Vec::new(),
            pk_field: "id".to_string(),
            ordering: None,
            verbose_name: "article".to_string(),
            verbose_name_plural: "articles".to_string(),
        };
        assert_eq!(meta.app_label, "blog");
        assert_eq!(meta.model_name, "Article");
        assert_eq!(meta.table_name, "blog_article");
        assert_eq!(meta.pk_field, "id");
        assert_eq!(meta.verbose_name, "article");
        assert_eq!(meta.verbose_name_plural, "articles");
        assert!(meta.ordering.is_none());
    }

    #[test]
    fn test_model_builder_new() {
        let _builder = ModelBuilder::new("myapp", "MyModel");
    }

    #[test]
    fn test_model_builder_build_defaults() {
        let meta = ModelBuilder::new("myapp", "MyModel").build();
        assert_eq!(meta.app_label, "myapp");
        assert_eq!(meta.model_name, "MyModel");
        assert_eq!(meta.table_name, "myapp_mymodel");
        assert_eq!(meta.pk_field, "id"); // default when no fields
        assert_eq!(meta.verbose_name, "MyModel");
        assert_eq!(meta.verbose_name_plural, "MyModels");
        assert!(meta.fields.is_empty());
    }

    #[test]
    fn test_model_builder_build_with_fields() {
        let meta = ModelBuilder::new("auth", "User")
            .field(Box::new(SimpleField::new("id", FieldTypes::AutoField)))
            .field(Box::new(SimpleField::new("username", FieldTypes::CharField)))
            .build();
        assert_eq!(meta.app_label, "auth");
        assert_eq!(meta.model_name, "User");
        assert_eq!(meta.table_name, "auth_user");
        assert_eq!(meta.pk_field, "id");
        assert_eq!(meta.fields.len(), 2);
    }

    #[test]
    fn test_model_builder_build_multiple_fields() {
        let meta = ModelBuilder::new("blog", "Post")
            .field(Box::new(SimpleField::new("id", FieldTypes::BigAutoField)))
            .field(Box::new(SimpleField::new("title", FieldTypes::CharField)))
            .field(Box::new(SimpleField::new("body", FieldTypes::TextField)))
            .field(Box::new(SimpleField::new("published", FieldTypes::BooleanField)))
            .build();
        assert_eq!(meta.table_name, "blog_post");
        assert_eq!(meta.fields.len(), 4);
        assert_eq!(meta.pk_field, "id");
    }

    #[test]
    fn test_model_builder_with_underscore_name() {
        let meta = ModelBuilder::new("my_app", "my_model").build();
        assert_eq!(meta.table_name, "my_app_my_model");
        assert_eq!(meta.verbose_name, "my model");
        assert_eq!(meta.verbose_name_plural, "my models");
    }

    #[test]
    fn test_model_builder_no_pk_field() {
        let meta = ModelBuilder::new("app", "Model")
            .field(Box::new(SimpleField::new("name", FieldTypes::CharField)))
            .build();
        assert_eq!(meta.pk_field, "id");
    }

    #[test]
    fn test_model_builder_empty_field_list() {
        let meta = ModelBuilder::new("app", "Empty").build();
        assert_eq!(meta.fields.len(), 0);
        assert_eq!(meta.pk_field, "id");
    }

    #[test]
    fn test_model_metadata_debug() {
        let meta = ModelBuilder::new("test", "TestModel").build();
        let debug = format!("{:?}", meta);
        assert!(debug.contains("test"));
        assert!(debug.contains("TestModel"));
    }

    #[test]
    fn test_model_metadata_clone() {
        let meta = ModelBuilder::new("app", "Source").build();
        let cloned = meta.clone();
        assert_eq!(cloned.app_label, meta.app_label);
        assert_eq!(cloned.model_name, meta.model_name);
        assert_eq!(cloned.table_name, meta.table_name);
        assert!(cloned.fields.is_empty());
    }
}
