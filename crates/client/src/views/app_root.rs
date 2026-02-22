use gpui::*;
use rusteze_models::ServerEvent;

use crate::state::AppState;
use crate::views::app_layout::AppLayout;
use crate::views::login::LoginView;

/// Root view that switches between Login and App based on auth state.
pub struct AppRoot {
    pub state: Entity<AppState>,
    login_view: Entity<LoginView>,
    app_layout: Option<Entity<AppLayout>>,
    focus_handle: FocusHandle,
}

impl AppRoot {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());
        let login_view = cx.new(|cx| LoginView::new(state.clone(), window, cx));
        let focus_handle = cx.focus_handle();

        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        // Focus the root so it receives key events
        focus_handle.focus(window);

        Self {
            state,
            login_view,
            app_layout: None,
            focus_handle,
        }
    }

    fn connect_gateway(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.state.read(cx);
        let token = match &state.token {
            Some(t) => t.clone(),
            None => return,
        };
        let gw_url = state.gateway_url.clone();

        let (event_tx, event_rx) = async_channel::unbounded::<ServerEvent>();

        // Spawn gateway — now returns a command sender
        let cmd_tx = crate::gateway::spawn_gateway(gw_url, token, event_tx);

        // Store command sender in state
        self.state.update(cx, |state, _cx| {
            state.gateway_tx = Some(cmd_tx);
        });

        // Spawn foreground task to process events
        let state_handle = self.state.clone();
        cx.spawn(async move |_this, cx| {
            while let Ok(event) = event_rx.recv().await {
                let _ = cx.update(|cx| {
                    state_handle.update(cx, |state, cx| {
                        handle_server_event(state, event, cx);
                    });
                });
            }
        })
        .detach();

        let layout = cx.new(|cx| AppLayout::new(self.state.clone(), window, cx));
        self.app_layout = Some(layout);
    }
}

impl Focusable for AppRoot {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AppRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_logged_in = self.state.read(cx).is_logged_in();

        if is_logged_in && self.app_layout.is_none() {
            self.connect_gateway(window, cx);
        }

        div()
            .id("app-root")
            .track_focus(&self.focus_handle)
            .key_context("AppRoot")
            .size_full()
            // Register keybind actions on the focused root element (Zed pattern)
            .on_action(cx.listener(|this, _: &crate::ToggleSettings, _window, cx| {
                this.state.update(cx, |s, cx| {
                    s.show_settings = !s.show_settings;
                    s.show_finder = false;
                    cx.notify();
                });
            }))
            .on_action(cx.listener(|this, _: &crate::ToggleFinder, _window, cx| {
                this.state.update(cx, |s, cx| {
                    s.show_finder = !s.show_finder;
                    s.show_settings = false;
                    s.finder_query.clear();
                    cx.notify();
                });
            }))
            .on_action(cx.listener(|this, _: &crate::DismissOverlay, _window, cx| {
                this.state.update(cx, |s, cx| {
                    if s.show_finder || s.show_settings {
                        s.show_finder = false;
                        s.show_settings = false;
                        cx.notify();
                    }
                });
            }))
            .child(if is_logged_in {
                if let Some(ref layout) = self.app_layout {
                    layout.clone().into_any_element()
                } else {
                    div().child("Connecting...").into_any_element()
                }
            } else {
                self.login_view.clone().into_any_element()
            })
    }
}

