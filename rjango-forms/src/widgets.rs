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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input() {
        let html = render_widget(&WidgetType::TextInput, "name", "John", "");
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"name="name""#));
        assert!(html.contains(r#"value="John""#));
    }

    #[test]
    fn test_email_input() {
        let html = render_widget(&WidgetType::EmailInput, "email", "a@b.com", "");
        assert!(html.contains(r#"type="email""#));
    }

    #[test]
    fn test_password_input() {
        let html = render_widget(&WidgetType::PasswordInput, "pass", "", "");
        assert!(html.contains(r#"type="password""#));
        assert!(!html.contains(r#"value="#)); // passwords don't show value
    }

    #[test]
    fn test_checkbox_checked() {
        let html = render_widget(&WidgetType::CheckboxInput, "agree", "true", "");
        assert!(html.contains("checked"));
    }

    #[test]
    fn test_checkbox_unchecked() {
        let html = render_widget(&WidgetType::CheckboxInput, "agree", "false", "");
        assert!(!html.contains("checked"));
    }

    #[test]
    fn test_textarea() {
        let html = render_widget(&WidgetType::Textarea, "bio", "Hello", "");
        assert!(html.contains("<textarea"));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_hidden_input() {
        let html = render_widget(&WidgetType::HiddenInput, "csrf", "token", "");
        assert!(html.contains(r#"type="hidden""#));
    }

    #[test]
    fn test_file_input() {
        let html = render_widget(&WidgetType::FileInput, "file", "", "");
        assert!(html.contains(r#"type="file""#));
    }

    #[test]
    fn test_escape_html_in_value() {
        let html = render_widget(&WidgetType::TextInput, "x", "<script>", "");
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_all_widgets_render() {
        let variants = vec![
            WidgetType::TextInput,
            WidgetType::EmailInput,
            WidgetType::PasswordInput,
            WidgetType::NumberInput,
            WidgetType::Textarea,
            WidgetType::Select,
            WidgetType::CheckboxInput,
            WidgetType::RadioSelect,
            WidgetType::DateInput,
            WidgetType::DateTimeInput,
            WidgetType::HiddenInput,
            WidgetType::FileInput,
        ];
        for w in &variants {
            let html = render_widget(w, "test", "val", "help");
            assert!(!html.is_empty());
        }
    }
}
