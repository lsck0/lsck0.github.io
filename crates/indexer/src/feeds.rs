use crate::parse::ContentPost;

// ============================================================
// RSS feed
// ============================================================

pub fn build_rss_feed(posts: &[&ContentPost], site_url: &str, site_title: &str) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n");
    out.push_str("<channel>\n");
    out.push_str(&format!("  <title>{site_title}</title>\n"));
    out.push_str(&format!("  <link>{site_url}</link>\n"));
    out.push_str(&format!(
        "  <atom:link href=\"{site_url}/rss.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n"
    ));
    out.push_str(&format!("  <description>{site_title}</description>\n"));

    for post in posts {
        let url = format!("{site_url}/blog/{}", post.slug);
        let title = xml_escape(post.title());
        let desc = xml_escape(post.description());
        out.push_str("  <item>\n");
        out.push_str(&format!("    <title>{title}</title>\n"));
        out.push_str(&format!("    <link>{url}</link>\n"));
        out.push_str(&format!("    <guid>{url}</guid>\n"));
        out.push_str(&format!("    <pubDate>{}</pubDate>\n", rfc822_date(post.date())));
        out.push_str(&format!("    <description>{desc}</description>\n"));
        out.push_str("  </item>\n");
    }

    out.push_str("</channel>\n");
    out.push_str("</rss>\n");
    return out;
}

// ============================================================
// Atom feed
// ============================================================

pub fn build_atom_feed(posts: &[&ContentPost], site_url: &str, site_title: &str) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    out.push_str(&format!("  <title>{site_title}</title>\n"));
    out.push_str(&format!("  <link href=\"{site_url}\" rel=\"alternate\"/>\n"));
    out.push_str(&format!(
        "  <link href=\"{site_url}/atom.xml\" rel=\"self\" type=\"application/atom+xml\"/>\n"
    ));
    out.push_str(&format!("  <id>{site_url}/</id>\n"));

    let latest = posts.iter().map(|p| p.date()).next().unwrap_or("");
    out.push_str(&format!("  <updated>{}</updated>\n", iso8601_date(latest)));

    for post in posts {
        let url = format!("{site_url}/blog/{}", post.slug);
        let title = xml_escape(post.title());
        let desc = xml_escape(post.description());
        out.push_str("  <entry>\n");
        out.push_str(&format!("    <title>{title}</title>\n"));
        out.push_str(&format!("    <link href=\"{url}\" rel=\"alternate\"/>\n"));
        out.push_str(&format!("    <id>{url}</id>\n"));
        out.push_str(&format!("    <updated>{}</updated>\n", iso8601_date(post.date())));
        out.push_str(&format!("    <summary>{desc}</summary>\n"));
        out.push_str("  </entry>\n");
    }

    out.push_str("</feed>\n");
    return out;
}

// ============================================================
// Sitemap
// ============================================================

pub fn build_sitemap(posts: &[&ContentPost], site_url: &str) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    let static_pages = [
        "",
        "/about",
        "/blog",
        "/projects",
        "/publications",
        "/graph",
        "/imprint",
        "/privacy",
        "/tos",
    ];
    for page in &static_pages {
        out.push_str(&format!("  <url><loc>{site_url}{page}</loc></url>\n"));
    }

    for post in posts {
        let url = format!("{site_url}/blog/{}", post.slug);
        out.push_str(&format!(
            "  <url><loc>{url}</loc><lastmod>{}</lastmod></url>\n",
            post.date()
        ));
    }

    out.push_str("</urlset>\n");
    return out;
}

// ============================================================
// OpenGraph metadata
// ============================================================

pub struct OpenGraphEntry {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub date: String,
    pub image: String,
}

pub fn build_opengraph_metadata(posts: &[&ContentPost], site_url: &str, site_image: &str) -> Vec<OpenGraphEntry> {
    return posts
        .iter()
        .map(|post| OpenGraphEntry {
            slug: post.slug.clone(),
            title: post.title().to_string(),
            description: post.description().to_string(),
            url: format!("{site_url}/blog/{}", post.slug),
            date: post.date().to_string(),
            image: site_image.to_string(),
        })
        .collect();
}

pub fn inject_opengraph_tags(base_html: &str, entry: &OpenGraphEntry, site_title: &str) -> String {
    let title = xml_escape(&entry.title);
    let desc = xml_escape(&entry.description);
    let url = xml_escape(&entry.url);
    let image = xml_escape(&entry.image);

    // Remove existing OG/meta tags that we'll replace
    let mut result = base_html.to_string();
    for pattern in ["og:title", "og:description", "og:url", "og:image", "og:type"] {
        let search = format!("<meta property=\"{pattern}\"");
        if let Some(start) = result.find(&search)
            && let Some(end) = result[start..].find("/>")
        {
            result = format!("{}{}", &result[..start], &result[start + end + 2..]);
        }
    }
    // Replace description meta
    if let Some(start) = result.find("<meta name=\"description\"")
        && let Some(end) = result[start..].find("/>")
    {
        result = format!("{}{}", &result[..start], &result[start + end + 2..]);
    }

    let tags = format!(
        r#"    <meta property="og:type" content="article"/>
    <meta property="og:title" content="{title}"/>
    <meta property="og:description" content="{desc}"/>
    <meta property="og:url" content="{url}"/>
    <meta property="og:image" content="{image}"/>
    <meta property="og:site_name" content="{site_title}"/>
    <meta name="twitter:card" content="summary"/>
    <meta name="twitter:title" content="{title}"/>
    <meta name="twitter:description" content="{desc}"/>
    <meta name="description" content="{desc}"/>
"#
    );

    return result.replace("</head>", &format!("{tags}  </head>"));
}

// ============================================================
// JSON-LD structured data
// ============================================================

pub fn build_jsonld(entry: &OpenGraphEntry, site_url: &str) -> String {
    let title = json_escape(&entry.title);
    let desc = json_escape(&entry.description);
    let image = json_escape(&entry.image);
    return format!(
        r#"<script type="application/ld+json">
{{
  "@context": "https://schema.org",
  "@type": "BlogPosting",
  "headline": "{title}",
  "author": {{ "@type": "Person", "name": "Luca Sandrock" }},
  "datePublished": "{}",
  "description": "{desc}",
  "url": "{}",
  "image": "{image}",
  "breadcrumb": {{
    "@type": "BreadcrumbList",
    "itemListElement": [
      {{ "@type": "ListItem", "position": 1, "name": "Home", "item": "{site_url}/" }},
      {{ "@type": "ListItem", "position": 2, "name": "Blog", "item": "{site_url}/blog" }}
    ]
  }}
}}
</script>"#,
        entry.date, entry.url
    );
}

// ============================================================
// Helpers
// ============================================================

fn xml_escape(input: &str) -> String {
    return input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;");
}

fn json_escape(input: &str) -> String {
    return input.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
}

fn rfc822_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }
    let month = match parts[1] {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        _ => return date.to_string(),
    };
    return format!("{} {} {} 00:00:00 +0000", parts[2], month, parts[0]);
}

fn iso8601_date(date: &str) -> String {
    if date.contains('T') {
        return date.to_string();
    }
    return format!("{date}T00:00:00Z");
}
