//! List migrations and their status.
//! Mirrors `rjango showmigrations`.

pub fn run() {
    println!("migrations:");
    println!("  (no migrations applied)");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_showmigrations_run() {
        // Just verify the function is callable
        let _ = super::run;
    }
}
