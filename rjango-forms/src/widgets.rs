/// Widget types for form field rendering.

#[derive(Debug, Clone)]
pub enum WidgetType {
    TextInput,
    EmailInput,
    PasswordInput,
    NumberInput,
    Textarea,
    Select,
    CheckboxInput,
    RadioSelect,
    DateInput,
    DateTimeInput,
    HiddenInput,
    FileInput,
}

/// Render a widget to HTML.
pub fn render_widget(widget: &WidgetType, name: &str, value: &str, _help_text: &str) -> String {
    let escaped = value.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;");
    match widget {
        WidgetType::TextInput => format!("<input type=\"text\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped),
        WidgetType::EmailInput => format!("<input type=\"email\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped),
        WidgetType::PasswordInput => format!("<input type=\"password\" name=\"{}\" class=\"form-control\">", name),
        WidgetType::NumberInput => format!("<input type=\"number\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped),
        WidgetType::Textarea => format!("<textarea name=\"{}\" class=\"form-control\" rows=\"4\">{}</textarea>", name, escaped),
        WidgetType::Select => format!("<select name=\"{}\" class=\"form-control\"><option value=\"{}\">{}</option></select>", name, escaped, escaped),
        WidgetType::CheckboxInput => format!("<input type=\"checkbox\" name=\"{}\" value=\"true\" {} class=\"form-check-input\">", name, if value == "true" { "checked" } else { "" }),
        WidgetType::RadioSelect => format!("<input type=\"radio\" name=\"{}\" value=\"{}\" class=\"form-check-input\">", name, escaped),
        WidgetType::DateInput => format!("<input type=\"date\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped),
        WidgetType::DateTimeInput => format!("<input type=\"datetime-local\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped),
        WidgetType::HiddenInput => format!("<input type=\"hidden\" name=\"{}\" value=\"{}\">", name, escaped),
        WidgetType::FileInput => format!("<input type=\"file\" name=\"{}\" class=\"form-control-file\">", name),
    }
}
