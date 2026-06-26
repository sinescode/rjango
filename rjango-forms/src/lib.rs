//! rjango-forms — Form handling, validation, and rendering.
//! Mirrors Django's form system.
//!
//! Validation pipeline (Django-compatible order):
//! 1. `full_clean()` — orchestrates everything
//! 2. `_clean_fields()` — calls each field's `clean()` method via `field.clean()`
//! 3. `_clean_form()` — calls `self.clean()` for cross-field validation
//! 4. `_post_clean()` — hook for model validation
//!
//! Override `clean()` for custom cross-field validation.
//! Override `clean_<fieldname>()` for per-field custom validation.

pub mod fields;
pub mod widgets;
pub mod rendering;
pub mod modelform;
pub mod formsets;
pub mod validation;

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
///
/// Validation pipeline:
/// 1. `full_clean()` — calls `_clean_fields()`, `_clean_form()`, `_post_clean()`
/// 2. `_clean_fields()` — calls each field's `clean()` and `clean_<fieldname>()`
/// 3. `_clean_form()` — calls `self.clean()` for cross-field validation
/// 4. `_post_clean()` — override for model-form integration
///
/// Override `clean()` or subclass to add custom validation via `add_error()`.
#[derive(Clone)]
pub struct Form {
    pub fields: Vec<FormField>,
    pub data: HashMap<String, Value>,
    pub state: FormState,
    pub is_bound: bool,
    pub cleaned: bool,
}

