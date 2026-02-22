use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::input::{Input, InputState};
use uuid::Uuid;

use crate::state::AppState;

/// Fuzzy finder palette (Cmd+K) for quickly jumping to servers, channels, and DMs.
pub struct FinderPalette {
    state: Entity<AppState>,
    search_input: Entity<InputState>,
}

#[derive(Clone)]
enum FinderResult {
    Server { id: Uuid, name: String },
    Channel { id: Uuid, server_id: Uuid, name: String, prefix: String },
}

impl FinderPalette {
    pub fn new(
        state: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Search servers, channels...")
        });

        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        // Update finder_query when input changes
        let state_for_observe = state.clone();
        cx.observe(&search_input, move |_this, input, cx| {
            let query = input.read(cx).text().to_string();
            state_for_observe.update(cx, |state, cx| {
                state.finder_query = query;
                cx.notify();
            });
        })
        .detach();

        Self { state, search_input }
    }

    fn get_results(&self, cx: &Context<Self>) -> Vec<FinderResult> {
        let state = self.state.read(cx);
        let query = state.finder_query.to_lowercase();
        let mut results = Vec::new();

        // Servers
        for server in &state.servers {
            if query.is_empty() || server.name.to_lowercase().contains(&query) {
                results.push(FinderResult::Server {
                    id: server.id,
                    name: server.name.clone(),
                });
            }
        }

        // Channels
        for channel in &state.channels {
            let ch_name = &channel.name;
            if query.is_empty() || ch_name.to_lowercase().contains(&query) {
                let server_name = channel.server_id
                    .and_then(|sid| state.servers.iter().find(|s| s.id == sid))
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let prefix = match channel.channel_type {
                    rusteze_models::ChannelType::Voice => "🔊".into(),
                    rusteze_models::ChannelType::DirectMessage => "💬".into(),
                    _ => "#".into(),
                };
                results.push(FinderResult::Channel {
                    id: channel.id,
                    server_id: channel.server_id.unwrap_or_default(),
                    name: format!("{} > {}", server_name, ch_name),
                    prefix,
                });
            }
        }

        results.truncate(15); // Cap at 15 results
        results
    }

    fn select_result(&mut self, result: &FinderResult, _window: &mut Window, cx: &mut Context<Self>) {
        match result {
            FinderResult::Server { id, .. } => {
                let server_id = *id;
                self.state.update(cx, |state, cx| {
                    state.selected_server = Some(server_id);
                    // Select first text channel
                    let first_ch = state.channels.iter()
                        .find(|c| c.server_id == Some(server_id) && c.channel_type == rusteze_models::ChannelType::Text)
                        .map(|c| c.id);
                    state.selected_channel = first_ch;
                    state.messages.clear();
                    state.show_finder = false;
                    state.finder_query.clear();
                    cx.notify();
                });

                // Fetch messages for the selected channel
                let api_url = self.state.read(cx).api_url.clone();
                let token = self.state.read(cx).token.clone().unwrap_or_default();
                let ch_id = self.state.read(cx).selected_channel;
                let state = self.state.clone();
                if let Some(ch_id) = ch_id {
                    self.state.read(cx).subscribe_channel(ch_id);
                    let task = cx.background_executor().spawn(async move {
                        crate::api::fetch_messages(&api_url, &token, ch_id, None)
                    });
                    cx.spawn(async move |_this, cx| {
                        if let Ok(mut rows) = task.await {
                            rows.reverse();
                            let msgs: Vec<rusteze_models::Message> = rows.into_iter()
                                .map(crate::views::app_root::msg_row_to_model).collect();
                            let _ = cx.update(|cx| {
                                state.update(cx, |state, cx| {
                                    state.messages = msgs;
                                    cx.notify();
                                });
                            });
                        }
                    }).detach();
                }
            }
            FinderResult::Channel { id, server_id, .. } => {
                let channel_id = *id;
                let srv_id = *server_id;
                self.state.read(cx).subscribe_channel(channel_id);
                self.state.update(cx, |state, cx| {
                    state.selected_server = Some(srv_id);
                    state.selected_channel = Some(channel_id);
                    state.messages.clear();
                    state.show_finder = false;
                    state.finder_query.clear();
                    cx.notify();
                });

                let api_url = self.state.read(cx).api_url.clone();
                let token = self.state.read(cx).token.clone().unwrap_or_default();
                let state = self.state.clone();
                let task = cx.background_executor().spawn(async move {
                    crate::api::fetch_messages(&api_url, &token, channel_id, None)
                });
                cx.spawn(async move |_this, cx| {
                    if let Ok(mut rows) = task.await {
                        rows.reverse();
                        let msgs: Vec<rusteze_models::Message> = rows.into_iter()
                            .map(crate::views::app_root::msg_row_to_model).collect();
                        let _ = cx.update(|cx| {
                            state.update(cx, |state, cx| {
                                state.messages = msgs;
                                cx.notify();
                            });
                        });
                    }
                }).detach();
            }
        }
    }
}

impl Render for FinderPalette {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_visible = self.state.read(cx).show_finder;

        if !is_visible {
            return div().into_any_element();
        }

        let results = self.get_results(cx);

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .justify_center()
            .pt(px(80.))
            .bg(rgba(0x00000066))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _e, _w, cx| {
                this.state.update(cx, |s, cx| {
                    s.show_finder = false;
                    cx.notify();
                });
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(500.))
                    .max_h(px(400.))
                    .rounded_xl()
                    .bg(rgb(0x1e1e2e))
                    .border_1()
                    .border_color(rgb(0x45475a))
                    .overflow_hidden()
                    // Prevent click-through to backdrop
                    .on_mouse_down(MouseButton::Left, |_e, _w, _cx| {})
                    // Search input
                    .child(
                        div()
                            .p_3()
                            .border_b_1()
                            .border_color(rgb(0x313244))
                            .child(Input::new(&self.search_input)),
                    )
                    // Results
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p_1()
                            .when(results.is_empty(), |d: Div| {
                                d.child(
                                    div()
                                        .p_3()
                                        .text_sm()
                                        .text_color(rgb(0x6c7086))
                                        .child("No results"),
                                )
                            })
                            .children(results.iter().enumerate().map(|(i, result)| {
                                let r = result.clone();
                                let (icon, label) = match result {
                                    FinderResult::Server { name, .. } => {
                                        ("🖥".to_string(), name.clone())
                                    }
                                    FinderResult::Channel { name, prefix, .. } => {
                                        (prefix.clone(), name.clone())
                                    }
                                };

                                div()
                                    .id(ElementId::Name(format!("finder-{i}").into()))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px_3()
                                    .py(px(6.))
                                    .mx_1()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x313244)))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x6c7086))
                                            .w(px(20.))
                                            .child(icon),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xcdd6f4))
                                            .child(label),
                                    )
                                    .on_click(cx.listener(move |this, _event, window, cx| {
                                        this.select_result(&r, window, cx);
                                    }))
                            })),
                    ),
            )
            .into_any_element()
    }
}
