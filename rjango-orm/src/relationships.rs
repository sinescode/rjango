/// Relationship types for ORM.

/// A foreign key relationship.
pub struct ForeignKey {
    pub field_name: String,
    pub to_model: String,
    pub to_field: String,
    pub on_delete: &'static str,
    pub related_name: Option<String>,
    pub related_query_name: Option<String>,
    pub db_constraint: bool,
}

impl ForeignKey {
    pub fn new(field_name: &str, to_model: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            to_model: to_model.to_string(),
            to_field: "id".into(),
            on_delete: "CASCADE",
            related_name: None,
            related_query_name: None,
            db_constraint: true,
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

    pub fn to_field(mut self, field: &str) -> Self {
        self.to_field = field.to_string();
        self
    }

    pub fn related_query_name(mut self, name: &str) -> Self {
        self.related_query_name = Some(name.to_string());
        self
    }

    pub fn db_constraint(mut self, enabled: bool) -> Self {
        self.db_constraint = enabled;
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

/// OneToOneField — alias for ForeignKey with unique=true.
/// Mirrors Django's `OneToOneField` which is conceptually a ForeignKey with a unique constraint.
pub struct OneToOneField {
    pub field_name: String,
    pub to_model: String,
    pub to_field: String,
    pub on_delete: &'static str,
    pub related_name: Option<String>,
    pub related_query_name: Option<String>,
    pub db_constraint: bool,
}

impl OneToOneField {
    pub fn new(field_name: &str, to_model: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            to_model: to_model.to_string(),
            to_field: "id".into(),
            on_delete: "CASCADE",
            related_name: None,
            related_query_name: None,
            db_constraint: true,
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

    pub fn to_field(mut self, field: &str) -> Self {
        self.to_field = field.to_string();
        self
    }

    pub fn related_query_name(mut self, name: &str) -> Self {
        self.related_query_name = Some(name.to_string());
        self
    }

    pub fn db_constraint(mut self, enabled: bool) -> Self {
        self.db_constraint = enabled;
        self
    }
}

/// A many-to-many relationship.
pub struct ManyToMany {
    pub field_name: String,
    pub to_model: String,
    pub through: Option<String>,
    pub db_table: Option<String>,
}

impl ManyToMany {
    pub fn new(field_name: &str, to_model: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            to_model: to_model.to_string(),
            through: None,
            db_table: None,
        }
    }

    pub fn through(mut self, table: &str) -> Self {
        self.through = Some(table.to_string());
        self
    }

    pub fn db_table(mut self, table: &str) -> Self {
        self.db_table = Some(table.to_string());
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
        assert!(fk.related_query_name.is_none());
        assert!(fk.db_constraint);
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
    fn test_foreign_key_to_field() {
        let fk = ForeignKey::new("author", "User").to_field("username");
        assert_eq!(fk.to_field, "username");
    }

    #[test]
    fn test_foreign_key_related_query_name() {
        let fk = ForeignKey::new("author", "User").related_query_name("authors");
        assert_eq!(fk.related_query_name, Some("authors".to_string()));
        assert!(fk.db_constraint);
    }

    #[test]
    fn test_foreign_key_db_constraint() {
        let fk = ForeignKey::new("author", "User").db_constraint(false);
        assert!(!fk.db_constraint);
    }

    #[test]
    fn test_foreign_key_all_new_fields() {
        let fk = ForeignKey::new("author", "User")
            .to_field("username")
            .related_query_name("authors")
            .db_constraint(false)
            .on_delete("SET_NULL")
            .related_name("articles");
        assert_eq!(fk.to_field, "username");
        assert_eq!(fk.related_query_name, Some("authors".to_string()));
        assert!(!fk.db_constraint);
        assert_eq!(fk.on_delete, "SET_NULL");
        assert_eq!(fk.related_name, Some("articles".to_string()));
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

    // OneToOneField tests

    #[test]
    fn test_one_to_one_field_new() {
        let o2of = OneToOneField::new("profile", "Profile");
        assert_eq!(o2of.field_name, "profile");
        assert_eq!(o2of.to_model, "Profile");
        assert_eq!(o2of.to_field, "id");
        assert_eq!(o2of.on_delete, "CASCADE");
        assert!(o2of.related_name.is_none());
        assert!(o2of.related_query_name.is_none());
        assert!(o2of.db_constraint);
    }

    #[test]
    fn test_one_to_one_field_builders() {
        let o2of = OneToOneField::new("profile", "Profile")
            .to_field("uid")
            .related_name("user_profile")
            .related_query_name("profiles")
            .db_constraint(false);
        assert_eq!(o2of.to_field, "uid");
        assert_eq!(o2of.related_name, Some("user_profile".to_string()));
        assert_eq!(o2of.related_query_name, Some("profiles".to_string()));
        assert!(!o2of.db_constraint);
    }

    #[test]
    fn test_one_to_one_field_on_delete() {
        let o2of = OneToOneField::new("profile", "Profile").on_delete("SET_NULL");
        assert_eq!(o2of.on_delete, "SET_NULL");
    }

    // ManyToMany tests

    #[test]
    fn test_many_to_many_new() {
        let m2m = ManyToMany::new("groups", "Group");
        assert_eq!(m2m.field_name, "groups");
        assert_eq!(m2m.to_model, "Group");
        assert!(m2m.through.is_none());
        assert!(m2m.db_table.is_none());
    }

    #[test]
    fn test_many_to_many_through() {
        let m2m = ManyToMany::new("categories", "Category").through("article_categories");
        assert_eq!(m2m.through, Some("article_categories".to_string()));
        assert_eq!(m2m.field_name, "categories");
        assert_eq!(m2m.to_model, "Category");
    }

    #[test]
    fn test_many_to_many_db_table() {
        let m2m = ManyToMany::new("tags", "Tag").db_table("myapp_tag_m2m");
        assert_eq!(m2m.db_table, Some("myapp_tag_m2m".to_string()));
        assert!(m2m.through.is_none());
    }

    #[test]
    fn test_many_to_many_through_and_db_table() {
        let m2m = ManyToMany::new("tags", "Tag")
            .through("myapp_article_tags")
            .db_table("myapp_article_tags");
        assert_eq!(m2m.through, Some("myapp_article_tags".to_string()));
        assert_eq!(m2m.db_table, Some("myapp_article_tags".to_string()));
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
        assert!(m2m.db_table.is_none());
    }

    #[test]
    fn test_many_to_many_through_empty() {
        let m2m = ManyToMany::new("tags", "Tag").through("");
        assert_eq!(m2m.through, Some("".to_string()));
    }

    #[test]
    fn test_many_to_many_db_table_empty() {
        let m2m = ManyToMany::new("tags", "Tag").db_table("");
        assert_eq!(m2m.db_table, Some("".to_string()));
    }
}
