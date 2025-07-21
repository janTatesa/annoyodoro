use annoyodoro_break_timer::{BreakTimeResult, CurrentFont, spawn_break_timer};
use annoyodoro_config::{duration::*, print_default_config, write_default_config};

use clap::{ArgAction::SetTrue, Parser, arg};
use cli_log::init_cli_log;
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Bind the break timer to a specific output
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short, long, value_parser = parse_duration, default_value_t = BREAK)]
    break_duration: Duration,
    #[arg(short, long, value_parser = parse_duration, default_value_t = WORK)]
    work_duration: Duration,
    #[arg(long, action = SetTrue)]
    long_break: bool,
    #[arg(long)]
    no_overtime: bool,
    /// If off time is being added, forgive durations that are at most this duration
    #[arg(short, long, value_parser = parse_duration, default_value_t = FORGIVE)]
    forgive_duration: Duration,
    /// How much time an overtime adds.
    #[arg(short = 'r', long, value_parser = parse_duration, default_value_t = OVERTIME)]
    overtime_duration: Duration,
    #[arg(short = 'l', long, value_parser = parse_duration, default_value_t = LONG_BREAK)]
    long_break_duration: Duration,
    #[arg(short = 'p', long, action = SetTrue, exclusive = true)]
    print_default_config: bool,
    #[arg(short = 'i', long, action = SetTrue, exclusive = true)]
    write_default_config: bool,
}

fn main() -> Result<()> {
    init_cli_log!();
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.print_default_config {
        print_default_config();
        return Ok(());
    }

    if cli.write_default_config {
        return write_default_config();
    };

    let duration = match cli.long_break {
        false => cli.break_duration,
        true => cli.long_break_duration,
    }
    .into();
    spawn_break_timer(
        cli.output,
        (!cli.no_overtime).then_some(cli.overtime_duration.into()),
        duration,
        cli.long_break,
        &mut CurrentFont::default(),
    )
    .map(|break_time_result| match break_time_result {
        BreakTimeResult::OvertimeUsed => println!("User has used overtime"),
        BreakTimeResult::BreakComplete(duration) if duration <= cli.forgive_duration.into() => {
            println!("Break complete!")
        }
        BreakTimeResult::BreakComplete(duration) => {
            println!("Break complete with extra duration: {:#?}", duration)
        }
    })
}
