use std::fmt::{Display, Error, Formatter};

use clap::{ArgAction, Parser, arg, command};
use color_eyre::{Result, eyre::eyre};

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

const BREAK: Duration = Duration::Minutes(5);
const LONG_BREAK: Duration = Duration::Minutes(10);
const OVERTIME: Duration = Duration::Minutes(2);
const WORK: Duration = Duration::Minutes(20);
const FORGIVE: Duration = Duration::Seconds(8);
const LONG_BREAK_IN: u16 = 4;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    /// Bind the break timer to a specific output
    #[arg(short, long)]
    pub output: Option<String>,
    #[arg(short, long, value_parser = parse_duration, default_value_t = BREAK)]
    pub break_duration: Duration,
    #[arg(short, long, value_parser = parse_duration, default_value_t = WORK)]
    pub work_duration: Duration,
    #[arg(short, long, value_parser = parse_duration, default_value_t = LONG_BREAK)]
    pub long_break_duration: Duration,
    /// If off time is being added, forgive durations that are at most this duration
    #[arg(short, long, value_parser = parse_duration, default_value_t = FORGIVE)]
    pub forgive_duration: Duration,
    /// How much time an overtime adds.
    #[arg(short = 'r', long, value_parser = parse_duration, default_value_t = OVERTIME)]
    pub overtime_duration: Duration,
    /// Indicates how many pomodori it takes to have long break
    #[arg(short = 'n', long, default_value_t = LONG_BREAK_IN)]
    pub long_break_in: u16,
    #[arg(short = 'p', long, action = ArgAction::SetTrue, exclusive = true)]
    pub print_default_config: bool,
    #[arg(short = 'i', long, action = ArgAction::SetTrue, exclusive = true)]
    pub write_default_config: bool,
    #[arg(short = 't', long, action = ArgAction::SetTrue)]
    pub test_break_timer: bool,
}

fn parse_duration(arg: &str) -> Result<Duration> {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        let (num, char) = match self {
            Duration::Seconds(num) => (num, 's'),
            Duration::Minutes(num) => (num, 'm'),
        };
        write!(f, "{num}{char}")
    }
}
