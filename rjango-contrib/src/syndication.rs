
/// A feed entry.
#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub author: Option<String>,
    pub pub_date: Option<String>,
    pub unique_id: Option<String>,
}

impl FeedItem {
    pub fn new(title: &str, link: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            link: link.to_string(),
            description: description.to_string(),
            author: None,
            pub_date: None,
            unique_id: None,
        }
    }

    pub fn author(mut self, name: &str) -> Self {
        self.author = Some(name.to_string());
        self
    }

    pub fn pub_date(mut self, date: &str) -> Self {
        self.pub_date = Some(date.to_string());
        self
    }

    pub fn unique_id(mut self, id: &str) -> Self {
        self.unique_id = Some(id.to_string());
        self
    }
}

/// A feed (RSS 2.0 + Atom).
#[derive(Debug, Clone)]
pub struct Feed {
    pub title: String,
    pub link: String,
    pub description: String,
    pub language: String,
    pub items: Vec<FeedItem>,
}

impl Feed {
    pub fn new(title: &str, link: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            link: link.to_string(),
            description: description.to_string(),
            language: "en".to_string(),
            items: vec![],
        }
    }

    pub fn add(&mut self, item: FeedItem) {
        self.items.push(item);
    }

    /// Render as RSS 2.0 XML.
    pub fn render_rss(&self) -> String {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<rss version="2.0">"#);
        xml.push_str("<channel>");
        xml.push_str(&format!("<title>{}</title>", self.title));
        xml.push_str(&format!("<link>{}</link>", self.link));
        xml.push_str(&format!("<description>{}</description>", self.description));
        xml.push_str(&format!("<language>{}</language>", self.language));
        for item in &self.items {
            xml.push_str("<item>");
            xml.push_str(&format!("<title>{}</title>", item.title));
            xml.push_str(&format!("<link>{}</link>", item.link));
            xml.push_str(&format!("<description>{}</description>", item.description));
            if let Some(ref a) = item.author {
                xml.push_str(&format!("<author>{}</author>", a));
            }
            if let Some(ref d) = item.pub_date {
                xml.push_str(&format!("<pubDate>{}</pubDate>", d));
            }
            if let Some(ref g) = item.unique_id {
                xml.push_str(&format!("<guid>{}</guid>", g));
            }
            xml.push_str("</item>");
        }
        xml.push_str("</channel>");
        xml.push_str("</rss>");
        xml
    }

    /// Render as Atom XML.
    pub fn render_atom(&self) -> String {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<feed xmlns="http://www.w3.org/2005/Atom">"#);
        xml.push_str(&format!("<title>{}</title>", self.title));
        xml.push_str(&format!("<link href=\"{}\"/>", self.link));
        xml.push_str(&format!("<subtitle>{}</subtitle>", self.description));
        for item in &self.items {
            let id = item.unique_id.as_deref().unwrap_or(&item.link);
            let author = item.author.as_deref().unwrap_or("Anonymous");
            xml.push_str("<entry>");
            xml.push_str(&format!("<title>{}</title>", item.title));
            xml.push_str(&format!("<link href=\"{}\"/>", item.link));
            xml.push_str(&format!("<id>{}</id>", id));
            xml.push_str(&format!("<author><name>{}</name></author>", author));
            xml.push_str(&format!("<summary>{}</summary>", item.description));
            if let Some(ref d) = item.pub_date {
                xml.push_str(&format!("<published>{}</published>", d));
            }
            xml.push_str("</entry>");
        }
        xml.push_str("</feed>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_item_basic() {
        let item = FeedItem::new("Post 1", "https://example.com/1", "First post");
        assert_eq!(item.title, "Post 1");
    }

    #[test]
    fn test_feed_item_builder() {
        let item = FeedItem::new("Post", "https://x.com/p", "desc")
            .author("Yaseen")
            .pub_date("Mon, 01 Jan 2026 00:00:00 GMT")
            .unique_id("tag:x,2026:post");
        assert_eq!(item.author, Some("Yaseen".into()));
    }

    #[test]
    fn test_render_rss() {
        let mut feed = Feed::new("My Blog", "https://example.com/", "A blog");
        feed.add(FeedItem::new("Hello", "https://example.com/hello", "First post"));
        let rss = feed.render_rss();
        assert!(rss.contains("<rss version=\"2.0\">"));
        assert!(rss.contains("Hello"));
        assert!(rss.contains("</rss>"));
    }

    #[test]
    fn test_render_atom() {
        let mut feed = Feed::new("Atom Feed", "https://example.com/", "An Atom feed");
        feed.add(FeedItem::new("Atom Post", "https://example.com/atom", "Content"));
        let atom = feed.render_atom();
        assert!(atom.contains("<feed xmlns="));
        assert!(atom.contains("Atom Post"));
        assert!(atom.contains("</feed>"));
    }

    #[test]
    fn test_multiple_items() {
        let mut feed = Feed::new("Blog", "https://example.com/", "Multi");
        for i in 0..10 {
            feed.add(FeedItem::new(
                &format!("Post {}", i),
                &format!("https://example.com/{}", i),
                &format!("Description {}", i),
            ));
        }
        assert_eq!(feed.items.len(), 10);
        let rss = feed.render_rss();
        assert_eq!(rss.matches("<item>").count(), 10);
    }

    #[test]
    fn test_empty_feed() {
        let feed = Feed::new("Empty", "https://example.com/", "No items");
        let rss = feed.render_rss();
        assert!(rss.contains("<channel>"));
        assert!(rss.contains("</channel>"));
    }
}
