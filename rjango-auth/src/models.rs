use std::collections::HashMap;
use std::sync::Mutex;

/// A single permission — like Django's Permission model.
#[derive(Debug, Clone, PartialEq)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub content_type_id: i64,
    pub codename: String,
}

impl Permission {
    pub fn new(id: i64, name: &str, content_type_id: i64, codename: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            content_type_id,
            codename: codename.to_string(),
        }
    }

    /// The natural key for this permission: "content_type_id:codename".
    pub fn natural_key(&self) -> String {
        format!("{}:{}", self.content_type_id, self.codename)
    }

    /// Check if this permission matches a Django-style perm string like "auth.add_user".
    /// This is a simpler check that doesn't resolve the app_label; use with care.
    pub fn matches(&self, perm_str: &str) -> bool {
        self.codename == perm_str || format!("{}:{}", self.content_type_id, self.codename) == perm_str
    }
}

/// Manager for Permission — like Django's PermissionManager.
#[derive(Debug)]
pub struct PermissionManager {
    permissions: HashMap<i64, Permission>,
    by_codename: HashMap<String, i64>,
    next_id: i64,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            by_codename: HashMap::new(),
            next_id: 1,
        }
    }

    /// Get a permission by codename.
    pub fn get_by_codename(&self, codename: &str) -> Option<&Permission> {
        self.by_codename.get(codename).and_then(|id| self.permissions.get(id))
    }

    /// Get all permissions.
    pub fn all(&self) -> Vec<&Permission> {
        self.permissions.values().collect()
    }

    /// Create a single permission.
    pub fn create(&mut self, name: &str, content_type_id: i64, codename: &str) -> Permission {
        if let Some(existing) = self.get_by_codename(codename) {
            return existing.clone();
        }
        let id = self.next_id;
        self.next_id += 1;
        let perm = Permission::new(id, name, content_type_id, codename);
        self.by_codename.insert(codename.to_string(), id);
        self.permissions.insert(id, perm.clone());
        perm
    }

    /// Bulk create permissions.
    pub fn bulk_create(&mut self, perms: Vec<(String, i64, String)>) -> Vec<Permission> {
        perms.into_iter().map(|(name, ct_id, codename)| {
            self.create(&name, ct_id, &codename)
        }).collect()
    }

    /// Auto-create default permissions (add, change, delete, view) for a given model.
    pub fn auto_create_default_permissions(&mut self, content_type_id: i64, _app_label: &str, model_name: &str) -> Vec<Permission> {
        let actions = ["add", "change", "delete", "view"];
        let human_names = [
            format!("Can add {}", model_name),
            format!("Can change {}", model_name),
            format!("Can delete {}", model_name),
            format!("Can view {}", model_name),
        ];
        actions.iter().enumerate().map(|(i, action)| {
            let codename = format!("{}_{}", action, model_name);
            self.create(&human_names[i], content_type_id, &codename)
        }).collect()
    }

    /// Get the total count of permissions.
    pub fn count(&self) -> usize {
        self.permissions.len()
    }

    /// Get multiple permissions by their codenames.
    pub fn filter_by_codenames(&self, codenames: &[&str]) -> Vec<&Permission> {
        codenames.iter().filter_map(|c| self.get_by_codename(c)).collect()
    }

    /// Clear all permissions (for testing).
    pub fn clear(&mut self) {
        self.permissions.clear();
        self.by_codename.clear();
        self.next_id = 1;
    }
}

/// A group — like Django's Group model.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub permissions: Vec<Permission>,
}

impl Group {
    pub fn new(id: i64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            permissions: Vec::new(),
        }
    }

    /// Add a permission to this group.
    pub fn add_permission(&mut self, perm: Permission) {
        if !self.permissions.iter().any(|p| p.id == perm.id) {
            self.permissions.push(perm);
        }
    }

    /// Remove a permission from this group.
    pub fn remove_permission(&mut self, perm_id: i64) {
        self.permissions.retain(|p| p.id != perm_id);
    }

    /// Check if the group has a specific permission.
    pub fn has_perm(&self, perm: &str) -> bool {
        self.permissions.iter().any(|p| p.codename == perm || format!("{}:{}", p.content_type_id, p.codename) == perm)
    }
}

