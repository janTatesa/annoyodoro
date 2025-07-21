use std::{fs, path::PathBuf, time::Duration};

use clap::crate_name;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use tap::Tap;

use crate::app::now;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct State {
    pub work_duration: Duration,
    // Extra break time, overtime, early break time
    pub off_duration: Duration,
    pub pomodori_today: u16,
    pub duration_today: Duration,
    pub in_overtime: bool,
}

impl State {
    pub fn get_path() -> PathBuf {
        dirs::state_dir()
            .or(dirs::data_dir())
            .unwrap()
            .tap_mut(|p| p.push(crate_name!()))
    }

    pub fn read() -> Result<Self> {
        Ok(serde_json::from_str(&fs::read_to_string(
            Self::get_path().tap_mut(|p| p.push(now().unwrap().to_string() + ".json")),
        )?)?)
    }

    pub fn write(&self, file_name: String) -> Result<()> {
        fs::write(
            Self::get_path().tap_mut(|path| path.push(file_name)),
            serde_json::to_string(&self)?,
        )?;
        Ok(())
    }
}
