use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::text::TextView;
use uuid::Uuid;

use crate::state::AppState;

/// Scrollable list of messages for the currently selected channel.
pub struct MessageList {
    state: Entity<AppState>,
    /// Right-click context menu: (msg_id, channel_id, is_own, is_pinned, author, content)
    context_menu: Option<(Uuid, Uuid, bool, bool, String, String)>,
}

impl MessageList {
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

    fn delete_message(&mut self, msg_id: Uuid, channel_id: Uuid, cx: &mut Context<Self>) {
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let task = cx.background_executor().spawn(async move {
            crate::api::delete_message(&api_url, &token, channel_id, msg_id)
        });

        cx.spawn(async move |_this, cx| {
            if task.await.is_ok() {
                let _ = cx.update(|cx| {
                    state.update(cx, |state, cx| {
                        state.messages.retain(|m| m.id != msg_id);
                        cx.notify();
                    });
                });
            }
        })
        .detach();
    }

    fn pin_message(&mut self, msg_id: Uuid, channel_id: Uuid, cx: &mut Context<Self>) {
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let task = cx.background_executor().spawn(async move {
            crate::api::pin_message(&api_url, &token, channel_id, msg_id)
        });

        cx.spawn(async move |_this, cx| {
            if let Ok(updated) = task.await {
                let _ = cx.update(|cx| {
                    state.update(cx, |state, cx| {
                        if let Some(m) = state.messages.iter_mut().find(|m| m.id == msg_id) {
                            m.pinned = updated.pinned;
                        }
                        cx.notify();
                    });
                });
            }
        })
        .detach();
    }

    fn start_reply(&mut self, msg_id: Uuid, author: String, content: String, cx: &mut Context<Self>) {
        let preview = if content.len() > 60 {
            format!("{}...", &content[..57])
        } else {
            content
        };
        self.state.update(cx, |state, cx| {
            state.replying_to = Some((msg_id, author, preview));
            cx.notify();
        });
    }
}

