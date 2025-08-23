pub mod break_timer;
pub mod config;
pub mod icons;

use iced::widget::text::{Fragment, IntoFragment};
use time::Duration;

pub struct HumanReadableDuration(pub Duration);
impl IntoFragment<'static> for HumanReadableDuration {
    fn into_fragment(self) -> Fragment<'static> {
        let sign = if self.0.is_negative() {
            "-"
        } else {
            Default::default()
        };

        let mins = self.0.whole_minutes().abs();
        let secs = self.0.whole_seconds().abs() % 60;
        Fragment::Owned(format!("{sign}{mins}:{secs:02}"))
    }
}
