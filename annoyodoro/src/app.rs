use std::time::Duration;

use ::time::{Date, OffsetDateTime};
use annoyodoro_break_timer::CurrentFont;
use color_eyre::Result;
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use ratatui::DefaultTerminal;
use tokio::{
    select,
    time::{self, Instant, Interval},
};

use crate::{cli::Cli, state::State};

pub struct App {
    pub running: bool,

    pub ticker: Interval,
    pub is_paused: bool,
    pub event_stream: EventStream,

    pub state: State,
    pub state_date: Date,
    pub state_backup_interval: Interval,

    pub cli: Cli,

    pub current_font: CurrentFont,
}

impl App {
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            select! {
                Some(result) = self.event_stream.next().fuse() => self.handle_event(result?, terminal)?,
                instant = self.ticker.tick(), if !self.is_paused => self.on_tick(instant, terminal)?,
                _ = self.state_backup_interval.tick() => self.backup_state(terminal)?,
                _ = time::sleep(Duration::from_millis(100)) => {
                    // Avoid busy waiting
                }
            }
        }

        self.state.write(self.state_date.to_string())
    }

    fn backup_state(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.state.write(self.state_date.to_string())?;

        let now = now()?;
        if self.state_date < now {
            self.state = State::default();
            self.state_date = now;
            terminal.draw(|frame| self.render(frame))?;
        }

        Ok(())
    }

    fn on_tick(&mut self, instant: Instant, terminal: &mut DefaultTerminal) -> Result<()> {
        self.add_delta_time(Duration::from_secs(1) + instant.elapsed());
        if self.state.work_duration > self.max_duration() {
            self.break_time(false)?
        }
        terminal.draw(|frame| self.render(frame))?;
        Ok(())
    }

    fn add_delta_time(&mut self, delta_time: Duration) {
        self.state.work_duration += delta_time;
        self.state.duration_today += delta_time;
        if self.state.in_overtime {
            self.state.off_duration += delta_time;
        }
    }

    pub fn max_duration(&self) -> Duration {
        Duration::from(self.cli.work_duration)
            + match self.state.in_overtime {
                true => self.cli.overtime_duration.into(),
                false => Duration::ZERO,
            }
    }
}

pub fn now() -> Result<Date> {
    Ok(OffsetDateTime::now_local()?.date())
}
