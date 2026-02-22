use gpui::*;
use gpui::prelude::FluentBuilder as _;

/// Common emoji organized by category.
const EMOJI_SMILEYS: &[&str] = &[
    "😀", "😃", "😄", "😁", "😆", "😅", "🤣", "😂", "🙂", "😊",
    "😇", "🥰", "😍", "🤩", "😘", "😗", "😚", "😋", "😛", "😜",
    "🤪", "😝", "🤑", "🤗", "🤭", "🤫", "🤔", "🤐", "🤨", "😐",
    "😑", "😶", "😏", "😒", "🙄", "😬", "🤥", "😌", "😔", "😪",
    "🤤", "😴", "😷", "🤒", "🤕", "🤢", "🤮", "🥵", "🥶", "🥴",
];

const EMOJI_HANDS: &[&str] = &[
    "👋", "🤚", "🖐", "✋", "🖖", "👌", "🤌", "🤏", "✌️", "🤞",
    "🤟", "🤘", "🤙", "👈", "👉", "👆", "👇", "☝️", "👍", "👎",
    "✊", "👊", "🤛", "🤜", "👏", "🙌", "👐", "🤲", "🤝", "🙏",
];

const EMOJI_HEARTS: &[&str] = &[
    "❤️", "🧡", "💛", "💚", "💙", "💜", "🖤", "🤍", "🤎", "💔",
    "❣️", "💕", "💞", "💓", "💗", "💖", "💘", "💝", "💟", "♥️",
];

const EMOJI_OBJECTS: &[&str] = &[
    "🔥", "⭐", "🌟", "✨", "💫", "🎉", "🎊", "🎈", "🎁", "🏆",
    "🥇", "🎯", "🎮", "🎲", "🎵", "🎶", "🔔", "📢", "💡", "📌",
    "📎", "✏️", "📝", "📁", "📊", "🔗", "🔒", "🔓", "🔑", "⚡",
];

const EMOJI_NATURE: &[&str] = &[
    "🌈", "☀️", "🌙", "⭐", "🌍", "🌊", "🌸", "🌺", "🌻", "🌹",
    "🍀", "🌿", "🍃", "🌴", "🌵", "🍄", "🐶", "🐱", "🐭", "🐰",
];

/// Emitted when an emoji is selected.
pub struct EmojiSelected {
    pub emoji: String,
}

/// A simple emoji picker overlay.
pub struct EmojiPicker {
    visible: bool,
    category: usize, // 0=smileys, 1=hands, 2=hearts, 3=objects, 4=nature
}

impl EmojiPicker {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            category: 0,
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.visible = !self.visible;
        cx.notify();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn get_emojis(&self) -> (&str, &[&str]) {
        match self.category {
            0 => ("Smileys", EMOJI_SMILEYS),
            1 => ("Hands", EMOJI_HANDS),
            2 => ("Hearts", EMOJI_HEARTS),
            3 => ("Objects", EMOJI_OBJECTS),
            4 => ("Nature", EMOJI_NATURE),
            _ => ("Smileys", EMOJI_SMILEYS),
        }
    }
}

impl EventEmitter<EmojiSelected> for EmojiPicker {}

impl Render for EmojiPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let (cat_name, emojis) = self.get_emojis();
        let categories = ["😀", "👋", "❤️", "🔥", "🌈"];

        div()
            .absolute()
            .bottom(px(60.))
            .left(px(20.))
            .w(px(320.))
            .max_h(px(300.))
            .rounded_xl()
            .bg(rgb(0x1e1e2e))
            .border_1()
            .border_color(rgb(0x45475a))
            .flex()
            .flex_col()
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_e, _w, _cx| {}) // Prevent click-through
            // Category tabs
            .child(
                div()
                    .flex()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .children(categories.iter().enumerate().map(|(i, &cat_emoji)| {
                        let is_active = self.category == i;
                        div()
                            .id(ElementId::Name(format!("ecat-{i}").into()))
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .when(is_active, |d: Stateful<Div>| d.bg(rgb(0x313244)))
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .child(cat_emoji)
                            .on_click(cx.listener(move |this, _e, _w, cx| {
                                this.category = i;
                                cx.notify();
                            }))
                    })),
            )
            // Category name
            .child(
                div()
                    .px_3()
                    .py_1()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0x6c7086))
                    .child(cat_name.to_string()),
            )
            // Emoji grid
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_0p5()
                    .px_2()
                    .py_1()
                    .overflow_hidden()
                    .children(emojis.to_vec().into_iter().map(|emoji| {
                        let e = emoji.to_string();
                        div()
                            .id(ElementId::Name(format!("emoji-{emoji}").into()))
                            .w(px(32.))
                            .h(px(32.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x313244)))
                            .child(emoji.to_string())
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.visible = false;
                                cx.emit(EmojiSelected { emoji: e.clone() });
                                cx.notify();
                            }))
                    })),
            )
            .into_any_element()
    }
}
