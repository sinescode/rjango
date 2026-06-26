/// Widget types for form field rendering.
/// Mirrors Django's `django.forms.widgets`.

#[derive(Debug, Clone)]
pub enum WidgetType {
    TextInput,
    EmailInput,
    PasswordInput,
    NumberInput,
    URLInput,
    Textarea,
    Select(Vec<(String, String)>),   // choices
    SelectMultiple(Vec<(String, String)>),
    CheckboxInput,
    CheckboxSelectMultiple(Vec<(String, String)>),
    RadioSelect(Vec<(String, String)>),
    NullBooleanSelect,
    DateInput,
    DateTimeInput,
    TimeInput,
    HiddenInput,
    FileInput,
    ClearableFileInput,
    SplitDateTimeInput,
    SelectDateWidget,
    MultipleHiddenInput,
}

impl WidgetType {
    /// Default choices iterator for select-type widgets.
    fn choices(&self) -> &[(String, String)] {
        match self {
            WidgetType::Select(c)
            | WidgetType::SelectMultiple(c)
            | WidgetType::CheckboxSelectMultiple(c)
            | WidgetType::RadioSelect(c) => c,
            _ => &[],
        }
    }

    /// Render HTML choices into options.
    fn render_choices(&self, _name: &str, value: &str) -> String {
        let choices = self.choices();
        if choices.is_empty() {
            return String::new();
        }
        let mut out = String::new();
        for (val, label) in choices {
            let selected = if val == value { " selected" } else { "" };
            out.push_str(&format!("<option value=\"{}\"{}>{}</option>\n", val, selected, label));
        }
        out
    }
}

