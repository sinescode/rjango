//! CLI command implementations

pub mod startproject;
pub mod startapp;
pub mod runserver;
pub mod migrate;
pub mod makemigrations;
pub mod showmigrations;
pub mod createsuperuser;
pub mod shell;
pub mod dbshell;
pub mod collectstatic;
pub mod check;
pub mod validate;
pub mod test;
pub mod console;

#[cfg(test)]
mod tests {
    #[test]
    fn test_commands_module_accessible() {
        // Verify all command modules are accessible
        let _ = super::check::run;
        let _ = super::runserver::run;
        let _ = super::migrate::run;
        let _ = super::startapp::run;
        let _ = super::startproject::run;
        let _ = super::collectstatic::run; // may not exist with run signature
    }
}
