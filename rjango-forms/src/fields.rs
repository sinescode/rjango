//! Field types for forms — CharField, EmailField, IntegerField, etc.
//! Like Django's `django.forms.fields`.

use serde_json::Value;
use rjango_core::validators::Validator;

/// A single form field definition.
#[derive(Clone)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub initial: Option<Value>,
    pub help_text: String,
    pub widget: super::widgets::WidgetType,
    pub validators: Vec<Validator>,
}

impl std::fmt::Debug for FormField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormField")
            .field("name", &self.name)
            .field("label", &self.label)
            .field("field_type", &self.field_type)
            .field("required", &self.required)
            .field("help_text", &self.help_text)
            .field("widget", &self.widget)
            .finish()
    }
}

impl FormField {
    pub fn new(name: &str, field_type: FieldType) -> Self {
        let widget = match &field_type {
            FieldType::Char => super::widgets::WidgetType::TextInput,
            FieldType::Email => super::widgets::WidgetType::EmailInput,
            FieldType::Password => super::widgets::WidgetType::PasswordInput,
            FieldType::Integer => super::widgets::WidgetType::NumberInput,
            FieldType::Float => super::widgets::WidgetType::NumberInput,
            FieldType::Boolean => super::widgets::WidgetType::CheckboxInput,
            FieldType::Choice(_) => super::widgets::WidgetType::Select(vec![]),
            FieldType::TextArea => super::widgets::WidgetType::Textarea,
            FieldType::Date => super::widgets::WidgetType::DateInput,
            FieldType::DateTime => super::widgets::WidgetType::DateTimeInput,
            FieldType::Hidden => super::widgets::WidgetType::HiddenInput,
            FieldType::File => super::widgets::WidgetType::FileInput,
        };
        Self {
            name: name.to_string(),
            label: name.replace('_', " "),
            field_type,
            required: true,
            initial: None,
            help_text: String::new(),
            widget,
            validators: vec![],
        }
    }

    pub fn label(mut self, label: &str) -> Self { self.label = label.to_string(); self }
    pub fn required(mut self, r: bool) -> Self { self.required = r; self }
    pub fn initial(mut self, val: Value) -> Self { self.initial = Some(val); self }
    pub fn help(mut self, text: &str) -> Self { self.help_text = text.to_string(); self }
    pub fn widget(mut self, w: super::widgets::WidgetType) -> Self { self.widget = w; self }
    pub fn validator(mut self, v: Validator) -> Self { self.validators.push(v); self }

    pub fn clean(&self, value: &Value) -> Result<Value, String> {
        if value.is_null() {
            if self.required {
                return Err(format!("{} is required.", self.label));
            }
            return Ok(self.initial.clone().unwrap_or(Value::Null));
        }

        // Type conversion
        match self.field_type {
            FieldType::Integer => {
                let s = value.as_str().unwrap_or("");
                s.parse::<i64>()
                    .map(|n| Value::Number(serde_json::Number::from(n as u64)))
                    .map_err(|_| format!("{} must be an integer.", self.label))
            }
            FieldType::Float => {
                let s = value.as_str().unwrap_or("");
                s.parse::<f64>()
                    .map(|n| serde_json::Number::from_f64(n).map(Value::Number).unwrap_or(Value::Null))
                    .map_err(|_| format!("{} must be a number.", self.label))
            }
            FieldType::Boolean => {
                match value.as_str().unwrap_or("").to_lowercase().as_str() {
                    "true" | "1" | "on" | "yes" => Ok(Value::Bool(true)),
                    "false" | "0" | "off" | "no" | "" => Ok(Value::Bool(false)),
                    _ => Err(format!("{} must be true or false.", self.label)),
                }
            }
            _ => Ok(value.clone()),
        }
    }

    pub fn render(&self, value: Option<&Value>) -> String {
        let val = value.map(|v| match v {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }).unwrap_or_default();
        super::widgets::render_widget(&self.widget, &self.name, &val, &self.help_text)
    }
}

