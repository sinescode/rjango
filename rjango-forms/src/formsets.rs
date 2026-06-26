//! Formsets — manage multiple forms on a single page.
//! Mirrors Django's `django.forms.formsets`.

use crate::Form;
use crate::fields::{FormField, FieldType};

/// Management form data for a formset.
#[derive(Debug, Clone)]
pub struct ManagementForm {
    pub total_forms: usize,
    pub initial_forms: usize,
    pub min_num: usize,
    pub max_num: usize,
}

impl ManagementForm {
    pub fn new(total: usize, initial: usize, min: usize, max: usize) -> Self {
        Self { total_forms: total, initial_forms: initial, min_num: min, max_num: max }
    }
}

/// A single form within a formset.
#[derive(Debug, Clone)]
pub struct FormsetForm {
    pub form: Form,
    pub index: usize,
    pub can_delete: bool,
    pub can_order: bool,
}

/// Base formset — manages a collection of forms.
#[derive(Debug, Clone)]
pub struct BaseFormSet {
    pub forms: Vec<FormsetForm>,
    pub management_form: ManagementForm,
    pub can_delete: bool,
    pub can_order: bool,
    pub min_num: usize,
    pub max_num: usize,
    pub absolute_max: usize,
    pub errors: Vec<Vec<String>>,
    pub is_valid_cache: Option<bool>,
}

impl BaseFormSet {
    /// Create a new formset from a factory function.
    pub fn new(
        factory: impl Fn(usize) -> Vec<FormField>,
        count: usize,
        can_delete: bool,
        can_order: bool,
        min_num: usize,
        max_num: usize,
    ) -> Self {
        let forms: Vec<FormsetForm> = (0..count)
            .map(|i| FormsetForm {
                form: Form::new(factory(i)),
                index: i,
                can_delete,
                can_order,
            })
            .collect();

        let absolute_max = if max_num > 0 { max_num * 2 } else { 1000 };

        Self {
            forms,
            management_form: ManagementForm::new(count, count, min_num, max_num),
            can_delete,
            can_order,
            min_num,
            max_num,
            absolute_max,
            errors: vec![],
            is_valid_cache: None,
        }
    }

    /// Validate all forms in the formset.
    pub fn is_valid(&mut self) -> bool {
        let mut all_valid = true;
        self.errors.clear();

        let total = self.forms.len();
        if self.min_num > 0 && total < self.min_num {
            self.errors.push(vec![format!(
                "Please submit at least {} form(s).",
                self.min_num
            )]);
            all_valid = false;
        }
        if self.max_num > 0 && total > self.max_num {
            self.errors.push(vec![format!(
                "Please submit at most {} form(s).",
                self.max_num
            )]);
            all_valid = false;
        }

        for f in self.forms.iter_mut() {
            if !f.form.is_valid() {
                all_valid = false;
                let errs = f.form.errors();
                let mut form_errs = Vec::new();
                for (_field, msgs) in errs.iter() {
                    form_errs.extend(msgs.iter().cloned());
                }
                self.errors.push(form_errs);
            }
        }

        if self.forms.is_empty() {
            self.errors.push(vec!["No forms in formset.".to_string()]);
            all_valid = false;
        }

        self.is_valid_cache = Some(all_valid);
        all_valid
    }

    /// Get total form count.
    pub fn total_form_count(&self) -> usize {
        self.forms.len()
    }

    /// Get initial form count.
    pub fn initial_form_count(&self) -> usize {
        self.management_form.initial_forms
    }

    /// Get cleaned data from all forms.
    pub fn cleaned_data(&self) -> Vec<std::collections::HashMap<String, serde_json::Value>> {
        self.forms.iter().map(|f| f.form.cleaned_data().clone()).collect()
    }

    /// Render all forms.
    pub fn render_all(&self, submit_label: &str, action: &str, method: &str) -> String {
        let mut html = format!("<form action=\"{}\" method=\"{}\">\n", action, method);
        html.push_str(&Form::csrf_input());
        for f in &self.forms {
            html.push_str("<fieldset>\n");
            html.push_str(&f.form.render(submit_label, action, method));
            html.push_str("</fieldset>\n");
        }
        html.push_str("</form>\n");
        html
    }
}

