use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    label::Label,
    Disableable as _,
    Sizable as _,
};

use crate::audio::AudioEngine;
use crate::state::AppState;

/// Voice connection panel at the bottom of the channel sidebar.
pub struct VoicePanel {
    state: Entity<AppState>,
    audio: AudioEngine,
}

impl VoicePanel {
    pub fn new(
        state: Entity<AppState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            state,
            audio: AudioEngine::new(),
        }
    }

    fn toggle_voice(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.audio.is_running() {
            self.audio.stop();
            self.state.update(cx, |state, cx| {
                state.voice_channel = None;
                state.is_muted = false;
                state.is_deafened = false;
                cx.notify();
            });
        } else {
            let voice_ch = self
                .state
                .read(cx)
                .current_voice_channels()
                .first()
                .map(|c| c.id);

            match self.audio.start() {
                Ok(()) => {
                    self.state.update(cx, |state, cx| {
                        state.voice_channel = voice_ch;
                        state.is_muted = false;
                        state.is_deafened = false;
                        cx.notify();
                    });
                }
                Err(e) => {
                    tracing::error!("failed to start audio: {e}");
                    self.state.update(cx, |state, cx| {
                        state.error = Some(format!("Audio error: {e}"));
                        cx.notify();
                    });
                }
            }
        }
        cx.notify();
    }

    /// Join a specific voice channel.
    pub fn join_channel(&mut self, channel_id: uuid::Uuid, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.read(cx).voice_channel == Some(channel_id) {
            self.audio.stop();
            self.state.update(cx, |state, cx| {
                state.voice_channel = None;
                state.is_muted = false;
                state.is_deafened = false;
                cx.notify();
            });
            cx.notify();
            return;
        }

        if self.audio.is_running() {
            self.audio.stop();
        }

        match self.audio.start() {
            Ok(()) => {
                self.state.update(cx, |state, cx| {
                    state.voice_channel = Some(channel_id);
                    state.is_muted = false;
                    state.is_deafened = false;
                    cx.notify();
                });
            }
            Err(e) => {
                tracing::error!("failed to start audio: {e}");
                self.state.update(cx, |state, cx| {
                    state.error = Some(format!("Audio error: {e}"));
                    cx.notify();
                });
            }
        }
        cx.notify();
    }

    fn toggle_mute(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.audio.toggle_mute();
        self.state.update(cx, |state, cx| {
            state.is_muted = self.audio.muted;
            cx.notify();
        });
        cx.notify();
    }

    fn toggle_deafen(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.audio.toggle_deafen();
        self.state.update(cx, |state, cx| {
            state.is_deafened = self.audio.deafened;
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for VoicePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_connected = self.audio.is_running();
        let is_muted = self.state.read(cx).is_muted;
        let is_deafened = self.state.read(cx).is_deafened;
        let has_server = self.state.read(cx).selected_server.is_some();
        let t = self.state.read(cx).theme_colors();

        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .border_t_1()
            .border_color(rgb(t.border))
            .bg(rgb(t.bg_tertiary))
            .p_3()
            .gap_2()
            // Connected status
            .when(is_connected, |d: Div| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .w(px(8.))
                                .h(px(8.))
                                .rounded_full()
                                .bg(rgb(t.success)),
                        )
                        .child(
                            Label::new("Voice Connected")
                                .text_color(rgb(t.success))
                                .text_size(px(12.)),
                        ),
                )
            })
            // Buttons row
            .child(
                div()
                    .flex()
                    .gap_1()
                    .when(!is_connected, |d: Div| {
                        d.child(
                            Button::new("voice-join")
                                .label("Join Voice")
                                .primary()
                                .small()
                                .w_full()
                                .disabled(!has_server)
                                .on_click(cx.listener(Self::toggle_voice))
                        )
                    })
                    .when(is_connected, |d: Div| {
                        d
                            // Mute button
                            .child(
                                div()
                                    .id("btn-mute")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(36.))
                                    .h(px(36.))
                                    .rounded_md()
                                    .cursor_pointer()
                                    .bg(if is_muted { rgb(t.danger) } else { rgb(t.bg_input) })
                                    .text_color(if is_muted { rgb(0xffffff) } else { rgb(t.text_secondary) })
                                    .hover(|s| s.bg(rgb(t.bg_hover)))
                                    .text_size(px(16.))
                                    .child(if is_muted { "🔇" } else { "🎤" })
                                    .on_click(cx.listener(Self::toggle_mute)),
                            )
                            // Deafen button
                            .child(
                                div()
                                    .id("btn-deafen")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(36.))
                                    .h(px(36.))
                                    .rounded_md()
                                    .cursor_pointer()
                                    .bg(if is_deafened { rgb(t.danger) } else { rgb(t.bg_input) })
                                    .text_color(if is_deafened { rgb(0xffffff) } else { rgb(t.text_secondary) })
                                    .hover(|s| s.bg(rgb(t.bg_hover)))
                                    .text_size(px(16.))
                                    .child(if is_deafened { "🔇" } else { "🔊" })
                                    .on_click(cx.listener(Self::toggle_deafen)),
                            )
                            // Disconnect button
                            .child(
                                div()
                                    .id("btn-disconnect")
                                    .flex()
                                    .flex_grow()
                                    .items_center()
                                    .justify_center()
                                    .h(px(36.))
                                    .rounded_md()
                                    .cursor_pointer()
                                    .bg(rgb(t.danger))
                                    .text_color(rgb(0xffffff))
                                    .hover(|s| s.bg(rgb(0xd63031)))
                                    .text_size(px(12.))
                                    .child("Disconnect")
                                    .on_click(cx.listener(Self::toggle_voice)),
                            )
                    }),
            )
    }
}
