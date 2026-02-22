use rusteze_models::{Channel, ChannelType, ClientEvent, Member, Message, PartialUser, Server};
use uuid::Uuid;

/// Central application state, owned by GPUI as an Entity.
pub struct AppState {
    /// Currently authenticated user (None = show login screen).
    pub user: Option<PartialUser>,
    /// JWT token for API calls.
    pub token: Option<String>,

    /// All servers the user has joined (populated from Ready event).
    pub servers: Vec<Server>,
    /// All channels across all servers + DMs.
    pub channels: Vec<Channel>,
    /// All members across all servers.
    pub members: Vec<Member>,

    /// Messages for the currently viewed channel.
    pub messages: Vec<Message>,

    /// Currently selected server.
    pub selected_server: Option<Uuid>,
    /// Currently selected channel.
    pub selected_channel: Option<Uuid>,

    /// Backend URLs.
    pub api_url: String,
    pub gateway_url: String,

    /// Sender for outbound gateway commands (Subscribe, TypingStart, etc.)
    pub gateway_tx: Option<async_channel::Sender<ClientEvent>>,

    /// Currently joined voice channel (None = not in a call).
    pub voice_channel: Option<Uuid>,
    /// Whether the microphone is muted.
    pub is_muted: bool,
    /// Whether audio output is deafened.
    pub is_deafened: bool,

    /// Who is currently typing in the selected channel.
    pub typing_users: Vec<(Uuid, std::time::Instant)>,

    /// Message being replied to (id, author_name, content_preview).
    pub replying_to: Option<(Uuid, String, String)>,

    /// Whether the member list panel is visible.
    pub show_members: bool,

    /// Send mode: true = Enter sends, Shift+Enter newline.
    ///            false = Enter newline, Cmd+Enter sends.
    pub enter_to_send: bool,

    /// Selected audio input device name.
    pub selected_input_device: Option<String>,
    /// Selected audio output device name.
    pub selected_output_device: Option<String>,

    /// Current theme name: "dark", "light", "gray".
    pub theme: String,

    /// Settings category currently selected.
    pub settings_category: String,

    /// Whether viewing DMs instead of a server.
    pub viewing_dms: bool,
    /// Cached DM channel list.
    pub dm_channels: Vec<crate::api::DmChannelInfo>,

    /// Whether invite dialog is open.
    pub show_invite: bool,
    /// Last generated invite code.
    pub invite_code: Option<String>,

    /// Whether settings panel is open.
    pub show_settings: bool,

    /// Whether fuzzy finder palette is open.
    pub show_finder: bool,
    /// Current search query in the fuzzy finder.
    pub finder_query: String,

    /// Error message to display.
    pub error: Option<String>,
    /// Whether a login request is in-flight.
    pub loading: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user: None,
            token: None,
            servers: vec![],
            channels: vec![],
            members: vec![],
            messages: vec![],
            selected_server: None,
            selected_channel: None,
            gateway_tx: None,
            voice_channel: None,
            is_muted: false,
            is_deafened: false,
            typing_users: vec![],
            replying_to: None,
            show_members: true,
            viewing_dms: false,
            dm_channels: vec![],
            show_invite: false,
            invite_code: None,
            selected_input_device: None,
            selected_output_device: None,
            enter_to_send: true,
            theme: "dark".into(),
            settings_category: "account".into(),
            show_settings: false,
            show_finder: false,
            finder_query: String::new(),
            api_url: "http://127.0.0.1:14702".into(),
            gateway_url: "ws://127.0.0.1:14703".into(),
            error: None,
            loading: false,
        }
    }

    /// Get the current theme colors.
    pub fn theme_colors(&self) -> crate::theme::ThemeColors {
        crate::theme::ThemeColors::from_name(&self.theme)
    }

    /// Is the user logged in?
    pub fn is_logged_in(&self) -> bool {
        self.user.is_some() && self.token.is_some()
    }

    /// Send a command to the gateway WebSocket.
    pub fn send_gateway(&self, event: ClientEvent) {
        if let Some(ref tx) = self.gateway_tx {
            let _ = tx.try_send(event);
        }
    }

    /// Subscribe to a channel via the gateway.
    pub fn subscribe_channel(&self, channel_id: Uuid) {
        self.send_gateway(ClientEvent::Subscribe { channel_id });
    }

    /// Get channels for the currently selected server.
    pub fn current_channels(&self) -> Vec<&Channel> {
        match self.selected_server {
            Some(sid) => self
                .channels
                .iter()
                .filter(|c| c.server_id == Some(sid) && c.channel_type == ChannelType::Text)
                .collect(),
            None => vec![],
        }
    }

    /// Get voice channels for the currently selected server.
    pub fn current_voice_channels(&self) -> Vec<&Channel> {
        match self.selected_server {
            Some(sid) => self
                .channels
                .iter()
                .filter(|c| c.server_id == Some(sid) && c.channel_type == ChannelType::Voice)
                .collect(),
            None => vec![],
        }
    }

    /// Get the display name for the current server.
    pub fn current_server_name(&self) -> &str {
        self.selected_server
            .and_then(|sid| self.servers.iter().find(|s| s.id == sid))
            .map(|s| s.name.as_str())
            .unwrap_or("Rusteze")
    }

    /// Get a user's display name from member list.
    pub fn username_for(&self, user_id: Uuid) -> String {
        if let Some(ref u) = self.user {
            if u.id == user_id {
                return u.display_name.clone().unwrap_or_else(|| u.username.clone());
            }
        }
        // Check members list
        for m in &self.members {
            if m.user_id == user_id {
                if let Some(ref nick) = m.nickname {
                    return nick.clone();
                }
            }
        }
        format!("user-{}", &user_id.to_string()[..8])
    }
}
