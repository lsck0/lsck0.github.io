use serde::Serialize;

use crate::parse::ContentPost;

// ============================================================
// Graph data
// ============================================================

#[derive(Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Serialize)]
pub struct GraphNode {
    pub slug: String,
    pub title: String,
    pub tags: Vec<String>,
    pub date: String,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Reference,
    Series,
}

pub fn build_graph_data(posts: &[&ContentPost]) -> GraphData {
    let all_slugs: Vec<&str> = posts.iter().map(|p| p.slug.as_str()).collect();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for post in posts {
        nodes.push(GraphNode {
            slug: post.slug.clone(),
            title: post.title().to_string(),
            tags: post.tags().iter().map(|t| t.to_string()).collect(),
            date: post.date().to_string(),
        });

        for linked in &post.internal_links {
            if all_slugs.contains(&linked.as_str()) {
                edges.push(GraphEdge {
                    source: post.slug.clone(),
                    target: linked.clone(),
                    kind: EdgeKind::Reference,
                });
            }
        }
    }

    // Series edges
    let mut series_groups: std::collections::HashMap<&str, Vec<&ContentPost>> = std::collections::HashMap::new();
    for post in posts {
        if let Some(name) = post.series() {
            series_groups.entry(name).or_default().push(post);
        }
    }
    for (_, mut members) in series_groups {
        members.sort_by_key(|p| p.series_order().unwrap_or(0));
        for window in members.windows(2) {
            edges.push(GraphEdge {
                source: window[0].slug.clone(),
                target: window[1].slug.clone(),
                kind: EdgeKind::Series,
            });
        }
    }

    return GraphData { nodes, edges };
}

// ============================================================
// Search index
// ============================================================

#[derive(Serialize)]
pub struct SearchEntry {
    pub slug: String,
    pub title: String,
    pub tags: Vec<String>,
    pub description: String,
    pub date: String,
}

pub fn build_search_index(posts: &[&ContentPost]) -> Vec<SearchEntry> {
    return posts
        .iter()
        .map(|p| SearchEntry {
            slug: p.slug.clone(),
            title: p.title().to_string(),
            tags: p.tags().iter().map(|t| t.to_string()).collect(),
            description: p.description().to_string(),
            date: p.date().to_string(),
        })
        .collect();
}
