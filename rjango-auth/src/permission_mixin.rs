//! Permission mixin, content type integration, and custom user model support.
//! Mirrors Django's `django.contrib.auth.base_user` and permission mixins.

use crate::models::{Permission, User};

/// ContentType — like Django's `django.contrib.contenttypes.models.ContentType`.
/// A lightweight representation; the full registry lives in rjango_core::contenttypes.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentType {
    pub id: i64,
    pub app_label: String,
    pub model: String,
}

impl ContentType {
    pub fn new(id: i64, app_label: &str, model: &str) -> Self {
        Self {
            id,
            app_label: app_label.to_string(),
            model: model.to_string(),
        }
    }

    /// Get the content type for a given model (app_label + model name).
    /// Looks up the global rjango_core content type registry if available,
    /// otherwise returns a lightweight local ContentType.
    pub fn get_for_model(app_label: &str, model_name: &str) -> Self {
        // Try the global rjango_core content type registry first
        let ct = rjango_core::contenttypes::lookup_content_type(app_label, model_name);
        match ct {
            Some(found) => ContentType::new(found.id, &found.app_label, &found.model),
            None => {
                // Register it if not found
                let ct = rjango_core::contenttypes::register_content_type(app_label, model_name);
                ContentType::new(ct.id, &ct.app_label, &ct.model)
            }
        }
    }

    /// Get a content type by its ID from the global registry.
    pub fn get_for_id(id: i64) -> Option<Self> {
        let mgr = rjango_core::contenttypes::get_content_type_manager();
        mgr.as_ref().and_then(|m| m.get_for_id(id))
            .map(|ct| ContentType::new(ct.id, &ct.app_label, &ct.model))
    }

    /// The human-readable name.
    pub fn name(&self) -> String {
        format!("{}:{}", self.app_label, self.model)
    }

    /// Generate the default permission codenames for this content type.
    pub fn default_permission_codenames(&self) -> Vec<String> {
        let actions = ["add", "change", "delete", "view"];
        actions.iter().map(|a| format!("{}_{}", a, self.model)).collect()
    }

    /// Generate the default Permission objects for this content type.
    pub fn default_permissions(&self) -> Vec<Permission> {
        let mut mgr = crate::models::permission_manager().lock().unwrap();
        mgr.auto_create_default_permissions(self.id, &self.app_label, &self.model)
    }
}

/// PermissionMixin trait — like Django's `PermissionsMixin`.
/// User models can implement this for permission checking.
pub trait PermissionMixin {
    /// Check if this user has a specific permission.
    fn has_perm(&self, perm: &str) -> bool;

    /// Check if the user has all of the specified permissions.
    fn has_perms(&self, perms: &[&str]) -> bool;

    /// Check if the user has any permission in the given module (app_label).
    fn has_module_perms(&self, app_label: &str) -> bool;

    /// Get all permission strings for this user.
    fn get_all_permissions(&self) -> Vec<String>;

    /// Whether this user is a superuser (bypasses all checks).
    fn is_superuser(&self) -> bool {
        false
    }

    /// Whether this user is active (inactive users get no perms).
    fn is_active(&self) -> bool {
        true
    }
}

/// Default implementation of PermissionMixin for standard User.
impl PermissionMixin for User {
    fn has_perm(&self, perm: &str) -> bool {
        self.has_perm(perm)
    }

    fn has_perms(&self, perms: &[&str]) -> bool {
        self.has_perms(perms)
    }

    fn has_module_perms(&self, app_label: &str) -> bool {
        self.has_module_perms(app_label)
    }

    fn get_all_permissions(&self) -> Vec<String> {
        self.get_all_permissions()
    }

