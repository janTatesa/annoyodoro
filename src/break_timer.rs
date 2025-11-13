use std::{
    mem,
    sync::mpsc::{SyncSender, sync_channel},
    time::Instant
};

use color_eyre::Result;
use iced::{
    Element, Length, Padding, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, focus_next, sensor, text, text_input},
    window::Id
};
use iced_sessionlock::{actions::UnLockAction, application};
use jiff::SignedDuration;

#[cfg(debug_assertions)]
use crate::button_style;
use crate::{Annoyodoro, HumanReadableDuration, config::Config, text_input_style};

#[derive(Clone)]
pub struct BreakTimer {
    needs_startup_focus: bool,
    work_goal_tx: SyncSender<String>,
    last_tick: Instant,
    long_break: bool,
    break_duration_left: SignedDuration,
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
            work_goal: String::new(),
            needs_startup_focus: true
        };

        application(
            move || (timer.clone(), Task::none()),
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
    StartupFocus,
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
                self.break_duration_left -= now.duration_since(self.last_tick).try_into().unwrap();
                self.last_tick = now;
                Task::none()
            }
            Message::WorkGoalChange(goal) => {
                self.work_goal = goal;
                Task::none()
            }
            Message::Exit => panic!("Iced sessionlock should exit on this action"),
            Message::StartupFocus => {
                self.needs_startup_focus = false;
                focus_next()
            }
        }
    }

    fn view(&self, _: Id) -> Element<'_, Message> {
        let (title_text, timer_color) = if self.break_duration_left <= SignedDuration::ZERO {
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

        let text_input = text_input("Enter the goal of the next work session", &self.work_goal)
            .size(Annoyodoro::TEXT_SIZE / 2)
            .on_input(Message::WorkGoalChange)
            .on_submit_maybe(on_submit)
            .style(text_input_style);

        let text_input_padding = Padding::default()
            .left(Annoyodoro::TIMER_SIZE * 2)
            .right(Annoyodoro::TIMER_SIZE * 2);

        let column = column![
            text(title_text).size(Annoyodoro::TEXT_SIZE),
            text(time_left)
                .color(timer_color)
                .size(Annoyodoro::TIMER_SIZE),
            container(text_input).padding(text_input_padding),
        ]
        .align_x(Horizontal::Center)
        .spacing(20);

        #[cfg(debug_assertions)]
        let column = column.push(
            button(
                text("Skip break button (enabled only when debugging)").size(Annoyodoro::TEXT_SIZE)
            )
            .style(button_style)
            .on_press(Message::ContinueWorking)
        );

        let container = container(column)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        if self.needs_startup_focus {
            return sensor(container).on_show(|_| Message::StartupFocus).into();
        }

        container.into()
    }
}
