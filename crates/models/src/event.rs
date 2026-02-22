use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Channel, Member, Message, PartialUser, Server};

/// Events sent from server to client over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    Ready {
        user: PartialUser,
        servers: Vec<Server>,
        channels: Vec<Channel>,
        members: Vec<Member>,
    },
    Pong {
        ts: u64,
    },

    // Messages
    MessageCreate(Message),
    MessageUpdate {
        id: Uuid,
        channel_id: Uuid,
        content: Option<String>,
        edited_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    MessageDelete {
        id: Uuid,
        channel_id: Uuid,
    },

    // Channels
    ChannelCreate(Channel),
    ChannelUpdate {
        id: Uuid,
        name: Option<String>,
        topic: Option<String>,
    },
    ChannelDelete {
        id: Uuid,
    },

    // Servers
    ServerUpdate {
        id: Uuid,
        name: Option<String>,
        icon_url: Option<String>,
    },

    // Members
    MemberJoin {
        server_id: Uuid,
        user: PartialUser,
    },
    MemberLeave {
        server_id: Uuid,
        user_id: Uuid,
    },

    // Reactions
    ReactionAdd {
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },
    ReactionRemove {
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    // Presence
    PresenceUpdate {
        user_id: Uuid,
        status: crate::UserStatus,
    },

    // Voice
    VoiceJoin {
        channel_id: Uuid,
        user_id: Uuid,
    },
    VoiceLeave {
        channel_id: Uuid,
        user_id: Uuid,
    },

    // Typing
    TypingStart {
        channel_id: Uuid,
        user_id: Uuid,
    },
}

/// Events sent from client to server over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientEvent {
    Authenticate { token: String },
    Ping { ts: u64 },
    TypingStart { channel_id: Uuid },
    Subscribe { channel_id: Uuid },
    UpdatePresence { status: crate::UserStatus },
}
