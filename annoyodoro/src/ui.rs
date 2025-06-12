mod keyhints;
mod state;

use clap::{crate_name, crate_version};
use keyhints::{KEYHINTS_LEN, render_keyhints};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::App;

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
            Constraint::Length(KEYHINTS_LEN + EXTRA_AREA_FOR_BLOCK_BORDERS),
        ])
        .flex(Flex::Center)
        .areas(app_area);

        render_ui_component(frame, state_area, APP_TITLE, |f, a| self.render_state(f, a));
        render_ui_component(frame, keyhints_area, "Keyhints", render_keyhints);
    }
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
