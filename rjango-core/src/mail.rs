/// Mail module — like Django's `django.core.mail`.
/// Provides `send_mail()`, `EmailMessage`, backends.

use std::collections::HashMap;

/// An email message (like Django's `EmailMessage`).
#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub subject: String,
    pub body: String,
    pub from_email: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub reply_to: Vec<String>,
    pub headers: HashMap<String, String>,
    pub attachments: Vec<(String, Vec<u8>, String)>,  // (filename, content, mimetype)
    pub content_subtype: String,
    pub alternatives: Vec<(String, String)>,  // (content, mimetype)
}

impl EmailMessage {
    pub fn new(subject: &str, body: &str, from_email: &str, to: Vec<String>) -> Self {
        Self {
            subject: subject.to_string(),
            body: body.to_string(),
            from_email: from_email.to_string(),
            to,
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            headers: HashMap::new(),
            attachments: vec![],
            content_subtype: "text/plain".to_string(),
            alternatives: vec![],
        }
    }

    /// Add an attachment.
    pub fn attach(mut self, filename: &str, content: Vec<u8>, mimetype: &str) -> Self {
        self.attachments.push((filename.to_string(), content, mimetype.to_string()));
        self
    }

    /// Set content subtype (e.g., "html").
    pub fn content_subtype(mut self, subtype: &str) -> Self {
        self.content_subtype = subtype.to_string();
        self
    }

    /// Add an alternative representation (like HTML alternative to plain text).
    pub fn attach_alternative(mut self, content: &str, mimetype: &str) -> Self {
        self.alternatives.push((content.to_string(), mimetype.to_string()));
        self
    }
}

/// EmailMultiAlternatives — like Django's `EmailMultiAlternatives`.
#[derive(Debug, Clone)]
pub struct EmailMultiAlternatives {
    pub email: EmailMessage,
}

impl EmailMultiAlternatives {
    pub fn new(subject: &str, body: &str, from_email: &str, to: Vec<String>) -> Self {
        Self {
            email: EmailMessage::new(subject, body, from_email, to)
                .content_subtype("plain"),
        }
    }

    /// Attach an alternative (e.g., HTML version).
    pub fn attach_alternative(mut self, content: &str, mimetype: &str) -> Self {
        self.email = self.email.attach_alternative(content, mimetype);
        self
    }

    /// Send the email.
    pub fn send(&self) -> usize {
        send_email(&self.email)
    }
}

/// A sent email record (for testing/inspection).
#[derive(Debug, Clone)]
pub struct SentEmail {
    pub subject: String,
    pub body: String,
    pub from_email: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub headers: HashMap<String, String>,
    pub content_subtype: String,
    pub alternatives: Vec<(String, String)>,
    pub attachments: Vec<(String, Vec<u8>, String)>,
}

static SENT_MAIL: std::sync::OnceLock<std::sync::Mutex<Vec<SentEmail>>> =
    std::sync::OnceLock::new();

fn sent_mail() -> &'static std::sync::Mutex<Vec<SentEmail>> {
    SENT_MAIL.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// Send an email via the console backend (collects for testing).
pub fn send_email(msg: &EmailMessage) -> usize {
    let sent = SentEmail {
        subject: msg.subject.clone(),
        body: msg.body.clone(),
        from_email: msg.from_email.clone(),
        to: msg.to.clone(),
        cc: msg.cc.clone(),
        bcc: msg.bcc.clone(),
        headers: msg.headers.clone(),
        content_subtype: msg.content_subtype.clone(),
        alternatives: msg.alternatives.clone(),
        attachments: msg.attachments.clone(),
    };
    sent_mail().lock().unwrap().push(sent);
    msg.to.len()
}

/// Send a simple text email.
pub fn send_mail(subject: &str, message: &str, from_email: &str, recipient_list: &[&str]) -> usize {
    let msg = EmailMessage::new(
        subject,
        message,
        from_email,
        recipient_list.iter().map(|s| s.to_string()).collect(),
    );
    send_email(&msg)
}

/// Send mail to admins.
pub fn mail_admins(subject: &str, message: &str) -> usize {
    send_mail(subject, message, "root@example.com", &["admin@example.com"])
}

/// Send mail to managers.
pub fn mail_managers(subject: &str, message: &str) -> usize {
    send_mail(subject, message, "root@example.com", &["manager@example.com"])
}

/// Send mass mail (one at a time, collected).
pub fn send_mass_mail(datatuple: &[(&str, &str, &str, &[&str])]) -> usize {
    let mut total = 0;
    for (subject, message, from_email, recipients) in datatuple {
        total += send_mail(subject, message, from_email, recipients);
    }
    total
}

/// Get all sent emails (for testing).
pub fn get_sent_emails() -> Vec<SentEmail> {
    sent_mail().lock().unwrap().clone()
}

