use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::scroll::ScrollableElement as _;
use uuid::Uuid;

use crate::state::AppState;

/// Emitted when user clicks a member to open a DM.
pub struct OpenDmRequested {
    pub user_id: Uuid,
}

/// Member list panel on the right side of the content area.
pub struct MemberList {
    state: Entity<AppState>,
    /// Right-click menu: (user_id, username, is_self)
    context_menu: Option<(Uuid, String, bool)>,
}

impl MemberList {
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

    fn open_dm(&mut self, user_id: Uuid, _window: &mut Window, cx: &mut Context<Self>) {
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let task = cx.background_executor().spawn(async move {
            crate::api::create_dm(&api_url, &token, user_id)
        });

        cx.spawn(async move |_this, cx| {
            match task.await {
                Ok(dm) => {
                    let ch_id = dm.id;
                    let _ = cx.update(|cx| {
                        state.update(cx, |state, cx| {
                            state.subscribe_channel(ch_id);
                            state.viewing_dms = true;
                            state.selected_server = None;
                            state.selected_channel = Some(ch_id);
                            state.messages.clear();
                            // Add to DM list if not there
                            if !state.dm_channels.iter().any(|d| d.id == ch_id) {
                                state.dm_channels.push(crate::api::DmChannelInfo {
                                    id: dm.id,
                                    server_id: dm.server_id,
                                    name: dm.name,
                                    channel_type: dm.channel_type,
                                    topic: dm.topic,
                                    position: dm.position,
                                    created_at: dm.created_at,
                                    participants: vec![],
                                });
                            }
                            cx.notify();
                        });
                    });
                    // Fetch messages
                    let _ = cx.update(|cx| {
                        let api = state.read(cx).api_url.clone();
                        let tok = state.read(cx).token.clone().unwrap_or_default();
                        let s = state.clone();
                        cx.background_executor()
                            .spawn(async move {
                                if let Ok(mut rows) = crate::api::fetch_messages(&api, &tok, ch_id, None) {
                                    rows.reverse();
                                    let msgs: Vec<rusteze_models::Message> = rows.into_iter()
                                        .map(crate::views::app_root::msg_row_to_model).collect();
                                    // Can't update state from here without cx, but the observe will pick it up
                                    tracing::info!("fetched {} DM messages", msgs.len());
                                }
                            })
                            .detach();
                    });
                }
                Err(e) => tracing::error!("create DM failed: {e}"),
            }
        })
        .detach();
    }
}

impl EventEmitter<OpenDmRequested> for MemberList {}

impl Render for MemberList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let server_id = state.selected_server;
        let current_user = state.user.as_ref().map(|u| u.id);

        let members: Vec<_> = state
            .members
            .iter()
            .filter(|m| Some(m.server_id) == server_id)
            .map(|m| {
                let name = state.username_for(m.user_id);
                let is_self = current_user == Some(m.user_id);
                (m.user_id, name, is_self)
            })
            .collect();

        let count = members.len();

        div()
            .flex()
            .flex_col()
            .h_full()
            .bg(rgb(0x181825))
            .overflow_y_scrollbar()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(48.))
                    .px_3()
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0x6c7086))
                            .child(format!("MEMBERS — {count}")),
                    ),
            )
            
            .children(members.into_iter().map(|(uid, name, is_self)| {
                let user_id = uid;
                let member_name = name.clone();

                div()
                    .id(ElementId::Name(format!("member-{uid}").into()))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py(px(4.))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .rounded_md()
                    .mx_1()
                    .child(
                        div()
                            .flex_shrink_0()
                            .w(px(28.))
                            .h(px(28.))
                            .rounded_full()
                            .bg(if is_self { rgb(0x89b4fa) } else { rgb(0x585b70) })
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(if is_self { rgb(0x1e1e2e) } else { rgb(0xcdd6f4) })
                            .child(
                                name.chars().next().unwrap_or('?').to_uppercase().to_string(),
                            ),
                    )
                    .child(
                        div()
                            .flex_grow()
                            .text_sm()
                            .text_color(if is_self { rgb(0x89b4fa) } else { rgb(0xa6adc8) })
                            .child(name),
                    )
                    .on_mouse_down(MouseButton::Right, cx.listener(move |this, _e, _w, cx| {
                        this.context_menu = Some((user_id, member_name.clone(), is_self));
                        cx.notify();
                    }))
            }))
            // Member context menu
            .when(self.context_menu.is_some(), |d: gpui_component::scroll::Scrollable<Div>| {
                let (m_uid, m_name, m_is_self) = self.context_menu.clone().unwrap();
                d.child(
                    div()
                        .mx_1().p_1().rounded_md().bg(rgb(0x1e1e2e)).border_1().border_color(rgb(0x45475a))
                        .flex().flex_col().gap_0p5()
                        // Profile header
                        .child(
                            div().px_2().py_1().text_xs().text_color(rgb(0x6c7086))
                                .child(m_name.clone()),
                        )
                        // Message (DM) — not for self
                        .when(!m_is_self, |d: Div| {
                            d.child(
                                div().id("mbr-ctx-dm").px_2().py_1().text_sm().text_color(rgb(0xcdd6f4)).cursor_pointer().rounded_sm()
                                    .hover(|s| s.bg(rgb(0x45475a))).child("💬 Message")
                                    .on_click(cx.listener(move |this, _e, window, cx| {
                                        this.context_menu = None;
                                        this.open_dm(m_uid, window, cx);
                                    })),
                            )
                        })
                        // Kick — not for self
                        .when(!m_is_self, |d: Div| {
                            d.child(
                                div().id("mbr-ctx-kick").px_2().py_1().text_sm().text_color(rgb(0xf38ba8)).cursor_pointer().rounded_sm()
                                    .hover(|s| s.bg(rgb(0x45475a))).child("🚫 Kick")
                                    .on_click(cx.listener(move |this, _e, _w, cx| {
                                        this.context_menu = None;
                                        // TODO: call kick API
                                        cx.notify();
                                    })),
                            )
                        }),
                )
            })
    }
}
