#![allow(clippy::needless_return)]

use macros::include_posts;

pub static POSTS: &[Post] = include_posts!();

pub struct Post {
    pub slug: &'static str,
    pub folder: &'static str,
    pub metadata: &'static [(&'static str, &'static str)],
    pub content: &'static str,
}

impl Post {
    pub fn get(&self, key: &str) -> Option<&'static str> {
        return self.metadata.iter().find(|(k, _)| *k == key).map(|(_, v)| *v);
    }

    pub fn title(&self) -> &'static str {
        return self.get("title").unwrap_or(self.slug);
    }

    pub fn date(&self) -> &'static str {
        return self.get("date").unwrap_or("");
    }

    pub fn date_formatted(&self) -> String {
        return format_date(self.date());
    }

    pub fn tags(&self) -> Vec<&'static str> {
        return self
            .get("tags")
            .map(|t| t.split(',').map(|s| s.trim()).collect())
            .unwrap_or_default();
    }

    pub fn description(&self) -> &'static str {
        return self.get("description").unwrap_or("");
    }

    pub fn project(&self) -> Option<&'static str> {
        return self.get("project");
    }

    pub fn publication(&self) -> Option<&'static str> {
        return self.get("publication");
    }
}

fn format_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }
    let year = parts[0];
    let month = match parts[1] {
        "01" => "January",
        "02" => "February",
        "03" => "March",
        "04" => "April",
        "05" => "May",
        "06" => "June",
        "07" => "July",
        "08" => "August",
        "09" => "September",
        "10" => "October",
        "11" => "November",
        "12" => "December",
        _ => return date.to_string(),
    };
    let day: u32 = match parts[2].parse() {
        Ok(d) => d,
        Err(_) => return date.to_string(),
    };
    let suffix = match day {
        1 | 21 | 31 => "st",
        2 | 22 => "nd",
        3 | 23 => "rd",
        _ => "th",
    };
    return format!("{day}{suffix} {month} {year}");
}