/// Clear sent email records (for testing).
pub fn clear_sent_emails() {
    sent_mail().lock().unwrap().clear();
}

/// BadHeaderError — like Django's `BadHeaderError`.
#[derive(Debug, Clone)]
pub struct BadHeaderError(pub String);

impl std::fmt::Display for BadHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad header: {}", self.0)
    }
}

impl std::error::Error for BadHeaderError {}

/// Prevent header injection.
pub fn forbid_mail_headers(headers: &HashMap<String, String>) -> Result<(), BadHeaderError> {
    for (_, value) in headers {
        if value.contains('\n') || value.contains('\r') {
            return Err(BadHeaderError(value.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_mail() {
        clear_sent_emails();
        send_mail("Subject", "Body", "from@test.com", &["to@test.com"]);
        let sent = get_sent_emails();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].subject, "Subject");
        assert_eq!(sent[0].body, "Body");
    }

    #[test]
    fn test_send_mail_multiple_recipients() {
        clear_sent_emails();
        let count = send_mail("Hello", "World", "a@b.com", &["a@b.com", "c@d.com", "e@f.com"]);
        assert_eq!(count, 3); // returns recipient count
        assert_eq!(get_sent_emails()[0].to.len(), 3);
    }

    #[test]
    fn test_email_message_with_cc_bcc() {
        clear_sent_emails();
        let mut msg = EmailMessage::new("Test", "Body", "from@b.com", vec!["to@b.com".into()]);
        msg.cc.push("cc@b.com".into());
        msg.bcc.push("bcc@b.com".into());
        send_email(&msg);
        let sent = get_sent_emails();
        assert_eq!(sent[0].cc, vec!["cc@b.com"]);
        assert_eq!(sent[0].bcc, vec!["bcc@b.com"]);
    }

    #[test]
    fn test_send_mass_mail() {
        clear_sent_emails();
        let msgs = vec![
            ("Sub1", "Body1", "a@b.com", &["r1@b.com"] as &[&str]),
            ("Sub2", "Body2", "a@b.com", &["r2@b.com"]),
        ];
        send_mass_mail(&msgs);
        assert_eq!(get_sent_emails().len(), 2);
    }

    #[test]
    fn test_mail_admins() {
        clear_sent_emails();
        mail_admins("Alert", "Something happened");
        let sent = get_sent_emails();
        assert_eq!(sent[0].subject, "Alert");
        assert!(sent[0].to.contains(&"admin@example.com".to_string()));
    }

    #[test]
    fn test_mail_managers() {
        clear_sent_emails();
        mail_managers("Manager alert", "Check");
        let sent = get_sent_emails();
        assert_eq!(sent[0].subject, "Manager alert");
        assert!(sent[0].to.contains(&"manager@example.com".to_string()));
    }

    #[test]
    fn test_email_attachment() {
        let msg = EmailMessage::new("With Attach", "Body", "f@b.com", vec!["t@b.com".into()])
            .attach("file.txt", b"content".to_vec(), "text/plain");
        assert_eq!(msg.attachments.len(), 1);
        assert_eq!(msg.attachments[0].0, "file.txt");
    }

    #[test]
    fn test_email_content_subtype() {
        let msg = EmailMessage::new("HTML", "<h1>Hello</h1>", "f@b.com", vec!["t@b.com".into()])
            .content_subtype("html");
        assert_eq!(msg.content_subtype, "html");
    }

    #[test]
    fn test_email_alternatives() {
        let msg = EmailMessage::new("Multi", "Plain text", "f@b.com", vec!["t@b.com".into()])
            .attach_alternative("<h1>HTML</h1>", "text/html");
        assert_eq!(msg.alternatives.len(), 1);
        assert_eq!(msg.alternatives[0].1, "text/html");
    }

    #[test]
    fn test_email_multi_alternatives() {
        clear_sent_emails();
        let msg = EmailMultiAlternatives::new("MultiAlt", "Plain", "from@b.com", vec!["to@b.com".into()])
            .attach_alternative("<b>Rich</b>", "text/html");
        msg.send();
        let sent = get_sent_emails();
        assert_eq!(sent[0].subject, "MultiAlt");
        assert_eq!(sent[0].alternatives.len(), 1);
    }

    #[test]
    fn test_forbid_mail_headers_bad_newline() {
        let mut h = HashMap::new();
        h.insert("X-Custom".into(), "bad\nheader".into());
        let result = forbid_mail_headers(&h);
        assert!(result.is_err());
    }

    #[test]
    fn test_forbid_mail_headers_ok() {
        let mut h = HashMap::new();
        h.insert("X-Custom".into(), "good value".into());
        assert!(forbid_mail_headers(&h).is_ok());
    }
}
