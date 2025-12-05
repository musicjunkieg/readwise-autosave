//! Content formatter
//!
//! Converts Bluesky posts and threads into Readwise API payloads.

use crate::bluesky::{PostView, ThreadViewPost};
use crate::readwise::client::{Document, Highlight};

/// Format a single post as a Readwise highlight
pub fn format_post_as_highlight(post: &PostView, note: Option<&str>) -> Highlight {
    let author_name = post
        .author
        .display_name
        .clone()
        .unwrap_or_else(|| post.author.handle.clone());

    let source_url = format!(
        "https://bsky.app/profile/{}/post/{}",
        post.author.handle,
        extract_rkey(&post.uri)
    );

    Highlight {
        text: post.record.text.clone(),
        title: Some(format!("Post by @{}", post.author.handle)),
        author: Some(author_name),
        source_url: Some(source_url),
        category: Some("tweets".to_string()),
        note: note.map(|s| s.to_string()),
    }
}

/// Format a thread as a Readwise Reader document
pub fn format_thread_as_document(thread: &ThreadViewPost) -> Document {
    let posts = collect_thread_posts(thread);
    let html = format_posts_as_html(&posts);

    let first_post = posts.first().map(|p| &p.post);
    let (title, author, source_url) = if let Some(post) = first_post {
        let author_name = post
            .author
            .display_name
            .clone()
            .unwrap_or_else(|| post.author.handle.clone());
        let url = format!(
            "https://bsky.app/profile/{}/post/{}",
            post.author.handle,
            extract_rkey(&post.uri)
        );
        (
            format!("Thread by @{}", post.author.handle),
            author_name,
            url,
        )
    } else {
        ("Thread".to_string(), "Unknown".to_string(), String::new())
    };

    Document {
        url: source_url,
        html: Some(html),
        title: Some(title),
        author: Some(author),
        tags: Some(vec!["bluesky".to_string(), "thread".to_string()]),
    }
}

/// Collect all posts in a thread (from root to leaves)
fn collect_thread_posts(thread: &ThreadViewPost) -> Vec<&ThreadViewPost> {
    let mut posts = Vec::new();

    // First, collect parent chain (going up)
    let mut current = thread.parent.as_ref();
    let mut parent_chain = Vec::new();
    while let Some(parent) = current {
        parent_chain.push(parent.as_ref());
        current = parent.parent.as_ref();
    }
    parent_chain.reverse();
    posts.extend(parent_chain);

    // Add the current post
    posts.push(thread);

    // Add replies (going down) - just first level for now
    if let Some(replies) = &thread.replies {
        for reply in replies {
            posts.push(reply);
        }
    }

    posts
}

/// Format posts as HTML article
fn format_posts_as_html(posts: &[&ThreadViewPost]) -> String {
    let mut html = String::from("<article class=\"bluesky-thread\">\n");

    for post in posts {
        let author_name = post
            .post
            .author
            .display_name
            .clone()
            .unwrap_or_else(|| post.post.author.handle.clone());

        html.push_str(&format!(
            r#"<div class="post">
<p class="author"><strong>{}</strong> <span class="handle">@{}</span></p>
<p class="content">{}</p>
<p class="timestamp">{}</p>
</div>
"#,
            html_escape(&author_name),
            html_escape(&post.post.author.handle),
            html_escape(&post.post.record.text),
            post.post.record.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
    }

    html.push_str("</article>");
    html
}

/// Extract rkey from AT-URI (at://did/collection/rkey)
fn extract_rkey(uri: &str) -> String {
    uri.split('/').last().unwrap_or("").to_string()
}

/// Basic HTML escaping
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Check if a post is part of a thread
pub fn is_thread(post: &PostView) -> bool {
    post.record.reply.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rkey() {
        let uri = "at://did:plc:abc123/app.bsky.feed.post/xyz789";
        assert_eq!(extract_rkey(uri), "xyz789");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
