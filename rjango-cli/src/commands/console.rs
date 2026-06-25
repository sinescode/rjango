//! Interactive Rjango console (REPL).
//! Mirrors Django's `django-extensions shell_plus`.

pub fn run() {
    println!("Rjango Console");
    println!("Type 'exit' to quit.");
    
    loop {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() || line.trim() == "exit" {
            break;
        }
        println!("  (not yet connected)");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_console_function_exists() {
        // Verify the run function compiles and is accessible
        let _ = super::run;
    }
}