/// Manager for Group — like Django's GroupManager.
#[derive(Debug)]
pub struct GroupManager {
    groups: HashMap<i64, Group>,
    by_name: HashMap<String, i64>,
    next_id: i64,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            by_name: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new group.
    pub fn create(&mut self, name: &str) -> Group {
        if let Some(id) = self.by_name.get(name) {
            return self.groups.get(id).unwrap().clone();
        }
        let id = self.next_id;
        self.next_id += 1;
        let group = Group::new(id, name);
        self.by_name.insert(name.to_string(), id);
        self.groups.insert(id, group.clone());
        group
    }

    /// Get a group by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Group> {
        self.by_name.get(name).and_then(|id| self.groups.get(id))
    }

    /// Get a group by ID.
    pub fn get_by_id(&self, id: i64) -> Option<&Group> {
        self.groups.get(&id)
    }

    /// Get all groups.
    pub fn all(&self) -> Vec<&Group> {
        self.groups.values().collect()
    }

    /// Delete a group by ID.
    pub fn delete(&mut self, id: i64) -> bool {
        if let Some(group) = self.groups.remove(&id) {
            self.by_name.remove(&group.name);
            true
        } else {
            false
        }
    }

    /// Add a permission to a group.
    pub fn add_permission_to_group(&mut self, group_id: i64, perm: Permission) -> bool {
        if let Some(group) = self.groups.get_mut(&group_id) {
            group.add_permission(perm);
            true
        } else {
            false
        }
    }

    /// Remove a permission from a group.
    pub fn remove_permission_from_group(&mut self, group_id: i64, perm_id: i64) -> bool {
        if let Some(group) = self.groups.get_mut(&group_id) {
            group.remove_permission(perm_id);
            true
        } else {
            false
        }
    }

    /// Get all unique permissions across all given group IDs.
    pub fn get_group_permissions(&self, group_ids: &[i64]) -> Vec<Permission> {
        let mut perms = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for gid in group_ids {
            if let Some(group) = self.groups.get(gid) {
                for perm in &group.permissions {
                    if seen.insert(perm.id) {
                        perms.push(perm.clone());
                    }
                }
            }
        }
        perms
    }

    /// Clear all groups (for testing).
    pub fn clear(&mut self) {
        self.groups.clear();
        self.by_name.clear();
        self.next_id = 1;
    }

    /// Get the total count of groups.
    pub fn count(&self) -> usize {
        self.groups.len()
    }
}

/// Global permission manager static.
static PERMISSION_MANAGER: std::sync::OnceLock<Mutex<PermissionManager>> = std::sync::OnceLock::new();

fn get_perm_manager() -> &'static Mutex<PermissionManager> {
    PERMISSION_MANAGER.get_or_init(|| Mutex::new(PermissionManager::new()))
}

/// Global group manager static.
static GROUP_MANAGER: std::sync::OnceLock<Mutex<GroupManager>> = std::sync::OnceLock::new();

fn get_group_manager_global() -> &'static Mutex<GroupManager> {
    GROUP_MANAGER.get_or_init(|| Mutex::new(GroupManager::new()))
}

/// Public helper to access the global permission manager.
pub fn permission_manager() -> &'static Mutex<PermissionManager> {
    get_perm_manager()
}

/// Public helper to access the global group manager.
pub fn group_manager() -> &'static Mutex<GroupManager> {
    get_group_manager_global()
}

