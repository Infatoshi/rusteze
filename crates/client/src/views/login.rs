use gpui::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    label::Label,
};

use crate::state::AppState;

pub struct LoginView {
    state: Entity<AppState>,
    email_input: Entity<InputState>,
    password_input: Entity<InputState>,
}

impl LoginView {
    pub fn new(
        state: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let email_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Email address")
        });
        let password_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Password")
        });

        Self {
            state,
            email_input,
            password_input,
        }
    }

    fn do_login(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let email = self.email_input.read(cx).text().to_string();
        let password = self.password_input.read(cx).text().to_string();

        if email.is_empty() || password.is_empty() {
            self.state.update(cx, |s, cx| {
                s.error = Some("Please enter email and password".into());
                cx.notify();
            });
            return;
        }

        let api_url = self.state.read(cx).api_url.clone();
        let state = self.state.clone();

        self.state.update(cx, |s, cx| {
            s.loading = true;
            s.error = None;
            cx.notify();
        });

        let task = cx.background_executor().spawn(async move {
            crate::api::login(&api_url, &email, &password)
        });

        cx.spawn(async move |_this, cx| {
            let result = task.await;
            let _ = cx.update(|cx| {
                state.update(cx, |s, cx| {
                    match result {
                        Ok(auth) => {
                            s.token = Some(auth.token);
                            s.user = Some(rusteze_models::PartialUser {
                                id: auth.user_id,
                                username: String::new(),
                                discriminator: String::new(),
                                display_name: None,
                                avatar_url: None,
                                status: rusteze_models::UserStatus::Online,
                            });
                            s.loading = false;
                        }
                        Err(e) => {
                            s.error = Some(format!("Login failed: {e}"));
                            s.loading = false;
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
    }

    fn do_register(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let email = self.email_input.read(cx).text().to_string();
        let password = self.password_input.read(cx).text().to_string();

        if email.is_empty() || password.is_empty() {
            self.state.update(cx, |s, cx| {
                s.error = Some("Please enter email and password".into());
                cx.notify();
            });
            return;
        }

        let username = email.split('@').next().unwrap_or("user").to_string();
        let api_url = self.state.read(cx).api_url.clone();
        let state = self.state.clone();

        self.state.update(cx, |s, cx| {
            s.loading = true;
            s.error = None;
            cx.notify();
        });

        let task = cx.background_executor().spawn(async move {
            crate::api::register(&api_url, &username, &email, &password)
        });

        cx.spawn(async move |_this, cx| {
            let result = task.await;
            let _ = cx.update(|cx| {
                state.update(cx, |s, cx| {
                    match result {
                        Ok(auth) => {
                            s.token = Some(auth.token);
                            s.user = Some(rusteze_models::PartialUser {
                                id: auth.user_id,
                                username: String::new(),
                                discriminator: String::new(),
                                display_name: None,
                                avatar_url: None,
                                status: rusteze_models::UserStatus::Online,
                            });
                            s.loading = false;
                        }
                        Err(e) => {
                            s.error = Some(format!("Registration failed: {e}"));
                            s.loading = false;
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
    }
}

impl Render for LoginView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error_msg = self.state.read(cx).error.clone();
        let is_loading = self.state.read(cx).loading;
        let has_error = error_msg.is_some();

        div()
            .flex()
            .size_full()
            .justify_center()
            .items_center()
            .bg(rgb(0x1a1a2e))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_8()
                    .w(px(380.))
                    .rounded_xl()
                    .bg(rgb(0x16213e))
                    .border_1()
                    .border_color(rgb(0x2a2a5a))
                    // Title
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(28.))
                                    .text_color(rgb(0xffffff))
                                    .font_weight(FontWeight::BOLD)
                                    .child("Rusteze"),
                            )
                            .child(
                                Label::new("Sign in to your account")
                                    .text_color(rgb(0x888888)),
                            ),
                    )
                    // Error
                    .when(has_error, |d: Div| {
                        d.child(
                            div()
                                .p_3()
                                .rounded_md()
                                .bg(rgb(0x3d1515))
                                .border_1()
                                .border_color(rgb(0x6b2020))
                                .child(
                                    Label::new(error_msg.unwrap_or_default())
                                        .text_color(rgb(0xff6b6b)),
                                ),
                        )
                    })
                    // Email
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Label::new("Email").text_color(rgb(0xcccccc)))
                            .child(Input::new(&self.email_input)),
                    )
                    // Password
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Label::new("Password").text_color(rgb(0xcccccc)))
                            .child(Input::new(&self.password_input)),
                    )
                    // Buttons
                    .child(
                        h_flex()
                            .gap_3()
                            .mt_2()
                            .w_full()
                            .child(
                                div().flex_1().child(
                                    Button::new("login")
                                        .label("Sign In")
                                        .primary()
                                        .w_full()
                                        .loading(is_loading)
                                        .on_click(cx.listener(|this, _event, window, cx| {
                                            this.do_login(window, cx);
                                        })),
                                ),
                            )
                            .child(
                                div().flex_1().child(
                                    Button::new("register")
                                        .label("Register")
                                        .w_full()
                                        .on_click(cx.listener(|this, _event, window, cx| {
                                            this.do_register(window, cx);
                                        })),
                                ),
                            ),
                    ),
            )
    }
}
