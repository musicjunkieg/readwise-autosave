# AT Protocol API Reference

## Crates (Nick Gerakines)

- `atproto-client` - HTTP client for AT Protocol services
- `atproto-oauth` - OAuth 2.0 (DPoP, PKCE, JWT)
- `atproto-oauth-axum` - Axum integration
- `atproto-identity` - DID/handle resolution
- `atproto-record` - Record operations, AT-URI parsing

## Bookmark API

**Lexicon:** `app.bsky.bookmark.*`

### Get Bookmarks (authenticated)
```
GET app.bsky.bookmark.getBookmarks
  params: limit (1-100, default 50), cursor
  returns: { cursor, bookmarks: [bookmarkView] }

bookmarkView: {
  subject: { uri, cid },
  createdAt: datetime,
  item: postView | blockedPost | notFoundPost
}
```

## DM/Chat API

**Lexicon:** `chat.bsky.convo.*`

**Required header:** `AT-Protocol-Proxy: did:web:api.bsky.chat`

### List Conversations
```
GET chat.bsky.convo.listConvos
  returns: { convos: [convoView] }
```

### Get Messages
```
GET chat.bsky.convo.getMessages
  params: convoId, limit, cursor
  returns: { messages: [messageView] }
```

### Send Message
```
POST chat.bsky.convo.sendMessage
  body: { convoId, message: { text } }
```

## Post/Thread API

**Lexicon:** `app.bsky.feed.*`

### Get Post Thread
```
GET app.bsky.feed.getPostThread
  params: uri, depth (0-1000), parentHeight (0-1000)
  returns: { thread: threadViewPost }
```

Thread detection: check if post record has `reply.root` field.

## OAuth Scopes

- `atproto` - Basic auth (required)
- `transition:generic` - Write/create records
- `transition:chat.bsky` - DM access

## AT-URI Format

```
at://did:plc:xxx/app.bsky.feed.post/yyy
```

Parse with `atproto-record` crate.

---

## Learnings

(Append discoveries here as we implement)
