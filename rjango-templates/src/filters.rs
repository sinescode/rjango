/// Built-in template filters.
/// Like Django's `{{ value|filter:arg }}`.

use serde_json::Value;

/// Filter function signature: takes a value and optional string args.
pub type FilterFn = fn(&Value, &[&str]) -> Value;

/// Filter registry: maps filter name to a function.
pub fn builtin_filters() -> Vec<(&'static str, FilterFn)> {
    vec![
        ("upper", |v, _| upper(v)),
        ("lower", |v, _| lower(v)),
        ("title", |v, _| title(v)),
        ("length", |v, _| length(v)),
        ("default", |v, args| default(v, args)),
        ("cut", |v, args| cut(v, args)),
        ("join", |v, args| join(v, args)),
        ("capfirst", |v, _| capfirst(v)),
        ("escape", |v, _| escape(v)),
        ("safe", |v, _| safe(v)),
        ("slugify", |v, _| slugify(v)),
        ("add", |v, args| add_filter(v, args)),
        ("addslashes", |v, _| addslashes(v)),
        ("date", |v, _| date_filter(v)),
        ("time", |v, _| time_filter(v)),
        ("timesince", |v, _| timesince_filter(v)),
        ("timeuntil", |v, _| timeuntil_filter(v)),
        ("floatformat", |v, args| floatformat(v, args)),
        ("filesizeformat", |v, _| filesizeformat(v)),
        ("first", |v, _| first(v)),
        ("last", |v, _| last(v)),
        ("linenumbers", |v, _| linenumbers(v)),
        ("linebreaks", |v, _| linebreaks(v)),
        ("linebreaksbr", |v, _| linebreaksbr(v)),
        ("pluralize", |v, args| pluralize(v, args)),
        ("random", |v, _| random_filter(v)),
        ("rjust", |v, args| rjust(v, args)),
        ("ljust", |v, args| ljust(v, args)),
        ("center", |v, args| center(v, args)),
        ("make_list", |v, _| make_list(v)),
        ("slice", |v, args| slice_filter(v, args)),
        ("stringformat", |v, args| stringformat(v, args)),
        ("striptags", |v, _| striptags(v)),
        ("truncatechars", |v, args| truncatechars(v, args)),
        ("truncatewords", |v, args| truncatewords(v, args)),
        ("urlencode", |v, _| urlencode(v)),
        ("urlize", |v, _| urlize(v)),
        ("wordcount", |v, _| wordcount(v)),
        ("yesno", |v, args| yesno(v, args)),
        ("divisibleby", |v, args| divisibleby(v, args)),
        ("phone2numeric", |v, _| phone2numeric(v)),
        ("unordered_list", |v, _| unordered_list(v)),
        ("dictsort", |v, args| dictsort(v, args)),
        ("dictsortreversed", |v, args| dictsortreversed(v, args)),
        ("pprint", |v, _| pprint(v)),
        ("escapejs", |v, _| escapejs(v)),
        ("truncatechars_html", |v, args| truncatechars_html(v, args)),
        ("truncatewords_html", |v, args| truncatewords_html(v, args)),
        ("wordwrap", |v, args| wordwrap(v, args)),
        ("iriencode", |v, _| iriencode(v)),
        ("json_script", |v, args| json_script(v, args)),
        ("safeseq", |v, _| safeseq(v)),
        ("force_escape", |v, _| force_escape(v)),
        ("get_digit", |v, args| get_digit(v, args)),
        ("urlizetrunc", |v, args| urlizetrunc(v, args)),
    ]
}

// ── Core filters ────────────────────────────────────────────────────────

fn upper(v: &Value) -> Value {
    Value::String(v.as_str().unwrap_or("").to_uppercase())
}

fn lower(v: &Value) -> Value {
    Value::String(v.as_str().unwrap_or("").to_lowercase())
}

