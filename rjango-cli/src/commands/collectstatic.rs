//! Collect static files.
//! Mirrors `rjango collectstatic`.

use std::path::Path;

pub fn run(static_root: &str, static_dirs: &[String]) {
    let dest = Path::new(static_root);
    if !dest.exists() {
        std::fs::create_dir_all(dest).expect("Failed to create static root");
    }
    
    let mut copied = 0;
    for dir in static_dirs {
        let src = Path::new(dir);
        if src.is_dir() {
            if let Ok(entries) = std::fs::read_dir(src) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let dest_path = dest.join(path.file_name().unwrap());
                        if std::fs::copy(&path, &dest_path).is_ok() {
                            copied += 1;
                        }
                    }
                }
            }
        }
    }
    
    println!("{} static file(s) copied to '{}'.", copied, static_root);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_collectstatic_empty_dirs() {
        let dir = std::env::temp_dir().join("_rjango_test_static_out");
        let _ = fs::remove_dir_all(&dir);
        run(dir.to_str().unwrap(), &[]);
        assert!(dir.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collectstatic_with_source_dir() {
        let out_dir = std::env::temp_dir().join("_rjango_test_static_out2");
        let src_dir = std::env::temp_dir().join("_rjango_test_static_src");
        let _ = fs::remove_dir_all(&out_dir);
        let _ = fs::remove_dir_all(&src_dir);
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("style.css"), "body {}").unwrap();

        run(out_dir.to_str().unwrap(), &[src_dir.to_str().unwrap().to_string()]);
        assert!(out_dir.join("style.css").exists());

        let _ = fs::remove_dir_all(&out_dir);
        let _ = fs::remove_dir_all(&src_dir);
    }
}