/// Render a widget to HTML.
pub fn render_widget(widget: &WidgetType, name: &str, value: &str, _help_text: &str) -> String {
    let escaped = value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    match widget {
        WidgetType::TextInput => {
            format!("<input type=\"text\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped)
        }
        WidgetType::EmailInput => {
            format!("<input type=\"email\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped)
        }
        WidgetType::PasswordInput => {
            format!("<input type=\"password\" name=\"{}\" class=\"form-control\">", name)
        }
        WidgetType::NumberInput => {
            format!("<input type=\"number\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped)
        }
        WidgetType::URLInput => {
            format!("<input type=\"url\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped)
        }
        WidgetType::Textarea => {
            format!("<textarea name=\"{}\" class=\"form-control\" rows=\"4\">{}</textarea>", name, escaped)
        }
        WidgetType::Select(_) => {
            let options = widget.render_choices(name, value);
            format!("<select name=\"{}\" class=\"form-control\">\n{}</select>", name, options)
        }
        WidgetType::SelectMultiple(_) => {
            let options = widget.render_choices(name, "");
            format!("<select name=\"{}\" multiple class=\"form-control\">\n{}</select>", name, options)
        }
        WidgetType::CheckboxInput => {
            format!("<input type=\"checkbox\" name=\"{}\" value=\"true\" {} class=\"form-check-input\">",
                name, if value == "true" { "checked" } else { "" })
        }
        WidgetType::CheckboxSelectMultiple(_) => {
            let mut out = String::from("<ul class=\"checkbox-select-multiple\">\n");
            for (val, label) in widget.choices() {
                let checked = if value == val || value.contains(val) { " checked" } else { "" };
                out.push_str(&format!(
                    "<li><label><input type=\"checkbox\" name=\"{}\" value=\"{}\"{}> {}</label></li>\n",
                    name, val, checked, label
                ));
            }
            out.push_str("</ul>");
            out
        }
        WidgetType::RadioSelect(_) => {
            let mut out = String::from("<ul class=\"radio-select\">\n");
            for (val, label) in widget.choices() {
                let checked = if val == value { " checked" } else { "" };
                out.push_str(&format!(
                    "<li><label><input type=\"radio\" name=\"{}\" value=\"{}\"{}> {}</label></li>\n",
                    name, val, checked, label
                ));
            }
            out.push_str("</ul>");
            out
        }
        WidgetType::NullBooleanSelect => {
            format!(
                "<select name=\"{}\" class=\"form-control\">\n\
                 <option value=\"unknown\"{}>Unknown</option>\n\
                 <option value=\"true\"{}>Yes</option>\n\
                 <option value=\"false\"{}>No</option>\n\
                 </select>",
                name,
                if value.is_empty() || value == "unknown" { " selected" } else { "" },
                if value == "true" { " selected" } else { "" },
                if value == "false" { " selected" } else { "" }
            )
        }
        WidgetType::DateInput => {
            format!("<input type=\"date\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped)
        }
        WidgetType::DateTimeInput => {
            format!("<input type=\"datetime-local\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped)
        }
        WidgetType::TimeInput => {
            format!("<input type=\"time\" name=\"{}\" value=\"{}\" class=\"form-control\">", name, escaped)
        }
        WidgetType::HiddenInput => {
            format!("<input type=\"hidden\" name=\"{}\" value=\"{}\">", name, escaped)
        }
        WidgetType::FileInput => {
            format!("<input type=\"file\" name=\"{}\" class=\"form-control-file\">", name)
        }
        WidgetType::ClearableFileInput => {
            format!(
                "<div class=\"clearable-file-input\">\n\
                 <input type=\"file\" name=\"{}\" class=\"form-control-file\">\n\
                 <label><input type=\"checkbox\" name=\"{}-clear\"> Clear</label>\n\
                 </div>",
                name, name
            )
        }
        WidgetType::SplitDateTimeInput => {
            format!(
                "<div class=\"split-date-time\">\n\
                 <input type=\"date\" name=\"{}\" value=\"{}\" class=\"form-control\" style=\"display:inline-block;width:auto\">\n\
                 <input type=\"time\" name=\"{}\" value=\"{}\" class=\"form-control\" style=\"display:inline-block;width:auto\">\n\
                 </div>",
                name, escaped, name, escaped
            )
        }
        WidgetType::SelectDateWidget => {
            format!(
                "<div class=\"select-date\">\n\
                 <select name=\"{}\"><option value=\"\">Year</option></select>\n\
                 <select name=\"{}\"><option value=\"\">Month</option></select>\n\
                 <select name=\"{}\"><option value=\"\">Day</option></select>\n\
                 </div>",
                name, name, name
            )
        }
        WidgetType::MultipleHiddenInput => {
            format!(
                "<input type=\"hidden\" name=\"{}\" value=\"{}\">\n\
                 <input type=\"hidden\" name=\"{}_0\" value=\"\">\n\
                 <input type=\"hidden\" name=\"{}_1\" value=\"\">",
                name, escaped, name, name
            )
        }
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
    fn test_url_input() {
        let html = render_widget(&WidgetType::URLInput, "url", "https://x.com", "");
        assert!(html.contains(r#"type="url""#));
    }

    #[test]
    fn test_password_input() {
        let html = render_widget(&WidgetType::PasswordInput, "pass", "", "");
        assert!(html.contains(r#"type="password""#));
        assert!(!html.contains(r#"value="#));
    }

    #[test]
    fn test_select_with_choices() {
        let choices = vec![("1".into(), "One".into()), ("2".into(), "Two".into())];
        let html = render_widget(&WidgetType::Select(choices), "num", "1", "");
        assert!(html.contains("<select"));
        assert!(html.contains("selected")); // value "1" matches first choice
        assert!(html.contains("One"));
    }

    #[test]
    fn test_select_multiple() {
        let choices = vec![("a".into(), "A".into()), ("b".into(), "B".into())];
        let html = render_widget(&WidgetType::SelectMultiple(choices), "letters", "", "");
        assert!(html.contains("multiple"));
        assert!(html.contains("<select"));
    }

    #[test]
    fn test_checkbox_select_multiple() {
        let choices = vec![("x".into(), "X".into())];
        let html = render_widget(&WidgetType::CheckboxSelectMultiple(choices), "xs", "x", "");
        assert!(html.contains("checkbox"));
        assert!(html.contains("checked"));
        assert!(html.contains("<ul"));
    }

    #[test]
    fn test_radio_select() {
        let choices = vec![("yes".into(), "Yes".into()), ("no".into(), "No".into())];
        let html = render_widget(&WidgetType::RadioSelect(choices), "choice", "yes", "");
        assert!(html.contains("radio"));
        assert!(html.contains("checked"));
        assert!(html.contains("<ul"));
    }

    #[test]
    fn test_null_boolean_select() {
        let html = render_widget(&WidgetType::NullBooleanSelect, "flag", "", "");
        assert!(html.contains("Unknown"));
        assert!(html.contains("Yes"));
        assert!(html.contains("No"));
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
    fn test_time_input() {
        let html = render_widget(&WidgetType::TimeInput, "t", "14:30", "");
        assert!(html.contains(r#"type="time""#));
    }

    #[test]
    fn test_clearable_file_input() {
        let html = render_widget(&WidgetType::ClearableFileInput, "doc", "", "");
        assert!(html.contains("file"));
        assert!(html.contains("Clear"));
        assert!(html.contains("-clear"));
    }

    #[test]
    fn test_split_date_time_input() {
        let html = render_widget(&WidgetType::SplitDateTimeInput, "dt", "", "");
        assert!(html.contains("type=\"date\""));
        assert!(html.contains("type=\"time\""));
    }

    #[test]
    fn test_select_date_widget() {
        let html = render_widget(&WidgetType::SelectDateWidget, "d", "", "");
        assert!(html.contains("Year"));
        assert!(html.contains("Month"));
        assert!(html.contains("Day"));
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
    fn test_all_widgets_render_without_panic() {
        let all: Vec<WidgetType> = vec![
            WidgetType::TextInput,
            WidgetType::EmailInput,
            WidgetType::PasswordInput,
            WidgetType::NumberInput,
            WidgetType::URLInput,
            WidgetType::Textarea,
            WidgetType::CheckboxInput,
            WidgetType::DateInput,
            WidgetType::DateTimeInput,
            WidgetType::TimeInput,
            WidgetType::HiddenInput,
            WidgetType::FileInput,
            WidgetType::ClearableFileInput,
            WidgetType::SplitDateTimeInput,
            WidgetType::SelectDateWidget,
            WidgetType::NullBooleanSelect,
            WidgetType::Select(vec![]),
            WidgetType::SelectMultiple(vec![]),
            WidgetType::CheckboxSelectMultiple(vec![]),
            WidgetType::RadioSelect(vec![]),
        ];
        for w in &all {
            let html = render_widget(w, "test", "val", "help");
            assert!(!html.is_empty(), "Widget {:?} returned empty", w);
        }
    }

    #[test]
    fn test_multiple_hidden_input() {
        let html = render_widget(&WidgetType::MultipleHiddenInput, "tags", "a,b,c", "");
        assert!(html.contains("type=\"hidden\""));
        assert!(html.contains("name=\"tags\""));
        assert!(html.contains("value=\"a,b,c\""));
    }
}
