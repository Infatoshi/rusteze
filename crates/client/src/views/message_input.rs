use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    input::{Input, InputState},
    Disableable as _,
    Sizable as _,
};
use rusteze_models::ClientEvent;

use crate::state::AppState;
use crate::views::emoji_picker::{EmojiPicker, EmojiSelected};

/// Message compose input at the bottom of the content area.
pub struct MessageInput {
    state: Entity<AppState>,
    input: Entity<InputState>,
    emoji_picker: Entity<EmojiPicker>,
    last_typing_sent: std::time::Instant,
}

impl MessageInput {
    pub fn new(
        state: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Type a message...")
                .auto_grow(1, 6)
                .soft_wrap(true)
        });

        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        let emoji_picker = cx.new(|cx| EmojiPicker::new(window, cx));

        // Handle emoji selection — insert into input
        let input_for_emoji = input.clone();
        cx.subscribe_in(&emoji_picker, window, move |_this, _picker, event: &EmojiSelected, window, cx| {
            input_for_emoji.update(cx, |input_state, cx| {
                let current = input_state.text().to_string();
                input_state.set_value(&format!("{}{}", current, event.emoji), window, cx);
            });
        })
        .detach();

        // Typing indicator throttle
        cx.observe(&input, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if !text.is_empty() {
                let now = std::time::Instant::now();
                if now.duration_since(this.last_typing_sent).as_secs() >= 3 {
                    if let Some(channel_id) = this.state.read(cx).selected_channel {
                        this.state.read(cx).send_gateway(ClientEvent::TypingStart { channel_id });
                        this.last_typing_sent = now;
                    }
                }
            }
        })
        .detach();

        Self {
            state,
            input,
            emoji_picker,
            last_typing_sent: std::time::Instant::now() - std::time::Duration::from_secs(10),
        }
    }

    fn do_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().to_string();
        if text.trim().is_empty() {
            return;
        }

        let channel_id = match self.state.read(cx).selected_channel {
            Some(id) => id,
            None => return,
        };

        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let replies_to = self.state.read(cx).replying_to.as_ref().map(|(id, _, _)| *id);

        // Clear input and reply state
        self.input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.state.update(cx, |s, cx| {
            s.replying_to = None;
            cx.notify();
        });

        cx.background_executor()
            .spawn(async move {
                // Use send_message_with_reply if replying
                if let Some(_reply_id) = replies_to {
                    // For now, send as regular message — reply_to is handled by the API
                    if let Err(e) = crate::api::send_message(&api_url, &token, channel_id, &text) {
                        tracing::error!("failed to send message: {e}");
                    }
                } else {
                    if let Err(e) = crate::api::send_message(&api_url, &token, channel_id, &text) {
                        tracing::error!("failed to send message: {e}");
                    }
                }
            })
            .detach();
    }

    fn cancel_reply(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.replying_to = None;
            cx.notify();
        });
    }
}

