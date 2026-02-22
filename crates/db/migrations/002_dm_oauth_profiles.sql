-- DM channel membership (for channels with server_id = NULL)
CREATE TABLE dm_members (
    channel_id  UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (channel_id, user_id)
);

CREATE INDEX idx_dm_members_user ON dm_members (user_id);

-- User profile fields
ALTER TABLE users ADD COLUMN IF NOT EXISTS bio TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS banner_url TEXT;

-- OAuth accounts
CREATE TABLE oauth_accounts (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider    TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    email       TEXT,
    access_token TEXT,
    refresh_token TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_id)
);

CREATE INDEX idx_oauth_user ON oauth_accounts (user_id);

-- Read receipts (track last-read message per user per channel)
CREATE TABLE read_states (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id  UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    last_read_id UUID, -- last message id the user has read
    mention_count INT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, channel_id)
);

-- Server bans
CREATE TABLE bans (
    server_id   UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason      TEXT,
    banned_by   UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (server_id, user_id)
);
