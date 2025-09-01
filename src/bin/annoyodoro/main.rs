mod cli;
mod pomodori_count;
mod subscription;
mod work_timer;

use std::{fs::OpenOptions, io::Write, iter};

use annoyodoro::{
    BORDER_RADIUS, HumanReadableDuration, break_timer,
    config::Config,
    data_dir,
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

use crate::{pomodori_count::PomodoriCountManager, work_timer::WorkTimer};

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
    break_time: bool,
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

#[derive(Debug, Clone)]
enum Message {
    TogglePause,
    Tick,

    Retry,
    RetrySaveStats,
    RetryReloadStats,
    RetryAddWorkGoal(String),
    RetryBreakTimer {
        long_break: bool
    },

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
            error: None,
            break_time: false
        })
    }

    fn on_err(&mut self, report: Report, error_msg: &'static str, retry_message: Message) {
        self.error = Some(ErrorState {
            report: report.wrap_err(error_msg),
            retry_message
        })
    }

    fn add_work_goal(&mut self, goal: String) {
        if let Err(report) = Self::try_add_work_goal(&goal) {
            self.on_err(
                report,
                "Failed to add work goal",
                Message::RetryAddWorkGoal(goal)
            )
        }
    }

    fn try_add_work_goal(goal: &str) -> Result<()> {
        let mut path = data_dir()?;
        path.push("work-goals.txt");
        let mut file = OpenOptions::new().append(true).create(true).open(&path)?;
        writeln!(file, "{goal}")?;
        Ok(())
    }

    fn break_time(&mut self, long_break: bool) {
        let result = break_timer::spawn_break_timer(long_break, &self.config);
        self.break_time = false;
        match result {
            Ok(work_goal) => {
                self.work_timer = WorkTimer::new(self.config.pomodoro.work_duration);
                self.pomodori_count.increment();
                self.save_stats();
                self.add_work_goal(work_goal);
            }
            Err(report) => {
                self.on_err(
                    report,
                    "Failed to spawn break timer",
                    Message::RetryBreakTimer { long_break }
                );
            }
        }
    }

    fn save_stats(&mut self) {
        match self.pomodori_count.save() {
            Ok(_) => self.reload_stats_if_needed(),
            Err(report) => self.on_err(report, "Failed to save stats", Message::RetrySaveStats)
        }
    }

    fn reload_stats_if_needed(&mut self) {
        if let Err(report) = self.pomodori_count.reload_if_needed() {
            self.on_err(report, "Failed to reload stats", Message::RetryReloadStats);
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.work_timer.on_tick();
                if self.work_timer.duration_remaning().is_zero() {
                    self.long_break_in -= 1;
                    let long_break = if self.long_break_in == 0 {
                        self.long_break_in = self.config.pomodoro.long_break_each.into();
                        true
                    } else {
                        false
                    };

                    self.break_time = true;
                    self.break_time(long_break);
                }
            }
            Message::TogglePause => self.work_timer.toggle_pause(),
            Message::RetrySaveStats => self.save_stats(),
            Message::RetryReloadStats => self.reload_stats_if_needed(),
            Message::Retry => {
                if let Some(ErrorState { retry_message, .. }) = self.error.take() {
                    return Task::done(retry_message)
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

                self.break_time = true;
                self.break_time(long_break);
            }
            Message::RetryAddWorkGoal(goal) => self.add_work_goal(goal),
            Message::RetryBreakTimer { long_break } => {
                self.error = None;
                self.break_time = true;
                self.break_time(long_break);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.break_time {
            return "If you're seeing this, the break timer ddidn't spawn and it's a bug".into();
        }

        let palette = self.config.theme().palette();

        if let Some(ErrorState {
            report,
            retry_message
        }) = &self.error
        {
            return Self::error_view(palette, report, retry_message.clone());
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
