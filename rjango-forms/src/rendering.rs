/// Form rendering utilities — renders form as HTML table, list, or divs.

use std::collections::HashMap;
use crate::FormState;
use crate::fields::FormField;

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
