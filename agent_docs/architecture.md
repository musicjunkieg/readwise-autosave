# Architecture

## Overview

readwise-autosave is a centralized Rust service that integrates Bluesky with Readwise.

```
┌─────────────────────────────────────────────────────────────┐
│                        Web Layer (axum)                      │
├─────────────────────────────────────────────────────────────┤
│  GET  /                    → Landing page                    │
│  GET  /auth/login          → Initiate Bluesky OAuth          │
│  GET  /auth/callback       → Handle OAuth callback           │
│  GET  /dashboard           → User settings page              │
│  POST /api/settings        → Update user preferences         │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│              Background Services (tokio tasks)               │
├─────────────────────────────────────────────────────────────┤
│  Bookmark Poller     │ Poll user bookmarks via OAuth         │
│  DM Poller           │ Poll bot account DMs                  │
│  Post Processor      │ Fetch posts, detect threads           │
│  Content Formatter   │ Format for Readwise APIs              │
│  Readwise Client     │ Save to Highlights/Reader             │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                     Database (PostgreSQL)                    │
├─────────────────────────────────────────────────────────────┤
│  users, user_tokens, user_settings                           │
│  processed_bookmarks, processed_dms                          │
└─────────────────────────────────────────────────────────────┘
```

## Components

- **web/**: axum handlers, OAuth flow, dashboard UI
- **services/**: bookmark_sync, dm_bot, processor
- **bluesky/**: AT Protocol client, bookmarks, chat APIs
- **readwise/**: Readwise Highlights (v2) and Reader (v3) APIs
- **content/**: Post/thread formatters, link extraction

## Data Flow

1. Trigger detected (bookmark polled or DM received)
2. Post URI extracted
3. Full post fetched via `app.bsky.feed.getPostThread`
4. Thread detection: check if post has `reply.root`
5. Content formatted for Readwise API
6. Saved: single post → Highlights, thread → Reader

## Design Decisions

- **Trait-based abstractions** for testability (mock external APIs)
- **Background tasks** via `tokio::spawn` for polling
- **PostgreSQL** for persistence with sqlx
- **Nick Gerakines' crates** for AT Protocol (atproto-client, atproto-oauth)
