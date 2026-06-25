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

#[cfg(test)]
mod tests {
    use super::*;

    // ForeignKey tests

    #[test]
    fn test_foreign_key_new() {
        let fk = ForeignKey::new("author", "User");
        assert_eq!(fk.field_name, "author");
        assert_eq!(fk.to_model, "User");
        assert_eq!(fk.to_field, "id");
        assert_eq!(fk.on_delete, "CASCADE");
        assert!(fk.related_name.is_none());
    }

    #[test]
    fn test_foreign_key_on_delete() {
        let fk = ForeignKey::new("author", "User").on_delete("SET_NULL");
        assert_eq!(fk.on_delete, "SET_NULL");
        assert_eq!(fk.field_name, "author");
        assert_eq!(fk.to_model, "User");
        assert_eq!(fk.to_field, "id");
    }

    #[test]
    fn test_foreign_key_on_delete_protect() {
        let fk = ForeignKey::new("owner", "Team").on_delete("PROTECT");
        assert_eq!(fk.on_delete, "PROTECT");
    }

    #[test]
    fn test_foreign_key_related_name() {
        let fk = ForeignKey::new("author", "User").related_name("articles");
        assert_eq!(fk.related_name, Some("articles".to_string()));
        assert_eq!(fk.to_field, "id");
        assert_eq!(fk.on_delete, "CASCADE");
    }

    #[test]
    fn test_foreign_key_related_name_and_on_delete() {
        let fk = ForeignKey::new("category", "Category")
            .on_delete("CASCADE")
            .related_name("articles");
        assert_eq!(fk.on_delete, "CASCADE");
        assert_eq!(fk.related_name, Some("articles".to_string()));
    }

    #[test]
    fn test_foreign_key_chain_order() {
        // Builder should work regardless of method order
        let fk = ForeignKey::new("editor", "User")
            .related_name("edited_articles")
            .on_delete("SET_DEFAULT");
        assert_eq!(fk.on_delete, "SET_DEFAULT");
        assert_eq!(fk.related_name, Some("edited_articles".to_string()));
        assert_eq!(fk.to_field, "id");
    }

    #[test]
    fn test_foreign_key_different_to_field() {
        // Default to_field is "id", builder doesn't override it
        let fk = ForeignKey::new("comment", "Comment");
        assert_eq!(fk.to_field, "id");
    }

    #[test]
    fn test_foreign_key_empty_strings() {
        let fk = ForeignKey::new("", "");
        assert_eq!(fk.field_name, "");
        assert_eq!(fk.to_model, "");
    }

    // OneToOne tests

    #[test]
    fn test_one_to_one_new() {
        let o2o = OneToOne::new("profile", "Profile");
        assert_eq!(o2o.field_name, "profile");
        assert_eq!(o2o.to_model, "Profile");
        assert_eq!(o2o.on_delete, "CASCADE");
    }

    #[test]
    fn test_one_to_one_defaults() {
        let o2o = OneToOne::new("user", "User");
        assert_eq!(o2o.field_name, "user");
        assert_eq!(o2o.to_model, "User");
        // on_delete is not chainable on OneToOne (no builder methods)
        assert_eq!(o2o.on_delete, "CASCADE");
    }

    #[test]
    fn test_one_to_one_empty_strings() {
        let o2o = OneToOne::new("", "");
        assert_eq!(o2o.field_name, "");
        assert_eq!(o2o.to_model, "");
    }

    // ManyToMany tests

    #[test]
    fn test_many_to_many_new() {
        let m2m = ManyToMany::new("groups", "Group");
        assert_eq!(m2m.field_name, "groups");
        assert_eq!(m2m.to_model, "Group");
        assert!(m2m.through.is_none());
    }

    #[test]
    fn test_many_to_many_through() {
        let m2m = ManyToMany::new("categories", "Category").through("article_categories");
        assert_eq!(m2m.through, Some("article_categories".to_string()));
        assert_eq!(m2m.field_name, "categories");
        assert_eq!(m2m.to_model, "Category");
    }

    #[test]
    fn test_many_to_many_with_special_chars() {
        let m2m = ManyToMany::new("my-tags", "Tag Model").through("my_app.article_tags");
        assert_eq!(m2m.field_name, "my-tags");
        assert_eq!(m2m.to_model, "Tag Model");
        assert_eq!(m2m.through, Some("my_app.article_tags".to_string()));
    }

    #[test]
    fn test_many_to_many_empty_strings() {
        let m2m = ManyToMany::new("", "");
        assert_eq!(m2m.field_name, "");
        assert_eq!(m2m.to_model, "");
        assert!(m2m.through.is_none());
    }

    #[test]
    fn test_many_to_many_through_empty() {
        let m2m = ManyToMany::new("tags", "Tag").through("");
        assert_eq!(m2m.through, Some("".to_string()));
    }
}
