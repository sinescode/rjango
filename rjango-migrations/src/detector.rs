
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