fn title(v: &Value) -> Value {
    Value::String(
        v.as_str()
            .unwrap_or("")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn length(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::Number(serde_json::Number::from(s.len() as u64)),
        Value::Array(a) => Value::Number(serde_json::Number::from(a.len() as u64)),
        Value::Object(o) => Value::Number(serde_json::Number::from(o.len() as u64)),
        _ => Value::Number(serde_json::Number::from(0u64)),
    }
}

fn default(v: &Value, args: &[&str]) -> Value {
    if v.is_null() || (v.is_string() && v.as_str().unwrap_or("").is_empty()) {
        Value::String(args.first().unwrap_or(&"").to_string())
    } else {
        v.clone()
    }
}

fn cut(v: &Value, args: &[&str]) -> Value {
    let arg = args.first().unwrap_or(&"");
    Value::String(v.as_str().unwrap_or("").replace(arg, ""))
}

fn join(v: &Value, args: &[&str]) -> Value {
    let joiner = args.first().unwrap_or(&",");
    match v {
        Value::Array(a) => Value::String(
            a.iter()
                .map(|x| match x {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(joiner),
        ),
        _ => v.clone(),
    }
}

fn capfirst(v: &Value) -> Value {
    Value::String(
        v.as_str()
            .unwrap_or("")
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap_or(c) } else { c })
            .collect(),
    )
}

fn escape(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    Value::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;"),
    )
}

fn safe(v: &Value) -> Value {
    v.clone()
}

fn slugify(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("").to_lowercase();
    let slug: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let cleaned: String = slug
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    Value::String(cleaned)
}

fn add_filter(v: &Value, args: &[&str]) -> Value {
    let arg = args.first().unwrap_or(&"0");
    match (v, arg.parse::<f64>()) {
        (Value::Number(n), Ok(a)) => {
            let v = n.as_f64().unwrap_or(0.0);
            let sum = v + a;
            // Use integer if whole, float otherwise
            if sum.fract() == 0.0 && sum.is_finite() {
                Value::Number(serde_json::Number::from(sum as i64))
            } else {
                Value::Number(serde_json::Number::from_f64(sum).unwrap_or(serde_json::Number::from(0)))
            }
        }
        (Value::String(s), _) => Value::String(format!("{}{}", s, arg)),
        _ => v.clone(),
    }
}

fn addslashes(v: &Value) -> Value {
    Value::String(
        v.as_str()
            .unwrap_or("")
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('"', "\\\"")
    )
}

fn date_filter(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    if let Some(date) = s.split('T').next() {
        Value::String(date.to_string())
    } else {
        Value::String(s.to_string())
    }
}

fn time_filter(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    if let Some(t) = s.split('T').nth(1) {
        Value::String(t[..5].to_string())
    } else {
        Value::String(s.to_string())
    }
}

fn timesince_filter(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    if s.is_empty() {
        return Value::String(String::new());
    }
    Value::String("some time ago".to_string())
}

fn timeuntil_filter(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    if s.is_empty() {
        return Value::String(String::new());
    }
    Value::String("some time from now".to_string())
}

fn floatformat(v: &Value, args: &[&str]) -> Value {
    let decimals = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    match v.as_f64() {
        Some(n) => Value::String(format!("{:.*}", decimals, n)),
        None => v.clone(),
    }
}

fn filesizeformat(v: &Value) -> Value {
    let bytes = v.as_f64().unwrap_or(0.0);
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < 4 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        Value::String(format!("{} {}", size as u64, units[unit_idx]))
    } else {
        Value::String(format!("{:.1} {}", size, units[unit_idx]))
    }
}