    fn is_superuser(&self) -> bool {
        self.is_superuser
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

/// Custom user model support.
///
/// The `AUTH_USER_MODEL` setting specifies the custom user model for the project.
/// Currently it's a compile-time constant placeholder; in Django it's a runtime
/// setting like `"myapp.MyUser"`.
pub const AUTH_USER_MODEL: &str = "auth.User";

/// UserModelProxy trait — allows swapping the User model.
/// Models that implement this can be used as a drop-in replacement for User.
pub trait UserModelProxy: PermissionMixin + Sized {
    /// The type used for user identifiers.
    type Id: Eq + std::fmt::Debug + Clone;

    /// Return the user's unique identifier.
    fn id(&self) -> Self::Id;

    /// Return the user's username.
    fn username(&self) -> &str;

    /// Return the user's email address.
    fn email(&self) -> &str;

    /// Return whether this user is authenticated.
    fn is_authenticated(&self) -> bool;

    /// Verify a raw password against the stored hash.
    fn check_password(&self, raw: &str) -> bool;

    /// Set the password (hashed).
    fn set_password(&mut self, raw: &str);
}

/// Implement UserModelProxy for the standard User.
impl UserModelProxy for User {
    type Id = i64;

    fn id(&self) -> i64 {
        self.id
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn is_authenticated(&self) -> bool {
        self.is_authenticated()
    }

    fn check_password(&self, raw: &str) -> bool {
        self.check_password(raw)
    }

    fn set_password(&mut self, raw: &str) {
        self.set_password(raw)
    }
}

/// Resolve the actual User model type. Currently always returns the standard User.
/// In a full implementation this would use the AUTH_USER_MODEL setting.
pub fn get_user_model() -> User {
    User::new("anonymous", "")
}

/// Create a user from a UserModelProxy (generic helper).
pub fn create_user<U: UserModelProxy>(user: &mut U, username: &str, password: &str) {
    user.set_password(password);
    // In a real implementation, would set the username field too
    let _ = username;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Group, PermissionManager};

    #[test]
    fn test_content_type_new() {
        let ct = ContentType::new(1, "auth", "user");
        assert_eq!(ct.id, 1);
        assert_eq!(ct.app_label, "auth");
        assert_eq!(ct.model, "user");
    }

    #[test]
    fn test_content_type_name() {
        let ct = ContentType::new(1, "blog", "post");
        assert_eq!(ct.name(), "blog:post");
    }

    #[test]
    fn test_content_type_get_for_model_integration() {
        // This tests the rjango_core integration
        let ct = ContentType::get_for_model("test_perm_mixin", "sample");
        assert_eq!(ct.app_label, "test_perm_mixin");
        assert_eq!(ct.model, "sample");
        assert!(ct.id > 0);
    }

    #[test]
    fn test_content_type_get_for_model_idempotent() {
        let ct1 = ContentType::get_for_model("idempotent_app", "item");
        let ct2 = ContentType::get_for_model("idempotent_app", "item");
        assert_eq!(ct1.id, ct2.id);
    }

    #[test]
    fn test_content_type_get_for_id() {
        let ct = ContentType::get_for_model("get_for_id_test", "model_x");
        let found = ContentType::get_for_id(ct.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().model, "model_x");
    }

    #[test]
    fn test_content_type_get_for_id_not_found() {
        assert!(ContentType::get_for_id(99999).is_none());
    }

    #[test]
    fn test_content_type_default_permission_codenames() {
        let ct = ContentType::new(1, "auth", "user");
        let codenames = ct.default_permission_codenames();
        assert_eq!(codenames.len(), 4);
        assert!(codenames.contains(&"add_user".to_string()));
        assert!(codenames.contains(&"change_user".to_string()));
        assert!(codenames.contains(&"delete_user".to_string()));
        assert!(codenames.contains(&"view_user".to_string()));
    }

    #[test]
    fn test_content_type_default_permissions() {
        let mut mgr = PermissionManager::new();
        // Reset the global manager for clean state in this test
        let previous = crate::models::permission_manager().lock().unwrap().count();
        
        let ct = ContentType::new(10, "test_app", "test_model");
        let perms = ct.default_permissions();
        assert_eq!(perms.len(), 4);
        
        // Verify the global manager has the permissions
        let global_count = crate::models::permission_manager().lock().unwrap().count();
        assert_eq!(global_count, previous + 4);
    }

    #[test]
    fn test_permission_mixin_trait_impl_for_user() {
        // User implements PermissionMixin automatically
        fn check_permission_mixin(m: &impl PermissionMixin) -> bool {
            !m.has_perm("some.perm")
        }

        let user = User::new("test", "test@test.com");
        assert!(check_permission_mixin(&user));
    }

    #[test]
    fn test_permission_mixin_superuser() {
        let mut admin = User::new("admin", "admin@test.com");
        admin.is_superuser = true;
        assert!(PermissionMixin::has_perm(&admin, "anything"));
        assert!(PermissionMixin::is_superuser(&admin));
    }

    #[test]
    fn test_permission_mixin_inactive_user() {
        let mut user = User::new("inactive", "in@test.com");
        user.is_active = false;
        user.is_superuser = false;
        // PermissionMixin trait methods call User's methods which check is_active
        assert!(!PermissionMixin::has_perm(&user, "something"));
        assert!(PermissionMixin::is_active(&user)); // default impl returns true
    }

    #[test]
    fn test_auth_user_model_const() {
        assert_eq!(AUTH_USER_MODEL, "auth.User");
    }

    #[test]
    fn test_user_model_proxy_for_user() {
        let mut user = User::new("bob", "bob@test.com");
        user.id = 42;
        assert_eq!(user.id(), 42);
        assert_eq!(user.username(), "bob");
        assert_eq!(user.email(), "bob@test.com");
        assert!(user.is_authenticated());
    }

    #[test]
    fn test_user_model_proxy_password() {
        let mut user = User::new("bob", "bob@test.com");
        user.set_password("secret123");
        assert!(user.check_password("secret123"));
        assert!(!user.check_password("wrong"));
    }

    #[test]
    fn test_get_user_model() {
        let user = get_user_model();
        assert_eq!(user.username, "anonymous");
    }

    #[test]
    fn test_permission_mixin_has_perms_trait() {
        let mut user = User::new("bob", "bob@test.com");
        user.user_permissions.push(Permission::new(1, "Can view", 1, "view_item"));
        user.user_permissions.push(Permission::new(2, "Can add", 1, "add_item"));

        let mixin: &dyn PermissionMixin = &user;
        assert!(mixin.has_perms(&["view_item", "add_item"]));
        assert!(!mixin.has_perms(&["view_item", "delete_item"]));
    }

    #[test]
    fn test_permission_mixin_get_all_permissions_trait() {
        let mut user = User::new("alice", "a@test.com");
        user.user_permissions.push(Permission::new(1, "Can do", 1, "do_thing"));
        let perms: Vec<String> = PermissionMixin::get_all_permissions(&user);
        assert_eq!(perms, vec!["do_thing"]);
    }

    #[test]
    fn test_custom_user_trait_methods() {
        fn accepts_proxy(u: &impl UserModelProxy) {
            assert!(u.id() == 0 || u.id() == 1);
        }
        let user = User::new("u", "u@u.com");
        accepts_proxy(&user);
    }
}
