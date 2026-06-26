/// Form rendering utilities — renders form as HTML table, list, or divs.

use std::collections::HashMap;
use crate::fields::FormField;
use crate::FormState;

use serde_json::Value;

/// Render a form as an HTML table (like Django's as_table).
pub fn as_table(fields: &[FormField], state: &FormState, data: &HashMap<String, Value>) -> String {
    let mut html = String::from("<table class=\"form-table\">\n");
    for field in fields {
        let value = data.get(&field.name);
        let error = state.field_error(&field.name)
            .map(|e| format!("<ul class=\"errorlist\">{}</ul>", e.iter().map(|m| format!("<li>{}</li>", m)).collect::<Vec<_>>().join("")))
            .unwrap_or_default();
        html.push_str(&format!(
            "  <tr><th><label for=\"{}\">{}{}:</label></th><td>{} {}<span class=\"helptext\">{}</span></td></tr>\n",
            field.name,
            field.label,
            if field.required { " *" } else { "" },
            field.render(value),
            error,
            field.help_text,
        ));
    }
    html.push_str("</table>");
    html
}

/// Render a form as a list of paragraphs (like Django's as_p).
pub fn as_p(fields: &[FormField], state: &FormState, data: &HashMap<String, Value>) -> String {
    let mut html = String::new();
    for field in fields {
        let value = data.get(&field.name);
        let error = state.field_error(&field.name)
            .map(|e| format!("<span class=\"error\">{}</span>", e.join(", ")))
            .unwrap_or_default();
        html.push_str(&format!(
            "<p><label for=\"{}\">{}{}:</label> {} {}<br><span class=\"helptext\">{}</span></p>\n",
            field.name,
            field.label,
            if field.required { " *" } else { "" },
            field.render(value),
            error,
            field.help_text,
        ));
    }
    html
}

/// Render a form as divs (like Django's as_div).
pub fn as_div(fields: &[FormField], state: &FormState, data: &HashMap<String, Value>) -> String {
    let mut html = String::new();
    for field in fields {
        let value = data.get(&field.name);
        let error = state.field_error(&field.name)
            .map(|e| format!("<div class=\"invalid-feedback\">{}</div>", e.join(", ")))
            .unwrap_or_default();
        html.push_str(&format!(
            "<div class=\"mb-3\"><label class=\"form-label\" for=\"{}\">{}{}:</label> {} {}</div>\n",
            field.name,
            field.label,
            if field.required { " *" } else { "" },
            field.render(value),
            error,
        ));
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::{FormField, FieldType};

    fn make_fields() -> Vec<FormField> {
        vec![
            FormField::new("name", FieldType::Char)
                .label("Name")
                .help("Your full name"),
            FormField::new("email", FieldType::Email)
                .label("Email"),
        ]
    }

    fn make_data() -> HashMap<String, Value> {
        let mut data = HashMap::new();
        data.insert("name".into(), Value::String("Alice".into()));
        data.insert("email".into(), Value::String("alice@example.com".into()));
        data
    }

    #[test]
    fn test_as_table() {
        let fields = make_fields();
        let state = crate::FormState::valid();
        let data = make_data();
        let html = as_table(&fields, &state, &data);
        assert!(html.contains("<table"));
        assert!(html.contains("Name"));
        assert!(html.contains("Email"));
        assert!(html.contains("Alice"));
        assert!(html.contains("</table>"));
    }

    #[test]
    fn test_as_table_with_errors() {
        let fields = make_fields();
        let mut errors = HashMap::new();
        errors.insert("name".into(), vec!["Required".into()]);
        let state = crate::FormState::invalid(errors);
        let data = make_data();
        let html = as_table(&fields, &state, &data);
        assert!(html.contains("errorlist"));
        assert!(html.contains("Required"));
    }

    #[test]
    fn test_as_p() {
        let fields = make_fields();
        let state = crate::FormState::valid();
        let data = make_data();
        let html = as_p(&fields, &state, &data);
        assert!(html.contains("<p>"));
        assert!(html.contains("Name"));
        assert!(html.contains("Alice"));
    }

    #[test]
    fn test_as_p_no_data() {
        let fields = make_fields();
        let state = crate::FormState::valid();
        let data = HashMap::new();
        let html = as_p(&fields, &state, &data);
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_as_div() {
        let fields = make_fields();
        let state = crate::FormState::valid();
        let data = make_data();
        let html = as_div(&fields, &state, &data);
        assert!(html.contains("mb-3"));
        assert!(html.contains("form-label"));
        assert!(html.contains("Alice"));
    }

    #[test]
    fn test_as_div_with_errors() {
        let fields = make_fields();
        let mut errors = HashMap::new();
        errors.insert("email".into(), vec!["Invalid email".into()]);
        let state = crate::FormState::invalid(errors);
        let data = make_data();
        let html = as_div(&fields, &state, &data);
        assert!(html.contains("invalid-feedback"));
        assert!(html.contains("Invalid email"));
    }

    #[test]
    fn test_as_table_required_marker() {
        let fields = vec![
            FormField::new("opt", FieldType::Char).required(false),
        ];
        let state = crate::FormState::valid();
        let data = HashMap::new();
        let html = as_table(&fields, &state, &data);
        assert!(!html.contains("opt *"));
    }

    #[test]
    fn test_empty_fields() {
        let html = as_table(&[], &crate::FormState::valid(), &HashMap::new());
        assert_eq!(html, "<table class=\"form-table\">\n</table>");
    }
}
