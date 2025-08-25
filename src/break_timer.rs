use std::{iter, time::Instant};

use color_eyre::Result;
use iced::{
    Border, Element, Event, Length, Pixels, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    event::{self, Status},
    executor, exit, keyboard,
    widget::{Button, Container, Text, button, column},
    window::Id
};
use iced_layershell::{
    Application,
    reexport::{Anchor, Layer},
    settings::{LayerShellSettings, Settings},
    to_layer_message
};
use itertools::chain;
use time::Duration;

use crate::{BORDER_RADIUS, HumanReadableDuration, config::Config};

struct BreakTimer {
    last_tick: Instant,
    long_break: bool,
    break_duration_left: Duration,
    theme: Theme
}

pub fn spawn_break_timer(long_break: bool, config: &Config) -> Result<()> {
    let layer_settings = LayerShellSettings {
        layer: Layer::Overlay,
        size: Some((0, 0)),
        anchor: Anchor::all(),
        ..Default::default()
    };

    let duration = if long_break {
        config.pomodoro.long_break_duration
    } else {
        config.pomodoro.break_duration
    };

    let timer = BreakTimer {
        last_tick: Instant::now(),
        long_break,
        break_duration_left: duration.try_into()?,
        theme: config.theme()
    };

    BreakTimer::run(Settings {
        layer_settings,
        flags: timer,
        default_font: config.font,
        default_text_size: Pixels(16.),
        id: None,
        fonts: Vec::new(),
        antialiasing: false,
        virtual_keyboard_support: None
    })?;

    Ok(())
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    ContinueWorking,
    Tick(Instant)
}

impl BreakTimer {
    fn any_key_subscription(event: Event, _status: Status, _id: Id) -> Option<Message> {
        if let Event::Keyboard(keyboard::Event::KeyPressed { .. }) = event {
            Some(Message::ContinueWorking)
        } else {
            None
        }
    }
}

impl Application for BreakTimer {
    type Message = Message;
    type Flags = Self;
    type Theme = Theme;
    type Executor = executor::Default;

    fn new(timer: Self) -> (Self, Task<Message>) {
        (timer, Task::none())
    }

    fn namespace(&self) -> String {
        String::from("annoyodoro-break-timer")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(chain![
            iter::once(iced::time::every(iced::time::Duration::from_secs(1)).map(Message::Tick)),
            (!self.break_duration_left.is_positive())
                .then_some(event::listen_with(Self::any_key_subscription))
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ContinueWorking => exit(),
            Message::Tick(now) => {
                self.break_duration_left -= now.duration_since(self.last_tick);
                self.last_tick = now;
                Task::none()
            }
            _ => Task::none()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let (title_text, timer_color) = if self.break_duration_left <= Duration::ZERO {
            ("Time to work!", self.theme.palette().danger)
        } else if self.long_break {
            ("Time for a long break", self.theme.palette().primary)
        } else {
            ("Time for a break!", self.theme.palette().primary)
        };

        let time_left = HumanReadableDuration(self.break_duration_left);
        let rounded_corners = |theme: &Theme, status| {
            let base = button::primary(theme, status);
            button::Style {
                border: Border {
                    radius: BORDER_RADIUS,
                    ..base.border
                },
                ..base
            }
        };

        let column = column![
            Text::new(title_text).size(80),
            Text::new(time_left).color(timer_color).size(120),
            Button::new(Text::new("Continue working (any key)").size(80))
                .on_press_maybe(
                    (self.break_duration_left <= Duration::ZERO)
                        .then_some(Message::ContinueWorking)
                )
                .style(rounded_corners)
        ]
        .align_x(Horizontal::Center)
        .spacing(20);

        #[cfg(debug_assertions)]
        let column = column.push(
            Button::new(Text::new("Skip break button (enabled only when debugging)").size(80))
                .style(rounded_corners)
                .on_press(Message::ContinueWorking)
        );

        Container::new(column)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }
}
