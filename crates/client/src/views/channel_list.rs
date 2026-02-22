use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::input::{Input, InputState};
use uuid::Uuid;

use crate::state::AppState;

/// Emitted to request voice join from the voice panel.
pub struct VoiceJoinRequested {
    pub channel_id: Uuid,
}

/// Channel list sidebar for the currently selected server.
pub struct ChannelList {
    state: Entity<AppState>,
    create_input: Option<Entity<InputState>>,
    creating_voice: bool,
    /// Right-click context menu: (channel_id, channel_name, is_voice)
    context_menu: Option<(Uuid, String, bool)>,
    /// Rename mode: (channel_id, input)
    rename_input: Option<(Uuid, Entity<InputState>)>,
}

impl ChannelList {
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
            create_input: None,
            creating_voice: false,
            context_menu: None,
            rename_input: None,
        }
    }

    fn select_channel(&mut self, channel_id: Uuid, _window: &mut Window, cx: &mut Context<Self>) {
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        self.state.read(cx).subscribe_channel(channel_id);
        self.context_menu = None;

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

    fn start_create(&mut self, voice: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.creating_voice = voice;
        self.context_menu = None;
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(if voice { "voice-channel-name" } else { "channel-name" })
        });
        self.create_input = Some(input);
        cx.notify();
    }

    fn submit_create(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let name = match &self.create_input {
            Some(input) => input.read(cx).text().to_string(),
            None => return,
        };
        if name.trim().is_empty() {
            self.create_input = None;
            cx.notify();
            return;
        }

        let server_id = match self.state.read(cx).selected_server {
            Some(id) => id,
            None => return,
        };
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let ch_type = if self.creating_voice { "voice" } else { "text" }.to_string();
        let state = self.state.clone();

        self.create_input = None;
        cx.notify();

        let task = cx.background_executor().spawn(async move {
            crate::api::create_channel(&api_url, &token, server_id, &name, &ch_type)
        });

        cx.spawn(async move |_this, cx| {
            match task.await {
                Ok(ch) => {
                    let _ = cx.update(|cx| {
                        state.update(cx, |state, _cx| {
                            state.subscribe_channel(ch.id);
                        });
                    });
                }
                Err(e) => tracing::error!("create channel failed: {e}"),
            }
        })
        .detach();
    }

    fn cancel_create(&mut self, cx: &mut Context<Self>) {
        self.create_input = None;
        cx.notify();
    }

    fn delete_channel(&mut self, channel_id: Uuid, _window: &mut Window, cx: &mut Context<Self>) {
        self.context_menu = None;
        let server_id = match self.state.read(cx).selected_server {
            Some(id) => id,
            None => return,
        };
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let task = cx.background_executor().spawn(async move {
            crate::api::delete_channel_api(&api_url, &token, server_id, channel_id)
        });

        cx.spawn(async move |_this, cx| {
            if task.await.is_ok() {
                let _ = cx.update(|cx| {
                    state.update(cx, |state, cx| {
                        state.channels.retain(|c| c.id != channel_id);
                        if state.selected_channel == Some(channel_id) {
                            state.selected_channel = None;
                            state.messages.clear();
                        }
                        cx.notify();
                    });
                });
            }
        })
        .detach();
    }

    fn start_rename(&mut self, channel_id: Uuid, current_name: String, window: &mut Window, cx: &mut Context<Self>) {
        self.context_menu = None;
        let input = cx.new(|cx| {
            let mut s = InputState::new(window, cx).placeholder("new name");
            s.set_value(&current_name, window, cx);
            s
        });
        self.rename_input = Some((channel_id, input));
        cx.notify();
    }

    fn submit_rename(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let (channel_id, ref input) = match &self.rename_input {
            Some(r) => r.clone(),
            None => return,
        };
        let new_name = input.read(cx).text().to_string();
        if new_name.trim().is_empty() {
            self.rename_input = None;
            cx.notify();
            return;
        }

        let server_id = match self.state.read(cx).selected_server {
            Some(id) => id,
            None => return,
        };
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        self.rename_input = None;
        cx.notify();

        let task = cx.background_executor().spawn(async move {
            crate::api::rename_channel(&api_url, &token, server_id, channel_id, &new_name)
        });

        cx.spawn(async move |_this, cx| {
            match task.await {
                Ok(updated) => {
                    let _ = cx.update(|cx| {
                        state.update(cx, |state, cx| {
                            if let Some(ch) = state.channels.iter_mut().find(|c| c.id == channel_id) {
                                ch.name = updated.name;
                            }
                            cx.notify();
                        });
                    });
                }
                Err(e) => tracing::error!("rename channel failed: {e}"),
            }
        })
        .detach();
    }
}