/// Parse URL-encoded form data into a HashMap.
pub fn parse_form_data(query_str: &str) -> std::collections::HashMap<String, Value> {
    let mut data = std::collections::HashMap::new();
    for pair in query_str.split('&') {
        if let Some(pos) = pair.find('=') {
            let key = urlencoding::decode(&pair[..pos]).unwrap_or_default().to_string();
            let value = urlencoding::decode(&pair[pos + 1..]).unwrap_or_default().to_string();
            data.insert(key, Value::String(value));
        }
    }
    data
}

/// Types of form fields.
#[derive(Debug, Clone)]
pub enum FieldType {
    Char,
    Email,
    Password,
    Integer,
    Float,
    Boolean,
    Choice(Vec<(String, String)>),
    TextArea,
    Date,
    DateTime,
    Hidden,
    File,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_new() {
        let field = FormField::new("username", FieldType::Char);
        assert_eq!(field.name, "username");
        assert_eq!(field.label, "username");
        assert!(field.required);
    }

    #[test]
    fn test_form_field_builder() {
        let field = FormField::new("email", FieldType::Email)
            .label("Email address")
            .required(false)
            .help("Enter your email");
        assert_eq!(field.label, "Email address");
        assert!(!field.required);
        assert_eq!(field.help_text, "Enter your email");
    }

    #[test]
    fn test_clean_char() {
        let field = FormField::new("name", FieldType::Char);
        let val = Value::String("Alice".into());
        let result = field.clean(&val).unwrap();
        assert_eq!(result, val);
    }

    #[test]
    fn test_clean_integer() {
        let field = FormField::new("age", FieldType::Integer);
        let result = field.clean(&Value::String("42".into())).unwrap();
        assert_eq!(result, Value::Number(serde_json::Number::from(42u64)));
    }

    #[test]
    fn test_clean_integer_invalid() {
        let field = FormField::new("age", FieldType::Integer);
        let result = field.clean(&Value::String("abc".into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_boolean_true() {
        let field = FormField::new("agree", FieldType::Boolean);
        let result = field.clean(&Value::String("true".into())).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_clean_boolean_false() {
        let field = FormField::new("agree", FieldType::Boolean);
        let result = field.clean(&Value::String("false".into())).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_clean_null_required() {
        let field = FormField::new("name", FieldType::Char);
        let result = field.clean(&Value::Null);
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_null_optional() {
        let field = FormField::new("name", FieldType::Char).required(false);
        let result = field.clean(&Value::Null).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_render_field() {
        let field = FormField::new("name", FieldType::Char);
        let html = field.render(Some(&Value::String("Alice".into())));
        assert!(html.contains(r#"value="Alice""#));
    }

    #[test]
    fn test_parse_form_data() {
        let data = parse_form_data("name=Alice&age=30");
        assert_eq!(data.get("name").unwrap(), &Value::String("Alice".into()));
        assert_eq!(data.get("age").unwrap(), &Value::String("30".into()));
    }

    #[test]
    fn test_parse_form_data_empty() {
        let data = parse_form_data("");
        assert!(data.is_empty());
    }

    #[test]
    fn test_field_type_choice_uses_select_widget() {
        let field = FormField::new("color", FieldType::Choice(vec![
            ("R".into(), "Red".into()),
            ("B".into(), "Blue".into()),
        ]));
        assert!(matches!(field.widget, crate::widgets::WidgetType::Select(_)));
    }

    #[test]
    fn test_clean_float() {
        let field = FormField::new("price", FieldType::Float);
        let result = field.clean(&Value::String("3.14".into())).unwrap();
        assert_eq!(result.as_f64(), Some(3.14));
    }

    #[test]
    fn test_clean_boolean_from_numbers() {
        let field = FormField::new("flag", FieldType::Boolean);
        assert_eq!(field.clean(&Value::String("1".into())).unwrap(), Value::Bool(true));
        assert_eq!(field.clean(&Value::String("0".into())).unwrap(), Value::Bool(false));
    }
}
