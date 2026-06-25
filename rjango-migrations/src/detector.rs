
use crate::operations::{MigrationField, ModelOptions, MigrationOperation};

/// Detect changes between model definitions and the database schema.
pub struct SchemaDetector;

impl SchemaDetector {
    pub fn new() -> Self { Self }

    /// Compare two sets of model fields and detect field additions/removals.
    pub fn detect_field_changes(
        old_fields: &[MigrationField],
        new_fields: &[MigrationField],
        model_name: &str,
    ) -> Vec<MigrationOperation> {
        let mut ops = Vec::new();

        // Detect added fields
        for new_f in new_fields {
            if !old_fields.iter().any(|f| f.name == new_f.name) {
                ops.push(MigrationOperation::AddField {
                    model_name: model_name.to_string(),
                    field: new_f.clone(),
                });
            }
        }

        // Detect removed fields
        for old_f in old_fields {
            if !new_fields.iter().any(|f| f.name == old_f.name) {
                ops.push(MigrationOperation::RemoveField {
                    model_name: model_name.to_string(),
                    field_name: old_f.name.clone(),
                });
            }
        }

        ops
    }

    /// Auto-generate a CreateModel operation from field declarations.
    pub fn create_model_op(name: &str, fields: Vec<MigrationField>) -> MigrationOperation {
        MigrationOperation::CreateModel {
            name: name.to_string(),
            fields,
            options: ModelOptions::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field(name: &str) -> MigrationField {
        MigrationField {
            name: name.to_string(),
            field_type: crate::operations::FieldTypes::CharField,
            null: false,
            primary_key: false,
            unique: false,
            default: None,
            max_length: Some(100),
        }
    }

    #[test]
    fn test_schema_detector_new() {
        let _detector = SchemaDetector::new();
        // Just verify it constructs
    }

    #[test]
    fn test_detect_field_changes_no_changes() {
        let fields = vec![make_field("id"), make_field("title"), make_field("body")];
        let ops = SchemaDetector::detect_field_changes(&fields, &fields, "Post");
        assert!(ops.is_empty());
    }

    #[test]
    fn test_detect_field_changes_added() {
        let old = vec![make_field("id"), make_field("title")];
        let new = vec![make_field("id"), make_field("title"), make_field("body")];
        let ops = SchemaDetector::detect_field_changes(&old, &new, "Post");
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            MigrationOperation::AddField { model_name, field } => {
                assert_eq!(model_name, "Post");
                assert_eq!(field.name, "body");
            }
            _ => panic!("Expected AddField"),
        }
    }

    #[test]
    fn test_detect_field_changes_removed() {
        let old = vec![make_field("id"), make_field("title"), make_field("body")];
        let new = vec![make_field("id"), make_field("body")];
        let ops = SchemaDetector::detect_field_changes(&old, &new, "Post");
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            MigrationOperation::RemoveField { model_name, field_name } => {
                assert_eq!(model_name, "Post");
                assert_eq!(field_name, "title");
            }
            _ => panic!("Expected RemoveField"),
        }
    }

    #[test]
    fn test_detect_field_changes_add_and_remove() {
        let old = vec![make_field("id"), make_field("title"), make_field("old_field")];
        let new = vec![make_field("id"), make_field("title"), make_field("new_field")];
        let ops = SchemaDetector::detect_field_changes(&old, &new, "Post");
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_detect_field_changes_empty_old() {
        let old = vec![];
        let new = vec![make_field("id"), make_field("name")];
        let ops = SchemaDetector::detect_field_changes(&old, &new, "NewModel");
        assert_eq!(ops.len(), 2);
        for op in &ops {
            match op {
                MigrationOperation::AddField { model_name, .. } => {
                    assert_eq!(model_name, "NewModel");
                }
                _ => panic!("Expected AddField"),
            }
        }
    }

    #[test]
    fn test_detect_field_changes_empty_new() {
        let old = vec![make_field("id"), make_field("name")];
        let new = vec![];
        let ops = SchemaDetector::detect_field_changes(&old, &new, "GoneModel");
        assert_eq!(ops.len(), 2);
        for op in &ops {
            match op {
                MigrationOperation::RemoveField { model_name, .. } => {
                    assert_eq!(model_name, "GoneModel");
                }
                _ => panic!("Expected RemoveField"),
            }
        }
    }

    #[test]
    fn test_create_model_op() {
        let fields = vec![
            make_field("id"),
            make_field("title"),
            make_field("body"),
        ];
        let op = SchemaDetector::create_model_op("Article", fields.clone());
        match op {
            MigrationOperation::CreateModel { name, fields: f, .. } => {
                assert_eq!(name, "Article");
                assert_eq!(f.len(), 3);
            }
            _ => panic!("Expected CreateModel"),
        }
    }

    #[test]
    fn test_create_model_op_with_empty_fields() {
        let op = SchemaDetector::create_model_op("EmptyModel", vec![]);
        match op {
            MigrationOperation::CreateModel { name, fields, .. } => {
                assert_eq!(name, "EmptyModel");
                assert!(fields.is_empty());
            }
            _ => panic!("Expected CreateModel"),
        }
    }

    #[test]
    fn test_field_type_access() {
        let f = make_field("test_field");
        assert_eq!(f.name, "test_field");
        assert_eq!(f.field_type, crate::operations::FieldTypes::CharField);
        assert!(!f.null);
        assert!(!f.primary_key);
        assert!(!f.unique);
        assert_eq!(f.max_length, Some(100));
    }
}
