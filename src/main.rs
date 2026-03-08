#![cfg_attr(not(debug_assertions), allow(unused_imports))]

mod break_timer;
mod cli;
mod config;
mod stats;
mod work_timer;

use std::{cell::RefCell, mem, time::Duration};

use break_timer::BreakTimer;
use clap::Parser;
use cli::Cli;
use config::Config;
use iced::{
    Alignment::Center,
    Element, Event, Font,
    Length::Fill,
    Subscription, Task,
    event::{self, Status},
    exit,
    keyboard::{self, Key, key::Named},
    never,
    widget::{self, Container, Sensor, button, column, operation::focus, rich_text, row, span},
    window::Id
};
use jiff::SignedDuration;
use lucide_icons::{Icon, LUCIDE_FONT_BYTES};
use notify_rust::Notification;
use stats::StatsManager;
use work_timer::WorkTimer;
use yanet::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.print_default_config {
        Config::print_default();
        return Ok(());
    }

    if cli.write_default_config {
        return Config::write_default();
    };

    let config = Config::new()?;
    let default_font = config.font;
    let theme = config.theme();
    let stats = StatsManager::load()?;
    let once_boot = RefCell::new(Some(Annoyodoro::new(config, stats)));
    let boot = move || (once_boot.borrow_mut().take().unwrap(), Task::none());

    iced::application(boot, Annoyodoro::update, Annoyodoro::view)
        .subscription(Annoyodoro::subscription)
        .default_font(default_font)
        .font(LUCIDE_FONT_BYTES)
        .theme(move |_: &Annoyodoro| theme.clone())
        .run()?;

    Ok(())
}

struct Annoyodoro {
    config: Config,
    stats: StatsManager,
    state: AppState
}

#[derive(Debug)]
enum AppState {
    Startup,
    InitialWorkGoalPrompt {
        goal: String
    },
    Running {
        long_break_in: u16,
        work_timer: WorkTimer,
        last_work_session: bool,
        shown_notification: bool
    }
}

#[derive(Debug, Clone)]
enum Message {
    StartupFocus,
    InitialWorkGoalChange(String),
    InitialWorkGoalSubmit,

    TogglePause,
    ToggleLastWorkSession,
    Tick,

    #[cfg(debug_assertions)]
    DebugEarlyBreak
}

const SPACING: u16 = 5;
const TIMER_SIZE: u32 = 120;

impl Annoyodoro {
    fn new(config: Config, stats: StatsManager) -> Self {
        Annoyodoro {
            config,
            stats,
            state: AppState::Startup
        }
    }

    fn break_time(&mut self, long_break: bool) -> Result<()> {
        let goal = BreakTimer::spawn(long_break, self.config)?;
        if let AppState::Running { work_timer, .. } = &mut self.state {
            *work_timer = WorkTimer::new(self.config.pomodoro.work_duration);
            self.stats.increment_pomodori_count();
            self.stats.add_work_goal(goal);
            self.stats.save()?;
            self.stats.reload_if_needed()?;
        }

        Ok(())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.try_update(message).unwrap_or_else(|err| {
            dbg!(err);
            Task::none()
        })
    }

    fn try_update(&mut self, message: Message) -> Result<Task<Message>> {
        match (message, &mut self.state) {
            (Message::StartupFocus, _) => {
                self.state = AppState::InitialWorkGoalPrompt {
                    goal: String::new()
                };
                return Ok(focus("work-goal"))
            }
            (
                Message::Tick,
                AppState::Running {
                    long_break_in,
                    work_timer,
                    last_work_session,
                    shown_notification
                }
            ) => {
                work_timer.on_tick();
                let duration_remaning = work_timer.duration_remaning();
                if duration_remaning <= self.config.pomodoro.notification_duration
                    && !*shown_notification
                {
                    *shown_notification = true;
                    let body = format!(
                        "Next break in {}:{:02}",
                        duration_remaning.as_secs() / 60,
                        duration_remaning.as_secs() % 60
                    );
                    Notification::new()
                        .summary("Annoyodoro")
                        .body(body.as_str())
                        .show()?;
                }

                if !work_timer.duration_remaning().is_zero() {
                    return Ok(Task::none());
                }

                if *last_work_session {
                    Notification::new()
                        .summary("Annoyodoro")
                        .body("Last work session is over! Exiting annoyodoro")
                        .show()?;
                    return Ok(exit());
                }

                *long_break_in -= 1;
                let long_break = if *long_break_in == 0 {
                    *long_break_in = self.config.pomodoro.long_break_each.into();
                    true
                } else {
                    false
                };

                self.break_time(long_break)?;
            }
            (Message::TogglePause, AppState::Running { work_timer, .. }) => {
                work_timer.toggle_pause()
            }
            #[cfg(debug_assertions)]
            (Message::DebugEarlyBreak, AppState::Running { long_break_in, .. }) => {
                *long_break_in -= 1;
                let long_break = if *long_break_in == 0 {
                    *long_break_in = self.config.pomodoro.long_break_each.into();
                    true
                } else {
                    false
                };
                self.break_time(long_break)?;
            }
            #[cfg(debug_assertions)]
            (Message::DebugEarlyBreak, _) => panic!(),
            (
                Message::InitialWorkGoalChange(goal),
                AppState::InitialWorkGoalPrompt { goal: work_goal }
            ) => *work_goal = goal,
            (
                Message::InitialWorkGoalSubmit,
                AppState::InitialWorkGoalPrompt { goal: work_goal }
            ) => {
                self.stats.add_work_goal(mem::take(work_goal));
                self.stats.increment_app_sessions_count();
                self.state = AppState::Running {
                    long_break_in: self.config.pomodoro.long_break_each.into(),
                    work_timer: WorkTimer::new(self.config.pomodoro.work_duration),
                    last_work_session: false,
                    shown_notification: false
                };
                self.stats.save()?;
                self.stats.reload_if_needed()?;
            }
            (Message::InitialWorkGoalChange(_), _) => panic!(),
            (
                Message::ToggleLastWorkSession,
                AppState::Running {
                    last_work_session, ..
                }
            ) => *last_work_session = !*last_work_session,
            (Message::InitialWorkGoalSubmit, AppState::Startup)
            | (Message::InitialWorkGoalSubmit, AppState::Running { .. })
            | (Message::TogglePause, AppState::Startup)
            | (Message::TogglePause, AppState::InitialWorkGoalPrompt { .. })
            | (Message::ToggleLastWorkSession, AppState::Startup)
            | (Message::ToggleLastWorkSession, AppState::InitialWorkGoalPrompt { .. })
            | (Message::Tick, AppState::Startup)
            | (Message::Tick, AppState::InitialWorkGoalPrompt { .. }) => panic!()
        }

        Ok(Task::none())
    }

