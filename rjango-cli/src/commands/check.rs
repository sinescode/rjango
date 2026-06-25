//! Check project for issues.
//! Mirrors `rjango check`.

use rjango_core::Settings;

pub fn run(settings: &Settings) {
    let mut issues = Vec::new();

    // Check debug mode
    let debug_msgs = rjango_core::checks::check_debug_mode(settings.debug);
    for msg in &debug_msgs {
        if msg.level == rjango_core::checks::CheckLevel::Error ||
           msg.level == rjango_core::checks::CheckLevel::Warning {
            issues.push(format!("[{}] {}: {}", msg.level.as_str(), msg.id, msg.msg));
        }
    }

    // Check secret key
    let key_msgs = rjango_core::checks::check_secret_key_length(&settings.secret_key);
    for msg in &key_msgs {
        if msg.level == rjango_core::checks::CheckLevel::Error ||
           msg.level == rjango_core::checks::CheckLevel::Warning {
            issues.push(format!("[{}] {}: {}", msg.level.as_str(), msg.id, msg.msg));
        }
    }

    if issues.is_empty() {
        println!("System check identified no issues (0 silenced).");
    } else {
        for issue in &issues {
            eprintln!("{}", issue);
        }
    }
}
