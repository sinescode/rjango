//! Run tests.
//! Mirrors `rjango test`.

use std::process::Command;

pub fn run(args: &[String]) {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    for arg in args {
        cmd.arg(arg);
    }

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                println!("Tests passed.");
            } else {
                eprintln!("Tests failed.");
            }
        }
        Err(e) => {
            eprintln!("Failed to run tests: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fn_exists() {
        fn _assert(_: &[String]) {}
    }
}
