use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    input::{Input, InputState},
    label::Label,
};

use crate::state::AppState;

/// Dialog for creating a new server.
pub struct CreateServerDialog {
    state: Entity<AppState>,
    name_input: Entity<InputState>,
    visible: bool,
}

impl CreateServerDialog {
    pub fn new(
        state: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Server name")
        });
        Self {
            state,
            name_input,
            visible: false,
        }
    }

    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.visible = true;
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.notify();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn on_create(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name_input.read(cx).text().to_string();
        if name.trim().is_empty() {
            return;
        }

        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        // Clear input and hide dialog
        self.name_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.visible = false;
        cx.notify();

        // Create server in background
        let task = cx.background_executor().spawn(async move {
            let server = crate::api::create_server(&api_url, &token, &name)?;
            let channels = crate::api::fetch_channels(&api_url, &token, server.id)?;
            Ok::<_, anyhow::Error>((server, channels))
        });

        cx.spawn(async move |_this, cx| {
            match task.await {
                Ok((server_row, channel_rows)) => {
                    let _ = cx.update(|cx| {
                        state.update(cx, |state, cx| {
                            let new_server = rusteze_models::Server {
                                id: server_row.id,
                                name: server_row.name,
                                owner_id: server_row.owner_id,
                                icon_url: server_row.icon_url,
                                banner_url: server_row.banner_url,
                                description: server_row.description,
                                created_at: server_row.created_at,
                            };
                            state.servers.push(new_server);
                            state.selected_server = Some(server_row.id);

                            // Add channels
                            for ch in channel_rows {
                                let channel_type = match ch.channel_type.as_str() {
                                    "voice" => rusteze_models::ChannelType::Voice,
                                    _ => rusteze_models::ChannelType::Text,
                                };
                                state.channels.push(rusteze_models::Channel {
                                    id: ch.id,
                                    server_id: ch.server_id,
                                    name: ch.name,
                                    channel_type,
                                    topic: ch.topic,
                                    position: ch.position,
                                    created_at: ch.created_at,
                                });
                            }

                            // Add current user as member of the new server
                            if let Some(ref user) = state.user {
                                state.members.push(rusteze_models::Member {
                                    server_id: server_row.id,
                                    user_id: user.id,
                                    nickname: None,
                                    roles: vec![],
                                    joined_at: chrono::Utc::now(),
                                });
                            }

                            // Subscribe to new channels via gateway
                            for ch in &state.channels {
                                if ch.server_id == Some(server_row.id) {
                                    state.subscribe_channel(ch.id);
                                }
                            }

                            // Select first text channel
                            let first_ch = state
                                .channels
                                .iter()
                                .find(|c| {
                                    c.server_id == Some(server_row.id)
                                        && c.channel_type == rusteze_models::ChannelType::Text
                                })
                                .map(|c| c.id);
                            state.selected_channel = first_ch;
                            state.messages.clear();
                            cx.notify();
                        });
                    });
                }
                Err(e) => {
                    tracing::error!("failed to create server: {e}");
                }
            }
        })
        .detach();
    }

    fn on_cancel(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.visible = false;
        cx.notify();
    }
}

impl Render for CreateServerDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        // Overlay backdrop + centered dialog
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000088))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_6()
                    .w(px(360.))
                    .rounded_xl()
                    .bg(rgb(0x1e1e2e))
                    .border_1()
                    .border_color(rgb(0x45475a))
                    // Title
                    .child(
                        div()
                            .text_color(rgb(0xcdd6f4))
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(18.))
                            .child("Create a Server"),
                    )
                    .child(
                        Label::new("Give your server a name to get started.")
                            .text_color(rgb(0xa6adc8)),
                    )
                    // Name input
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Label::new("Server Name").text_color(rgb(0xbac2de)))
                            .child(Input::new(&self.name_input)),
                    )
                    // Buttons
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .justify_end()
                            .child(
                                Button::new("cancel")
                                    .label("Cancel")
                                    .on_click(cx.listener(Self::on_cancel)),
                            )
                            .child(
                                Button::new("create")
                                    .label("Create")
                                    .primary()
                                    .on_click(cx.listener(Self::on_create)),
                            ),
                    ),
            )
            .into_any_element()
    }
}
