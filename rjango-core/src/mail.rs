/// Email framework — like Django's `django.core.mail`.
/// Provides `ConsoleBackend` and `SMTPBackend`.

/// Represents an email message — like Django's `EmailMessage`.
#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub subject: String,
    pub body: String,
    pub from_email: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub reply_to: Vec<String>,
    pub headers: Vec<(String, String)>,
}

impl EmailMessage {
    pub fn new(subject: &str, body: &str, from_email: &str, to: Vec<String>) -> Self {
        Self {
            subject: subject.to_string(),
            body: body.to_string(),
            from_email: from_email.to_string(),
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: Vec::new(),
            headers: Vec::new(),
        }
    }

    /// Build the raw SMTP message (RFC 2822).
    pub fn to_smtp_string(&self) -> String {
        let mut msg = String::new();
        msg.push_str(&format!("From: {}\r\n", self.from_email));
        msg.push_str(&format!("To: {}\r\n", self.to.join(", ")));
        if !self.cc.is_empty() {
            msg.push_str(&format!("Cc: {}\r\n", self.cc.join(", ")));
        }
        msg.push_str(&format!("Subject: {}\r\n", self.subject));
        msg.push_str("MIME-Version: 1.0\r\n");
        msg.push_str("Content-Type: text/plain; charset=\"UTF-8\"\r\n");
        msg.push_str("Content-Transfer-Encoding: 7bit\r\n");
        msg.push_str(&format!("Date: {}\r\n", http_date()));
        for (k, v) in &self.headers {
            msg.push_str(&format!("{}: {}\r\n", k, v));
        }
        msg.push_str("\r\n");
        msg.push_str(&self.body);
        msg
    }
}

/// Email backend trait — like Django's `EMAIL_BACKEND`.
pub trait EmailBackend: Send + Sync {
    fn send(&self, message: &EmailMessage) -> Result<(), String>;
}

/// Console backend — prints emails to stdout (like Django's `ConsoleBackend`).
pub struct ConsoleBackend;

impl EmailBackend for ConsoleBackend {
    fn send(&self, message: &EmailMessage) -> Result<(), String> {
        println!("[Rjango Email Console Backend]");
        println!("Subject: {}", message.subject);
        println!("From: {}", message.from_email);
        println!("To: {}", message.to.join(", "));
        println!("--- Body ---");
        println!("{}", message.body);
        println!("--- End ---");
        Ok(())
    }
}

/// SMTP backend — sends via SMTP (like Django's `SMTPBackend`).
pub struct SMTPBackend {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Default for SMTPBackend {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 25,
            username: None,
            password: None,
            use_tls: false,
        }
    }
}

impl SMTPBackend {
    pub fn new(host: &str, port: u16, username: Option<&str>, password: Option<&str>, use_tls: bool) -> Self {
        Self {
            host: host.to_string(),
            port,
            username: username.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
            use_tls,
        }
    }
}

impl EmailBackend for SMTPBackend {
    fn send(&self, message: &EmailMessage) -> Result<(), String> {
        // Basic SMTP sending via std::net::TcpStream
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| format!("Invalid addr: {}", e))?,
            Duration::from_secs(10),
        ).map_err(|e| format!("Connection failed: {}", e))?;

        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
        let mut reader = BufReader::new(&stream);
        let mut writer = &stream;

        // Read greeting
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| format!("Read error: {}", e))?;

        // EHLO
        writeln!(writer, "EHLO rjango").map_err(|e| format!("Write error: {}", e))?;
        reader.read_line(&mut line).ok();
        reader.read_line(&mut line).ok();

        // Auth if needed
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            writeln!(writer, "AUTH LOGIN").map_err(|e| format!("Auth error: {}", e))?;
            reader.read_line(&mut line).ok();
            writeln!(writer, "{}", base64_encode(user.as_bytes())).map_err(|e| format!("Auth error: {}", e))?;
            reader.read_line(&mut line).ok();
            writeln!(writer, "{}", base64_encode(pass.as_bytes())).map_err(|e| format!("Auth error: {}", e))?;
            reader.read_line(&mut line).ok();
        }

        // MAIL FROM
        writeln!(writer, "MAIL FROM:<{}>", message.from_email).map_err(|e| format!("MAIL FROM error: {}", e))?;
        reader.read_line(&mut line).ok();

        // RCPT TO
        for recipient in &message.to {
            writeln!(writer, "RCPT TO:<{}>", recipient).map_err(|e| format!("RCPT TO error: {}", e))?;
            reader.read_line(&mut line).ok();
        }

        // DATA
        writeln!(writer, "DATA").map_err(|e| format!("DATA error: {}", e))?;
        reader.read_line(&mut line).ok();
        let smtp_msg = message.to_smtp_string();
        writeln!(writer, "{}\r\n.", smtp_msg).map_err(|e| format!("Write body error: {}", e))?;
        reader.read_line(&mut line).ok();

        // QUIT
        writeln!(writer, "QUIT").map_err(|e| format!("QUIT error: {}", e))?;

        Ok(())
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    // Manual Base64 encoding (no external dep needed for this simple case)
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn http_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    // Simple RFC 2822 date (not perfect, but sufficient)
    format!("{}", secs)
}

/// Convenience function — like Django's `send_mail()`.
pub fn send_mail(
    backend: &dyn EmailBackend,
    subject: &str,
    body: &str,
    from_email: &str,
    recipient_list: Vec<String>,
) -> Result<(), String> {
    let msg = EmailMessage::new(subject, body, from_email, recipient_list);
    backend.send(&msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_message_creation() {
        let msg = EmailMessage::new("Test", "Hello", "a@b.com", vec!["c@d.com".to_string()]);
        assert_eq!(msg.subject, "Test");
        assert_eq!(msg.to, vec!["c@d.com"]);
    }

    #[test]
    fn test_smtp_string_format() {
        let msg = EmailMessage::new("Subject", "Body", "from@test.com", vec!["to@test.com".to_string()]);
        let smtp = msg.to_smtp_string();
        assert!(smtp.contains("From: from@test.com"));
        assert!(smtp.contains("To: to@test.com"));
        assert!(smtp.contains("Subject: Subject"));
        assert!(smtp.contains("Body"));
    }

    #[test]
    fn test_console_backend() {
        let backend = ConsoleBackend;
        let msg = EmailMessage::new("Test", "Body", "a@b.com", vec!["c@d.com".to_string()]);
        // Just verify it doesn't panic
        let result = backend.send(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_base64_encode() {
        let encoded = base64_encode(b"hello");
        assert_eq!(encoded, "aGVsbG8=");
    }

    #[test]
    fn test_send_mail_convenience() {
        let backend = ConsoleBackend;
        let result = send_mail(&backend, "Hi", "There", "a@b.com", vec!["b@c.com".to_string()]);
        assert!(result.is_ok());
    }
}
