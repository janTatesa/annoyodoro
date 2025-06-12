use std::fmt::{self, Display, Formatter};

use color_eyre::eyre::{Result, eyre};

#[derive(Debug, Clone, Copy)]
pub enum Duration {
    Seconds(u16),
    Minutes(u16),
}

impl From<Duration> for std::time::Duration {
    fn from(value: Duration) -> Self {
        match value {
            Duration::Seconds(seconds) => Self::from_secs(seconds as u64),
            Duration::Minutes(minutes) => Self::from_secs(minutes as u64 * 60),
        }
    }
}

pub const BREAK: Duration = Duration::Minutes(5);
pub const LONG_BREAK: Duration = Duration::Minutes(10);
pub const OVERTIME: Duration = Duration::Minutes(2);
pub const WORK: Duration = Duration::Minutes(20);
pub const FORGIVE: Duration = Duration::Seconds(8);
pub const LONG_BREAK_IN: u16 = 4;

pub fn parse_duration(arg: &str) -> Result<Duration> {
    let num = arg[0..arg.len() - 1].parse()?;
    match arg.chars().last() {
        Some('s') => Ok(Duration::Seconds(num)),
        Some('m') => Ok(Duration::Minutes(num)),
        _ => Err(eyre!(
            "Duration can be a number followed by s for seconds or m for minutes"
        )),
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        let (num, char) = match self {
            Duration::Seconds(num) => (num, 's'),
            Duration::Minutes(num) => (num, 'm'),
        };
        write!(f, "{num}{char}")
    }
}
