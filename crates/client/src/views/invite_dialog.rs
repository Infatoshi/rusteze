use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    input::{Input, InputState},
    label::Label,
    Sizable as _,
};

use crate::state::AppState;

/// Dialog for creating/sharing server invites and joining by code.
pub struct InviteDialog {
    state: Entity<AppState>,
    join_input: Entity<InputState>,
    visible: bool,
}

impl InviteDialog {
    pub fn new(
        state: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let join_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Enter invite code...")
        });

        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            state,
            join_input,
            visible: false,
        }
    }

    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.visible = true;
        cx.notify();
    }

    fn generate_invite(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let server_id = match self.state.read(cx).selected_server {
            Some(id) => id,
            None => return,
        };
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let task = cx.background_executor().spawn(async move {
            crate::api::create_invite(&api_url, &token, server_id)
        });

        cx.spawn(async move |_this, cx| {
            match task.await {
                Ok(invite) => {
                    let _ = cx.update(|cx| {
                        state.update(cx, |s, cx| {
                            s.invite_code = Some(invite.code);
                            cx.notify();
                        });
                    });
                }
                Err(e) => tracing::error!("create invite failed: {e}"),
            }
        })
        .detach();
    }

    fn join_by_code(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let code = self.join_input.read(cx).text().to_string();
        if code.trim().is_empty() {
            return;
        }

        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let task = cx.background_executor().spawn(async move {
            crate::api::join_invite(&api_url, &token, &code)
        });

        cx.spawn(async move |_this, cx| {
            match task.await {
                Ok(()) => {
                    tracing::info!("joined server via invite");
                    // Refresh servers — the gateway Ready will update on reconnect
                    // For now just close the dialog
                    let _ = cx.update(|cx| {
                        state.update(cx, |s, cx| {
                            s.show_invite = false;
                            cx.notify();
                        });
                    });
                }
                Err(e) => tracing::error!("join invite failed: {e}"),
            }
        })
        .detach();
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        self.state.update(cx, |s, cx| {
            s.show_invite = false;
            s.invite_code = None;
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for InviteDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let show = self.state.read(cx).show_invite || self.visible;
        if !show {
            return div().into_any_element();
        }

        let invite_code = self.state.read(cx).invite_code.clone();
        let has_server = self.state.read(cx).selected_server.is_some();
        let server_name = self.state.read(cx).current_server_name().to_string();

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
                    .w(px(420.))
                    .rounded_xl()
                    .bg(rgb(0x1e1e2e))
                    .border_1()
                    .border_color(rgb(0x45475a))
                    .on_mouse_down(MouseButton::Left, |_e, _w, _cx| {})
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(rgb(0xcdd6f4))
                                    .font_weight(FontWeight::BOLD)
                                    .text_size(px(18.))
                                    .child("Invite People"),
                            )
                            .child(
                                div()
                                    .id("close-invite")
                                    .text_color(rgb(0x6c7086))
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(rgb(0xcdd6f4)))
                                    .child("✕")
                                    .on_click(cx.listener(|this, _e, _w, cx| {
                                        this.close(cx);
                                    })),
                            ),
                    )
                    // Generate invite section
                    .when(has_server, |d: Div| {
                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_3()
                                .rounded_lg()
                                .bg(rgb(0x181825))
                                .child(
                                    Label::new(format!("Create an invite link for {server_name}"))
                                        .text_color(rgb(0xa6adc8)),
                                )
                                .child(
                                    Button::new("gen-invite")
                                        .label("Generate Invite Link")
                                        .primary()
                                        .small()
                                        .on_click(cx.listener(|this, _e, window, cx| {
                                            this.generate_invite(window, cx);
                                        })),
                                ),
                        )
                    })
                    // Show generated code
                    .when(invite_code.is_some(), |d: Div| {
                        let code = invite_code.clone().unwrap();
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .p_3()
                                .rounded_lg()
                                .bg(rgb(0x313244))
                                .child(
                                    div()
                                        .flex_grow()
                                        .text_sm()
                                        .text_color(rgb(0xa6e3a1))
                                        .font_weight(FontWeight::BOLD)
                                        .child(code),
                                )
                                .child(
                                    Label::new("Share this code")
                                        .text_color(rgb(0x6c7086))
                                        .text_size(px(11.)),
                                ),
                        )
                    })
                    // Divider
                    .child(div().h(px(1.)).w_full().bg(rgb(0x313244)))
                    // Join by code section
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                Label::new("Join a server by invite code")
                                    .text_color(rgb(0xa6adc8)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(
                                        div().flex_grow().child(Input::new(&self.join_input)),
                                    )
                                    .child(
                                        Button::new("join-invite")
                                            .label("Join")
                                            .small()
                                            .on_click(cx.listener(|this, _e, window, cx| {
                                                this.join_by_code(window, cx);
                                            })),
                                    ),
                            ),
                    )
                    // Close
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .child(
                                Button::new("close-invite-btn")
                                    .label("Done")
                                    .small()
                                    .on_click(cx.listener(|this, _e, _w, cx| {
                                        this.close(cx);
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}