impl Render for MessageList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let messages = self.state.read(cx).messages.clone();
        let current_user_id = self.state.read(cx).user.as_ref().map(|u| u.id);
        let state = self.state.read(cx);

        // Clean stale typing indicators for display
        let now = std::time::Instant::now();
        let typing: Vec<String> = state
            .typing_users
            .iter()
            .filter(|(_, t)| now.duration_since(*t).as_secs() < 5)
            .map(|(uid, _)| state.username_for(*uid))
            .collect();

        // Build a lookup for replied-to messages
        let msg_lookup: std::collections::HashMap<Uuid, (String, String)> = messages
            .iter()
            .map(|m| {
                let author = state.username_for(m.author_id);
                let content = m.content.clone().unwrap_or_default();
                (m.id, (author, content))
            })
            .collect();

        let entries: Vec<_> = messages
            .iter()
            .map(|msg| {
                let author = state.username_for(msg.author_id);
                let content = msg.content.clone().unwrap_or_default();
                let time = msg.created_at.format("%H:%M").to_string();
                let is_edited = msg.edited_at.is_some();
                let is_own = current_user_id == Some(msg.author_id);
                let is_pinned = msg.pinned;
                let has_image = msg.attachments.iter().any(|a| a.content_type.starts_with("image/"));
                let has_file = !msg.attachments.is_empty();
                let attachments_info: Vec<(String, String, u64, bool)> = msg.attachments.iter()
                    .map(|a| {
                        let is_img = a.content_type.starts_with("image/");
                        (a.filename.clone(), a.content_type.clone(), a.size, is_img)
                    })
                    .collect();
                let reply_ref = msg.replies_to.and_then(|rid| {
                    msg_lookup.get(&rid).map(|(a, c)| {
                        let preview = if c.len() > 50 { format!("{}...", &c[..47]) } else { c.clone() };
                        (a.clone(), preview)
                    })
                });
                (msg.id, msg.channel_id, author, content, time, is_edited, is_own, is_pinned, reply_ref, has_file, attachments_info)
            })
            .collect();

        let is_empty = entries.is_empty();

        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scrollbar()
            .px_4()
            .py_2()
            
            .when(is_empty, |d: gpui_component::scroll::Scrollable<Div>| {
                d.child(
                    div()
                        .flex()
                        .flex_grow()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(0x6c7086))
                        .child("No messages yet. Say something!"),
                )
            })
            .children(entries.into_iter().map(|(id, ch_id, author, content, time, edited, is_own, pinned, reply_ref, has_file, attachments_info)| {
                let msg_id = id;
                let channel_id = ch_id;
                let author_for_reply = author.clone();
                let content_for_reply = content.clone();

                div()
                    .flex()
                    .flex_col()
                    // Reply reference line
                    .when(reply_ref.is_some(), |d: Div| {
                        let (ref_author, ref_content) = reply_ref.unwrap();
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .pl(px(50.))
                                .pb_0p5()
                                .text_xs()
                                .text_color(rgb(0x6c7086))
                                .child("↩")
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x89b4fa))
                                        .child(ref_author),
                                )
                                .child(ref_content),
                        )
                    })
                    .child(
                        div()
                            .id(ElementId::Name(format!("msg-{id}").into()))
                            .flex()
                            .gap_3()
                            .py_1()
                            .px_2()
                            .rounded_md()
                            .hover(|s| s.bg(rgb(0x2a2a3e)))
                            .when(pinned, |d: Stateful<Div>| {
                                d.border_l_2().border_color(rgb(0xf9e2af))
                            })
                            // Avatar
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .w(px(36.))
                                    .h(px(36.))
                                    .mt_1()
                                    .rounded_full()
                                    .bg(rgb(0x585b70))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .text_color(rgb(0xcdd6f4))
                                    .child(
                                        author.chars().next().unwrap_or('?').to_uppercase().to_string(),
                                    ),
                            )
                            // Content
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .min_w_0()
                                    .flex_grow()
                                    .child(
                                        div()
                                            .flex()
                                            .items_baseline()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(rgb(0xcdd6f4))
                                                    .child(author.clone()),
                                            )
                                            .child(
                                                div().text_xs().text_color(rgb(0x6c7086)).child(time),
                                            )
                                            .when(edited, |d: Div| {
                                                d.child(div().text_xs().text_color(rgb(0x6c7086)).child("(edited)"))
                                            })
                                            .when(pinned, |d: Div| {
                                                d.child(div().text_xs().text_color(rgb(0xf9e2af)).child("📌"))
                                            }),
                                    )
                                    // Message text (skip if empty image-only message)
                                    .when(!content.is_empty(), |d: Div| {
                                        d.child(
                                            div().text_sm().text_color(rgb(0xbac2de)).child(
                                                TextView::markdown(
                                                    SharedString::from(format!("msg-text-{id}")),
                                                    SharedString::from(content.clone()),
                                                    _window,
                                                    cx,
                                                )
                                                .selectable(true),
                                            ),
                                        )
                                    })
                                    // File attachments
                                    .when(has_file, |d: Div| {
                                        d.children(attachments_info.iter().map(|(fname, ctype, size, is_img)| {
                                            let icon = if *is_img {
                                                "🖼"
                                            } else if ctype.starts_with("video/") {
                                                "🎬"
                                            } else if ctype.starts_with("audio/") {
                                                "🎵"
                                            } else if ctype == "application/pdf" {
                                                "📄"
                                            } else if ctype.starts_with("text/") {
                                                "📝"
                                            } else if ctype == "application/zip" || ctype == "application/gzip" {
                                                "📦"
                                            } else {
                                                "📎"
                                            };

                                            let size_str = if *size > 1_000_000 {
                                                format!("{:.1} MB", *size as f64 / 1_000_000.0)
                                            } else if *size > 1_000 {
                                                format!("{:.0} KB", *size as f64 / 1_000.0)
                                            } else {
                                                format!("{} B", size)
                                            };

                                            div()
                                                .mt_1()
                                                .px_3()
                                                .py_2()
                                                .rounded_md()
                                                .bg(rgb(0x313244))
                                                .border_1()
                                                .border_color(rgb(0x45475a))
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div().text_size(px(20.)).child(icon.to_string()),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .text_color(rgb(0x89b4fa))
                                                                .child(fname.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0x6c7086))
                                                                .child(size_str),
                                                        ),
                                                )
                                        }))
                                    }),
                            )
                            // Right-click for context menu
                            .on_mouse_down(MouseButton::Right, cx.listener(move |this, _e, _w, cx| {
                                this.context_menu = Some((msg_id, channel_id, is_own, pinned, author_for_reply.clone(), content_for_reply.clone()));
                                cx.notify();
                            })),
                    )
            }))
            // Message context menu
            .when(self.context_menu.is_some(), |d: gpui_component::scroll::Scrollable<Div>| {
                let (m_id, m_ch, m_own, m_pinned, m_author, m_content) = self.context_menu.clone().unwrap();
                d.child(
                    div()
                        .mx_4().p_1().rounded_md().bg(rgb(0x1e1e2e)).border_1().border_color(rgb(0x45475a))
                        .flex().flex_col().gap_0p5()
                        // Reply
                        .child(
                            div().id("msg-ctx-reply").px_3().py_1().text_sm().text_color(rgb(0xcdd6f4)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("↩ Reply")
                                .on_click(cx.listener(move |this, _e, _w, cx| {
                                    this.start_reply(m_id, m_author.clone(), m_content.clone(), cx);
                                    this.context_menu = None;
                                    cx.notify();
                                })),
                        )
                        // Pin/Unpin
                        .child(
                            div().id("msg-ctx-pin").px_3().py_1().text_sm().text_color(rgb(0xf9e2af)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a)))
                                .child(if m_pinned { "📌 Unpin" } else { "📌 Pin" })
                                .on_click(cx.listener(move |this, _e, _w, cx| {
                                    this.pin_message(m_id, m_ch, cx);
                                    this.context_menu = None;
                                    cx.notify();
                                })),
                        )
                        // Copy text
                        .child(
                            div().id("msg-ctx-copy").px_3().py_1().text_sm().text_color(rgb(0xcdd6f4)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("📋 Copy Text")
                                .on_click(cx.listener(move |this, _e, _window, cx| {
                                    let text = this.context_menu.as_ref().map(|m| m.5.clone()).unwrap_or_default();
                                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                                    this.context_menu = None;
                                    cx.notify();
                                })),
                        )
                        // Delete (own only)
                        .when(m_own, |d: Div| {
                            d.child(
                                div().id("msg-ctx-delete").px_3().py_1().text_sm().text_color(rgb(0xf38ba8)).cursor_pointer().rounded_sm()
                                    .hover(|s| s.bg(rgb(0x45475a))).child("🗑 Delete")
                                    .on_click(cx.listener(move |this, _e, _w, cx| {
                                        this.delete_message(m_id, m_ch, cx);
                                        this.context_menu = None;
                                        cx.notify();
                                    })),
                            )
                        }),
                )
            })
            // Typing indicator
            .when(!typing.is_empty(), |d: gpui_component::scroll::Scrollable<Div>| {
                let text = if typing.len() == 1 {
                    format!("{} is typing...", typing[0])
                } else {
                    format!("{} are typing...", typing.join(", "))
                };
                d.child(
                    div().px_2().py_1().text_xs().text_color(rgb(0xa6adc8)).child(text),
                )
            })
    }
}
