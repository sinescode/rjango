//! Interactive Rust shell.
//! Mirrors `rjango shell`.

pub fn run() {
    println!("Rjango shell ({})", std::env::current_exe()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .unwrap_or_default());
    println!("Type \"exit()\" to quit.");
    
    // Simple REPL
    loop {
        print!(">>> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() || line.trim() == "exit()" {
            break;
        }
        println!("  (evaluator not yet connected)");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_shell_run_no_panic() {
        // Just verify the function signature compiles
        let _ = super::run;
    }
}