impl EventEmitter<VoiceJoinRequested> for ChannelList {}

impl Render for ChannelList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text_channels: Vec<_> = self.state.read(cx).current_channels()
            .into_iter().map(|c| (c.id, c.name.clone())).collect();
        let voice_channels: Vec<_> = self.state.read(cx).current_voice_channels()
            .into_iter().map(|c| (c.id, c.name.clone())).collect();
        let selected = self.state.read(cx).selected_channel;
        let in_voice = self.state.read(cx).voice_channel;
        let has_server = self.state.read(cx).selected_server.is_some();
        let is_creating = self.create_input.is_some();
        let is_creating_voice = self.creating_voice;
        let ctx_menu = self.context_menu.clone();
        let renaming = self.rename_input.as_ref().map(|(id, _)| *id);

        div()
            .flex()
            .flex_col()
            .py_2()
            .size_full()
            .overflow_y_scrollbar()
            
            // TEXT CHANNELS header + "+"
            .child(
                div()
                    .flex().items_center().justify_between().px_3().py_1()
                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0x6c7086)).child("TEXT CHANNELS"))
                    .when(has_server, |d: Div| {
                        d.child(
                            div().id("add-text-ch").text_xs().text_color(rgb(0x6c7086)).cursor_pointer()
                                .hover(|s| s.text_color(rgb(0xcdd6f4))).child("+")
                                .on_click(cx.listener(|this, _e, window, cx| { this.start_create(false, window, cx); })),
                        )
                    }),
            )
            // Create text channel input
            .when(is_creating && !is_creating_voice, |d: gpui_component::scroll::Scrollable<Div>| {
                if let Some(ref input) = self.create_input {
                    d.child(
                        div().mx_2().mb_1().flex().items_center().gap_1().px_1().rounded_md().bg(rgb(0x313244))
                            .child(div().text_color(rgb(0x6c7086)).text_sm().child("#"))
                            .child(div().flex_grow().child(Input::new(input).appearance(false)))
                            .child(div().id("create-ch-ok").text_xs().text_color(rgb(0xa6e3a1)).cursor_pointer().child("✓")
                                .on_click(cx.listener(|this, _e, window, cx| { this.submit_create(window, cx); })))
                            .child(div().id("create-ch-cancel").text_xs().text_color(rgb(0xf38ba8)).cursor_pointer().child("✕")
                                .on_click(cx.listener(|this, _e, _w, cx| { this.cancel_create(cx); }))),
                    )
                } else { d }
            })
            // Text channels — flat_map so context menu appears right below the clicked channel
            .children(text_channels.into_iter().flat_map(|(id, name)| {
                let is_selected = selected == Some(id);
                let ch_id = id;
                let ch_name = name.clone();
                let ch_name2 = name.clone();
                let is_renaming = renaming == Some(id);
                let show_menu = ctx_menu.as_ref().map(|m| m.0 == id && !m.2).unwrap_or(false);

                let mut items: Vec<AnyElement> = Vec::new();

                if is_renaming {
                    let input = &self.rename_input.as_ref().unwrap().1;
                    items.push(div()
                        .mx_2().mb_1().flex().items_center().gap_1().px_1().rounded_md().bg(rgb(0x313244))
                        .child(div().text_color(rgb(0x6c7086)).text_sm().child("#"))
                        .child(div().flex_grow().child(Input::new(input).appearance(false)))
                        .child(div().id(ElementId::Name(format!("rename-ok-{id}").into())).text_xs().text_color(rgb(0xa6e3a1)).cursor_pointer().child("✓")
                            .on_click(cx.listener(|this, _e, window, cx| { this.submit_rename(window, cx); })))
                        .child(div().id(ElementId::Name(format!("rename-cancel-{id}").into())).text_xs().text_color(rgb(0xf38ba8)).cursor_pointer().child("✕")
                            .on_click(cx.listener(|this, _e, _w, cx| { this.rename_input = None; cx.notify(); })))
                        .into_any_element());
                } else {
                    items.push(div()
                        .id(ElementId::Name(format!("ch-{id}").into()))
                        .flex().items_center().gap_1().mx_2().px_2().py(px(5.)).rounded_md().cursor_pointer()
                        .when(is_selected, |d: Stateful<Div>| d.bg(rgb(0x313244)))
                        .hover(|s| s.bg(rgb(0x45475a)))
                        .child(div().text_color(rgb(0x6c7086)).text_sm().child("#"))
                        .child(div().text_sm().text_color(if is_selected { rgb(0xcdd6f4) } else { rgb(0xa6adc8) }).flex_grow().child(name))
                        .on_click(cx.listener(move |this, _e, window, cx| { this.select_channel(id, window, cx); }))
                        .on_mouse_down(MouseButton::Right, cx.listener(move |this, _e, _w, cx| {
                            this.context_menu = Some((ch_id, ch_name.clone(), false));
                            cx.notify();
                        }))
                        .into_any_element());
                }

                // Context menu right below this channel
                if show_menu {
                    items.push(div()
                        .mx_2().p_1().rounded_md().bg(rgb(0x1e1e2e)).border_1().border_color(rgb(0x45475a))
                        .flex().flex_col().gap_0p5()
                        .child(
                            div().id(ElementId::Name(format!("ctx-rename-{ch_id}").into())).px_2().py_1().text_sm().text_color(rgb(0xcdd6f4)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("✏️ Rename Channel")
                                .on_click(cx.listener(move |this, _e, window, cx| {
                                    this.start_rename(ch_id, ch_name2.clone(), window, cx);
                                })),
                        )
                        .child(
                            div().id(ElementId::Name(format!("ctx-delete-{ch_id}").into())).px_2().py_1().text_sm().text_color(rgb(0xf38ba8)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("🗑 Delete Channel")
                                .on_click(cx.listener(move |this, _e, window, cx| {
                                    this.delete_channel(ch_id, window, cx);
                                })),
                        )
                        .into_any_element());
                }

                items
            }))
            // VOICE CHANNELS header + "+"
            .child(
                div().flex().items_center().justify_between().px_3().py_1().mt_3()
                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0x6c7086)).child("VOICE CHANNELS"))
                    .when(has_server, |d: Div| {
                        d.child(
                            div().id("add-voice-ch").text_xs().text_color(rgb(0x6c7086)).cursor_pointer()
                                .hover(|s| s.text_color(rgb(0xcdd6f4))).child("+")
                                .on_click(cx.listener(|this, _e, window, cx| { this.start_create(true, window, cx); })),
                        )
                    }),
            )
            // Create voice channel input
            .when(is_creating && is_creating_voice, |d: gpui_component::scroll::Scrollable<Div>| {
                if let Some(ref input) = self.create_input {
                    d.child(
                        div().mx_2().mb_1().flex().items_center().gap_1().px_1().rounded_md().bg(rgb(0x313244))
                            .child(div().text_color(rgb(0x6c7086)).text_sm().child("🔊"))
                            .child(div().flex_grow().child(Input::new(input).appearance(false)))
                            .child(div().id("create-vc-ok").text_xs().text_color(rgb(0xa6e3a1)).cursor_pointer().child("✓")
                                .on_click(cx.listener(|this, _e, window, cx| { this.submit_create(window, cx); })))
                            .child(div().id("create-vc-cancel").text_xs().text_color(rgb(0xf38ba8)).cursor_pointer().child("✕")
                                .on_click(cx.listener(|this, _e, _w, cx| { this.cancel_create(cx); }))),
                    )
                } else { d }
            })
            // Voice channels — flat_map for inline context menu
            .children(voice_channels.into_iter().flat_map(|(id, name)| {
                let is_active = in_voice == Some(id);
                let vc_id = id;
                let vc_name = name.clone();
                let vc_name2 = name.clone();
                let is_renaming = renaming == Some(id);
                let show_menu = ctx_menu.as_ref().map(|m| m.0 == id && m.2).unwrap_or(false);

                let mut items: Vec<AnyElement> = Vec::new();

                if is_renaming {
                    let input = &self.rename_input.as_ref().unwrap().1;
                    items.push(div()
                        .mx_2().mb_1().flex().items_center().gap_1().px_1().rounded_md().bg(rgb(0x313244))
                        .child(div().text_color(rgb(0x6c7086)).text_sm().child("🔊"))
                        .child(div().flex_grow().child(Input::new(input).appearance(false)))
                        .child(div().id(ElementId::Name(format!("rename-ok-{id}").into())).text_xs().text_color(rgb(0xa6e3a1)).cursor_pointer().child("✓")
                            .on_click(cx.listener(|this, _e, window, cx| { this.submit_rename(window, cx); })))
                        .child(div().id(ElementId::Name(format!("rename-cancel-{id}").into())).text_xs().text_color(rgb(0xf38ba8)).cursor_pointer().child("✕")
                            .on_click(cx.listener(|this, _e, _w, cx| { this.rename_input = None; cx.notify(); })))
                        .into_any_element());
                } else {
                    items.push(div()
                        .id(ElementId::Name(format!("vc-{id}").into()))
                        .flex().items_center().gap_1().mx_2().px_2().py(px(5.)).rounded_md().cursor_pointer()
                        .when(is_active, |d: Stateful<Div>| d.bg(rgb(0x313244)))
                        .hover(|s| s.bg(rgb(0x45475a)))
                        .child(div().text_color(if is_active { rgb(0xa6e3a1) } else { rgb(0x6c7086) }).text_sm().child("🔊"))
                        .child(div().text_sm().text_color(if is_active { rgb(0xa6e3a1) } else { rgb(0xa6adc8) }).flex_grow().child(name))
                        .on_click(cx.listener(move |_this, _e, _w, cx| { cx.emit(VoiceJoinRequested { channel_id: id }); }))
                        .on_mouse_down(MouseButton::Right, cx.listener(move |this, _e, _w, cx| {
                            this.context_menu = Some((vc_id, vc_name.clone(), true));
                            cx.notify();
                        }))
                        .into_any_element());
                }

                if show_menu {
                    items.push(div()
                        .mx_2().p_1().rounded_md().bg(rgb(0x1e1e2e)).border_1().border_color(rgb(0x45475a))
                        .flex().flex_col().gap_0p5()
                        .child(
                            div().id(ElementId::Name(format!("ctx-rename-vc-{vc_id}").into())).px_2().py_1().text_sm().text_color(rgb(0xcdd6f4)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("✏️ Rename Channel")
                                .on_click(cx.listener(move |this, _e, window, cx| {
                                    this.start_rename(vc_id, vc_name2.clone(), window, cx);
                                })),
                        )
                        .child(
                            div().id(ElementId::Name(format!("ctx-delete-vc-{vc_id}").into())).px_2().py_1().text_sm().text_color(rgb(0xf38ba8)).cursor_pointer().rounded_sm()
                                .hover(|s| s.bg(rgb(0x45475a))).child("🗑 Delete Channel")
                                .on_click(cx.listener(move |this, _e, window, cx| {
                                    this.delete_channel(vc_id, window, cx);
                                })),
                        )
                        .into_any_element());
                }

                items
            }))
    }
}
