//! Stats are not yet displayed but they're here for forwards compatibility
use std::{collections::BTreeMap, fs, fs::File, io::BufWriter, path::PathBuf};

use bincode::{Decode, Encode, decode_from_slice, encode_into_std_write};
#[cfg(not(debug_assertions))]
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::{Context, Result};
use jiff::{
    Zoned,
    civil::{Date, DateTime}
};
use serde::{Deserialize, Serialize};

pub struct StatsManager {
    current_date: Date,
    stats: Stats
}

#[derive(Encode, Decode, Default)]
struct Stats {
    #[bincode(with_serde)]
    work_goals: Vec<(DateTime, String)>,
    #[bincode(with_serde)]
    day: CountMap<Date>,
    week: CountMap<Week>,
    month: CountMap<Month>,
    year: CountMap<Year>,
    all_time: Count
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Week {
    pub year: Year,
    pub iso_week: u8
}

impl From<Date> for Week {
    fn from(value: Date) -> Self {
        Self {
            year: value.into(),
            iso_week: value.iso_week_date().week() as u8
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Month {
    pub year: Year,
    pub month: u8
}

impl From<Date> for Month {
    fn from(value: Date) -> Self {
        Self {
            year: value.into(),
            month: value.month() as u8
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Year(u16);
impl From<Date> for Year {
    fn from(value: Date) -> Self {
        Self(value.year() as u16)
    }
}

#[derive(Encode, Decode, Serialize, Deserialize)]
struct CountMap<K: Ord>(BTreeMap<K, Count>);
impl<K: Ord> Default for CountMap<K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Clone, Copy, Default)]
struct Count {
    sessions: u32,
    pomodori: u32
}

impl<K: From<Date> + Ord> CountMap<K> {
    fn increment_pomodori(&mut self, date: Date) {
        self.0.entry(K::from(date)).or_default().pomodori += 1;
    }

    fn increment_app_sessions(&mut self, date: Date) {
        self.0.entry(K::from(date)).or_default().sessions += 1;
    }

    fn get(&self, date: Date) -> Count {
        self.0.get(&K::from(date)).copied().unwrap_or_default()
    }
}

impl StatsManager {
    #[cfg(not(debug_assertions))]
    fn path() -> Result<PathBuf> {
        let mut path = dirs::data_dir().ok_or_eyre("Cannot determine data dir")?;
        path.push("annoyodoro");
        fs::create_dir_all(&path)?;
        path.push("stats.bin");
        Ok(data_dir)
    }

    #[cfg(debug_assertions)]
    fn path() -> Result<PathBuf> {
        let mut path = PathBuf::from_iter(["testing-files"]);
        fs::create_dir_all(&path)?;
        path.push("stats.bin");
        Ok(path)
    }

    pub fn load() -> Result<Self> {
        let current_date = Zoned::now().date();
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self {
                current_date,
                stats: Stats::default()
            })
        }

        let bytes =
            fs::read(&path).wrap_err_with(|| format!("Cannot open {}", path.to_string_lossy()))?;

        Ok(Self {
            current_date,
            stats: decode_from_slice(&bytes, bincode::config::standard())
                .wrap_err_with(|| format!("Cannot decode {}", path.to_string_lossy()))?
                .0
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let file = File::create(&path)
            .wrap_err_with(|| format!("Cannot open {}", path.to_string_lossy()))?;
        let mut writer = BufWriter::new(file);

        encode_into_std_write(&self.stats, &mut writer, bincode::config::standard())?;
        Ok(())
    }

    pub fn increment_pomodori_count(&mut self) {
        let stats = &mut self.stats;
        let date = self.current_date;
        stats.day.increment_pomodori(date);
        stats.week.increment_pomodori(date);
        stats.month.increment_pomodori(date);
        stats.year.increment_pomodori(date);
        stats.all_time.pomodori += 1;
    }

    pub fn increment_app_sessions_count(&mut self) {
        let stats = &mut self.stats;
        let date = self.current_date;
        stats.day.increment_app_sessions(date);
        stats.week.increment_app_sessions(date);
        stats.month.increment_app_sessions(date);
        stats.year.increment_app_sessions(date);
        stats.all_time.sessions += 1;
    }

    pub fn add_work_goal(&mut self, goal: String) {
        self.stats.work_goals.push((Zoned::now().datetime(), goal))
    }

    pub fn reload_if_needed(&mut self) -> Result<()> {
        // If the date is changed at the runtime of application it needs to be updated
        let current_date = Zoned::now().date();
        if current_date != self.current_date {
            *self = Self::load()?;
        }

        Ok(())
    }

    pub fn pomodori_daily(&self) -> u32 {
        self.stats.day.get(self.current_date).pomodori
    }
}
