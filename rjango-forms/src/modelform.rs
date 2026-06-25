//! ModelForm — automatically generates a Form from a model.
//! Like Django's `django.forms.ModelForm`.

use crate::fields::{FormField, FieldType};
use crate::widgets::WidgetType;
use crate::Form;
use std::collections::HashMap;
use serde_json::Value;

/// Options for generating a ModelForm.
pub struct ModelFormOptions {
    pub fields: Vec<String>,
    pub exclude: Vec<String>,
    pub labels: HashMap<String, String>,
    pub help_texts: HashMap<String, String>,
    pub widgets: HashMap<String, WidgetType>,
}

impl Default for ModelFormOptions {
    fn default() -> Self {
        Self {
            fields: vec![],
            exclude: vec![],
            labels: HashMap::new(),
            help_texts: HashMap::new(),
            widgets: HashMap::new(),
        }
    }
}

/// Build a Form from a model definition (fields description).
pub fn modelform_factory(
    model_fields: &[(&str, &str)],
    options: ModelFormOptions,
) -> Form {
    let mut fields = Vec::new();

    for (name, type_str) in model_fields {
        if !options.fields.is_empty() && !options.fields.contains(&name.to_string()) {
            continue;
        }
        if options.exclude.contains(&name.to_string()) {
            continue;
        }

        let label = options.labels.get(*name).cloned().unwrap_or_else(|| name.to_string());
        let field_type = field_type_from_string(type_str);
        let widget = options.widgets.get(*name).cloned().unwrap_or_else(|| {
            default_widget(&field_type)
        });

        fields.push(FormField {
            name: name.to_string(),
            field_type,
            label,
            required: true,
            widget,
            help_text: options.help_texts.get(*name).cloned().unwrap_or_default(),
            initial: None,
            validators: vec![],
        });
    }

    Form { fields, data: HashMap::new(), state: crate::FormState::valid(), is_bound: false }
}

fn field_type_from_string(s: &str) -> FieldType {
    match s {
        "CharField" | "TextField" => FieldType::Char,
        "IntegerField" => FieldType::Integer,
        "BooleanField" => FieldType::Boolean,
        "EmailField" => FieldType::Email,
        "URLField" => FieldType::Char,
        "DateField" => FieldType::Date,
        "DateTimeField" => FieldType::DateTime,
        "ChoiceField" => FieldType::Choice(vec![]),
        "FloatField" | "DecimalField" => FieldType::Float,
        "TextArea" => FieldType::TextArea,
        "PasswordField" => FieldType::Password,
        _ => FieldType::Char,
    }
}

fn default_widget(field_type: &FieldType) -> WidgetType {
    match field_type {
        FieldType::Char => WidgetType::TextInput,
        FieldType::TextArea => WidgetType::Textarea,
        FieldType::Integer | FieldType::Float => WidgetType::NumberInput,
        FieldType::Boolean => WidgetType::CheckboxInput,
        FieldType::Email => WidgetType::EmailInput,
        FieldType::Password => WidgetType::PasswordInput,
        FieldType::Date => WidgetType::DateInput,
        FieldType::DateTime => WidgetType::DateTimeInput,
        FieldType::Choice(_) => WidgetType::Select,
        _ => WidgetType::TextInput,
    }
}

/// Save a ModelForm's cleaned data to a model.
pub fn save_modelform(cleaned_data: HashMap<String, Value>) -> HashMap<String, Value> {
    cleaned_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modelform_factory() {
        let options = ModelFormOptions::default();
        let form = modelform_factory(&[
            ("title", "CharField"),
            ("content", "TextField"),
            ("published", "BooleanField"),
        ], options);
        assert_eq!(form.fields.len(), 3);
        assert_eq!(form.fields[0].name, "title");
    }

    #[test]
    fn test_modelform_exclude() {
        let mut options = ModelFormOptions::default();
        options.exclude.push("content".to_string());
        let form = modelform_factory(&[
            ("title", "CharField"),
            ("content", "TextField"),
        ], options);
        assert_eq!(form.fields.len(), 1);
    }

    #[test]
    fn test_modelform_fields_filter() {
        let mut options = ModelFormOptions::default();
        options.fields.push("title".to_string());
        let form = modelform_factory(&[
            ("title", "CharField"),
            ("content", "TextField"),
        ], options);
        assert_eq!(form.fields.len(), 1);
    }

    #[test]
    fn test_field_type_mapping() {
        assert!(matches!(field_type_from_string("IntegerField"), FieldType::Integer));
        assert!(matches!(field_type_from_string("BooleanField"), FieldType::Boolean));
        assert!(matches!(field_type_from_string("EmailField"), FieldType::Email));
    }

    #[test]
    fn test_default_widget_mapping() {
        assert!(matches!(default_widget(&FieldType::Char), WidgetType::TextInput));
        assert!(matches!(default_widget(&FieldType::Boolean), WidgetType::CheckboxInput));
    }
}
