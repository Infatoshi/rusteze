-- Users
CREATE TABLE users (
    id          UUID PRIMARY KEY,
    username    TEXT NOT NULL,
    discriminator TEXT NOT NULL DEFAULT '0000',
    display_name TEXT,
    avatar_url  TEXT,
    email       TEXT UNIQUE,
    phone       TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    flags       INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_username ON users (username);

-- Sessions (replaces authifier)
CREATE TABLE sessions (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL,
    device_name TEXT,
    ip_address  INET,
    last_seen   TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user ON sessions (user_id);
CREATE INDEX idx_sessions_token ON sessions (token_hash);

-- MFA
CREATE TABLE mfa_secrets (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    totp_secret TEXT,
    backup_codes TEXT[] DEFAULT '{}',
    enabled     BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Servers
CREATE TABLE servers (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    owner_id    UUID NOT NULL REFERENCES users(id),
    icon_url    TEXT,
    banner_url  TEXT,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Roles
CREATE TABLE roles (
    id          UUID PRIMARY KEY,
    server_id   UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    color       INT,
    permissions BIGINT NOT NULL DEFAULT 0,
    position    INT NOT NULL DEFAULT 0
);

CREATE INDEX idx_roles_server ON roles (server_id);

-- Members
CREATE TABLE members (
    server_id   UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nickname    TEXT,
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (server_id, user_id)
);

CREATE INDEX idx_members_user ON members (user_id);

-- Member roles (many-to-many)
CREATE TABLE member_roles (
    server_id   UUID NOT NULL,
    user_id     UUID NOT NULL,
    role_id     UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (server_id, user_id, role_id),
    FOREIGN KEY (server_id, user_id) REFERENCES members(server_id, user_id) ON DELETE CASCADE
);

-- Channels
CREATE TABLE channels (
    id          UUID PRIMARY KEY,
    server_id   UUID REFERENCES servers(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    channel_type TEXT NOT NULL DEFAULT 'text',
    topic       TEXT,
    position    INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_channels_server ON channels (server_id);

-- Messages (the big table)
CREATE TABLE messages (
    id          UUID PRIMARY KEY,
    channel_id  UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    author_id   UUID NOT NULL REFERENCES users(id),
    content     TEXT,
    replies_to  UUID REFERENCES messages(id) ON DELETE SET NULL,
    pinned      BOOLEAN NOT NULL DEFAULT false,
    edited_at   TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- UUID v7 is time-ordered, so id DESC = chronological DESC. No separate timestamp index needed.
CREATE INDEX idx_messages_channel ON messages (channel_id, id DESC);
CREATE INDEX idx_messages_author ON messages (author_id);

-- Full-text search on message content
CREATE INDEX idx_messages_search ON messages USING gin(to_tsvector('english', coalesce(content, '')));

-- Attachments
CREATE TABLE attachments (
    id          UUID PRIMARY KEY,
    message_id  UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename    TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size        BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    iv          TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_attachments_message ON attachments (message_id);

-- Reactions
CREATE TABLE reactions (
    message_id  UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji       TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (message_id, user_id, emoji)
);

-- Invites
CREATE TABLE invites (
    code        TEXT PRIMARY KEY,
    server_id   UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    channel_id  UUID REFERENCES channels(id) ON DELETE SET NULL,
    creator_id  UUID NOT NULL REFERENCES users(id),
    max_uses    INT,
    uses        INT NOT NULL DEFAULT 0,
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Push notification queue (replaces RabbitMQ)
CREATE TABLE push_queue (
    id          BIGSERIAL PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    payload     JSONB NOT NULL,
    delivered   BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_push_queue_pending ON push_queue (user_id) WHERE NOT delivered;

-- Updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
