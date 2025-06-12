use iced::{
    Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{Container, column, rich_text, span, text},
};

use crate::{
    BreakTimer, BreakTimerMode,
    iced_implementation::{Message, config_color_to_iced_color},
};

impl BreakTimer {
    pub fn view(&self) -> Element<Message> {
        let primary = config_color_to_iced_color(self.colors_config.accent);
        let warning = config_color_to_iced_color(self.colors_config.warning);
        let (title_text, sign, timer_color) = match self.mode {
            BreakTimerMode::AfterBreak => ("Time to work!", "-", warning),
            _ if self.long_break => ("Time for a long break!", "", primary),
            _ => ("Time for a break!", "", primary),
        };

        let time_left = format!("{sign}{}:{:02}", self.seconds / 60, self.seconds % 60);
        let press_key_message: Option<Element<Message>> = match self.mode {
            BreakTimerMode::Running => None,
            BreakTimerMode::RunningWithOvertimeOption(duration) => Some(
                rich_text![
                    "Press ",
                    span("o").color(primary),
                    " to add ",
                    span(format!(
                        "{}:{:02}",
                        duration.as_secs() / 60,
                        duration.as_secs() % 60
                    ))
                    .color(primary),
                    " of overtime"
                ]
                .into(),
            ),
            BreakTimerMode::AfterBreak => Some("Press any key to continue working".into()),
        };

        let column = column![text(title_text), text(time_left).color(timer_color)]
            .push_maybe(press_key_message)
            .align_x(Horizontal::Center);
        Container::new(column)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
