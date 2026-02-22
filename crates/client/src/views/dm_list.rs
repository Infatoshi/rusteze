use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::scroll::ScrollableElement as _;
use uuid::Uuid;

use crate::state::AppState;

/// DM channel list — shown when the user clicks the DM button.
pub struct DmList {
    state: Entity<AppState>,
}

impl DmList {
    pub fn new(
        state: Entity<AppState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        Self { state }
    }

    fn select_dm(&mut self, channel_id: Uuid, _window: &mut Window, cx: &mut Context<Self>) {
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();

        self.state.read(cx).subscribe_channel(channel_id);

        self.state.update(cx, |state, cx| {
            state.selected_channel = Some(channel_id);
            state.messages.clear();
            cx.notify();
        });

        let state = self.state.clone();
        let task = cx.background_executor().spawn(async move {
            crate::api::fetch_messages(&api_url, &token, channel_id, None)
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

    fn open_dm_with(&mut self, user_id: Uuid, _window: &mut Window, cx: &mut Context<Self>) {
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let task = cx.background_executor().spawn(async move {
            crate::api::create_dm(&api_url, &token, user_id)
        });

        cx.spawn(async move |this, cx| {
            match task.await {
                Ok(dm) => {
                    let ch_id = dm.id;
                    let _ = cx.update(|cx| {
                        state.update(cx, |state, cx| {
                            state.subscribe_channel(ch_id);
                            // Add to DM channels if not already there
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
                            state.selected_channel = Some(ch_id);
                            state.messages.clear();
                            cx.notify();
                        });
                    });
                }
                Err(e) => tracing::error!("create DM failed: {e}"),
            }
        })
        .detach();
    }
}

impl Render for DmList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dms = self.state.read(cx).dm_channels.clone();
        let selected = self.state.read(cx).selected_channel;

        div()
            .flex()
            .flex_col()
            .py_2()
            .size_full()
            .overflow_y_scrollbar()
            .child(
                div()
                    .px_3()
                    .py_1()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0x6c7086))
                    .child("DIRECT MESSAGES"),
            )
            .when(dms.is_empty(), |d: gpui_component::scroll::Scrollable<Div>| {
                d.child(
                    div()
                        .px_3()
                        .py_4()
                        .text_sm()
                        .text_color(rgb(0x6c7086))
                        .child("No conversations yet"),
                )
            })
            .children(dms.into_iter().map(|dm| {
                let is_selected = selected == Some(dm.id);
                let dm_id = dm.id;
                let display_name = if dm.name == "DM" {
                    format!("DM #{}", &dm.id.to_string()[..6])
                } else {
                    dm.name.clone()
                };

                div()
                    .id(ElementId::Name(format!("dm-{}", dm.id).into()))
                    .flex()
                    .items_center()
                    .gap_2()
                    .mx_2()
                    .px_2()
                    .py(px(6.))
                    .rounded_md()
                    .cursor_pointer()
                    .when(is_selected, |d: Stateful<Div>| d.bg(rgb(0x313244)))
                    .hover(|s| s.bg(rgb(0x45475a)))
                    // Avatar
                    .child(
                        div()
                            .w(px(28.))
                            .h(px(28.))
                            .rounded_full()
                            .bg(rgb(0x585b70))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0xcdd6f4))
                            .child("💬"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_selected { rgb(0xcdd6f4) } else { rgb(0xa6adc8) })
                            .child(display_name),
                    )
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.select_dm(dm_id, window, cx);
                    }))
            }))
    }
}
