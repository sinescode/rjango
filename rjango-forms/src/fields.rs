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
            FieldType::Choice(_) => super::widgets::WidgetType::Select,
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

        // Validate
        for v in &self.validators {
            v.validate(value)?;
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