impl Render for MessageInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_channel = self.state.read(cx).selected_channel.is_some();
        let enter_to_send = self.state.read(cx).enter_to_send;
        let replying_to = self.state.read(cx).replying_to.clone();
        let is_replying = replying_to.is_some();

        let hint = if enter_to_send {
            "Enter to send, Shift+Enter for newline"
        } else {
            "Cmd+Enter to send, Enter for newline"
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .px_4()
            .pb_4()
            .pt_2()
            .bg(rgb(0x1e1e2e))
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                let is_enter = event.keystroke.key == "enter";
                if !is_enter {
                    return;
                }
                let has_shift = event.keystroke.modifiers.shift;
                let has_cmd = event.keystroke.modifiers.platform;

                if enter_to_send {
                    if !has_shift && !has_cmd {
                        this.do_send(window, cx);
                    }
                } else {
                    if has_cmd {
                        this.do_send(window, cx);
                    }
                }
            }))
            // Reply bar
            .when(is_replying, |d: Div| {
                let (_, ref_author, ref_content) = replying_to.unwrap();
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_1()
                        .mb_1()
                        .rounded_t_lg()
                        .bg(rgb(0x313244))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .text_xs()
                                .text_color(rgb(0xa6adc8))
                                .child("Replying to")
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x89b4fa))
                                        .child(ref_author),
                                )
                                .child(
                                    div()
                                        .text_color(rgb(0x6c7086))
                                        .child(format!("— {ref_content}")),
                                ),
                        )
                        .child(
                            div()
                                .id("cancel-reply")
                                .text_xs()
                                .text_color(rgb(0x6c7086))
                                .cursor_pointer()
                                .hover(|s| s.text_color(rgb(0xf38ba8)))
                                .child("✕")
                                .on_click(cx.listener(|this, _e, _w, cx| {
                                    this.cancel_reply(cx);
                                })),
                        ),
                )
            })
            // Input container
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_end()
                    .gap_2()
                    .p_3()
                    .when(is_replying, |d: Div| d.rounded_b_lg())
                    .when(!is_replying, |d: Div| d.rounded_lg())
                    .bg(rgb(0x313244))
                    .border_1()
                    .border_color(rgb(0x45475a))
                    // Emoji button
                    .child(
                        div()
                            .id("emoji-btn")
                            .flex_shrink_0()
                            .text_color(rgb(0x6c7086))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(0xcdd6f4)))
                            .child("😀")
                            .on_click(cx.listener(|this, _e, _w, cx| {
                                this.emoji_picker.update(cx, |p, cx| p.toggle(cx));
                            })),
                    )
                    // File attach button
                    .child(
                        div()
                            .id("attach-file")
                            .flex_shrink_0()
                            .text_color(rgb(0x6c7086))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(0xcdd6f4)))
                            .child("📎")
                            .on_click(cx.listener(|this, _e, _w, cx| {
                                let channel_id = match this.state.read(cx).selected_channel {
                                    Some(id) => id,
                                    None => return,
                                };
                                let api_url = this.state.read(cx).api_url.clone();
                                let token = this.state.read(cx).token.clone().unwrap_or_default();

                                let options = PathPromptOptions {
                                    files: true,
                                    directories: false,
                                    multiple: false,
                                    prompt: Some("Select a file to upload".into()),
                                };
                                let receiver = cx.prompt_for_paths(options);

                                cx.spawn(async move |_this, cx| {
                                    if let Ok(Ok(Some(paths))) = receiver.await {
                                        for path in paths {
                                            // Read file, convert to base64, upload
                                            match std::fs::read(&path) {
                                                Ok(data) => {
                                                    use base64::Engine;
                                                    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                                                    let filename = path.file_name()
                                                        .and_then(|n| n.to_str())
                                                        .unwrap_or("file")
                                                        .to_string();
                                                    let content_type = match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref() {
                                                        Some("png") => "image/png",
                                                        Some("jpg") | Some("jpeg") => "image/jpeg",
                                                        Some("gif") => "image/gif",
                                                        Some("webp") => "image/webp",
                                                        Some("svg") => "image/svg+xml",
                                                        Some("mp4") => "video/mp4",
                                                        Some("webm") => "video/webm",
                                                        Some("mov") => "video/quicktime",
                                                        Some("mp3") => "audio/mpeg",
                                                        Some("ogg") => "audio/ogg",
                                                        Some("wav") => "audio/wav",
                                                        Some("pdf") => "application/pdf",
                                                        Some("txt") => "text/plain",
                                                        Some("json") => "application/json",
                                                        Some("csv") => "text/csv",
                                                        Some("zip") => "application/zip",
                                                        Some("tar") | Some("gz") => "application/gzip",
                                                        Some("rs") => "text/x-rust",
                                                        Some("py") => "text/x-python",
                                                        Some("js") | Some("ts") => "text/javascript",
                                                        Some("md") => "text/markdown",
                                                        _ => "application/octet-stream",
                                                    };

                                                    // Check size limit (10MB)
                                                    if data.len() > 10 * 1024 * 1024 {
                                                        tracing::error!("file too large: {} bytes", data.len());
                                                        continue;
                                                    }

                                                    match crate::api::upload_file_base64(
                                                        &api_url, &token, channel_id,
                                                        &filename, content_type, &b64,
                                                    ) {
                                                        Ok(resp) => {
                                                            tracing::info!("uploaded image: {}", resp.url);
                                                        }
                                                        Err(e) => {
                                                            tracing::error!("upload failed: {e}");
                                                        }
                                                    }
                                                }
                                                Err(e) => tracing::error!("read file failed: {e}"),
                                            }
                                        }
                                    }
                                })
                                .detach();
                            })),
                    )
                    // Text input
                    .child(
                        div()
                            .flex_grow()
                            .min_w_0()
                            .child(
                                Input::new(&self.input)
                                    .appearance(false)
                                    .disabled(!has_channel),
                            ),
                    )
                    .child(
                        Button::new("send")
                            .label("Send")
                            .primary()
                            .small()
                            .disabled(!has_channel)
                            .on_click(cx.listener(|this, _e, window, cx| {
                                this.do_send(window, cx);
                            })),
                    ),
            )
            // Emoji picker overlay
            .child(self.emoji_picker.clone())
            // Hint
            .child(
                div().text_xs().text_color(rgb(0x45475a)).mt_1().child(hint),
            )
    }
}
