use gpui::*;
use gpui::prelude::FluentBuilder as _;

use crate::state::AppState;
use crate::views::channel_list::{ChannelList, VoiceJoinRequested};
use crate::views::create_server::CreateServerDialog;
use crate::views::dm_list::DmList;
use crate::views::finder::FinderPalette;
use crate::views::invite_dialog::InviteDialog;
use crate::views::member_list::MemberList;
use crate::views::message_input::MessageInput;
use crate::views::message_list::MessageList;
use crate::views::server_strip::{CreateServerRequested, DmsRequested, SettingsRequested, ServerStrip};
use crate::views::settings_panel::SettingsPanel;
use crate::views::voice_panel::VoicePanel;

/// Main application layout
pub struct AppLayout {
    state: Entity<AppState>,
    server_strip: Entity<ServerStrip>,
    channel_list: Entity<ChannelList>,
    dm_list: Entity<DmList>,
    message_list: Entity<MessageList>,
    message_input: Entity<MessageInput>,
    voice_panel: Entity<VoicePanel>,
    member_list: Entity<MemberList>,
    create_dialog: Entity<CreateServerDialog>,
    invite_dialog: Entity<InviteDialog>,
    settings_panel: Entity<SettingsPanel>,
    finder: Entity<FinderPalette>,
    sidebar_width: f32,
    dragging: bool,
}

impl AppLayout {
    pub fn new(
        state: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let server_strip = cx.new(|cx| ServerStrip::new(state.clone(), window, cx));
        let channel_list = cx.new(|cx| ChannelList::new(state.clone(), window, cx));
        let dm_list = cx.new(|cx| DmList::new(state.clone(), window, cx));
        let message_list = cx.new(|cx| MessageList::new(state.clone(), window, cx));
        let message_input = cx.new(|cx| MessageInput::new(state.clone(), window, cx));
        let voice_panel = cx.new(|cx| VoicePanel::new(state.clone(), window, cx));
        let member_list = cx.new(|cx| MemberList::new(state.clone(), window, cx));
        let create_dialog = cx.new(|cx| CreateServerDialog::new(state.clone(), window, cx));
        let invite_dialog = cx.new(|cx| InviteDialog::new(state.clone(), window, cx));
        let settings_panel = cx.new(|cx| SettingsPanel::new(state.clone(), window, cx));
        let finder = cx.new(|cx| FinderPalette::new(state.clone(), window, cx));

        // "+" button → create server dialog
        let dialog_handle = create_dialog.clone();
        cx.subscribe_in(&server_strip, window, move |_this, _strip, _event: &CreateServerRequested, _window, cx| {
            dialog_handle.update(cx, |dialog, cx| {
                dialog.show(cx);
            });
        })
        .detach();

        // Voice channel join
        let vp = voice_panel.clone();
        cx.subscribe_in(&channel_list, window, move |_this, _ch_list, event: &VoiceJoinRequested, window, cx| {
            vp.update(cx, |panel, cx| {
                panel.join_channel(event.channel_id, window, cx);
            });
        })
        .detach();

        // Settings gear
        let state_for_settings = state.clone();
        cx.subscribe_in(&server_strip, window, move |_this, _strip, _event: &SettingsRequested, _window, cx| {
            state_for_settings.update(cx, |s, cx| {
                s.show_settings = true;
                cx.notify();
            });
        })
        .detach();

        // DM button
        let state_for_dms = state.clone();
        cx.subscribe_in(&server_strip, window, move |_this, _strip, _event: &DmsRequested, _window, cx| {
            let api_url = state_for_dms.read(cx).api_url.clone();
            let token = state_for_dms.read(cx).token.clone().unwrap_or_default();
            let s = state_for_dms.clone();

            s.update(cx, |state, cx| {
                state.viewing_dms = true;
                state.selected_server = None;
                state.selected_channel = None;
                state.messages.clear();
                cx.notify();
            });

            // Fetch DM list in background
            let task = cx.background_executor().spawn(async move {
                crate::api::list_dms(&api_url, &token)
            });

            cx.spawn(async move |_this, cx| {
                if let Ok(dms) = task.await {
                    let _ = cx.update(|cx| {
                        s.update(cx, |state, cx| {
                            state.dm_channels = dms;
                            cx.notify();
                        });
                    });
                }
            })
            .detach();
        })
        .detach();

        // Re-render when state changes
        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            state,
            server_strip,
            channel_list,
            dm_list,
            message_list,
            message_input,
            voice_panel,
            member_list,
            create_dialog,
            invite_dialog,
            settings_panel,
            finder,
            sidebar_width: 220.0,
            dragging: false,
        }
    }
}

impl Render for AppLayout {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewing_dms = self.state.read(cx).viewing_dms;
        let server_name = if viewing_dms {
            "Direct Messages".to_string()
        } else {
            self.state.read(cx).current_server_name().to_string()
        };
        let sidebar_w = self.sidebar_width;
        let show_members = self.state.read(cx).show_members && !viewing_dms;
        let has_server = self.state.read(cx).selected_server.is_some();