/// Authenticated user model.
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub password: String,
    pub groups: Vec<Group>,
    pub user_permissions: Vec<Permission>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub date_joined: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(username: &str, email: &str) -> Self {
        Self {
            id: 0,
            username: username.to_string(),
            email: email.to_string(),
            first_name: String::new(),
            last_name: String::new(),
            is_active: true,
            is_staff: false,
            is_superuser: false,
            password: String::new(),
            groups: Vec::new(),
            user_permissions: Vec::new(),
            last_login: None,
            date_joined: chrono::Utc::now(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        true
    }

    /// Check if the user has a specific permission (like Django's user.has_perm).
    /// Superusers bypass all permission checks.
    /// Checks both user-level permissions and group-level permissions.
    pub fn has_perm(&self, perm: &str) -> bool {
        if self.is_superuser {
            return true;
        }
        if !self.is_active {
            return false;
        }
        // Check user-level permissions
        if self.user_permissions.iter().any(|p| p.codename == perm) {
            return true;
        }
        // Check group-level permissions
        self.groups.iter().any(|g| g.permissions.iter().any(|p| p.codename == perm))
    }

    /// Check if the user has all specified permissions.
    pub fn has_perms(&self, perms: &[&str]) -> bool {
        perms.iter().all(|p| self.has_perm(p))
    }

    /// Check if the user has any permission in the given module (app_label).
    pub fn has_module_perms(&self, app_label: &str) -> bool {
        if self.is_superuser {
            return true;
        }
        if !self.is_active {
            return false;
        }
        let prefix = format!("{}.", app_label);
        // Check user-level permissions matching the app_label prefix
        if self.user_permissions.iter().any(|p| p.codename.starts_with(&prefix)) {
            return true;
        }
        // Check group-level permissions
        self.groups.iter().any(|g| g.permissions.iter().any(|p| p.codename.starts_with(&prefix)))
    }

    /// Get all unique permission strings for this user (from both user and group perms).
    pub fn get_all_permissions(&self) -> Vec<String> {
        if self.is_superuser {
            // Superusers have all permissions conceptually
            return vec!["*.*".to_string()];
        }
        let mut perms: Vec<String> = self.user_permissions.iter().map(|p| p.codename.clone()).collect();
        for group in &self.groups {
            for perm in &group.permissions {
                let codename = &perm.codename;
                if !perms.iter().any(|p| p == codename) {
                    perms.push(codename.clone());
                }
            }
        }
        perms
    }

    /// Set password (hashed).
    pub fn set_password(&mut self, raw: &str) {
        self.password = crate::hashers::make_password(raw);
    }

    /// Check raw password against stored hash.
    pub fn check_password(&self, raw: &str) -> bool {
        if self.password.is_empty() {
            return raw.is_empty();
        }
        crate::hashers::check_password(raw, &self.password)
    }

    /// Get full name (like Django's User.get_full_name()).
    pub fn get_full_name(&self) -> String {
        if self.first_name.is_empty() && self.last_name.is_empty() {
            self.username.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name).trim().to_string()
        }
    }

    /// Get short name (like Django's User.get_short_name()).
    pub fn get_short_name(&self) -> String {
        if self.first_name.is_empty() {
            self.username.clone()
        } else {
            self.first_name.clone()
        }
    }
}

/// Anonymous (unauthenticated) user.
#[derive(Debug, Clone)]
pub struct AnonymousUser;

impl AnonymousUser {
    pub fn is_authenticated(&self) -> bool {
        false
    }

    /// Anonymous users never have permissions.
    pub fn has_perm(&self, _perm: &str) -> bool {
        false
    }

    pub fn has_perms(&self, _perms: &[&str]) -> bool {
        false
    }

    pub fn has_module_perms(&self, _app_label: &str) -> bool {
        false
    }

    pub fn get_all_permissions(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new("alice", "alice@example.com");
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
        assert!(user.is_active);
        assert!(!user.is_staff);
        assert!(!user.is_superuser);
        assert!(user.is_authenticated());
    }

    #[test]
    fn test_anonymous_user() {
        let anon = AnonymousUser;
        assert!(!anon.is_authenticated());
    }