fn first(v: &Value) -> Value {
    match v {
        Value::Array(a) => a.first().cloned().unwrap_or(Value::Null),
        Value::String(s) => s.chars().next().map(|c| Value::String(c.to_string())).unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

fn last(v: &Value) -> Value {
    match v {
        Value::Array(a) => a.last().cloned().unwrap_or(Value::Null),
        Value::String(s) => s.chars().last().map(|c| Value::String(c.to_string())).unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

fn linenumbers(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    let numbered: Vec<String> = s.lines()
        .enumerate()
        .map(|(i, line)| format!("{}. {}", i + 1, line))
        .collect();
    Value::String(numbered.join("\n"))
}

fn linebreaks(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    let html: String = s.lines()
        .map(|l| {
            if l.is_empty() { "</p><p>".to_string() }
            else { format!("{}\n<br>", l) }
        })
        .collect();
    Value::String(format!("<p>{}</p>", html.trim_end_matches("<br>")))
}

fn linebreaksbr(v: &Value) -> Value {
    Value::String(v.as_str().unwrap_or("").replace('\n', "<br>\n"))
}

fn pluralize(v: &Value, args: &[&str]) -> Value {
    let count = v.as_f64().unwrap_or(0.0) as u64;
    if args.is_empty() {
        // Default: add "s" suffix for non-1 counts
        return if count == 1 { Value::String(String::new()) } else { Value::String("s".to_string()) };
    }
    let parts: Vec<&str> = args[0].split(',').collect();
    if parts.len() == 1 {
        // Single suffix: add nothing for 1, suffix otherwise
        let suffix = parts[0];
        if count == 1 { Value::String(String::new()) } else { Value::String(suffix.to_string()) }
    } else {
        // Two parts: singular, plural
        let singular = parts[0];
        let plural = parts[1];
        if count == 1 { Value::String(singular.to_string()) } else { Value::String(plural.to_string()) }
    }
}

fn random_filter(v: &Value) -> Value {
    match v {
        Value::Array(a) if !a.is_empty() => {
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let idx = (nanos as usize) % a.len();
            a[idx].clone()
        }
        _ => Value::Null,
    }
}

fn rjust(v: &Value, args: &[&str]) -> Value {
    let width = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    Value::String(format!("{:>width$}", v.as_str().unwrap_or(""), width = width))
}

fn ljust(v: &Value, args: &[&str]) -> Value {
    let width = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    Value::String(format!("{:<width$}", v.as_str().unwrap_or(""), width = width))
}

fn center(v: &Value, args: &[&str]) -> Value {
    let width = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    Value::String(format!("{:^width$}", v.as_str().unwrap_or(""), width = width))
}

fn make_list(v: &Value) -> Value {
    match v {
        Value::String(s) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Value::Array(chars)
        }
        Value::Array(a) => Value::Array(a.clone()),
        Value::Number(n) => {
            let chars: Vec<Value> = n.to_string().chars().map(|c| Value::String(c.to_string())).collect();
            Value::Array(chars)
        }
        _ => Value::Array(vec![]),
    }
}

fn slice_filter(v: &Value, args: &[&str]) -> Value {
    let parts: Vec<&str> = args.first().unwrap_or(&"").split(':').collect();
    let start = parts.first().and_then(|p| p.parse::<usize>().ok()).unwrap_or(0);
    let end = parts.get(1).and_then(|p| {
        if p.is_empty() { None } else { p.parse::<usize>().ok() }
    });
    match v {
        Value::Array(a) => {
            let sliced: Vec<Value> = if let Some(e) = end {
                a.iter().skip(start).take(e.saturating_sub(start)).cloned().collect()
            } else {
                a.iter().skip(start).cloned().collect()
            };
            Value::Array(sliced)
        }
        Value::String(s) => {
            let sliced: String = if let Some(e) = end {
                s.chars().skip(start).take(e.saturating_sub(start)).collect()
            } else {
                s.chars().skip(start).collect()
            };
            Value::String(sliced)
        }
        _ => v.clone(),
    }
}

fn stringformat(v: &Value, args: &[&str]) -> Value {
    let fmt = args.first().unwrap_or(&"%s");
    let s = v.as_str().map(|s| s.to_string())
        .or_else(|| v.as_f64().map(|n| n.to_string()))
        .unwrap_or_default();
    Value::String(fmt.replace("%s", &s).replace("%d", &(v.as_f64().unwrap_or(0.0) as i64).to_string()))
}

fn striptags(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    Value::String(re.replace_all(s, "").to_string())
}

fn truncatechars(v: &Value, args: &[&str]) -> Value {
    let n = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    let s = v.as_str().unwrap_or("");
    if s.chars().count() > n && n > 3 {
        Value::String(format!("{}...", s.chars().take(n - 3).collect::<String>()))
    } else if s.chars().count() > n {
        Value::String(s.chars().take(n).collect())
    } else {
        Value::String(s.to_string())
    }
}

fn truncatewords(v: &Value, args: &[&str]) -> Value {
    let n = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    let s = v.as_str().unwrap_or("");
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() > n {
        Value::String(format!("{} ...", words[..n].join(" ")))
    } else {
        v.clone()
    }
}

fn urlencode(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    Value::String(urlencoding::encode(s).replace('+', "%20"))
}

fn urlize(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    let re = regex::Regex::new(r"(https?://[^\s<]+)").unwrap();
    let result = re.replace_all(s, |caps: &regex::Captures| {
        format!(r#"<a href="{}">{}</a>"#, &caps[1], &caps[1])
    });
    Value::String(result.to_string())
}

fn wordcount(v: &Value) -> Value {
    let count = v.as_str().unwrap_or("").split_whitespace().count();
    Value::Number(serde_json::Number::from(count as u64))
}

fn yesno(v: &Value, args: &[&str]) -> Value {
    let parts: Vec<&str> = args.first().unwrap_or(&"yes,no,maybe").split(',').collect();
    let yes_val = parts.first().unwrap_or(&"yes");
    let no_val = parts.get(1).unwrap_or(&"no");
    let maybe_val = parts.get(2).unwrap_or(&"maybe");
    match v {
        Value::Bool(true) | Value::Null => Value::String(yes_val.to_string()),
        Value::Bool(false) => Value::String(no_val.to_string()),
        Value::String(s) => match s.as_str() {
            "true" | "1" | "True" | "yes" => Value::String(yes_val.to_string()),
            "false" | "0" | "False" | "no" => Value::String(no_val.to_string()),
            _ => Value::String(maybe_val.to_string()),
        },
        Value::Number(n) => {
            if n.as_f64().unwrap_or(0.0) != 0.0 {
                Value::String(yes_val.to_string())
            } else {
                Value::String(no_val.to_string())
            }
        }
        _ => Value::String(maybe_val.to_string()),
    }
}

fn divisibleby(v: &Value, args: &[&str]) -> Value {
    let divisor = args.first().and_then(|a| a.parse::<f64>().ok()).unwrap_or(1.0);
    if divisor == 0.0 { return Value::Bool(false); }
    match v.as_f64() {
        Some(n) => Value::Bool(n % divisor == 0.0),
        None => Value::Bool(false),
    }
}

fn phone2numeric(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("").to_lowercase();
    let result: String = s.chars().map(|c| match c {
        'a' | 'b' | 'c' => '2',
        'd' | 'e' | 'f' => '3',
        'g' | 'h' | 'i' => '4',
        'j' | 'k' | 'l' => '5',
        'm' | 'n' | 'o' => '6',
        'p' | 'q' | 'r' | 's' => '7',
        't' | 'u' | 'v' => '8',
        'w' | 'x' | 'y' | 'z' => '9',
        other => other,
    }).collect();
    Value::String(result)
}

fn unordered_list(v: &Value) -> Value {
    fn render_list(items: &[Value]) -> String {
        let mut html = String::from("<ul>\n");
        for item in items {
            match item {
                Value::Array(sub) if sub.len() == 2 => {
                    let label = sub[0].as_str().unwrap_or("");
                    html.push_str(&format!("  <li>{}\n", label));
                    if let Value::Array(children) = &sub[1] {
                        html.push_str(&render_list(children));
                    }
                    html.push_str("  </li>\n");
                }
                Value::String(s) => {
                    html.push_str(&format!("  <li>{}</li>\n", s));
                }
                _ => {
                    html.push_str(&format!("  <li>{}</li>\n", item));
                }
            }
        }
        html.push_str("</ul>\n");
        html
    }
    match v {
        Value::Array(a) => Value::String(render_list(a)),
        _ => v.clone(),
    }
}

fn dictsort(v: &Value, args: &[&str]) -> Value {
    let key = args.first().unwrap_or(&"");
    match v {
        Value::Array(items) => {
            let mut sorted = items.clone();
            sorted.sort_by(|a, b| {
                let av = a.get(key).and_then(|v| v.as_str()).unwrap_or("");
                let bv = b.get(key).and_then(|v| v.as_str()).unwrap_or("");
                av.cmp(bv)
            });
            Value::Array(sorted)
        }
        _ => v.clone(),
    }
}

fn dictsortreversed(v: &Value, args: &[&str]) -> Value {
    let key = args.first().unwrap_or(&"");
    match v {
        Value::Array(items) => {
            let mut sorted = items.clone();
            sorted.sort_by(|a, b| {
                let av = a.get(key).and_then(|v| v.as_str()).unwrap_or("");
                let bv = b.get(key).and_then(|v| v.as_str()).unwrap_or("");
                bv.cmp(av) // Reversed
            });
            Value::Array(sorted)
        }
        _ => v.clone(),
    }
}

fn pprint(v: &Value) -> Value {
    Value::String(format!("{:#}", v))
}

/// Escape JavaScript string.
fn escapejs(v: &Value) -> Value {
    let s = match v {
        Value::String(s) => s.clone(),
        _ => return v.clone(),
    };
    let escaped = s
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('/', "\\/");
    Value::String(escaped)
}

/// Truncate at N chars with HTML tag awareness (simplified: strips tags then truncates).
/// Strip HTML tags (char-by-char scanner).
fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

fn truncatechars_html(v: &Value, args: &[&str]) -> Value {
    let s = match v {
        Value::String(s) => s.clone(),
        _ => return v.clone(),
    };
    let n: usize = args.first().and_then(|a| a.parse().ok()).unwrap_or(10);
    let stripped = strip_html_tags(&s);
    if stripped.chars().count() <= n {
        return Value::String(stripped);
    }
    let truncated: String = stripped.chars().take(n).collect();
    Value::String(format!("{}...", truncated))
}

/// Truncate at N words with HTML awareness.
fn truncatewords_html(v: &Value, args: &[&str]) -> Value {
    let s = match v {
        Value::String(s) => s.clone(),
        _ => return v.clone(),
    };
    let n: usize = args.first().and_then(|a| a.parse().ok()).unwrap_or(5);
    let stripped = strip_html_tags(&s);
    let words: Vec<&str> = stripped.split_whitespace().collect();
    if words.len() <= n {
        return Value::String(words.join(" "));
    }
    Value::String(format!("{} ...", words[..n].join(" ")))
}

/// Wordwrap text at N characters.
fn wordwrap(v: &Value, args: &[&str]) -> Value {
    let s = match v {
        Value::String(s) => s.clone(),
        _ => return v.clone(),
    };
    let width: usize = args.first().and_then(|a| a.parse().ok()).unwrap_or(80);
    let mut result = String::new();
    let mut line_len = 0;
    for word in s.split_whitespace() {
        if line_len + word.len() + 1 > width && line_len > 0 {
            result.push('\n');
            line_len = 0;
        }
        if line_len > 0 {
            result.push(' ');
            line_len += 1;
        }
        result.push_str(word);
        line_len += word.len();
    }
    Value::String(result)
}

/// Encode an IRI (simplified: URL percent-encoding).
fn iriencode(v: &Value) -> Value {
    let s = match v {
        Value::String(s) => s.clone(),
        _ => return v.clone(),
    };
    Value::String(urlencoding::encode(&s).into_owned())
}

/// Output value as JSON in a <script> tag.
fn json_script(v: &Value, args: &[&str]) -> Value {
    let id = args.first().unwrap_or(&"data");
    let json_str = serde_json::to_string(v).unwrap_or_else(|_| "null".to_string());
    Value::String(format!(
        "<script id=\"{}\" type=\"application/json\">{}</script>",
        id, json_str
    ))
}

/// Mark each item in a sequence as safe (simply returns the same array).
fn safeseq(v: &Value) -> Value {
    v.clone()
}

/// Force-escape HTML — always escapes even if value is marked safe.
fn force_escape(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    Value::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;"),
    )
}

/// Get the nth digit from a number (1-indexed from right).
/// E.g., {{ 12345|get_digit:"2" }} → "4"
fn get_digit(v: &Value, args: &[&str]) -> Value {
    let n = v.as_f64().map(|f| f.abs() as u64).unwrap_or(0);
    let pos: usize = args.first().and_then(|a| a.parse().ok()).unwrap_or(0);
    if pos == 0 || n == 0 {
        return Value::Number(serde_json::Number::from(0u64));
    }
    let digits = n.to_string();
    if pos > digits.len() {
        return Value::Number(serde_json::Number::from(0u64));
    }
    let ch = digits.chars().rev().nth(pos - 1).unwrap_or('0');
    Value::Number(serde_json::Number::from(ch.to_digit(10).unwrap_or(0)))
}

/// URLize with truncated display text.
/// E.g., {{ "https://example.com/long/path"|urlizetrunc:"15" }}
fn urlizetrunc(v: &Value, args: &[&str]) -> Value {
    let s = v.as_str().unwrap_or("");
    let truncate_len: usize = args.first().and_then(|a| a.parse().ok()).unwrap_or(30);
    let re = regex::Regex::new(r"(https?://[^\s<]+)").unwrap();
    let result = re.replace_all(s, |caps: &regex::Captures| {
        let url = &caps[1];
        let truncated: String = if url.chars().count() > truncate_len {
            format!("{}...", url.chars().take(truncate_len).collect::<String>())
        } else {
            url.to_string()
        };
        format!(r##"<a href="{}">{}</a>"##, url, truncated)
    });
    Value::String(result.to_string())
}

// ── Tests ───────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_upper() {
        assert_eq!(upper(&json!("hello")), json!("HELLO"));
    }

    #[test]
    fn test_lower() {
        assert_eq!(lower(&json!("HELLO")), json!("hello"));
    }

    #[test]
    fn test_title() {
        assert_eq!(title(&json!("hello world")), json!("Hello World"));
    }

    #[test]
    fn test_length_string() {
        assert_eq!(length(&json!("hello")), json!(5));
    }

    #[test]
    fn test_length_array() {
        assert_eq!(length(&json!([1, 2, 3])), json!(3));
    }

    #[test]
    fn test_default_fallback() {
        assert_eq!(default(&json!(null), &["fallback"]), json!("fallback"));
    }

    #[test]
    fn test_default_empty_string() {
        assert_eq!(default(&json!(""), &["fallback"]), json!("fallback"));
    }

    #[test]
    fn test_default_keeps_value() {
        assert_eq!(default(&json!("hello"), &["fallback"]), json!("hello"));
    }

    #[test]
    fn test_escape() {
        assert_eq!(escape(&json!("<script>")), json!("&lt;script&gt;"));
    }

    #[test]
    fn test_cut() {
        assert_eq!(cut(&json!("hello world"), &["l"]), json!("heo word"));
    }

    #[test]
    fn test_join() {
        assert_eq!(join(&json!(["a", "b", "c"]), &["|"]), json!("a|b|c"));
    }

    #[test]
    fn test_capfirst() {
        assert_eq!(capfirst(&json!("hello")), json!("Hello"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify(&json!("Hello World!")), json!("hello-world"));
        assert_eq!(slugify(&json!("  leading spaces")), json!("leading-spaces"));
        assert_eq!(slugify(&json!("multiple   spaces")), json!("multiple-spaces"));
    }

    #[test]
    fn test_add_numeric() {
        assert_eq!(add_filter(&json!(5), &["3"]), json!(8));
    }

    #[test]
    fn test_add_string() {
        assert_eq!(add_filter(&json!("hello "), &["world"]), json!("hello world"));
    }

    #[test]
    fn test_addslashes() {
        assert_eq!(addslashes(&json!("it's \"fun\"")), json!("it\\'s \\\"fun\\\""));
    }

    #[test]
    fn test_floatformat() {
        assert_eq!(floatformat(&json!(3.14159), &["2"]), json!("3.14"));
        assert_eq!(floatformat(&json!(3.0), &["0"]), json!("3"));
    }

    #[test]
    fn test_filesizeformat() {
        assert_eq!(filesizeformat(&json!(1024)), json!("1.0 KB"));
        assert_eq!(filesizeformat(&json!(500)), json!("500 B"));
        assert_eq!(filesizeformat(&json!(1048576)), json!("1.0 MB"));
    }

    #[test]
    fn test_first() {
        assert_eq!(first(&json!([1, 2, 3])), json!(1));
        assert_eq!(first(&json!("abc")), json!("a"));
        assert_eq!(first(&json!([])), json!(null));
    }

    #[test]
    fn test_last() {
        assert_eq!(last(&json!([1, 2, 3])), json!(3));
        assert_eq!(last(&json!("abc")), json!("c"));
    }

    #[test]
    fn test_linenumbers() {
        assert_eq!(linenumbers(&json!("a\nb\nc")), json!("1. a\n2. b\n3. c"));
    }

    #[test]
    fn test_linebreaks() {
        let result = linebreaks(&json!("hello\n\nworld")).as_str().unwrap_or("").to_string();
        assert!(result.contains("<p>"));
        assert!(result.contains("<br>"));
    }

    #[test]
    fn test_linebreaksbr() {
        assert_eq!(linebreaksbr(&json!("hello\nworld")), json!("hello<br>\nworld"));
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize(&json!(1), &["s"]), json!(""));
    }

    #[test]
    fn test_pluralize_default() {
        assert_eq!(pluralize(&json!(1), &[]), json!(""));
        assert_eq!(pluralize(&json!(2), &[]), json!("s"));
        assert_eq!(pluralize(&json!(2), &["s"]), json!("s"));
        assert_eq!(pluralize(&json!(1), &["es"]), json!(""));
        assert_eq!(pluralize(&json!(2), &["es"]), json!("es"));
    }

    #[test]
    fn test_make_list() {
        assert_eq!(make_list(&json!("abc")), json!(["a", "b", "c"]));
    }

    #[test]
    fn test_slice() {
        assert_eq!(slice_filter(&json!(["a", "b", "c", "d"]), &["1:3"]), json!(["b", "c"]));
        assert_eq!(slice_filter(&json!("hello"), &["1:4"]), json!("ell"));
    }

    #[test]
    fn test_striptags() {
        assert_eq!(striptags(&json!("<b>Hello</b> <i>world</i>")), json!("Hello world"));
    }

    #[test]
    fn test_truncatechars() {
        assert_eq!(truncatechars(&json!("Hello World!"), &["10"]), json!("Hello W..."));
    }

    #[test]
    fn test_truncatewords() {
        assert_eq!(truncatewords(&json!("This is a long sentence"), &["3"]), json!("This is a ..."));
    }

    #[test]
    fn test_urlencode() {
        assert_eq!(urlencode(&json!("hello world")), json!("hello%20world"));
    }

    #[test]
    fn test_urlize() {
        let result = urlize(&json!("Visit https://example.com")).as_str().unwrap_or("").to_string();
        assert!(result.contains(r#"<a href="https://example.com">"#));
    }

    #[test]
    fn test_wordcount() {
        assert_eq!(wordcount(&json!("one two three")), json!(3));
        assert_eq!(wordcount(&json!("")), json!(0));
    }

    #[test]
    fn test_yesno() {
        assert_eq!(yesno(&json!(true), &[]), json!("yes"));
        assert_eq!(yesno(&json!(false), &[]), json!("no"));
        assert_eq!(yesno(&json!(null), &[]), json!("yes"));
        assert_eq!(yesno(&json!(true), &["ok,nok"]), json!("ok"));
        assert_eq!(yesno(&json!("maybe"), &["yes,no,perhaps"]), json!("perhaps"));
    }

    #[test]
    fn test_divisibleby() {
        assert_eq!(divisibleby(&json!(10), &["5"]), json!(true));
        assert_eq!(divisibleby(&json!(10), &["3"]), json!(false));
    }

    #[test]
    fn test_phone2numeric() {
        assert_eq!(phone2numeric(&json!("800-FLOWERS")), json!("800-3569377"));
    }

    #[test]
    fn test_unordered_list() {
        let data = json!(["item1", ["item2", []]]);
        let result = unordered_list(&data);
        assert!(result.as_str().unwrap_or("").contains("<ul>"));
        assert!(result.as_str().unwrap_or("").contains("<li>item1</li>"));
    }

    #[test]
    fn test_dictsort() {
        let data = json!([
            {"name": "zoe", "age": 25},
            {"name": "alice", "age": 30},
            {"name": "bob", "age": 20}
        ]);
        let sorted = dictsort(&data, &["name"]);
        if let Value::Array(items) = sorted {
            assert_eq!(items[0].get("name").and_then(|v| v.as_str()), Some("alice"));
            assert_eq!(items[1].get("name").and_then(|v| v.as_str()), Some("bob"));
            assert_eq!(items[2].get("name").and_then(|v| v.as_str()), Some("zoe"));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_dictsortreversed() {
        let data = json!([
            {"name": "zoe", "age": 25},
            {"name": "alice", "age": 30},
            {"name": "bob", "age": 20}
        ]);
        let sorted = dictsortreversed(&data, &["name"]);
        if let Value::Array(items) = sorted {
            assert_eq!(items[0].get("name").and_then(|v| v.as_str()), Some("zoe"));
            assert_eq!(items[1].get("name").and_then(|v| v.as_str()), Some("bob"));
            assert_eq!(items[2].get("name").and_then(|v| v.as_str()), Some("alice"));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_dictsortreversed_non_array() {
        let result = dictsortreversed(&json!("hello"), &["key"]);
        assert_eq!(result, json!("hello"));
    }

    #[test]
    fn test_rjust() {
        assert_eq!(rjust(&json!("hi"), &["5"]), json!("   hi"));
    }

    #[test]
    fn test_ljust() {
        assert_eq!(ljust(&json!("hi"), &["5"]), json!("hi   "));
    }

    #[test]
    fn test_center() {
        let result = center(&json!("hi"), &["5"]);
        assert!(result.as_str().map(|s| s.len() == 5).unwrap_or(false));
    }

    #[test]
    fn test_safe() {
        assert_eq!(safe(&json!("<b>bold</b>")), json!("<b>bold</b>"));
    }

    #[test]
    fn test_pprint() {
        let result = pprint(&json!({"a": 1}));
        assert!(result.as_str().unwrap_or("").contains("\"a\""));
    }

    #[test]
    fn test_stringformat() {
        assert_eq!(stringformat(&json!("hello"), &["%s world"]), json!("hello world"));
    }

    #[test]
    fn test_escapejs() {
        let result = escapejs(&json!("it's \"fun\"\n"));
        let s = result.as_str().unwrap();
        assert!(s.contains("\\'"));
        assert!(s.contains("\\\""));
        assert!(s.contains("\\n"));
    }

    #[test]
    fn test_truncatechars_html_strips_tags() {
        let result = truncatechars_html(&json!("<b>Hello</b> <i>World</i>"), &["5"]);
        assert_eq!(result, json!("Hello..."));
    }

    #[test]
    fn test_truncatechars_html_shorter_than_limit() {
        let result = truncatechars_html(&json!("<b>Hi</b>"), &["10"]);
        assert_eq!(result, json!("Hi"));
    }

    #[test]
    fn test_truncatewords_html() {
        let result = truncatewords_html(&json!("<p>One two three four five six</p>"), &["3"]);
        assert_eq!(result, json!("One two three ..."));
    }

    #[test]
    fn test_wordwrap() {
        let result = wordwrap(&json!("this is a long line of text"), &["10"]);
        let s = result.as_str().unwrap();
        assert!(s.contains('\n'));
    }

    #[test]
    fn test_iriencode() {
        let result = iriencode(&json!("hello world"));
        assert!(result.as_str().unwrap().contains("%20"));
    }

    #[test]
    fn test_json_script() {
        let result = json_script(&json!({"key": "value"}), &["mydata"]);
        let s = result.as_str().unwrap();
        assert!(s.contains("<script id=\"mydata\""));
        assert!(s.contains("\"key\":\"value\""));
        assert!(s.contains("</script>"));
    }

    #[test]
    fn test_json_script_default_id() {
        let result = json_script(&json!("plain"), &[]);
        assert!(result.as_str().unwrap().contains("id=\"data\""));
    }

    #[test]
    fn test_safeseq_passthrough() {
        let arr = json!(["a", "b"]);
        assert_eq!(safeseq(&arr), arr);
    }

    #[test]
    fn test_force_escape() {
        assert_eq!(force_escape(&json!("<b>bold</b>")), json!("&lt;b&gt;bold&lt;/b&gt;"));
    }

    #[test]
    fn test_force_escape_ampersand() {
        assert_eq!(force_escape(&json!("a&b")), json!("a&amp;b"));
    }

    #[test]
    fn test_force_escape_quotes() {
        assert_eq!(force_escape(&json!("\"hello\"")), json!("&quot;hello&quot;"));
    }

    #[test]
    fn test_force_escape_multiple() {
        assert_eq!(force_escape(&json!("<a href='x'>")), json!("&lt;a href=&#x27;x&#x27;&gt;"));
    }

    #[test]
    fn test_get_digit_basic() {
        assert_eq!(get_digit(&json!(12345), &["2"]), json!(4));
    }

    #[test]
    fn test_get_digit_first() {
        assert_eq!(get_digit(&json!(12345), &["1"]), json!(5));
    }

    #[test]
    fn test_get_digit_last() {
        assert_eq!(get_digit(&json!(12345), &["5"]), json!(1));
    }

    #[test]
    fn test_get_digit_out_of_range() {
        assert_eq!(get_digit(&json!(123), &["10"]), json!(0));
    }

    #[test]
    fn test_get_digit_zero_pos() {
        assert_eq!(get_digit(&json!(123), &["0"]), json!(0));
    }

    #[test]
    fn test_get_digit_zero_value() {
        assert_eq!(get_digit(&json!(0), &["1"]), json!(0));
    }

    #[test]
    fn test_get_digit_negative_number() {
        // Negative: use absolute value
        assert_eq!(get_digit(&json!(-12345), &["2"]), json!(4));
    }

    #[test]
    fn test_urlizetrunc_basic() {
        let result = urlizetrunc(&json!("Visit https://example.com"), &["20"]);
        let s = result.as_str().unwrap();
        assert!(s.contains(r##"<a href="https://example.com">"##));
        assert!(s.contains("https://example.com"));
    }

    #[test]
    fn test_urlizetrunc_truncated() {
        let long = "https://example.com/very/long/path/that/should/be/truncated";
        let result = urlizetrunc(&json!(long), &["20"]);
        let s = result.as_str().unwrap();
        assert!(s.contains("..."));
        // The display text should be truncated, but the full URL wrapper adds HTML length
        assert!(s.contains(long));  // The href has the full URL
    }

    #[test]
    fn test_urlizetrunc_no_truncate_short() {
        let short = "https://goo.gl/abc";
        let result = urlizetrunc(&json!(short), &["30"]);
        let s = result.as_str().unwrap();
        assert!(!s.contains("..."));
    }

    #[test]
    fn test_urlizetrunc_no_urls() {
        let result = urlizetrunc(&json!("plain text"), &["10"]);
        assert_eq!(result, json!("plain text"));
    }
}
