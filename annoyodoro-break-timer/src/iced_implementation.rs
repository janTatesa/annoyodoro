use iced::{
    Color, Element, Event, Subscription, Task, Theme,
    event::{self, Status},
    exit,
    keyboard::{self, Key},
    theme::Palette,
    window::Id,
};
use iced_layershell::{Appearance, Application, to_layer_message};

use crate::{BreakTimer, BreakTimerMode, NAME};

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    // The user can press o and add overtime to the timer so it has to be a distinct message. Feel the pain. These days ima opressed aswell
    OPressed,
    // If the break is over the user can press any key to continue working
    OtherKeyPressed,
    Tick,
    // HACK: this message is never constructed but used by the macro
    IcedEvent(Event),
}

fn handle_iced_event(event: Event, _status: Status, _id: Id) -> Option<Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };

    Some(match (key, modifiers.is_empty()) {
        (Key::Character(char), true) if char.as_str() == "o" => Message::OPressed,
        _ => Message::OtherKeyPressed,
    })
}

impl Application for BreakTimer {
    type Message = Message;
    type Flags = BreakTimer;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(flags: BreakTimer) -> (Self, Task<Message>) {
        (flags, Task::none())
    }

    fn namespace(&self) -> String {
        NAME.to_string()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch([
            iced::time::every(iced::time::Duration::from_secs(1)).map(|_| Message::Tick),
            event::listen_with(handle_iced_event),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match (self.mode, message) {
            (BreakTimerMode::AfterBreak, Message::Tick) => self.seconds += 1,
            (_, Message::Tick) => match self.seconds {
                0 => {
                    *self = BreakTimer {
                        seconds: 1,
                        mode: BreakTimerMode::AfterBreak,
                        ..*self
                    }
                }
                _ => self.seconds -= 1,
            },
            (BreakTimerMode::RunningWithOvertimeOption(_), Message::OPressed)
            | (BreakTimerMode::AfterBreak, Message::OPressed | Message::OtherKeyPressed) => {
                return exit();
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        self.view()
    }

    fn style(&self, _theme: &Self::Theme) -> Appearance {
        Appearance {
            background_color: self.theme().palette().background,
            text_color: self.theme().palette().text,
        }
    }

    fn theme(&self) -> Self::Theme {
        Theme::custom(
            "Custom",
            Palette {
                background: config_color_to_iced_color(self.colors_config.background),
                text: config_color_to_iced_color(self.colors_config.text),
                primary: config_color_to_iced_color(self.colors_config.accent),
                success: config_color_to_iced_color(self.colors_config.accent),
                danger: config_color_to_iced_color(self.colors_config.warning),
            },
        )
    }
}

pub fn config_color_to_iced_color(color: annoyodoro_config::Color) -> Color {
    Color {
        r: color.0 / 255.,
        g: color.1 / 255.,
        b: color.2 / 255.,
        a: 1.,
    }
}
