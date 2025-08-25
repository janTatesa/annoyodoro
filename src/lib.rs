pub mod break_timer;
pub mod config;
pub mod icons;

use std::{fs, path::PathBuf};

use color_eyre::Result;
#[cfg(not(debug_assertions))]
use color_eyre::eyre::OptionExt;
use iced::{
    border::Radius,
    widget::text::{Fragment, IntoFragment}
};
use time::Duration;

pub const BORDER_RADIUS: Radius = Radius {
    top_left: 20.,
    top_right: 20.,
    bottom_right: 20.,
    bottom_left: 20.
};

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

#[cfg(not(debug_assertions))]
pub fn data_dir() -> Result<PathBuf> {
    let mut data_dir = dirs::data_dir().ok_or_eyre("Cannot determine data dir")?;
    data_dir.push("annoyodoro");
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

#[cfg(debug_assertions)]
pub fn data_dir() -> Result<PathBuf> {
    let data_dir = PathBuf::from_iter(["testing-files"]);
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}
