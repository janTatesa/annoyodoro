use std::time::Duration;

use annoyodoro_break_timer::{BreakTimeResult, spawn_break_timer};
use color_eyre::Result;

use crate::{App, State};
impl App {
    pub fn break_time(&mut self, early_break: bool) -> Result<()> {
        if early_break {
            let off_time = self
                .state
                .work_duration
                .abs_diff(self.cli.work_duration.into());
            if off_time > Duration::from(self.cli.forgive_duration) {
                self.state.off_duration += off_time;
            }
        }

        let available_overtime =
            (!self.state.in_overtime && !early_break).then_some(self.cli.overtime_duration.into());
        let result = spawn_break_timer(
            self.cli.output.clone(),
            available_overtime,
            self.break_duration(),
            self.should_do_long_break(),
            &mut self.current_font,
        )?;

        match result {
            BreakTimeResult::OvertimeUsed => self.state.in_overtime = true,
            BreakTimeResult::BreakComplete(duration) => self.after_break_complete(duration),
        }

        self.ticker.reset();
        Ok(())
    }

    fn should_do_long_break(&self) -> bool {
        ((self.state.pomodori_today + 1) % self.cli.long_break_in) == 0
    }

    fn break_duration(&self) -> Duration {
        match self.should_do_long_break() {
            true => self.cli.long_break_duration,
            false => self.cli.break_duration,
        }
        .into()
    }

    fn after_break_complete(&mut self, duration: Duration) {
        self.state.duration_today += duration;
        let extra_duration = duration.saturating_sub(self.break_duration());
        let off_time = self.state.off_duration
            + (extra_duration > Duration::from(self.cli.forgive_duration))
                .then_some(extra_duration)
                .unwrap_or_default();
        self.state = State {
            pomodori_today: self.state.pomodori_today + 1,
            duration_today: self.state.duration_today + self.state.work_duration,
            in_overtime: false,
            off_duration: off_time,
            ..Default::default()
        }
    }
}
