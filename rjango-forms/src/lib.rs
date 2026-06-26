//! rjango-forms — Form handling, validation, and rendering.
//! Mirrors Django's form system.

pub mod fields;
pub mod widgets;
pub mod rendering;
pub mod modelform;
pub mod formsets;

use std::collections::HashMap;
use serde_json::Value;
use fields::FormField;

/// A validated form's state.
#[derive(Debug, Clone)]
pub struct FormState {
    pub is_valid: bool,
    pub cleaned_data: HashMap<String, Value>,
    pub errors: HashMap<String, Vec<String>>,
}

impl FormState {
    pub fn valid() -> Self {
        Self { is_valid: true, cleaned_data: HashMap::new(), errors: HashMap::new() }
    }

    pub fn invalid(errors: HashMap<String, Vec<String>>) -> Self {
        Self { is_valid: false, cleaned_data: HashMap::new(), errors }
    }

    pub fn field_error(&self, field: &str) -> Option<&Vec<String>> {
        self.errors.get(field)
    }

    pub fn non_field_errors(&self) -> Vec<String> {
        self.errors.get("__all__").cloned().unwrap_or_default()
    }
}

/// A concrete Form with fields, data binding, and validation.
/// Like Django's `forms.Form`.
#[derive(Clone)]
pub struct Form {
    pub fields: Vec<FormField>,
    pub data: HashMap<String, Value>,
    pub state: FormState,
    pub is_bound: bool,
}

impl std::fmt::Debug for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Form")
            .field("is_bound", &self.is_bound)
            .field("fields", &self.fields)
            .finish()
    }
}

impl Form {
    pub fn new(fields: Vec<FormField>) -> Self {
        Self {
            fields,
            data: HashMap::new(),
            state: FormState::valid(),
            is_bound: false,
        }
    }

    /// Bind data to the form and validate.
    pub fn bind(data: HashMap<String, Value>) -> Self {
        Self {
            fields: Vec::new(),
            data,
            state: FormState::valid(),
            is_bound: true,
        }
    }

    /// Bind data from query string.
    pub fn from_query(fields: Vec<FormField>, query_str: &str) -> Self {
        let data = crate::fields::parse_form_data(query_str);
        let mut form = Self::new(fields);
        form.data = data;
        form.is_bound = true;
        form
    }

    /// Validate all fields.
    pub fn validate(&mut self) -> &FormState {
        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        let mut cleaned = HashMap::new();

        for field in &self.fields {
            let raw_value = self.data.get(&field.name).cloned().unwrap_or(Value::Null);
            let mut field_errors = Vec::new();

            // Required check
            if field.required && (raw_value.is_null() || raw_value.as_str().map_or(true, |s| s.is_empty())) {
                field_errors.push(format!("{} is required.", field.label));
            }

            // Run validators
            for validator in &field.validators {
                if let Some(msg) = validator.validate(&raw_value) {
                    field_errors.push(msg);
                }
            }

            // Type-specific validation
            if !field_errors.is_empty() {
                errors.insert(field.name.clone(), field_errors);
            } else if !raw_value.is_null() {
                cleaned.insert(field.name.clone(), raw_value);
            }
        }

        self.state = if errors.is_empty() {
            FormState {
                is_valid: true,
                cleaned_data: cleaned,
                errors: HashMap::new(),
            }
        } else {
            FormState {
                is_valid: false,
                cleaned_data: HashMap::new(),
                errors,
            }
        };

        &self.state
    }

    pub fn is_valid(&mut self) -> bool {
        self.validate().is_valid
    }

    /// Render as HTML table.
    pub fn as_table(&self) -> String {
        use rendering::as_table;
        as_table(&self.fields, &self.state, &self.data)
    }

    /// Render as HTML paragraphs.
    pub fn as_p(&self) -> String {
        use rendering::as_p;
        as_p(&self.fields, &self.state, &self.data)
    }

    /// Render as HTML divs.
    pub fn as_div(&self) -> String {
        use rendering::as_div;
        as_div(&self.fields, &self.state, &self.data)
    }

    pub fn cleaned_data(&self) -> &HashMap<String, Value> {
        &self.state.cleaned_data
    }

    pub fn errors(&self) -> &HashMap<String, Vec<String>> {
        &self.state.errors
    }

    /// HTML helper: hidden CSRF token input.
    pub fn csrf_input() -> &'static str {
        r#"<input type="hidden" name="csrfmiddlewaretoken" value="rjango-csrf-token">"#
    }

    /// HTML helper: submit button.
    pub fn submit_button(label: &str) -> String {
        format!(r#"<button type="submit" class="btn btn-primary">{}</button>"#, label)
    }

    /// Render a complete form with CSRF, submit, and error summary.
    pub fn render(&self, submit_label: &str, action: &str, method: &str) -> String {
        let mut html = String::new();
        html.push_str(&format!(r#"<form action="{}" method="{}">"#, action, method));
        html.push_str(Self::csrf_input());

        // Non-field errors
        let non_field = self.state.non_field_errors();
        if !non_field.is_empty() {
            html.push_str(r#"<ul class="errorlist nonfield">"#);
            for err in non_field {
                html.push_str(&format!("<li>{}</li>", err));
            }
            html.push_str("</ul>");
        }

        html.push_str(&self.as_table());
        html.push_str(&Self::submit_button(submit_label));
        html.push_str("</form>");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::*;

    fn simple_form_fields() -> Vec<FormField> {
        vec![
            FormField::new("username", FieldType::Char).label("Username").required(true),
            FormField::new("email", FieldType::Email).label("Email").required(true),
            FormField::new("age", FieldType::Integer).label("Age").required(false),
        ]
    }

    #[test]
    fn test_empty_form_valid() {
        let form = Form::new(simple_form_fields());
        assert!(!form.is_bound);
    }

    #[test]
    fn test_form_validation_passes() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));
        data.insert("email".into(), Value::String("john@example.com".into()));
        data.insert("age".into(), Value::Number(30.into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        assert!(form.is_valid());
    }

    #[test]
    fn test_form_validation_fails_required() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        assert!(!form.is_valid());
        assert!(form.errors().contains_key("username"));
        assert!(form.errors().contains_key("email"));
    }

    #[test]
    fn test_form_render() {
        let form = Form::new(simple_form_fields());
        let html = form.render("Submit", "/submit/", "post");
        assert!(html.contains("<form"));
        assert!(html.contains("csrfmiddlewaretoken"));
        assert!(html.contains("Submit"));
        assert!(html.contains("Username"));
        assert!(html.contains("Email"));
    }

    #[test]
    fn test_from_query() {
        let fields = simple_form_fields();
        let mut form = Form::from_query(fields, "username=john&email=john@example.com&age=30");
        assert!(form.is_bound);
        assert!(form.is_valid()); // basic required validation passes with values
    }
}
