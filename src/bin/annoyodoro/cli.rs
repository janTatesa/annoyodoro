use clap::{ArgAction, Parser, arg, command};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(short = 'p', long, action = ArgAction::SetTrue, exclusive = true)]
    pub print_default_config: bool,
    #[arg(short = 'w', long, action = ArgAction::SetTrue, exclusive = true)]
    pub write_default_config: bool
}
