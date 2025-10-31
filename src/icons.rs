use iced::{
    Font, Pixels, Theme,
    widget::{Button, button, text, text::Shaping::Advanced}
};

pub static ICON_FONT: &[u8] = include_bytes!(env!("LUCIDE_PATH"));

macro_rules! icon_button {
    ($name:ident = $icon:literal) => {
        pub fn $name<'a, T>(size: impl Into<Pixels>) -> Button<'a, T> {
            button(
                text($icon)
                    .shaping(Advanced)
                    .font(Font::with_name("lucide"))
                    .size(size)
            )
            .style(|theme: &Theme, status| match status {
                button::Status::Hovered => button::Style {
                    text_color: theme.palette().primary,
                    ..Default::default()
                },
                button::Status::Pressed => button::Style {
                    text_color: theme.extended_palette().primary.strong.color,
                    ..Default::default()
                },
                button::Status::Disabled | button::Status::Active => button::Style {
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
        }
    };
}

// https://unpkg.com/lucide-static@latest/font/info.json
icon_button!(pause_button = '\u{e12e}');
icon_button!(resume_button = '\u{e13c}');
