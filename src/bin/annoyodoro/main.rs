mod cli;
mod stats;
mod subscription;
mod work_timer;

use std::time::Instant;

use annoyodoro::{
    HumanReadableDuration, break_timer,
    config::Config,
    icons::{ICON_FONT, pause_button, resume_button}
};
use clap::{Parser, crate_name};
use cli::Cli;
use color_eyre::{Result, eyre::Report};
use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Task,
    widget::{Container, Text, column, rich_text, row, text::Span}
};
use time::Duration;

use crate::{stats::PomodoriCountManager, work_timer::WorkTimer};

fn main() -> Result<()> {
    color_eyre::install()?;
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
    let app = Annoyodoro::new(config)?;
    iced::application(crate_name!(), Annoyodoro::update, Annoyodoro::view)
        .subscription(Annoyodoro::subscription)
        .default_font(default_font)
        .font(ICON_FONT)
        .theme(move |_| theme.clone())
        .run_with(|| (app, Task::none()))?;

    Ok(())
}

struct Annoyodoro {
    long_break_in: u16,
    work_timer: WorkTimer,
    config: Config,
    error: Option<ErrorState>,
    pomodori_count: PomodoriCountManager
}

struct ErrorState {
    report: Report,
    retry_message: Message
}

#[derive(Debug, Clone, Copy)]
enum Message {
    TogglePause,
    Tick(Instant),

    Retry,
    RetryBreakTimer { long_break: bool },
    RetrySaveStats,
    RetryReloadStats
}

impl Annoyodoro {
    fn new(config: Config) -> Result<Self> {
        Ok(Annoyodoro {
            long_break_in: config.pomodoro.long_break_each.into(),
            work_timer: WorkTimer::new(config.pomodoro.work_duration),
            config,
            pomodori_count: PomodoriCountManager::load()?,
            error: None
        })
    }

    fn break_time(&mut self, long_break: bool) {
        match break_timer::spawn_break_timer(long_break, &self.config) {
            Ok(_) => {
                self.pomodori_count.increment();
                self.save_stats()
            }
            Err(report) => {
                self.error = Some(ErrorState {
                    report: report.wrap_err("Failed to spawn break timer"),
                    retry_message: Message::RetryBreakTimer { long_break }
                })
            }
        }
    }

    fn save_stats(&mut self) {
        match self.pomodori_count.save() {
            Ok(_) => self.reload_stats_if_needed(),
            Err(report) => {
                self.error = Some(ErrorState {
                    report: report.wrap_err("Failed to save stats"),
                    retry_message: Message::RetrySaveStats
                })
            }
        }
    }

    fn reload_stats_if_needed(&mut self) {
        if let Err(report) = self.pomodori_count.reload_if_needed() {
            self.error = Some(ErrorState {
                report: report.wrap_err("Failed to reload stats"),
                retry_message: Message::RetryReloadStats
            })
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                self.work_timer.on_tick(now);
                if self.work_timer.duration_remaning().is_zero() {
                    self.long_break_in -= 1;
                    let long_break = if self.long_break_in == 0 {
                        self.long_break_in = self.config.pomodoro.long_break_each.into();
                        true
                    } else {
                        false
                    };

                    self.break_time(long_break);
                }
            }
            Message::TogglePause => self.work_timer.toggle_pause(),

            Message::RetryBreakTimer { long_break } => {
                self.error = None;
                self.break_time(long_break)
            }
            Message::RetrySaveStats => {
                self.error = None;
                self.save_stats()
            }
            Message::RetryReloadStats => {
                self.error = None;
                self.reload_stats_if_needed()
            }
            Message::Retry => {
                if let Some(ErrorState { retry_message, .. }) = self.error {
                    return Task::done(retry_message)
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let palette = self.config.theme().palette();
        if let Some(ErrorState { report, .. }) = &self.error {
            todo!("Error handling not yet supported {report}")
        }

        let time_left = Text::new(HumanReadableDuration(
            self.work_timer
                .duration_remaning()
                .try_into()
                .unwrap_or(Duration::MAX)
        ))
        .size(120)
        .color(palette.primary);

        let toggle_pause_button = if self.work_timer.is_paused() {
            resume_button(80)
        } else {
            pause_button(80)
        }
        .on_press(Message::TogglePause);

        let timer_row = row![time_left, toggle_pause_button]
            .align_y(Center)
            .spacing(20);

        let column = column![
            timer_row,
            rich_text![
                "Next long break in ",
                Span::new(self.long_break_in.to_string()).color(palette.primary),
                " pomodori"
            ]
            .size(60)
        ]
        .align_x(Center);

        Container::new(column)
            .align_x(Center)
            .align_y(Center)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
