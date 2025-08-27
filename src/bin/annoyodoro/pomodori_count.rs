//! Stats are not yet displayed but they're here for forwards compatibility

use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter, ErrorKind}
};

use annoyodoro::data_dir;
use bincode::{Decode, Encode, decode_from_reader, encode_into_std_write};
#[cfg(debug_assertions)]
use clap::crate_name;
#[cfg(debug_assertions)]
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::{Context, Report, Result};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

pub struct PomodoriCountManager {
    current_date: Date,
    pomodori_count: PomodoriCount
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
pub struct Week {
    pub year: Year,
    pub iso_week: u8
}

impl From<Date> for Week {
    fn from(value: Date) -> Self {
        Self {
            year: value.into(),
            iso_week: value.iso_week()
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
struct Month {
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

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
pub struct Year(u16);
impl From<Date> for Year {
    fn from(value: Date) -> Self {
        Self(value.year() as u16)
    }
}

#[derive(Encode, Decode, Default)]
struct PomodoriCount {
    #[bincode(with_serde)]
    day: PomodoriCountMap<Date>,
    week: PomodoriCountMap<Week>,
    month: PomodoriCountMap<Month>,
    year: PomodoriCountMap<Year>,
    all_time: u32
}

#[derive(Encode, Decode, Serialize, Deserialize)]
struct PomodoriCountMap<K: Ord>(BTreeMap<K, u32>);
impl<K: Ord> Default for PomodoriCountMap<K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: From<Date> + Ord> PomodoriCountMap<K> {
    fn increment(&mut self, date: Date) {
        *self.0.entry(K::from(date)).or_insert(0) += 1;
    }

    fn get(&self, date: Date) -> u32 {
        self.0.get(&K::from(date)).copied().unwrap_or(0)
    }
}

impl PomodoriCountManager {
    fn today() -> Result<Date> {
        Ok(OffsetDateTime::now_local()
            .wrap_err("Cannot determine current date")?
            .date())
    }

    pub fn load() -> Result<Self> {
        let current_date = Self::today()?;
        let mut path = data_dir()?;
        path.push("stats.bin");
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(Self {
                    current_date,
                    pomodori_count: PomodoriCount::default()
                })
            }
            Err(error) => {
                return Err(
                    Report::new(error).wrap_err(format!("Cannot open {}", path.to_string_lossy()))
                )
            }
        };
        let reader = BufReader::new(file);
        Ok(Self {
            current_date,
            pomodori_count: decode_from_reader(reader, bincode::config::standard())?
        })
    }

    pub fn save(&self) -> Result<()> {
        let mut path = data_dir()?;
        path.push("stats.bin");
        let mut writer = BufWriter::new(
            File::create(&path)
                .wrap_err_with(|| format!("Cannot open {}", path.to_string_lossy()))?
        );

        encode_into_std_write(
            &self.pomodori_count,
            &mut writer,
            bincode::config::standard()
        )?;
        Ok(())
    }

    pub fn increment(&mut self) {
        let count = &mut self.pomodori_count;
        let date = self.current_date;
        count.day.increment(date);
        count.week.increment(date);
        count.month.increment(date);
        count.year.increment(date);
        count.all_time += 1;
    }

    pub fn reload_if_needed(&mut self) -> Result<()> {
        // If the date is changed at the runtime of application it needs to be updated
        let current_date = Self::today()?;
        if current_date != self.current_date {
            *self = Self::load()?;
        }

        Ok(())
    }

    pub fn daily(&self) -> u32 {
        self.pomodori_count.day.get(self.current_date)
    }
}
