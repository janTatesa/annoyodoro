mod cli;
mod stats;
mod subscription;
mod work_timer;

use std::{iter, time::Instant};

use annoyodoro::{
    BORDER_RADIUS, HumanReadableDuration, break_timer,
    config::Config,
    icons::{ICON_FONT, pause_button, resume_button}
};
use clap::{Parser, crate_name};
use cli::Cli;
use color_eyre::{Result, eyre::Report};
use iced::{
    Alignment::Center,
    Border, Element,
    Length::Fill,
    Padding, Task, Theme,
    theme::Palette,
    widget::{Button, Column, Container, Text, button, column, rich_text, row, text::Span}
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
    RetryBreakTimer {
        long_break: bool
    },
    RetrySaveStats,
    RetryReloadStats,

    #[cfg(debug_assertions)]
    DebugEarlyBreak
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
                self.work_timer = WorkTimer::new(self.config.pomodoro.work_duration);
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
                if let Some(ErrorState { retry_message, .. }) = self.error.take() {
                    return self.update(retry_message)
                }
            }

            #[cfg(debug_assertions)]
            Message::DebugEarlyBreak => {
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

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let palette = self.config.theme().palette();

        if let Some(ErrorState {
            ref report,
            retry_message
        }) = self.error
        {
            return Self::error_view(palette, report, retry_message);
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
            .size(60),
            rich_text![
                "Pomodori today: ",
                Span::new(self.pomodori_count.daily().to_string()).color(palette.primary)
            ]
            .size(60)
        ]
        .align_x(Center);

        #[cfg(debug_assertions)]
        let column = column.push(
            Button::new(Text::new("Early break (enabled only in debug mode)").size(60))
                .on_press(Message::DebugEarlyBreak)
                .style(Self::button_rounded_corners)
        );

        Container::new(column)
            .align_x(Center)
            .align_y(Center)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn error_view(
        palette: Palette,
        report: &Report,
        retry_message: Message
    ) -> Element<'_, Message> {
        let column = Column::from_iter(
            iter::once(
                row![
                    Text::new(report.to_string()).color(palette.danger).size(80),
                    Button::new(Text::new("Retry").size(60))
                        .on_press(retry_message)
                        .style(Self::button_rounded_corners)
                ]
                .spacing(20)
                .align_y(Center)
                .into()
            )
            .chain(report.chain().skip(1).map(|err| {
                Container::new(Text::new(err.to_string()).size(60))
                    .padding(Padding::default().left(60))
                    .into()
            }))
        );

        Container::new(column)
            .width(Fill)
            .height(Fill)
            .align_x(Center)
            .align_y(Center)
            .into()
    }

    fn button_rounded_corners(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::primary(theme, status);
        button::Style {
            border: Border {
                radius: BORDER_RADIUS,
                ..base.border
            },
            ..base
        }
    }
}
