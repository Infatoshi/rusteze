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
}
