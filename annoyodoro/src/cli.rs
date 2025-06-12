use annoyodoro_config::duration::*;
use clap::{ArgAction, Parser, arg, command};

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
}
