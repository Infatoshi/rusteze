use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    input::{Input, InputState},
    label::Label,
    switch::Switch,
    Sizable as _,
};

use crate::state::AppState;
use crate::theme::ThemeColors;

/// Full-page settings panel (replaces content area when open).
pub struct SettingsPanel {
    state: Entity<AppState>,
    // Account fields
    display_name_input: Entity<InputState>,
    bio_input: Entity<InputState>,
    save_message: Option<String>,
}

impl SettingsPanel {
    pub fn new(
        state: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let display_name_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Display name")
        });
        let bio_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Tell us about yourself")
                .auto_grow(1, 3)
                .soft_wrap(true)
        });

        cx.observe(&state, |_this, _state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            state,
            display_name_input,
            bio_input,
            save_message: None,
        }
    }

    fn save_profile(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let display_name = self.display_name_input.read(cx).text().to_string();
        let bio = self.bio_input.read(cx).text().to_string();
        let api_url = self.state.read(cx).api_url.clone();
        let token = self.state.read(cx).token.clone().unwrap_or_default();
        let state = self.state.clone();

        let dn = if display_name.trim().is_empty() { None } else { Some(display_name) };
        let b = if bio.trim().is_empty() { None } else { Some(bio) };

        let task = cx.background_executor().spawn(async move {
            crate::api::update_profile(
                &api_url,
                &token,
                dn.as_deref(),
                b.as_deref(),
            )
        });

        cx.spawn(async move |this, cx| {
            match task.await {
                Ok(profile) => {
                    let _ = cx.update(|cx| {
                        state.update(cx, |state, cx| {
                            if let Some(ref mut user) = state.user {
                                user.display_name = profile.display_name;
                                user.username = profile.username;
                            }
                            cx.notify();
                        });
                        let _ = this.update(cx, |this, cx| {
                            this.save_message = Some("Profile saved!".into());
                            cx.notify();
                        });
                    });
                }
                Err(e) => {
                    let _ = cx.update(|cx| {
                        let _ = this.update(cx, |this, cx| {
                            this.save_message = Some(format!("Error: {e}"));
                            cx.notify();
                        });
                    });
                }
            }
        })
        .detach();
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.show_settings = false;
            cx.notify();
        });
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.state.read(cx).theme_colors();
        let selected = self.state.read(cx).settings_category.clone();

        let categories = [
            ("account", "My Account"),
            ("appearance", "Appearance"),
            ("voice", "Voice & Video"),
            ("chat", "Chat"),
        ];

        div()
            .flex()
            .flex_col()
            .w(px(200.))
            .flex_shrink_0()
            .h_full()
            .bg(rgb(t.bg_tertiary))
            .p_4()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(t.text_muted))
                    .mb_2()
                    .child("USER SETTINGS"),
            )
            .children(categories.into_iter().map(|(id, label)| {
                let is_selected = selected == id;
                let cat_id = id.to_string();

                div()
                    .id(ElementId::Name(format!("settings-{id}").into()))
                    .px_3()
                    .py(px(6.))
                    .rounded_md()
                    .cursor_pointer()
                    .text_sm()
                    .when(is_selected, |d: Stateful<Div>| d.bg(rgb(t.bg_hover)).text_color(rgb(t.text_primary)))
                    .when(!is_selected, |d: Stateful<Div>| d.text_color(rgb(t.text_secondary)))
                    .hover(|s| s.bg(rgb(t.bg_hover)))
                    .child(label)
                    .on_click(cx.listener(move |this, _e, _w, cx| {
                        this.state.update(cx, |s, cx| {
                            s.settings_category = cat_id.clone();
                            cx.notify();
                        });
                    }))
            }))
            .child(
                div().flex_grow(), // spacer
            )
            .child(
                div()
                    .id("settings-close-sidebar")
                    .px_3()
                    .py(px(6.))
                    .rounded_md()
                    .cursor_pointer()
                    .text_sm()
                    .text_color(rgb(t.danger))
                    .hover(|s| s.bg(rgb(t.bg_hover)))
                    .child("Close Settings")
                    .on_click(cx.listener(|this, _e, _w, cx| {
                        this.close(cx);
                    })),
            )
    }

    fn render_account(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.state.read(cx).theme_colors();
        let user = self.state.read(cx).user.clone();
        let username = user.as_ref().map(|u| u.username.clone()).unwrap_or_default();
        let display = user.as_ref().and_then(|u| u.display_name.clone()).unwrap_or_default();
        let email = self.state.read(cx).user.as_ref()
            .map(|_| "***@***.com".to_string()) // email not in PartialUser
            .unwrap_or_default();
        let save_msg = self.save_message.clone();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .max_w(px(600.))
            .child(
                div()
                    .text_color(rgb(t.text_primary))
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(20.))
                    .child("My Account"),
            )
            // Avatar + username card
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .p_4()
                    .rounded_lg()
                    .bg(rgb(t.bg_tertiary))
                    .child(
                        div()
                            .w(px(60.))
                            .h(px(60.))
                            .rounded_full()
                            .bg(rgb(t.accent))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(24.))
                            .text_color(rgb(t.bg_primary))
                            .font_weight(FontWeight::BOLD)
                            .child(
                                display.chars().next()
                                    .or(username.chars().next())
                                    .unwrap_or('?')
                                    .to_uppercase()
                                    .to_string(),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_color(rgb(t.text_primary))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(if display.is_empty() { username.clone() } else { display.clone() }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(t.text_muted))
                                    .child(format!("@{username}")),
                            ),
                    ),
            )
            // Display name
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(Label::new("Display Name").text_color(rgb(t.text_secondary)).text_size(px(12.)))
                    .child(Input::new(&self.display_name_input)),
            )
            // Bio
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(Label::new("About Me").text_color(rgb(t.text_secondary)).text_size(px(12.)))
                    .child(Input::new(&self.bio_input)),
            )
            // Email (readonly display)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(Label::new("Email").text_color(rgb(t.text_secondary)).text_size(px(12.)))
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(t.bg_input))
                            .text_sm()
                            .text_color(rgb(t.text_muted))
                            .child(email),
                    ),
            )
            // Save message
            .when(save_msg.is_some(), |d: Div| {
                d.child(
                    div()
                        .text_sm()
                        .text_color(rgb(t.success))
                        .child(save_msg.unwrap_or_default()),
                )
            })
            // Save button
            .child(
                div()
                    .flex()
                    .justify_end()
                    .child(
                        Button::new("save-profile")
                            .label("Save Changes")
                            .primary()
                            .small()
                            .on_click(cx.listener(|this, _e, window, cx| {
                                this.save_profile(window, cx);
                            })),
                    ),
            )
    }

    fn render_appearance(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.state.read(cx).theme_colors();
        let current_theme = self.state.read(cx).theme.clone();

        let themes = [
            ("dark", "Dark", ThemeColors::dark()),
            ("light", "Light", ThemeColors::light()),
            ("gray", "Gray", ThemeColors::gray()),
        ];

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .max_w(px(600.))
            .child(
                div()
                    .text_color(rgb(t.text_primary))
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(20.))
                    .child("Appearance"),
            )
            .child(
                Label::new("Choose your theme")
                    .text_color(rgb(t.text_secondary)),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .children(themes.into_iter().map(|(id, label, preview)| {
                        let is_selected = current_theme == id;
                        let theme_id = id.to_string();

                        div()
                            .id(ElementId::Name(format!("theme-{id}").into()))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .p_3()
                            .rounded_lg()
                            .cursor_pointer()
                            .border_2()
                            .when(is_selected, |d: Stateful<Div>| d.border_color(rgb(t.accent)))
                            .when(!is_selected, |d: Stateful<Div>| d.border_color(rgb(t.border)))
                            .hover(|s| s.border_color(rgb(t.accent)))
                            // Preview swatch
                            .child(
                                div()
                                    .w(px(80.))
                                    .h(px(50.))
                                    .rounded_md()
                                    .bg(rgb(preview.bg_primary))
                                    .border_1()
                                    .border_color(rgb(preview.border))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .w(px(20.))
                                            .h(px(20.))
                                            .rounded_full()
                                            .bg(rgb(preview.accent)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_selected { rgb(t.accent) } else { rgb(t.text_secondary) })
                                    .child(label),
                            )
                            .on_click(cx.listener(move |this, _e, _w, cx| {
                                this.state.update(cx, |s, cx| {
                                    s.theme = theme_id.clone();
                                    cx.notify();
                                });
                            }))
                    })),
            )
    }

    fn render_voice(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.state.read(cx).theme_colors();
        let selected_input = self.state.read(cx).selected_input_device.clone();
        let selected_output = self.state.read(cx).selected_output_device.clone();

        // Enumerate audio devices
        let host = cpal::default_host();
        use cpal::traits::HostTrait;
        let input_devices: Vec<String> = host
            .input_devices()
            .map(|devs| {
                use cpal::traits::DeviceTrait;
                devs.filter_map(|d| d.name().ok()).collect()
            })
            .unwrap_or_default();
        let output_devices: Vec<String> = host
            .output_devices()
            .map(|devs| {
                use cpal::traits::DeviceTrait;
                devs.filter_map(|d| d.name().ok()).collect()
            })
            .unwrap_or_default();

        // Default to first device if none selected
        let active_input = selected_input
            .or_else(|| input_devices.first().cloned())
            .unwrap_or_default();
        let active_output = selected_output
            .or_else(|| output_devices.first().cloned())
            .unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .max_w(px(600.))
            .child(
                div()
                    .text_color(rgb(t.text_primary))
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(20.))
                    .child("Voice & Video"),
            )
            // Input devices
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(Label::new("INPUT DEVICE").text_color(rgb(t.text_muted)).text_size(px(11.)))
                    .children(input_devices.into_iter().enumerate().map(|(i, name)| {
                        let is_active = name == active_input;
                        let dev_name = name.clone();

                        div()
                            .id(ElementId::Name(format!("input-dev-{i}").into()))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(if is_active { rgb(t.bg_hover) } else { rgb(t.bg_input) })
                            .hover(|s| s.bg(rgb(t.bg_hover)))
                            .when(is_active, |d: Stateful<Div>| {
                                d.border_l_2().border_color(rgb(t.success))
                            })
                            .child(
                                div()
                                    .w(px(16.))
                                    .text_sm()
                                    .text_color(if is_active { rgb(t.success) } else { rgb(t.text_muted) })
                                    .child(if is_active { "●" } else { "○" }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_active { rgb(t.text_primary) } else { rgb(t.text_secondary) })
                                    .child(name),
                            )
                            .on_click(cx.listener(move |this, _e, _w, cx| {
                                this.state.update(cx, |s, cx| {
                                    s.selected_input_device = Some(dev_name.clone());
                                    cx.notify();
                                });
                            }))
                    })),
            )
            // Output devices
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .child(Label::new("OUTPUT DEVICE").text_color(rgb(t.text_muted)).text_size(px(11.)))
                    .children(output_devices.into_iter().enumerate().map(|(i, name)| {
                        let is_active = name == active_output;
                        let dev_name = name.clone();

                        div()
                            .id(ElementId::Name(format!("output-dev-{i}").into()))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(if is_active { rgb(t.bg_hover) } else { rgb(t.bg_input) })
                            .hover(|s| s.bg(rgb(t.bg_hover)))
                            .when(is_active, |d: Stateful<Div>| {
                                d.border_l_2().border_color(rgb(t.success))
                            })
                            .child(
                                div()
                                    .w(px(16.))
                                    .text_sm()
                                    .text_color(if is_active { rgb(t.success) } else { rgb(t.text_muted) })
                                    .child(if is_active { "●" } else { "○" }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_active { rgb(t.text_primary) } else { rgb(t.text_secondary) })
                                    .child(name),
                            )
                            .on_click(cx.listener(move |this, _e, _w, cx| {
                                this.state.update(cx, |s, cx| {
                                    s.selected_output_device = Some(dev_name.clone());
                                    cx.notify();
                                });
                            }))
                    })),
            )
    }

    fn render_chat(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.state.read(cx).theme_colors();
        let enter_to_send = self.state.read(cx).enter_to_send;

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .max_w(px(600.))
            .child(
                div()
                    .text_color(rgb(t.text_primary))
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(20.))
                    .child("Chat"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(Label::new("Enter to send").text_color(rgb(t.text_primary)))
                            .child(
                                Label::new(if enter_to_send {
                                    "Enter sends message, Shift+Enter for newline"
                                } else {
                                    "Enter for newline, Cmd+Enter sends message"
                                })
                                .text_color(rgb(t.text_muted))
                                .text_size(px(11.)),
                            ),
                    )
                    .child(
                        Switch::new("enter-send-toggle")
                            .checked(enter_to_send)
                            .on_click(cx.listener(|this, _checked, _window, cx| {
                                this.state.update(cx, |s, cx| {
                                    s.enter_to_send = !s.enter_to_send;
                                    cx.notify();
                                });
                            })),
                    ),
            )
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_visible = self.state.read(cx).show_settings;
        if !is_visible {
            return div().into_any_element();
        }

        let t = self.state.read(cx).theme_colors();
        let category = self.state.read(cx).settings_category.clone();

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .bg(rgb(t.bg_primary))
            // Sidebar
            .child(self.render_sidebar(cx))
            // Content
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .h_full()
                    .bg(rgb(t.bg_secondary))
                    .overflow_hidden()
                    // Close button top-right
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .p_4()
                            .child(
                                div()
                                    .id("settings-close-x")
                                    .w(px(36.))
                                    .h(px(36.))
                                    .rounded_full()
                                    .bg(rgb(t.bg_input))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(rgb(t.text_muted))
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(rgb(t.text_primary)))
                                    .child("✕")
                                    .on_click(cx.listener(|this, _e, _w, cx| {
                                        this.close(cx);
                                    })),
                            ),
                    )
                    // Category content
                    .child(match category.as_str() {
                        "account" => self.render_account(cx).into_any_element(),
                        "appearance" => self.render_appearance(cx).into_any_element(),
                        "voice" => self.render_voice(cx).into_any_element(),
                        "chat" => self.render_chat(cx).into_any_element(),
                        _ => self.render_account(cx).into_any_element(),
                    }),
            )
            .into_any_element()
    }
}
