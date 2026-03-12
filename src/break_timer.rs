use std::{
    mem,
    sync::mpsc::{SyncSender, sync_channel},
    time::Instant
};

use iced::{
    Element, Length, Task, Theme,
    alignment::{Horizontal, Vertical},
    exit, never,
    widget::{self, button, column, container, operation::focus, rich_text, span, text_input}
};
use iced_layershell::{application, to_layer_message};
use jiff::SignedDuration;
use mpris::{PlaybackStatus, PlayerFinder};
use yanet::Result;

use crate::{TIMER_SIZE, config::Config};

#[derive(Clone)]
pub struct BreakTimer {
    work_goal_tx: SyncSender<String>,
    last_tick: Instant,
    long_break: bool,
    break_duration_left: SignedDuration,
    work_goal: String,
    theme: Theme
}

impl BreakTimer {
    pub fn spawn(long_break: bool, config: Config) -> Result<String> {
        let player = PlayerFinder::new()?.find_active()?;
        let was_playing_before_break = player.get_playback_status()? == PlaybackStatus::Playing;
        if was_playing_before_break {
            player.pause()?
        }

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
            move || (timer.clone(), Task::none()),
            "annoyodoro",
            BreakTimer::update,
            BreakTimer::view
        )
        .default_font(config.font)
        .theme(|app: &BreakTimer| app.theme.clone())
        .subscription(|_| iced::time::every(iced::time::Duration::from_secs(1)).map(Message::Tick))
        .run()?;

        let work_goal = work_goal_rx
            .try_recv()
            .expect("Work goal should have been sent");

        if was_playing_before_break {
            player.play()?
        }

        Ok(work_goal)
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    ContinueWorking,
    WorkGoalChange(String),
    Tick(Instant)
}

impl BreakTimer {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ContinueWorking if !self.work_goal.is_empty() => {
                self.work_goal_tx
                    .send(mem::take(&mut self.work_goal))
                    .unwrap();
                return exit()
            }
            Message::ContinueWorking => {}
            Message::Tick(now) => {
                self.break_duration_left -= now.duration_since(self.last_tick).try_into().unwrap();
                self.last_tick = now;
            }
            Message::WorkGoalChange(goal) => self.work_goal = goal,
            Message::AnchorChange(_)
            | Message::SetInputRegion(_)
            | Message::AnchorSizeChange(..)
            | Message::LayerChange(_)
            | Message::MarginChange(_)
            | Message::SizeChange(_)
            | Message::ExclusiveZoneChange(_)
            | Message::KeyboardInteractivityChange(_)
            | Message::VirtualKeyboardPressed { .. } => {}
        }

        focus("work-goal")
    }

    fn view(&self) -> Element<'_, Message> {
        let palette = self.theme.palette();
        let (title_text, timer_color) = if self.break_duration_left <= SignedDuration::ZERO {
            ("Time to work! (submit your work reason)", palette.danger)
        } else if self.long_break {
            ("Time for a long break", palette.primary)
        } else {
            ("Time for a break!", palette.primary)
        };

        let time_left = self.break_duration_left;
        let on_submit =
            (!self.break_duration_left.is_positive()).then_some(Message::ContinueWorking);

        let text_input = text_input("Work goal", &self.work_goal)
            .id("work-goal")
            .on_input(Message::WorkGoalChange)
            .on_submit_maybe(on_submit);

        let column = column![
            widget::text(title_text).size(30),
            rich_text![
                span(time_left.as_mins().to_string()).color(timer_color),
                span(":").color(self.theme.extended_palette().background.strong.color),
                span(format!("{:02}", time_left.as_secs().abs() % 60)).color(timer_color)
            ]
            .on_link_click(never)
            .size(TIMER_SIZE),
            "Enter the goal of your next work session",
            text_input,
        ]
        .align_x(Horizontal::Center)
        .max_width(TIMER_SIZE * 3.0);

        #[cfg(debug_assertions)]
        let column = column.push(
            button("Skip break button (enabled only when debugging)")
                .on_press(Message::ContinueWorking)
        );

        let container = container(column)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        container.into()
    }
}
