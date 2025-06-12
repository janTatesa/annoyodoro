use std::time::Duration;

use clap::{crate_name, crate_version};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::App;
use itertools::{Itertools, chain};

const KEYHINTS: [(&str, &str); 3] = [("Quit", "q"), ("Early break", "e"), ("Toggle pause", "p")];
const APP_TITLE: &str = concat!(crate_name!(), " v", crate_version!());
const APP_WIDTH: u16 = 60;
const VARIABLES_HEIGHT: u16 = 5;
const OFF_DURATION_HINT_HEIGHT: u16 = 1;
const OFF_DURATION_HINT: &str = "Off duration is extra/early break duration + overtime";
const EXTRA_AREA_FOR_BLOCK_BORDERS: u16 = 2;
impl App {
    pub fn render(&self, frame: &mut Frame) {
        let [app_area] = Layout::horizontal([Constraint::Length(APP_WIDTH)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [state_area, keyhints_area] = Layout::vertical([
            Constraint::Length(
                VARIABLES_HEIGHT
                    + OFF_DURATION_HINT_HEIGHT
                    + (self.is_paused || self.state.in_overtime) as u16
                    + EXTRA_AREA_FOR_BLOCK_BORDERS,
            ),
            Constraint::Length(KEYHINTS.len() as u16 + EXTRA_AREA_FOR_BLOCK_BORDERS),
        ])
        .flex(Flex::Center)
        .areas(app_area);

        render_ui_component(frame, state_area, APP_TITLE, |f, a| self.render_state(f, a));
        render_ui_component(frame, keyhints_area, "Keyhints", render_keyhints);
    }

    #[allow(unstable_name_collisions)]
    fn render_state(&self, frame: &mut Frame, area: Rect) {
        let title_spans = chain!(
            self.is_paused.then_some("Paused".green()),
            self.state.in_overtime.then_some("Overtime".red())
        )
        .intersperse(" ".to_span());

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

fn render_keyhints(frame: &mut Frame, area: Rect) {
    KEYHINTS
        .into_iter()
        .map(|hint| TextOnTwoSides(hint.0, hint.1))
        .enumerate()
        .for_each(|(position, hint)| {
            frame.render_widget(hint, list_item_area(area, position as u16))
        })
}

fn render_ui_component<F>(frame: &mut Frame, area: Rect, title: &str, render_function: F)
where
    F: FnOnce(&mut Frame, Rect),
{
    frame.render_widget(block().title(title), area);
    render_function(frame, block().inner(area))
}

fn list_item_area(original: Rect, position: u16) -> Rect {
    Rect {
        y: original.y + position,
        ..original
    }
}

fn block() -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::Blue))
}

struct TextOnTwoSides<T, U>(T, U);

impl<'a, T: Into<Span<'a>>, U: Into<Span<'a>>> Widget for TextOnTwoSides<T, U> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.0.into().render(area, buf);
        let span = self.1.into();
        let [area] = Layout::horizontal([Constraint::Length(span.content.len() as u16)])
            .flex(Flex::End)
            .areas(area);
        span.green().render(area, buf);
    }
}