    #[test]
    fn test_user_has_perm() {
        let user = User::new("bob", "bob@example.com");
        assert!(!user.has_perm("app.view_model")); // not superuser

        let mut admin = User::new("admin", "admin@example.com");
        admin.is_superuser = true;
        assert!(admin.has_perm("app.view_model"));
    }

    #[test]
    fn test_user_has_perm_by_user_permission() {
        let mut user = User::new("alice", "alice@example.com");
        let perm = Permission::new(1, "Can view model", 1, "view_model");
        user.user_permissions.push(perm);
        assert!(user.has_perm("view_model"));
        assert!(!user.has_perm("change_model"));
    }

    #[test]
    fn test_user_has_perm_by_group() {
        let mut user = User::new("bob", "bob@example.com");
        let mut group = Group::new(1, "Editors");
        group.add_permission(Permission::new(2, "Can edit", 1, "edit_post"));
        user.groups.push(group);
        assert!(user.has_perm("edit_post"));
        assert!(!user.has_perm("delete_post"));
    }

    #[test]
    fn test_user_has_perm_superuser_bypass() {
        let mut user = User::new("admin", "admin@example.com");
        user.is_superuser = true;
        user.is_active = false;
        // Superuser bypasses all checks including is_active
        assert!(user.has_perm("anything.at_all"));
    }

    #[test]
    fn test_user_has_perm_inactive_user() {
        let mut user = User::new("bob", "bob@example.com");
        user.is_active = false;
        let perm = Permission::new(1, "Can view", 1, "view_model");
        user.user_permissions.push(perm);
        // Inactive users with perms still show has_perm false
        assert!(!user.has_perm("view_model"));
    }

    #[test]
    fn test_user_has_perms() {
        let mut user = User::new("alice", "alice@example.com");
        user.user_permissions.push(Permission::new(1, "Can add", 1, "add_item"));
        user.user_permissions.push(Permission::new(2, "Can view", 1, "view_item"));
        assert!(user.has_perms(&["add_item", "view_item"]));
        assert!(!user.has_perms(&["add_item", "delete_item"]));
    }

    #[test]
    fn test_user_has_module_perms() {
        let mut user = User::new("alice", "alice@example.com");
        user.user_permissions.push(Permission::new(1, "Can add post", 1, "blog.add_post"));
        assert!(user.has_module_perms("blog"));
        assert!(!user.has_module_perms("auth"));
    }

