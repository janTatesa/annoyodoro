mod app;
mod break_time;
mod cli;
mod event;
mod state;
mod ui;

use std::{fs, time::Duration};

use annoyodoro_config::{print_default_config, write_default_config};
use app::{App, now};
use clap::Parser;
use cli::Cli;
use cli_log::{error, init_cli_log};
use color_eyre::Result;
use crossterm::event::EventStream;
use state::State;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<()> {
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

    fs::create_dir_all(State::get_path())?;
    let result = App {
        running: true,
        state: State::read()
            .inspect_err(|e| error!("Failed to read state: {e}, using default"))
            .unwrap_or_default(),
        cli,
        ticker: interval(Duration::from_secs(1)),
        is_paused: false,
        event_stream: EventStream::new(),
        state_date: now()?,
        state_backup_interval: interval(Duration::from_secs(200)),
    }
    .run(&mut ratatui::init())
    .await;
    ratatui::restore();
    result
}
