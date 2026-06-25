/// Relationship types for ORM.

/// A foreign key relationship.
pub struct ForeignKey {
    pub field_name: String,
    pub to_model: String,
    pub to_field: String,
    pub on_delete: &'static str,
    pub related_name: Option<String>,
}

impl ForeignKey {
    pub fn new(field_name: &str, to_model: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            to_model: to_model.to_string(),
            to_field: "id".into(),
            on_delete: "CASCADE",
            related_name: None,
        }
    }

    pub fn on_delete(mut self, action: &'static str) -> Self {
        self.on_delete = action;
        self
    }

    pub fn related_name(mut self, name: &str) -> Self {
        self.related_name = Some(name.to_string());
        self
    }
}

/// A one-to-one relationship.
pub struct OneToOne {
    pub field_name: String,
    pub to_model: String,
    pub on_delete: &'static str,
}

impl OneToOne {
    pub fn new(field_name: &str, to_model: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            to_model: to_model.to_string(),
            on_delete: "CASCADE",
        }
    }
}

/// A many-to-many relationship.
pub struct ManyToMany {
    pub field_name: String,
    pub to_model: String,
    pub through: Option<String>,
}

impl ManyToMany {
    pub fn new(field_name: &str, to_model: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            to_model: to_model.to_string(),
            through: None,
        }
    }

    pub fn through(mut self, table: &str) -> Self {
        self.through = Some(table.to_string());
        self
    }
}