        div()
            .flex()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .relative()
            // Global mouse-up to stop dragging anywhere in the window
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                if this.dragging {
                    this.dragging = false;
                    cx.notify();
                }
            }))
            // Global mouse-move for resize dragging
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if this.dragging {
                    let x: f32 = event.position.x.into();
                    let new_w = (x - 60.0).clamp(140.0, 500.0);
                    this.sidebar_width = new_w;
                    cx.notify();
                }
            }))
            // Server strip
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .w(px(60.))
                    .h_full()
                    .bg(rgb(0x11111b))
                    .border_r_1()
                    .border_color(rgb(0x313244))
                    .child(self.server_strip.clone()),
            )
            // Channel/DM sidebar
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .w(px(sidebar_w))
                    .h_full()
                    .bg(rgb(0x181825))
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .h(px(48.))
                            .px_4()
                            .border_b_1()
                            .border_color(rgb(0x313244))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xcdd6f4))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(server_name),
                            )
                            // Invite button (only for servers)
                            .when(has_server, |d: Div| {
                                d.child(
                                    div()
                                        .id("invite-btn")
                                        .text_xs()
                                        .text_color(rgb(0x6c7086))
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(rgb(0xcdd6f4)))
                                        .child("📨")
                                        .on_click(cx.listener(|this, _e, _w, cx| {
                                            this.invite_dialog.update(cx, |d, cx| d.show(cx));
                                        })),
                                )
                            }),
                    )
                    // Channel list or DM list
                    .child(
                        div()
                            .flex()
                            .flex_grow()
                            .overflow_hidden()
                            .child(if viewing_dms {
                                self.dm_list.clone().into_any_element()
                            } else {
                                self.channel_list.clone().into_any_element()
                            }),
                    )
                    // Voice panel (only for servers)
                    .when(!viewing_dms, |d: Div| {
                        d.child(self.voice_panel.clone())
                    }),
            )
            // Resize handle — invisible, just changes cursor
            .child(
                div()
                    .id("resize-handle")
                    .flex_shrink_0()
                    .w(px(8.))
                    .h_full()
                    .cursor(CursorStyle::ResizeLeftRight)
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                        this.dragging = true;
                        cx.notify();
                    })),
            )
            // Content area
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .min_w_0()
                    .h_full()
                    // Channel header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .h(px(48.))
                            .px_4()
                            .border_b_1()
                            .border_color(rgb(0x313244))
                            .child({
                                let (icon, ch_name) = if viewing_dms {
                                    ("💬".to_string(), self.state.read(cx).selected_channel
                                        .and_then(|ch_id| {
                                            self.state.read(cx).dm_channels.iter()
                                                .find(|d| d.id == ch_id)
                                                .map(|d| d.name.clone())
                                        })
                                        .unwrap_or_else(|| "Select a conversation".into()))
                                } else {
                                    ("#".to_string(), self.state.read(cx).selected_channel
                                        .and_then(|ch_id| {
                                            self.state.read(cx).channels.iter()
                                                .find(|c| c.id == ch_id)
                                                .map(|c| c.name.clone())
                                        })
                                        .unwrap_or_else(|| "select a channel".into()))
                                };
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().text_color(rgb(0x6c7086)).text_sm().child(icon))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xcdd6f4))
                                            .child(ch_name),
                                    )
                            })
                            .child(
                                div()
                                    .id("toggle-members")
                                    .cursor_pointer()
                                    .text_sm()
                                    .text_color(if show_members { rgb(0xcdd6f4) } else { rgb(0x6c7086) })
                                    .hover(|s| s.text_color(rgb(0xcdd6f4)))
                                    .child("👥")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.state.update(cx, |state, cx| {
                                            state.show_members = !state.show_members;
                                            cx.notify();
                                        });
                                    })),
                            ),
                    )
                    // Messages + input column alongside member list
                    .child(
                        div()
                            .flex()
                            .flex_grow()
                            .min_h_0()
                            // Left: messages + input (flex column, grows)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_grow()
                                    .min_w_0()
                                    // Message list (grows to fill)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_grow()
                                            .min_h_0()
                                            .overflow_hidden()
                                            .child(self.message_list.clone()),
                                    )
                                    // Input (fixed at bottom, only in message column)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_shrink_0()
                                            .w_full()
                                            .child(self.message_input.clone()),
                                    ),
                            )
                            // Right: member list (fixed width, scrollable)
                            .when(show_members, |d: Div| {
                                d.child(
                                    div()
                                        .flex_shrink_0()
                                        .w(px(200.))
                                        .h_full()
                                        .border_l_1()
                                        .border_color(rgb(0x313244))
                                        .overflow_hidden()
                                        .child(self.member_list.clone()),
                                )
                            }),
                    ),
            )
            // Overlays
            .child(self.create_dialog.clone())
            .child(self.invite_dialog.clone())
            .child(self.settings_panel.clone())
            .child(self.finder.clone())
    }
}
