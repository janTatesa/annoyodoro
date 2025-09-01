use annoyodoro::{break_timer::BreakTimer, config::Config};
use clap::{ArgAction::SetTrue, Parser, arg};
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short = 'l', long, action = SetTrue)]
    is_long_break: bool,
    #[arg(short = 'p', long, action = SetTrue, exclusive = true)]
    print_default_config: bool,
    #[arg(short = 'w', long, action = SetTrue, exclusive = true)]
    write_default_config: bool
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.print_default_config {
        Config::print_default();
        return Ok(());
    }

    if cli.write_default_config {
        return Config::write_default();
    };

    let config = Config::new()?;

    let goal = BreakTimer::spawn(cli.is_long_break, config)?;
    println!("{goal}");
    Ok(())
}
