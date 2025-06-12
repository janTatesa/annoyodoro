use ratatui::{Frame, layout::Rect};

use super::{TextOnTwoSides, list_item_area};

const KEYHINTS: [(&str, &str); 3] = [("Quit", "q"), ("Early break", "e"), ("Toggle pause", "p")];
pub const KEYHINTS_LEN: u16 = KEYHINTS.len() as u16;
pub fn render_keyhints(frame: &mut Frame, area: Rect) {
    KEYHINTS
        .into_iter()
        .map(|hint| TextOnTwoSides(hint.0, hint.1))
        .enumerate()
        .for_each(|(position, hint)| {
            frame.render_widget(hint, list_item_area(area, position as u16))
        })
}