    #[test]
    fn test_user_get_all_permissions() {
        let mut user = User::new("alice", "alice@example.com");
        user.user_permissions.push(Permission::new(1, "Can add", 1, "add_item"));
        user.user_permissions.push(Permission::new(2, "Can view", 1, "view_item"));
        let all = user.get_all_permissions();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&"add_item".to_string()));
        assert!(all.contains(&"view_item".to_string()));
    }

    #[test]
    fn test_user_get_all_permissions_includes_group_perms() {
        let mut user = User::new("bob", "bob@example.com");
        let mut group = Group::new(1, "Moderators");
        group.add_permission(Permission::new(1, "Can delete", 1, "delete_item"));
        user.user_permissions.push(Permission::new(2, "Can view", 1, "view_item"));
        user.groups.push(group);
        let all = user.get_all_permissions();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&"view_item".to_string()));
        assert!(all.contains(&"delete_item".to_string()));
    }

    #[test]
    fn test_superuser_get_all_permissions() {
        let mut user = User::new("admin", "admin@example.com");
        user.is_superuser = true;
        let all = user.get_all_permissions();
        assert_eq!(all, vec!["*.*"]);
    }

    #[test]
    fn test_anonymous_user_has_no_perms() {
        let anon = AnonymousUser;
        assert!(!anon.has_perm("anything"));
        assert!(!anon.has_perms(&["a", "b"]));
        assert!(!anon.has_module_perms("any"));
        assert!(anon.get_all_permissions().is_empty());
    }

    // === Permission Model Tests ===

    #[test]
    fn test_permission_new() {
        let perm = Permission::new(1, "Can add user", 1, "add_user");
        assert_eq!(perm.id, 1);
        assert_eq!(perm.name, "Can add user");
        assert_eq!(perm.content_type_id, 1);
        assert_eq!(perm.codename, "add_user");
    }

    #[test]
    fn test_permission_natural_key() {
        let perm = Permission::new(1, "Can view", 3, "view_post");
        assert_eq!(perm.natural_key(), "3:view_post");
    }

    #[test]
    fn test_permission_matches() {
        let perm = Permission::new(1, "Can add", 1, "add_user");
        assert!(perm.matches("add_user"));
        assert!(perm.matches("1:add_user"));
        assert!(!perm.matches("change_user"));
    }

    #[test]
    fn test_permission_manager_create() {
        let mut mgr = PermissionManager::new();
        let perm = mgr.create("Can add user", 1, "add_user");
        assert_eq!(perm.id, 1);
        assert_eq!(perm.codename, "add_user");
    }

    #[test]
    fn test_permission_manager_get_by_codename() {
        let mut mgr = PermissionManager::new();
        mgr.create("Can add", 1, "add_user");
        let perm = mgr.get_by_codename("add_user");
        assert!(perm.is_some());
        assert_eq!(perm.unwrap().name, "Can add");
    }

    #[test]
    fn test_permission_manager_get_by_codename_not_found() {
        let mgr = PermissionManager::new();
        assert!(mgr.get_by_codename("nonexistent").is_none());
    }

    #[test]
    fn test_permission_manager_all() {
        let mut mgr = PermissionManager::new();
        mgr.create("Can add", 1, "add_user");
        mgr.create("Can change", 1, "change_user");
        let all = mgr.all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_permission_manager_bulk_create() {
        let mut mgr = PermissionManager::new();
        let perms = mgr.bulk_create(vec![
            ("Can add".to_string(), 1, "add_user".to_string()),
            ("Can change".to_string(), 1, "change_user".to_string()),
        ]);
        assert_eq!(perms.len(), 2);
        assert_eq!(perms[0].id, 1);
        assert_eq!(perms[1].id, 2);
        assert_eq!(mgr.count(), 2);
    }

    #[test]
    fn test_permission_manager_idempotent_create() {
        let mut mgr = PermissionManager::new();
        let p1 = mgr.create("Can add", 1, "add_user");
        let p2 = mgr.create("Can add again", 1, "add_user");
        // Same codename returns the existing permission
        assert_eq!(p1.id, p2.id);
        assert_eq!(p2.name, "Can add"); // original name preserved
    }

    #[test]
    fn test_permission_manager_auto_create_default() {
        let mut mgr = PermissionManager::new();
        let perms = mgr.auto_create_default_permissions(1, "auth", "user");
        assert_eq!(perms.len(), 4);
        let codenames: Vec<&str> = perms.iter().map(|p| p.codename.as_str()).collect();
        assert!(codenames.contains(&"add_user"));
        assert!(codenames.contains(&"change_user"));
        assert!(codenames.contains(&"delete_user"));
        assert!(codenames.contains(&"view_user"));
    }

    #[test]
    fn test_permission_manager_auto_create_duplicate_call() {
        let mut mgr = PermissionManager::new();
        let p1 = mgr.auto_create_default_permissions(1, "auth", "user");
        let p2 = mgr.auto_create_default_permissions(1, "auth", "user");
        // Second call returns the same permissions (idempotent)
        for (a, b) in p1.iter().zip(p2.iter()) {
            assert_eq!(a.id, b.id);
        }
        assert_eq!(mgr.count(), 4);
    }

    #[test]
    fn test_permission_manager_clear() {
        let mut mgr = PermissionManager::new();
        mgr.create("Can add", 1, "add_user");
        mgr.clear();
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_permission_manager_filter_by_codenames() {
        let mut mgr = PermissionManager::new();
        mgr.create("Can add", 1, "add_user");
        mgr.create("Can change", 1, "change_user");
        mgr.create("Can delete", 1, "delete_user");
        let filtered = mgr.filter_by_codenames(&["add_user", "delete_user"]);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|p| p.codename == "add_user"));
        assert!(filtered.iter().any(|p| p.codename == "delete_user"));
    }

    // === Group Model Tests ===

    #[test]
    fn test_group_new() {
        let group = Group::new(1, "Editors");
        assert_eq!(group.id, 1);
        assert_eq!(group.name, "Editors");
        assert!(group.permissions.is_empty());
    }

    #[test]
    fn test_group_add_permission() {
        let mut group = Group::new(1, "Editors");
        let perm = Permission::new(1, "Can edit", 1, "edit_post");
        group.add_permission(perm);
        assert_eq!(group.permissions.len(), 1);
    }

    #[test]
    fn test_group_add_permission_idempotent() {
        let mut group = Group::new(1, "Editors");
        let perm = Permission::new(1, "Can edit", 1, "edit_post");
        group.add_permission(perm.clone());
        group.add_permission(perm);
        assert_eq!(group.permissions.len(), 1);
    }

    #[test]
    fn test_group_remove_permission() {
        let mut group = Group::new(1, "Editors");
        let perm = Permission::new(1, "Can edit", 1, "edit_post");
        group.add_permission(perm);
        assert_eq!(group.permissions.len(), 1);
        group.remove_permission(1);
        assert!(group.permissions.is_empty());
    }

    #[test]
    fn test_group_has_perm() {
        let mut group = Group::new(1, "Editors");
        group.add_permission(Permission::new(1, "Can edit", 1, "edit_post"));
        assert!(group.has_perm("edit_post"));
        assert!(!group.has_perm("delete_post"));
    }

    #[test]
    fn test_group_manager_create() {
        let mut mgr = GroupManager::new();
        let group = mgr.create("Admins");
        assert_eq!(group.name, "Admins");
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_group_manager_create_idempotent() {
        let mut mgr = GroupManager::new();
        let g1 = mgr.create("Editors");
        let g2 = mgr.create("Editors");
        assert_eq!(g1.id, g2.id);
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_group_manager_get_by_name() {
        let mut mgr = GroupManager::new();
        mgr.create("Moderators");
        let group = mgr.get_by_name("Moderators");
        assert!(group.is_some());
        let group = mgr.get_by_name("NonExistent");
        assert!(group.is_none());
    }

    #[test]
    fn test_group_manager_get_by_id() {
        let mut mgr = GroupManager::new();
        mgr.create("Moderators");
        let group = mgr.get_by_id(1);
        assert!(group.is_some());
        assert_eq!(group.unwrap().name, "Moderators");
        assert!(mgr.get_by_id(999).is_none());
    }

    #[test]
    fn test_group_manager_all() {
        let mut mgr = GroupManager::new();
        mgr.create("Group A");
        mgr.create("Group B");
        assert_eq!(mgr.all().len(), 2);
    }

    #[test]
    fn test_group_manager_delete() {
        let mut mgr = GroupManager::new();
        let group = mgr.create("Temp");
        assert!(mgr.delete(group.id));
        assert_eq!(mgr.count(), 0);
        assert!(mgr.get_by_name("Temp").is_none());
    }

    #[test]
    fn test_group_manager_delete_nonexistent() {
        let mut mgr = GroupManager::new();
        assert!(!mgr.delete(999));
    }

    #[test]
    fn test_group_manager_add_permission_to_group() {
        let mut mgr = GroupManager::new();
        let group = mgr.create("Admins");
        let perm = Permission::new(1, "Can add", 1, "add_user");
        assert!(mgr.add_permission_to_group(group.id, perm));
        let found = mgr.get_by_id(group.id).unwrap();
        assert_eq!(found.permissions.len(), 1);
    }

    #[test]
    fn test_group_manager_add_permission_to_nonexistent_group() {
        let mut mgr = GroupManager::new();
        let perm = Permission::new(1, "Can add", 1, "add_user");
        assert!(!mgr.add_permission_to_group(999, perm));
    }

    #[test]
    fn test_group_manager_remove_permission_from_group() {
        let mut mgr = GroupManager::new();
        let group = mgr.create("Admins");
        let perm = Permission::new(1, "Can add", 1, "add_user");
        mgr.add_permission_to_group(group.id, perm);
        assert!(mgr.remove_permission_from_group(group.id, 1));
        let found = mgr.get_by_id(group.id).unwrap();
        assert!(found.permissions.is_empty());
    }

    #[test]
    fn test_group_manager_get_group_permissions() {
        let mut mgr = GroupManager::new();
        let g1 = mgr.create("Admins");
        let g2 = mgr.create("Moderators");
        let p1 = Permission::new(1, "Can add", 1, "add_user");
        let p2 = Permission::new(2, "Can change", 1, "change_user");
        mgr.add_permission_to_group(g1.id, p1);
        mgr.add_permission_to_group(g2.id, p2);
        let perms = mgr.get_group_permissions(&[g1.id, g2.id]);
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn test_group_manager_get_group_permissions_deduplicates() {
        let mut mgr = GroupManager::new();
        let g1 = mgr.create("Admins");
        let g2 = mgr.create("SuperAdmins");
        let perm = Permission::new(1, "Can delete", 1, "delete_all");
        mgr.add_permission_to_group(g1.id, perm.clone());
        mgr.add_permission_to_group(g2.id, perm);
        let perms = mgr.get_group_permissions(&[g1.id, g2.id]);
        assert_eq!(perms.len(), 1); // deduplicated
    }

    #[test]
    fn test_group_manager_clear() {
        let mut mgr = GroupManager::new();
        mgr.create("Test Group");
        mgr.clear();
        assert_eq!(mgr.count(), 0);
    }

    // === User->Group integration tests ===

    #[test]
    fn test_user_groups_field_exists() {
        let user = User::new("alice", "alice@example.com");
        assert!(user.groups.is_empty());
    }

    #[test]
    fn test_user_user_permissions_field_exists() {
        let user = User::new("alice", "alice@example.com");
        assert!(user.user_permissions.is_empty());
    }

    #[test]
    fn test_user_set_password() {
        let mut user = User::new("alice", "alice@example.com");
        user.set_password("my_secure_pass");
        assert!(!user.password.is_empty());
        assert_ne!(user.password, "my_secure_pass");
    }

    #[test]
    fn test_user_check_password() {
        let mut user = User::new("bob", "bob@example.com");
        user.set_password("my_secure_pass");
        assert!(user.check_password("my_secure_pass"));
        assert!(!user.check_password("wrong_pass"));
    }

    #[test]
    fn test_user_check_empty_password() {
        let user = User::new("test", "test@example.com");
        assert!(user.check_password(""));
        assert!(!user.check_password("something"));
    }

    #[test]
    fn test_user_get_full_name_with_names() {
        let mut user = User::new("alice", "alice@example.com");
        user.first_name = "Alice".to_string();
        user.last_name = "Smith".to_string();
        assert_eq!(user.get_full_name(), "Alice Smith");
    }

    #[test]
    fn test_user_get_full_name_fallback() {
        let user = User::new("alice", "alice@example.com");
        assert_eq!(user.get_full_name(), "alice");
    }

    #[test]
    fn test_user_get_short_name() {
        let mut user = User::new("alice", "alice@example.com");
        user.first_name = "Alice".to_string();
        assert_eq!(user.get_short_name(), "Alice");
    }

    #[test]
    fn test_user_get_short_name_fallback() {
        let user = User::new("bob", "bob@example.com");
        assert_eq!(user.get_short_name(), "bob");
    }
}
