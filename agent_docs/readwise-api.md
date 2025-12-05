# Readwise API Reference

## Authentication

All requests require:
```
Authorization: Token YOUR_ACCESS_TOKEN
```

Get token from: https://readwise.io/access_token

## Highlights API (v2)

**Endpoint:** `POST https://readwise.io/api/v2/highlights/`

**Rate limit:** 240 requests/minute

### Save Highlight

```json
{
  "highlights": [{
    "text": "The quote or content to save (required, max 8191 chars)",
    "title": "Source title (optional, max 511 chars)",
    "author": "Author name (optional, max 1024 chars)",
    "source_url": "https://... (optional, max 2047 chars)",
    "source_type": "string (optional, 3-64 chars)",
    "category": "books|articles|tweets|podcasts|emails",
    "note": "Your note (optional, max 8191 chars)",
    "highlighted_at": "ISO 8601 datetime (optional)"
  }]
}
```

For Bluesky posts, use:
- `category`: "tweets"
- `source_url`: Bluesky post URL
- `title`: "Post by @handle"
- `author`: Display name

## Reader API (v3)

**Endpoint:** `POST https://readwise.io/api/v3/save/`

**Rate limit:** 50 requests/minute

### Save Document

```json
{
  "url": "https://... (required)",
  "html": "<article>...</article> (optional, for full content)",
  "title": "Custom title (optional)",
  "author": "Author name (optional)",
  "tags": ["tag1", "tag2"],
  "location": "new|later|archive|feed",
  "category": "article|email|pdf|epub|tweet|video"
}
```

For threads, provide formatted HTML with all posts.

## Verify Token

```
GET https://readwise.io/api/v2/auth/
Authorization: Token XXX
```
Returns 204 if valid.

---

## Learnings

(Append discoveries here as we implement)