/// Process a single ServerEvent and update the AppState.
fn handle_server_event(state: &mut AppState, event: ServerEvent, cx: &mut Context<AppState>) {
    // Clean stale typing indicators (>5s old)
    let now = std::time::Instant::now();
    state.typing_users.retain(|(_, t)| now.duration_since(*t).as_secs() < 5);
    match event {
        ServerEvent::Ready {
            user,
            servers,
            channels,
            members,
        } => {
            tracing::info!(
                "Ready: {} servers, {} channels",
                servers.len(),
                channels.len()
            );
            state.user = Some(user);
            state.servers = servers;
            state.channels = channels;
            state.members = members;

            // Subscribe to ALL channels via the gateway
            for ch in &state.channels {
                state.subscribe_channel(ch.id);
            }

            // Auto-select first server and its first channel
            if let Some(first) = state.servers.first() {
                state.selected_server = Some(first.id);
                let first_ch = state
                    .channels
                    .iter()
                    .find(|c| c.server_id == Some(first.id));
                if let Some(ch) = first_ch {
                    state.selected_channel = Some(ch.id);
                    let api = state.api_url.clone();
                    let token = state.token.clone().unwrap_or_default();
                    let ch_id = ch.id;
                    cx.spawn(async move |state_handle, cx| {
                        match crate::api::fetch_messages(&api, &token, ch_id, None) {
                            Ok(mut rows) => {
                                rows.reverse();
                                let messages: Vec<rusteze_models::Message> =
                                    rows.into_iter().map(msg_row_to_model).collect();
                                let _ = cx.update(|cx| {
                                    let _ = state_handle.update(cx, |state, cx| {
                                        state.messages = messages;
                                        cx.notify();
                                    });
                                });
                            }
                            Err(e) => tracing::error!("fetch messages failed: {e}"),
                        }
                    })
                    .detach();
                }
            }
            cx.notify();
        }

        ServerEvent::MessageCreate(msg) => {
            // Clear typing indicator for the author
            state.typing_users.retain(|(uid, _)| *uid != msg.author_id);

            if state.selected_channel == Some(msg.channel_id) {
                state.messages.push(msg);
                cx.notify();
            }
        }

        ServerEvent::MessageUpdate {
            id,
            channel_id,
            content,
            ..
        } => {
            if state.selected_channel == Some(channel_id) {
                if let Some(m) = state.messages.iter_mut().find(|m| m.id == id) {
                    if let Some(c) = content {
                        m.content = Some(c);
                    }
                    cx.notify();
                }
            }
        }

        ServerEvent::MessageDelete { id, channel_id } => {
            if state.selected_channel == Some(channel_id) {
                state.messages.retain(|m| m.id != id);
                cx.notify();
            }
        }

        ServerEvent::TypingStart { channel_id, user_id } => {
            // Only show typing for current channel, and not from self
            if state.selected_channel == Some(channel_id) {
                if state.user.as_ref().map(|u| u.id) != Some(user_id) {
                    // Remove old entry for this user, add fresh one
                    state.typing_users.retain(|(uid, _)| *uid != user_id);
                    state.typing_users.push((user_id, std::time::Instant::now()));
                    cx.notify();
                }
            }
        }

        ServerEvent::ChannelCreate(ch) => {
            // Deduplicate — only add if not already present
            if !state.channels.iter().any(|c| c.id == ch.id) {
                state.subscribe_channel(ch.id);
                state.channels.push(ch);
                cx.notify();
            }
        }

        ServerEvent::ChannelDelete { id } => {
            state.channels.retain(|c| c.id != id);
            if state.selected_channel == Some(id) {
                state.selected_channel = None;
                state.messages.clear();
            }
            cx.notify();
        }

        ServerEvent::MemberJoin { server_id, user } => {
            state.members.push(rusteze_models::Member {
                server_id,
                user_id: user.id,
                nickname: user.display_name.clone(),
                roles: vec![],
                joined_at: chrono::Utc::now(),
            });
            cx.notify();
        }

        ServerEvent::MemberLeave { server_id, user_id } => {
            state.members.retain(|m| !(m.server_id == server_id && m.user_id == user_id));
            cx.notify();
        }

        _ => {}
    }
}

pub fn msg_row_to_model(row: crate::api::MessageRow) -> rusteze_models::Message {
    rusteze_models::Message {
        id: row.id,
        channel_id: row.channel_id,
        author_id: row.author_id,
        content: row.content,
        attachments: vec![],
        embeds: vec![],
        mentions: vec![],
        replies_to: row.replies_to,
        pinned: row.pinned,
        edited_at: row.edited_at,
        created_at: row.created_at,
    }
}