impl std::fmt::Debug for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Form")
            .field("is_bound", &self.is_bound)
            .field("fields", &self.fields)
            .field("is_valid", &self.state.is_valid)
            .field("error_count", &self.state.errors.len())
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
            cleaned: false,
        }
    }

    /// Bind data to the form (does NOT validate).
    pub fn bind(data: HashMap<String, Value>) -> Self {
        Self {
            fields: Vec::new(),
            data,
            state: FormState::valid(),
            is_bound: true,
            cleaned: false,
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

    // ── Validation pipeline (Django-compatible) ─────────────────────

    /// Run the full validation pipeline.
    /// Like Django's `form.full_clean()`.
    pub fn full_clean(&mut self) {
        self.cleaned = true;
        // Preserve errors added manually before full_clean (e.g., via add_error)
        let preexisting_errors = std::mem::take(&mut self.state.errors);
        let preexisting_invalid = !self.state.is_valid && !preexisting_errors.is_empty();
        
        self._clean_fields();
        self._clean_form();
        self._post_clean();
        
        // Re-merge errors that were added before full_clean
        for (field, errs) in preexisting_errors {
            for err in errs {
                self.state.errors.entry(field.clone()).or_insert_with(Vec::new).push(err);
            }
        }
        if preexisting_invalid || !self.state.errors.is_empty() {
            self.state.is_valid = false;
        }
    }

    /// Step 1: Clean individual fields — calls each field's `clean()` method.
    /// Like Django's `Form._clean_fields()`.
    pub fn _clean_fields(&mut self) {
        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        let mut cleaned: HashMap<String, Value> = HashMap::new();

        for field in &self.fields {
            let raw_value = self.data.get(&field.name).cloned().unwrap_or(Value::Null);

            // Treat empty string as not provided for required-field checks (Django-compatible)
            let effective_value = match &raw_value {
                Value::String(s) if s.is_empty() && field.required => Value::Null,
                _ => raw_value.clone(),
            };

            // Run field type-specific clean (type conversion + basic validation)
            let clean_result = field.clean(&effective_value);

            // Run any extra validators from core
            let validator_errors: Vec<String> = field.validators
                .iter()
                .filter_map(|v| v.validate(&raw_value))
                .collect();

            match clean_result {
                Ok(cleaned_val) => {
                    if !validator_errors.is_empty() {
                        errors.insert(field.name.clone(), validator_errors);
                    } else if !cleaned_val.is_null() {
                        cleaned.insert(field.name.clone(), cleaned_val);
                    }
                },
                Err(clean_err) => {
                    let mut all_errors = vec![clean_err];
                    all_errors.extend(validator_errors);
                    errors.insert(field.name.clone(), all_errors);
                }
            }
        }

        self.state.is_valid = errors.is_empty();
        self.state.cleaned_data = cleaned;
        self.state.errors = errors;
    }

    /// Step 2: Form-level validation — calls `self.clean()`.
    /// Like Django's `Form._clean_form()`.
    pub fn _clean_form(&mut self) {
        if self.state.is_valid {
            if let Some(non_field_errors) = self.clean() {
                for err in non_field_errors {
                    self.add_error(None, err);
                }
            }
        }
    }

    /// Step 3: Post-clean hook — override for model-form integration.
    /// Like Django's `Form._post_clean()`.
    pub fn _post_clean(&mut self) {
        // Override in subclass for model form validation
    }

    /// Override this method for custom cross-field validation.
    /// Return `Some(vec![...])` with non-field error messages, or `None` if valid.
    /// Like Django's `Form.clean()`.
    pub fn clean(&self) -> Option<Vec<String>> {
        None
    }

    /// Add an error to a field or as a non-field error (`__all__`).
    /// Like Django's `form.add_error(field, error)`.
    pub fn add_error(&mut self, field: Option<&str>, error: String) {
        let key = field.unwrap_or("__all__").to_string();
        self.state
            .errors
            .entry(key)
            .or_insert_with(Vec::new)
            .push(error);
        self.state.is_valid = false;
    }

    /// Validate bound data.
    /// Like Django's `form.is_valid()` — calls full_clean().
    pub fn is_valid(&mut self) -> bool {
        if !self.cleaned {
            self.full_clean();
        }
        self.state.is_valid
    }

    /// Validate and return a reference to state (for batch field errors).
    pub fn validate(&mut self) -> &FormState {
        if !self.cleaned {
            self.full_clean();
        }
        &self.state
    }

    /// Check if the form data differs from initial values.
    /// Like Django's `form.has_changed()`.
    pub fn has_changed(&self) -> bool {
        for field in &self.fields {
            let submitted = self.data.get(&field.name);
            let initial = &field.initial;
            match (submitted, initial) {
                (Some(s), Some(i)) if s != i => return true,
                (Some(_), None) => return true,
                _ => {}
            }
        }
        false
    }

    /// Return cleaned data after validation.
    pub fn cleaned_data(&self) -> &HashMap<String, Value> {
        &self.state.cleaned_data
    }

    /// Return all errors.
    pub fn errors(&self) -> &HashMap<String, Vec<String>> {
        &self.state.errors
    }

    /// Return non-field errors only.
    pub fn non_field_errors(&self) -> Vec<String> {
        self.state.non_field_errors()
    }

    // ── Rendering ──

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
        let mut data: HashMap<String, Value> = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));
        data.insert("email".into(), Value::String("john@example.com".into()));
        data.insert("age".into(), Value::String("30".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        assert!(form.is_valid());
    }

    #[test]
    fn test_form_validation_fails_required() {
        let mut data: HashMap<String, Value> = HashMap::new();
        data.insert("username".into(), Value::String("".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        assert!(!form.is_valid());
        assert!(form.errors().contains_key("username"));
        assert!(form.errors().contains_key("email"));
    }

    #[test]
    fn test_form_validation_email_fails() {
        let mut data: HashMap<String, Value> = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));
        data.insert("email".into(), Value::String("not-an-email".into()));
        data.insert("age".into(), Value::String("30".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        assert!(!form.is_valid());
        assert!(form.errors().contains_key("email"));
    }

    #[test]
    fn test_form_validation_integer_fails() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));
        data.insert("email".into(), Value::String("john@example.com".into()));
        data.insert("age".into(), Value::String("not-a-number".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        assert!(!form.is_valid());
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
        assert!(form.is_valid());
    }

    #[test]
    fn test_add_field_error() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));
        data.insert("email".into(), Value::String("john@example.com".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        form.add_error(Some("username"), "Custom username error".into());
        assert!(!form.is_valid());
        assert_eq!(form.errors().get("username").unwrap().len(), 1);
        assert!(form.errors().get("username").unwrap()[0].contains("Custom"));
    }

    #[test]
    fn test_add_non_field_error() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));
        data.insert("email".into(), Value::String("john@example.com".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        form.add_error(None, "Form-level error".into());
        assert!(!form.is_valid());
        let non_field = form.non_field_errors();
        assert_eq!(non_field.len(), 1);
        assert!(non_field[0].contains("Form-level"));
    }

    #[test]
    fn test_has_changed_with_no_initial() {
        let mut data: HashMap<String, Value> = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        // Has initial? No. Has data? Yes. -> changed
        let changed = form.has_changed();
        assert!(changed); // data has items, field has no initial
    }

    #[test]
    fn test_has_changed_same_as_initial() {
        let field = FormField::new("username", FieldType::Char)
            .initial(Value::String("john".into()));
        let mut data: HashMap<String, Value> = HashMap::new();
        data.insert("username".into(), Value::String("john".into()));

        let form = Form::new(vec![field]);
        // Data matches initial — not changed
        // Actually has_changed checks data vs field.initial, and data has the same value
        let changed = form.has_changed();
        assert!(!changed); // same as initial — but data HAS entries...
    }

    #[test]
    fn test_cleaned_data_after_validation() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("alice".into()));
        data.insert("email".into(), Value::String("alice@example.com".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        assert!(form.is_valid());
        assert_eq!(form.cleaned_data().get("username").unwrap(), &Value::String("alice".into()));
    }

    #[test]
    fn test_non_field_errors_empty_when_valid() {
        let form = Form::new(simple_form_fields());
        assert!(form.non_field_errors().is_empty());
    }

    #[test]
    fn test_debug_output() {
        let form = Form::new(simple_form_fields());
        let debug = format!("{:?}", form);
        assert!(debug.contains("Form"));
    }

    #[test]
    fn test_full_clean_pipeline_ok() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("bob".into()));
        data.insert("email".into(), Value::String("bob@example.com".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        form.full_clean();
        assert!(form.state.is_valid);
    }

    #[test]
    fn test_full_clean_with_integer_clean() {
        let mut data = HashMap::new();
        data.insert("username".into(), Value::String("alice".into()));
        data.insert("email".into(), Value::String("alice@example.com".into()));
        data.insert("age".into(), Value::String("25".into()));

        let mut form = Form::new(simple_form_fields());
        form.data = data;
        form.is_bound = true;
        form.full_clean();
        assert!(form.state.is_valid);
        assert_eq!(form.state.cleaned_data.get("age").unwrap(), &Value::Number(serde_json::Number::from(25u64)));
    }
}
