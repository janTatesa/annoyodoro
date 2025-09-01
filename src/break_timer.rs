use std::{
    mem,
    sync::mpsc::{SyncSender, sync_channel},
    time::Instant
};

use color_eyre::Result;
#[cfg(debug_assertions)]
use iced::widget::Button;
use iced::{
    Border, Element, Length, Padding, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{Container, Text, TextInput, button, column, focus_next, text_input},
    window::Id
};
use iced_sessionlock::{actions::UnLockAction, application};
use time::Duration;

use crate::{BORDER_RADIUS, HumanReadableDuration, config::Config};

#[derive(Clone)]
pub struct BreakTimer {
    work_goal_tx: SyncSender<String>,
    last_tick: Instant,
    long_break: bool,
    break_duration_left: Duration,
    work_goal: String,
    theme: Theme
}

impl BreakTimer {
    pub fn spawn(long_break: bool, config: Config) -> Result<String> {
        let duration = if long_break {
            config.pomodoro.long_break_duration
        } else {
            config.pomodoro.break_duration
        };

        let (work_goal_tx, work_goal_rx) = sync_channel(1);
        let timer = BreakTimer {
            last_tick: Instant::now(),
            long_break,
            break_duration_left: duration.try_into()?,
            theme: config.theme(),
            work_goal_tx,
            work_goal: String::new()
        };

        application(
            move || (timer.clone(), focus_next()),
            BreakTimer::update,
            BreakTimer::view
        )
        .default_font(config.font)
        .theme(move |_, _| config.theme())
        .subscription(|_| iced::time::every(iced::time::Duration::from_secs(1)).map(Message::Tick))
        .run()?;

        let work_goal = work_goal_rx
            .try_recv()
            .expect("Work goal should have been sent");

        Ok(work_goal)
    }
}

#[derive(Debug, Clone)]
enum Message {
    Exit,
    ContinueWorking,
    WorkGoalChange(String),
    Tick(Instant)
}

impl TryInto<UnLockAction> for Message {
    type Error = Self;

    fn try_into(self) -> Result<UnLockAction, Self> {
        match self {
            Self::Exit => Ok(UnLockAction),
            _ => Err(self)
        }
    }
}

impl BreakTimer {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ContinueWorking if !self.work_goal.is_empty() => {
                self.work_goal_tx
                    .send(mem::take(&mut self.work_goal))
                    .unwrap();
                Task::done(Message::Exit)
            }
            Message::ContinueWorking => Task::none(),
            Message::Tick(now) => {
                self.break_duration_left -= now.duration_since(self.last_tick);
                self.last_tick = now;
                Task::none()
            }
            Message::WorkGoalChange(goal) => {
                self.work_goal = goal;
                Task::none()
            }
            Message::Exit => panic!("Iced sessionlock should exit on this action")
        }
    }

    fn view(&self, _: Id) -> Element<'_, Message> {
        let (title_text, timer_color) = if self.break_duration_left <= Duration::ZERO {
            (
                "Time to work! (submit your work reason)",
                self.theme.palette().danger
            )
        } else if self.long_break {
            ("Time for a long break", self.theme.palette().primary)
        } else {
            ("Time for a break!", self.theme.palette().primary)
        };

        let time_left = HumanReadableDuration(self.break_duration_left);
        let on_submit =
            (!self.break_duration_left.is_positive()).then_some(Message::ContinueWorking);

        let column = column![
            Text::new(title_text).size(80),
            Text::new(time_left).color(timer_color).size(120),
            Container::new(
                TextInput::new("Enter the goal of the next work session", &self.work_goal)
                    .size(40)
                    .on_input(Message::WorkGoalChange)
                    .on_submit_maybe(on_submit)
                    .style(|theme: &Theme, status| {
                        let mut style = text_input::default(theme, status);
                        style.border.width = 4.;
                        style.border.radius = BORDER_RADIUS;
                        if let text_input::Status::Focused { .. } = status {
                            style.border.color = theme.palette().primary
                        }

                        style
                    })
            )
            .padding(Padding::default().left(240).right(240))
        ]
        .align_x(Horizontal::Center)
        .spacing(20);

        #[cfg(debug_assertions)]
        let column = column.push(
            Button::new(Text::new("Skip break button (enabled only when debugging)").size(40))
                .style(|theme: &Theme, status| {
                    let base = button::primary(theme, status);
                    button::Style {
                        border: Border {
                            radius: BORDER_RADIUS,
                            ..base.border
                        },
                        ..base
                    }
                })
                .on_press(Message::ContinueWorking)
        );

        Container::new(column)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
