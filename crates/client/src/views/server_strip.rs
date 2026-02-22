use gpui::*;
use gpui::prelude::FluentBuilder as _;
use uuid::Uuid;

use crate::state::AppState;

/// Emitted when the user clicks the "+" button to create a server.
pub struct CreateServerRequested;

/// Emitted when the user clicks the settings gear.
pub struct SettingsRequested;

/// Emitted when the user clicks the DM button.
pub struct DmsRequested;

/// Emitted when the user clicks the invite button.
pub struct InviteRequested;

/// Vertical strip of server icons on the far left.
pub struct ServerStrip {
    state: Entity<AppState>,
    /// Right-click context menu for a server: (server_id, server_name)
    context_menu: Option<(uuid::Uuid, String)>,
}

impl ServerStrip {
    pub fn new(
        state: Entity<AppState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        Self { state, context_menu: None }
    }

    fn select_server(
        &mut self,
        server_id: Uuid,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();

        self.state.update(cx, |state, cx| {
            state.viewing_dms = false;
            state.selected_server = Some(server_id);
            let first_ch = state
                .channels
                .iter()
                .find(|c| {
                    c.server_id == Some(server_id)
                        && c.channel_type == rusteze_models::ChannelType::Text
                })
                .map(|c| c.id);
            state.selected_channel = first_ch;
            state.messages.clear();
            cx.notify();
        });

        let state = self.state.clone();
        let channel_id = self.state.read(cx).selected_channel;

        if let Some(ch_id) = channel_id {
            let task = cx.background_executor().spawn(async move {
                crate::api::fetch_messages(&api_url, &token, ch_id, None)
            });

            cx.spawn(async move |_this, cx| {
                if let Ok(mut rows) = task.await {
                    rows.reverse();
                    let messages: Vec<rusteze_models::Message> = rows
                        .into_iter()
                        .map(crate::views::app_root::msg_row_to_model)
                        .collect();
                    let _ = cx.update(|cx| {
                        state.update(cx, |state, cx| {
                            state.messages = messages;
                            cx.notify();
                        });
                    });
                }
            })
            .detach();
        }
    }
}

impl Render for ServerStrip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let servers = self.state.read(cx).servers.clone();
        let selected = self.state.read(cx).selected_server;

        div()
            .flex()
            .flex_col()
            .items_center()
            .py_3()
            .size_full()
            .justify_between()
            .relative()
            // DM button at the very top
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .items_center()
                    .gap_2()
                    .pb_2()
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .id("dm-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(42.))
                            .h(px(42.))
                            .rounded(if selected.is_none() && self.state.read(cx).viewing_dms { px(14.) } else { px(21.) })
                            .bg(if self.state.read(cx).viewing_dms { rgb(0x89b4fa) } else { rgb(0x313244) })
                            .text_color(if self.state.read(cx).viewing_dms { rgb(0x1e1e2e) } else { rgb(0xcdd6f4) })
                            .text_size(px(18.))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x585b70)))
                            .child("💬")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(DmsRequested);
                            })),
                    ),
            )
            // Server icons (scrollable)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .items_center()
                    .gap_2()
                    .pt_2()
                    .overflow_hidden()
                    .children(servers.into_iter().map(|server| {
                        let is_selected = selected == Some(server.id);
                        let server_id = server.id;
                        let srv_name = server.name.clone();
                        let initial = server
                            .name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();

                        div()
                            .id(ElementId::Name(format!("srv-{server_id}").into()))
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(42.))
                            .h(px(42.))
                            .flex_shrink_0()
                            .rounded(if is_selected { px(14.) } else { px(21.) })
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x585b70)))
                            .child(initial)
                            .on_click(cx.listener(move |this, _event, window, cx| {
                                this.context_menu = None;
                                this.select_server(server_id, window, cx);
                            }))
                            .on_mouse_down(MouseButton::Right, cx.listener(move |this, _e, _w, cx| {
                                this.context_menu = Some((server_id, srv_name.clone()));
                                cx.notify();
                            }))
                    })),
            )
            // Server context menu — absolute positioned, pops out to the right
            .when(self.context_menu.is_some(), |d: Div| {
                let (srv_id, _srv_name) = self.context_menu.clone().unwrap();
                d.child(
                    div()
                        .absolute()
                        .left(px(64.))
                        .top(px(100.))
                        .w(px(160.))
                        .p_1()
                        .rounded_md()
                        .bg(rgb(0x1e1e2e))
                        .border_1()
                        .border_color(rgb(0x45475a))
                        .flex().flex_col().gap_0p5()
                        .on_mouse_down(MouseButton::Left, |_e, _w, _cx| {}) // prevent click-through
                        .child(
                            div().id("srv-ctx-invite").px_3().py(px(6.)).text_sm().text_color(rgb(0xcdd6f4)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("📨 Invite People")
                                .on_click(cx.listener(move |this, _e, _w, cx| {
                                    this.context_menu = None;
                                    cx.emit(CreateServerRequested);
                                    cx.notify();
                                })),
                        )
                        .child(div().h(px(1.)).w_full().bg(rgb(0x313244))) // divider
                        .child(
                            div().id("srv-ctx-leave").px_3().py(px(6.)).text_sm().text_color(rgb(0xf38ba8)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("Leave Server")
                                .on_click(cx.listener(move |this, _e, _w, cx| {
                                    this.context_menu = None;
                                    this.state.update(cx, |state, cx| {
                                        state.servers.retain(|s| s.id != srv_id);
                                        state.channels.retain(|c| c.server_id != Some(srv_id));
                                        state.members.retain(|m| m.server_id != srv_id);
                                        if state.selected_server == Some(srv_id) {
                                            state.selected_server = state.servers.first().map(|s| s.id);
                                            state.selected_channel = None;
                                            state.messages.clear();
                                        }
                                        cx.notify();
                                    });
                                })),
                        ),
                )
            })
            // Bottom: "+" and gear grouped together
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .items_center()
                    .gap_2()
                    .pt_2()
                    .border_t_1()
                    .border_color(rgb(0x313244))
                    // "+" create server
                    .child(
                        div()
                            .id("create-server-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(42.))
                            .h(px(42.))
                            .rounded(px(21.))
                            .bg(rgb(0x313244))
                            .text_color(rgb(0xa6e3a1))
                            .text_size(px(22.))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0xa6e3a1)).text_color(rgb(0x1e1e2e)).rounded(px(14.)))
                            .child("+")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(CreateServerRequested);
                            })),
                    )
                    // Settings gear
                    .child(
                        div()
                            .id("settings-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(42.))
                            .h(px(42.))
                            .mb_1()
                            .rounded(px(21.))
                            .bg(rgb(0x313244))
                            .text_color(rgb(0x6c7086))
                            .text_size(px(18.))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x585b70)).text_color(rgb(0xcdd6f4)).rounded(px(14.)))
                            .child("⚙")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(SettingsRequested);
                            })),
                    ),
            )
    }
}

impl EventEmitter<CreateServerRequested> for ServerStrip {}
impl EventEmitter<SettingsRequested> for ServerStrip {}
impl EventEmitter<DmsRequested> for ServerStrip {}
impl EventEmitter<InviteRequested> for ServerStrip {}
