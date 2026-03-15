#![cfg_attr(not(debug_assertions), allow(unused_imports))]

mod break_timer;
mod circular;
mod cli;
mod config;
mod stats;
mod view;
mod work_timer;

use std::{cell::RefCell, mem};

use break_timer::BreakTimer;
use clap::Parser;
use cli::Cli;
use config::Config;
use iced::{
    Event, Subscription, Task,
    event::Status,
    exit,
    keyboard::{self, Key},
    widget::operation::focus,
    window::{self, Id}
};
use lucide_icons::LUCIDE_FONT_BYTES;
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
    let boot = move || (once_boot.borrow_mut().take().unwrap(), focus("work-goal"));

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
    state: AppState,
    error: Option<String>
}

#[derive(Debug)]
enum AppState {
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
    InitialWorkGoalChange(String),
    InitialWorkGoalSubmit,
    FocusTextInput,

    TogglePause,
    ToggleLastWorkSession,
    Tick,

    #[cfg(debug_assertions)]
    DebugEarlyBreak
}

impl Annoyodoro {
    fn new(config: Config, stats: StatsManager) -> Self {
        Annoyodoro {
            config,
            stats,
            state: AppState::InitialWorkGoalPrompt {
                goal: String::new()
            },
            error: None
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
            dbg!(&err);
            self.error = Some(err.to_string());
            Task::none()
        })
    }

    fn try_update(&mut self, message: Message) -> Result<Task<Message>> {
        match (message, &mut self.state) {
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
            (
                Message::ToggleLastWorkSession,
                AppState::Running {
                    last_work_session, ..
                }
            ) => *last_work_session = !*last_work_session,
            (Message::FocusTextInput, AppState::InitialWorkGoalPrompt { .. }) => {
                return Ok(focus("worK-goal"))
            }
            (Message::InitialWorkGoalChange(_), AppState::Running { .. }) => {}
            (Message::InitialWorkGoalSubmit, AppState::Running { .. }) => {}
            (Message::FocusTextInput, AppState::Running { .. }) => {}
            (Message::TogglePause, AppState::InitialWorkGoalPrompt { .. }) => {}
            (Message::ToggleLastWorkSession, AppState::InitialWorkGoalPrompt { .. }) => {}
            (Message::Tick, AppState::InitialWorkGoalPrompt { .. }) => {}
        }

        Ok(Task::none())
    }

    fn key_subscription(event: Event, _: Status, _: Id) -> Option<Message> {
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            && modifiers.is_empty()
        {
            return match key {
                Key::Character(char) if char == "p" => Some(Message::TogglePause),
                Key::Character(char) if char == "l" => Some(Message::ToggleLastWorkSession),
                _ => None
            }
        }

        None
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::Tick),
            iced::event::listen_with(Self::key_subscription)
        ])
    }
}
