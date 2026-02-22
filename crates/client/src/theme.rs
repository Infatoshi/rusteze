/// Color palette for the UI theme.
#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg_primary: u32,    // main content area
    pub bg_secondary: u32,  // channel sidebar
    pub bg_tertiary: u32,   // server strip
    pub bg_input: u32,      // input fields, cards
    pub bg_hover: u32,      // hover state
    pub text_primary: u32,  // main text
    pub text_secondary: u32,// dimmed text
    pub text_muted: u32,    // very dim (labels, hints)
    pub accent: u32,        // blue accent (selected items)
    pub border: u32,        // borders, dividers
    pub success: u32,       // green (online, voice connected)
    pub danger: u32,        // red (disconnect, delete)
    pub warning: u32,       // yellow (pins)
}

impl ThemeColors {
    /// Catppuccin Mocha — the current default dark theme.
    pub fn dark() -> Self {
        Self {
            bg_primary: 0x1e1e2e,
            bg_secondary: 0x181825,
            bg_tertiary: 0x11111b,
            bg_input: 0x313244,
            bg_hover: 0x45475a,
            text_primary: 0xcdd6f4,
            text_secondary: 0xa6adc8,
            text_muted: 0x6c7086,
            accent: 0x89b4fa,
            border: 0x313244,
            success: 0xa6e3a1,
            danger: 0xf38ba8,
            warning: 0xf9e2af,
        }
    }

    /// Light theme.
    pub fn light() -> Self {
        Self {
            bg_primary: 0xffffff,
            bg_secondary: 0xf2f3f5,
            bg_tertiary: 0xe3e5e8,
            bg_input: 0xebedef,
            bg_hover: 0xd4d7dc,
            text_primary: 0x2e3338,
            text_secondary: 0x4f5660,
            text_muted: 0x80848e,
            accent: 0x5865f2,
            border: 0xd4d7dc,
            success: 0x3ba55c,
            danger: 0xed4245,
            warning: 0xfaa61a,
        }
    }

    /// Gray / neutral theme.
    pub fn gray() -> Self {
        Self {
            bg_primary: 0x2b2d31,
            bg_secondary: 0x232428,
            bg_tertiary: 0x1e1f22,
            bg_input: 0x383a40,
            bg_hover: 0x404249,
            text_primary: 0xdbdee1,
            text_secondary: 0xb5bac1,
            text_muted: 0x80848e,
            accent: 0x5865f2,
            border: 0x3f4147,
            success: 0x23a55a,
            danger: 0xf23f43,
            warning: 0xf0b232,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "light" => Self::light(),
            "gray" => Self::gray(),
            _ => Self::dark(),
        }
    }
}