/// Create a formset factory.
pub fn formset_factory(
    form_builder: fn(usize) -> Vec<FormField>,
    extra: usize,
    can_delete: bool,
    can_order: bool,
    min_num: usize,
    max_num: usize,
) -> impl Fn() -> BaseFormSet {
    move || BaseFormSet::new(form_builder, extra, can_delete, can_order, min_num, max_num)
}

/// Helper: create a simple name field.
pub fn name_fieldset(_index: usize) -> Vec<FormField> {
    vec![
        FormField::new("name", FieldType::Char)
            .required(true)
            .label("Name"),
        FormField::new("email", FieldType::Email)
            .required(true)
            .label("Email"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formset_new() {
        let mut fs = BaseFormSet::new(name_fieldset, 3, false, false, 0, 0);
        assert_eq!(fs.total_form_count(), 3);
        assert_eq!(fs.initial_form_count(), 3);
        assert!(!fs.can_delete);
        assert!(!fs.can_order);
        // Unbound forms with required fields are not valid by default
        // (same as Django — empty required fields fail validation)
        assert!(!fs.is_valid());
        assert_eq!(fs.errors.len(), 3); // 3 forms each with errors
    }

    #[test]
    fn test_formset_with_constraints() {
        let fs = BaseFormSet::new(name_fieldset, 2, false, false, 1, 10);
        assert_eq!(fs.min_num, 1);
        assert_eq!(fs.max_num, 10);
    }

    #[test]
    fn test_formset_is_valid_empty() {
        let mut fs = BaseFormSet::new(name_fieldset, 0, false, false, 0, 0);
        assert!(!fs.is_valid()); // No forms = invalid
    }

    #[test]
    fn test_formset_below_min() {
        let mut fs = BaseFormSet::new(name_fieldset, 0, false, false, 2, 10);
        assert!(!fs.is_valid());
    }

    #[test]
    fn test_formset_above_max() {
        let mut fs = BaseFormSet::new(name_fieldset, 15, false, false, 0, 10);
        assert!(!fs.is_valid());
    }

    #[test]
    fn test_formset_cleaned_data() {
        let fs = BaseFormSet::new(name_fieldset, 2, false, false, 0, 0);
        let data = fs.cleaned_data();
        assert_eq!(data.len(), 2);
    }

    #[test]
    fn test_formset_factory() {
        let factory = formset_factory(name_fieldset, 2, true, false, 0, 10);
        let mut fs = factory();
        assert_eq!(fs.total_form_count(), 2);
        assert!(fs.can_delete);
        assert!(!fs.can_order);
        assert!(!fs.is_valid()); // empty required fields
    }

    #[test]
    fn test_management_form_new() {
        let mf = ManagementForm::new(10, 3, 1, 50);
        assert_eq!(mf.total_forms, 10);
        assert_eq!(mf.initial_forms, 3);
    }

    #[test]
    fn test_formset_form_construction() {
        let ff = FormsetForm {
            form: Form::new(name_fieldset(0)),
            index: 0,
            can_delete: true,
            can_order: false,
        };
        assert_eq!(ff.index, 0);
        assert!(ff.can_delete);
    }

    #[test]
    fn test_render_all() {
        let fs = BaseFormSet::new(name_fieldset, 2, false, false, 0, 0);
        let html = fs.render_all("Submit", "/submit", "post");
        assert!(html.contains("<form"));
        assert!(html.contains("</form>"));
        assert!(html.contains("fieldset"));
    }

    #[test]
    fn test_name_fieldset() {
        let fields = name_fieldset(0);
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "name");
        assert_eq!(fields[1].name, "email");
    }

    #[test]
    fn test_clone() {
        let mut fs = BaseFormSet::new(name_fieldset, 2, false, false, 0, 0);
        let _cloned = fs.clone();
        // Empty required fields => not valid
        assert!(!fs.is_valid());
    }
}
