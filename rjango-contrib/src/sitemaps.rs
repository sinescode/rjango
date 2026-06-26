
/// A single sitemap entry.
#[derive(Debug, Clone)]
pub struct SitemapEntry {
    pub location: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}

impl SitemapEntry {
    pub fn new(location: &str) -> Self {
        Self {
            location: location.to_string(),
            lastmod: None,
            changefreq: None,
            priority: None,
        }
    }

    pub fn lastmod(mut self, date: &str) -> Self {
        self.lastmod = Some(date.to_string());
        self
    }

    pub fn changefreq(mut self, freq: &str) -> Self {
        self.changefreq = Some(freq.to_string());
        self
    }

    pub fn priority(mut self, p: f32) -> Self {
        self.priority = Some(p.clamp(0.0, 1.0));
        self
    }
}

/// A sitemap section.
#[derive(Debug, Clone)]
pub struct Sitemap {
    pub section: String,
    pub entries: Vec<SitemapEntry>,
}

impl Sitemap {
    pub fn new(section: &str) -> Self {
        Self { section: section.to_string(), entries: vec![] }
    }

    pub fn add(&mut self, entry: SitemapEntry) {
        self.entries.push(entry);
    }

    /// Render as XML sitemap.
    pub fn render_xml(&self) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        );
        xml.push_str(
            r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
        );
        for entry in &self.entries {
            xml.push_str("<url>");
            xml.push_str(&format!("<loc>{}</loc>", entry.location));
            if let Some(ref lm) = entry.lastmod {
                xml.push_str(&format!("<lastmod>{}</lastmod>", lm));
            }
            if let Some(ref cf) = entry.changefreq {
                xml.push_str(&format!("<changefreq>{}</changefreq>", cf));
            }
            if let Some(p) = entry.priority {
                xml.push_str(&format!("<priority>{:.1}</priority>", p));
            }
            xml.push_str("</url>");
        }
        xml.push_str("</urlset>");
        xml
    }

    /// Render a sitemap index (multiple sitemaps).
    pub fn render_index(sitemaps: &[Sitemap]) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        );
        xml.push_str(
            r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
        );
        for sm in sitemaps {
            xml.push_str("<sitemap>");
            xml.push_str(&format!("<loc>/sitemap-{}.xml</loc>", sm.section));
            xml.push_str("</sitemap>");
        }
        xml.push_str("</sitemapindex>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sitemap_entry_basic() {
        let e = SitemapEntry::new("/about/");
        assert_eq!(e.location, "/about/");
    }

    #[test]
    fn test_sitemap_entry_builder() {
        let e = SitemapEntry::new("/blog/")
            .lastmod("2026-06-26")
            .changefreq("weekly")
            .priority(0.8);
        assert_eq!(e.lastmod, Some("2026-06-26".into()));
        assert_eq!(e.changefreq, Some("weekly".into()));
        assert_eq!(e.priority, Some(0.8));
    }

    #[test]
    fn test_priority_clamped() {
        let e = SitemapEntry::new("/").priority(2.0);
        assert_eq!(e.priority, Some(1.0));
    }

    #[test]
    fn test_render_xml() {
        let mut sm = Sitemap::new("main");
        sm.add(SitemapEntry::new("/home/").priority(1.0));
        sm.add(SitemapEntry::new("/contact/"));
        let xml = sm.render_xml();
        assert!(xml.contains("<loc>/home/</loc>"));
        assert!(xml.contains("<priority>1.0</priority>"));
        assert!(xml.contains("</urlset>"));
    }

    #[test]
    fn test_render_index() {
        let sm1 = Sitemap::new("pages");
        let sm2 = Sitemap::new("posts");
        let xml = Sitemap::render_index(&[sm1, sm2]);
        assert!(xml.contains("sitemapindex"));
        assert!(xml.contains("/sitemap-pages.xml"));
        assert!(xml.contains("/sitemap-posts.xml"));
    }

    #[test]
    fn test_sitemap_add_multiple() {
        let mut sm = Sitemap::new("test");
        for i in 0..5 {
            sm.add(SitemapEntry::new(&format!("/page/{}/", i)));
        }
        assert_eq!(sm.entries.len(), 5);
    }

    #[test]
    fn test_empty_sitemap() {
        let sm = Sitemap::new("empty");
        let xml = sm.render_xml();
        assert!(xml.contains("<urlset"));
        assert!(xml.contains("</urlset>"));
    }
}
