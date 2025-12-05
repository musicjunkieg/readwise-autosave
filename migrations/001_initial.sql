-- Initial database schema for readwise-autosave

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bluesky_did TEXT UNIQUE NOT NULL,
    bluesky_handle TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- OAuth tokens (should be encrypted at rest in production)
CREATE TABLE IF NOT EXISTS user_tokens (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- User settings
CREATE TABLE IF NOT EXISTS user_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    readwise_token TEXT NOT NULL,
    bookmark_sync_enabled BOOLEAN DEFAULT TRUE NOT NULL,
    extract_links BOOLEAN DEFAULT FALSE NOT NULL,
    last_bookmark_cursor TEXT,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Processed bookmarks (for deduplication)
CREATE TABLE IF NOT EXISTS processed_bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_uri TEXT NOT NULL,
    processed_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE(user_id, post_uri)
);

-- Processed DMs
CREATE TABLE IF NOT EXISTS processed_dms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    message_id TEXT UNIQUE NOT NULL,
    post_uri TEXT,
    status TEXT NOT NULL,
    processed_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_users_did ON users(bluesky_did);
CREATE INDEX IF NOT EXISTS idx_processed_bookmarks_user ON processed_bookmarks(user_id);
CREATE INDEX IF NOT EXISTS idx_processed_dms_message ON processed_dms(message_id);
