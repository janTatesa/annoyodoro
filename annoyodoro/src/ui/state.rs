use std::time::Duration;

use itertools::{Itertools, chain};
use ratatui::{Frame, layout::Rect, style::Stylize, text::Line};

use crate::app::App;

use super::{
    OFF_DURATION_HINT, OFF_DURATION_HINT_HEIGHT, TextOnTwoSides, VARIABLES_HEIGHT, list_item_area,
};

impl App {
    #[allow(unstable_name_collisions)]
    pub(crate) fn render_state(&self, frame: &mut Frame, area: Rect) {
        let title_spans = chain!(
            self.is_paused.then_some("Paused".green()),
            self.state.in_overtime.then_some("Overtime".red())
        )
        .intersperse(" ".into());

        frame.render_widget(Line::from_iter(title_spans).centered(), area);

        let starting_index = (self.is_paused || self.state.in_overtime) as u16;

        let off_duration_percentage = format!(
            "{:.2}%",
            (self.state.off_duration.as_secs_f32() * 100.
                / self.state.duration_today.as_secs_f32())
        );

        let next_break_in = self.max_duration().saturating_sub(self.state.work_duration);

        [
            ("Next break in", format_duration(next_break_in)),
            ("Pomodori today", self.state.pomodori_today.to_string()),
            ("Duration today", format_duration(self.state.duration_today)),
            ("Off duration", format_duration(self.state.off_duration)),
            ("Off duration percentage", off_duration_percentage),
        ]
        .into_iter()
        .enumerate()
        .for_each(|(idx, item)| {
            frame.render_widget(
                TextOnTwoSides(item.0, item.1),
                list_item_area(area, starting_index + idx as u16),
            )
        });

        let off_time_hint_area = Rect {
            y: area.y + starting_index + VARIABLES_HEIGHT,
            height: OFF_DURATION_HINT_HEIGHT,
            ..area
        };

        frame.render_widget(
            Line::from(OFF_DURATION_HINT).italic().green().centered(),
            off_time_hint_area,
        );
    }
}

fn format_duration(duration: Duration) -> String {
    format!("{}:{:02}", duration.as_secs() / 60, duration.as_secs() % 60)
}