    fn key_subscription(event: Event, _: Status, _: Id) -> Option<Message> {
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            && modifiers.is_empty()
        {
            return match key {
                Key::Named(Named::Escape) => todo!(),
                Key::Character(char) if char == "p" => Some(Message::TogglePause),
                Key::Character(char) if char == "l" => Some(Message::ToggleLastWorkSession),
                _ => None
            }
        }

        None
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            AppState::InitialWorkGoalPrompt { .. } | AppState::Startup => Subscription::none(),
            AppState::Running { work_timer, .. }
                if !work_timer.is_paused() && !work_timer.duration_remaning().is_zero() =>
            {
                Subscription::batch([
                    event::listen_with(Self::key_subscription),
                    iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
                ])
            }
            AppState::Running { .. } => event::listen_with(Self::key_subscription)
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self.state {
            AppState::Running { work_timer, .. } if work_timer.duration_remaning().is_zero() => {
                "If you're seeing this, the break timer didn't spawn and it's a bug".into()
            }
            AppState::Running {
                long_break_in,
                work_timer,
                last_work_session,
                ..
            } => self.main_view(long_break_in, work_timer, last_work_session),
            AppState::InitialWorkGoalPrompt { ref goal } => Self::initial_work_goal_prompt(goal),
            AppState::Startup => Sensor::new(Self::initial_work_goal_prompt(""))
                .on_show(|_| Message::StartupFocus)
                .into()
        }
    }

    fn main_view(
        &self,
        long_break_in: u16,
        work_timer: WorkTimer,
        last_work_session: bool
    ) -> Element<'_, Message> {
        let palette = self.config.theme().palette();
        let time_left = work_timer
            .duration_remaning()
            .try_into()
            .unwrap_or(SignedDuration::MAX);
        let time_left = rich_text![
            span(time_left.as_mins().to_string()).color(palette.primary),
            span(":").color(
                self.config
                    .theme()
                    .extended_palette()
                    .background
                    .strong
                    .color
            ),
            span(format!("{:02}", time_left.as_secs().abs() % 60)).color(palette.primary)
        ]
        .on_link_click(never)
        .size(TIMER_SIZE);
        let toggle_pause_button = if work_timer.is_paused() {
            button(widget::text(Icon::Play.unicode()).font(Font::with_name("lucide")))
        } else {
            button(widget::text(Icon::Pause.unicode()).font(Font::with_name("lucide")))
        }
        .on_press(Message::TogglePause);

        let timer_row = row![time_left, toggle_pause_button]
            .align_y(Center)
            .spacing(SPACING as u32);
        let column = column![
            timer_row,
            widget::checkbox(last_work_session)
                .on_toggle(|_| Message::ToggleLastWorkSession)
                .label("Last work session"),
            rich_text![
                "Next long break in ",
                span(long_break_in.to_string()).color(palette.primary),
                " pomodori"
            ]
            .on_link_click(never),
            rich_text![
                "Pomodori today: ",
                span(self.stats.pomodori_daily().to_string()).color(palette.primary)
            ]
            .on_link_click(never),
            widget::text!(
                "Current work goal: {}",
                self.stats.work_goals().last().unwrap().1
            )
        ]
        .align_x(Center);
        #[cfg(debug_assertions)]
        let column = column.push(
            button("Early break (enabled only in debug mode)").on_press(Message::DebugEarlyBreak)
        );
        Container::new(column)
            .align_x(Center)
            .align_y(Center)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn initial_work_goal_prompt<'a>(work_goal: &str) -> Element<'a, Message> {
        let text_input = widget::text_input("Enter the goal of the first work session", work_goal)
            .id("work-goal")
            .on_input(Message::InitialWorkGoalChange)
            .on_submit(Message::InitialWorkGoalSubmit);
        Container::new(text_input)
            .padding(SPACING)
            .align_x(Center)
            .align_y(Center)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
