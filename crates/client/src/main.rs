mod api;
mod audio;
mod gateway;
pub mod state;
pub mod theme;
mod views;

use gpui::*;
use gpui_component::Root;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _};

use crate::views::app_root::AppRoot;

actions!(rusteze, [Quit, ToggleSettings, ToggleFinder, DismissOverlay]);

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("rusteze_client=debug")),
        )
        .init();

    let app = Application::new();

    app.run(move |cx| {
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(1100.), px(750.)), cx);
        let window_opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Rusteze".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(window_opts, |window, cx| {
            gpui_component::init(cx);
            let view = AppRoot::view(window, cx);
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap();

        // Global action handlers at App level (like Zed does in zed::init)
        cx.on_action(|_: &Quit, cx| cx.quit());

        // Keybindings
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            // These are scoped to "AppRoot" context — handlers registered on the root div
            KeyBinding::new("cmd-,", ToggleSettings, Some("AppRoot")),
            KeyBinding::new("cmd-k", ToggleFinder, Some("AppRoot")),
            KeyBinding::new("escape", DismissOverlay, Some("AppRoot")),
        ]);

        cx.activate(true);
    });
}
