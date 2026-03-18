use macros::include_meta;

pub static META: SiteMeta = include_meta!();

pub struct SiteMeta {
    pub title: &'static str,
    pub description: &'static str,
    pub author: &'static str,
    pub url: &'static str,
    pub pages: &'static [(&'static str, PageMeta)],
}

pub struct PageMeta {
    pub title: &'static str,
    pub description: &'static str,
}

impl SiteMeta {
    pub fn page(&self, key: &str) -> Option<&'static PageMeta> {
        return self.pages.iter().find(|(page_key, _)| *page_key == key).map(|(_, page_meta)| page_meta);
    }

    /// Format a page title like: "λ /dev/lsck0 — page_title"
    pub fn page_title(&self, key: &str) -> String {
        if let Some(page) = self.page(key) {
            if page.title == self.title {
                return format!("\u{03bb} {}", self.title);
            }
            return format!("\u{03bb} {} \u{2014} {}", self.title, page.title);
        }
        return format!("\u{03bb} {}", self.title);
    }
}
