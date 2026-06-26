/// Humanize — like `django.contrib.humanize`.
/// Template filters for making numbers/dates more readable.

use chrono::NaiveDate;

/// Convert an integer to ordinal string (1st, 2nd, 3rd, ...).
pub fn ordinal(n: i64) -> String {
    let abs = n.unsigned_abs();
    let suffix = match (abs % 100, abs % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{}{}", n, suffix)
}

/// Format an integer with comma separators: 1000 → "1,000".
pub fn intcomma(n: i64) -> String {
    let negative = n < 0;
    let abs_str = n.unsigned_abs().to_string();
    let mut result = String::new();
    let chars: Vec<char> = abs_str.chars().collect();
    let len = chars.len();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    if negative {
        format!("-{}", result)
    } else {
        result
    }
}

/// Format a number as "1.0 million", "2.5 billion", etc.
pub fn intword(n: f64) -> String {
    let abs = n.abs();
    let (value, suffix) = if abs >= 1_000_000_000_000.0 {
        (abs / 1_000_000_000_000.0, "trillion")
    } else if abs >= 1_000_000_000.0 {
        (abs / 1_000_000_000.0, "billion")
    } else if abs >= 1_000_000.0 {
        (abs / 1_000_000.0, "million")
    } else if abs >= 1_000.0 {
        (abs / 1_000.0, "thousand")
    } else {
        return format!("{}", n as i64);
    };
    format!("{:.1} {}", value, suffix)
}

/// Show a datetime as "now", "1 minute ago", "2 hours ago", etc.
pub fn naturaltime(seconds_ago: i64) -> String {
    let abs = seconds_ago.unsigned_abs();
    let future = seconds_ago < 0;

    if abs == 0 {
        return "now".to_string();
    }

    let minutes = abs / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;

    let prefix = if future { "from now" } else { "ago" };

    let time_str = if abs < 60 {
        format!("{} second{}", abs, if abs == 1 { "" } else { "s" })
    } else if minutes < 60 {
        format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" })
    } else if hours < 24 {
        format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else if days < 7 {
        format!("{} day{}", days, if days == 1 { "" } else { "s" })
    } else if weeks < 5 {
        format!("{} week{}", weeks, if weeks == 1 { "" } else { "s" })
    } else if months < 12 {
        format!("{} month{}", months, if months == 1 { "" } else { "s" })
    } else {
        format!("{} year{}", years, if years == 1 { "" } else { "s" })
    };

    format!("{} {}", time_str, prefix)
}

/// Convert a date to "today", "yesterday", "tomorrow", or a formatted date.
pub fn naturalday(date: NaiveDate, today: NaiveDate) -> String {
    let diff = date.signed_duration_since(today).num_days();
    match diff {
        0 => "today".to_string(),
        1 => "tomorrow".to_string(),
        -1 => "yesterday".to_string(),
        _ => date.format("%b %d, %Y").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordinal_basic() {
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(3), "3rd");
        assert_eq!(ordinal(4), "4th");
        assert_eq!(ordinal(11), "11th");
        assert_eq!(ordinal(12), "12th");
        assert_eq!(ordinal(13), "13th");
        assert_eq!(ordinal(21), "21st");
        assert_eq!(ordinal(101), "101st");
    }

    #[test]
    fn test_ordinal_negative() {
        assert_eq!(ordinal(-1), "-1st");
    }

    #[test]
    fn test_intcomma() {
        assert_eq!(intcomma(1000), "1,000");
        assert_eq!(intcomma(1234567), "1,234,567");
        assert_eq!(intcomma(100), "100");
        assert_eq!(intcomma(0), "0");
    }

    #[test]
    fn test_intcomma_negative() {
        assert_eq!(intcomma(-1000), "-1,000");
    }

    #[test]
    fn test_intword() {
        assert_eq!(intword(1_500_000.0), "1.5 million");
        assert_eq!(intword(2_000_000_000.0), "2.0 billion");
        assert_eq!(intword(500.0), "500");
    }

    #[test]
    fn test_naturaltime_now() {
        assert_eq!(naturaltime(0), "now");
    }

    #[test]
    fn test_naturaltime_seconds() {
        assert_eq!(naturaltime(30), "30 seconds ago");
    }

    #[test]
    fn test_naturaltime_minutes() {
        assert_eq!(naturaltime(120), "2 minutes ago");
    }

    #[test]
    fn test_naturaltime_hours() {
        assert_eq!(naturaltime(7200), "2 hours ago");
    }

    #[test]
    fn test_naturaltime_days() {
        assert_eq!(naturaltime(172800), "2 days ago");
    }

    #[test]
    fn test_naturaltime_future() {
        assert_eq!(naturaltime(-3600), "1 hour from now");
    }

    #[test]
    fn test_naturalday_today() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        assert_eq!(naturalday(today, today), "today");
    }

    #[test]
    fn test_naturalday_yesterday() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let yesterday = NaiveDate::from_ymd_opt(2026, 6, 25).unwrap();
        assert_eq!(naturalday(yesterday, today), "yesterday");
    }

    #[test]
    fn test_naturalday_tomorrow() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2026, 6, 27).unwrap();
        assert_eq!(naturalday(tomorrow, today), "tomorrow");
    }

    #[test]
    fn test_naturalday_other() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let other = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let result = naturalday(other, today);
        assert!(result.contains("Jan"));
    }
}
