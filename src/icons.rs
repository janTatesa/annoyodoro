use iced::{
    Font, Pixels, Theme,
    widget::{Button, button, text, text::Shaping::Advanced}
};

pub static ICON_FONT: &[u8] = include_bytes!(env!("LUCIDE_PATH"));

macro_rules! icon {
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

// https://unpkg.com/lucide-static@0.541.0/font/info.json
icon!(pause_button = '\u{e132}');
icon!(resume_button = '\u{e140}');
