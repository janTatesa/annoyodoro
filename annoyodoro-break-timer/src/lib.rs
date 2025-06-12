mod iced_implementation;
mod view;

use std::time::{Duration, SystemTime};

use annoyodoro_config::{ColorsConfig, Config};
use cli_log::error;
use color_eyre::Result;
use iced::{Font, Pixels};
use iced_layershell::{
    Application,
    reexport::{Anchor, KeyboardInteractivity, Layer},
    settings::{LayerShellSettings, Settings, StartMode},
};

const NAME: &str = "Break time";

#[derive(Debug, Clone, Copy)]
pub enum BreakTimeResult {
    OvertimeUsed,
    BreakComplete(Duration),
}

pub fn spawn_break_timer(
    binded_output_name: Option<String>,
    overtime: Option<Duration>,
    break_duration: Duration,
    long_break: bool,
) -> Result<BreakTimeResult> {
    let config = Config::parse().inspect_err(|e| {
        let message = format!("Cannot parse config: {e}");
        eprintln!("{message}");
        error!("{message}");
    })?;

    let start_mode = match binded_output_name {
        Some(output) => StartMode::TargetScreen(output),
        None => StartMode::Active,
    };

    let layer_settings = LayerShellSettings {
        size: Some((0, 0)),
        start_mode,
        anchor: Anchor::all(),
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        layer: Layer::Overlay,
        ..Default::default()
    };

    let flags = BreakTimer {
        seconds: break_duration.as_secs(),
        mode: overtime
            .map(BreakTimerMode::RunningWithOvertimeOption)
            .unwrap_or_default(),
        long_break,
        colors_config: config.colors,
    };

    let start_time = SystemTime::now();

    BreakTimer::run(Settings {
        layer_settings,
        flags,
        default_font: Font::with_name("sans-serif"),
        default_text_size: Pixels(config.font.size),
        id: None,
        fonts: Vec::default(),
        antialiasing: false,
        virtual_keyboard_support: None,
    })?;

    Ok(match SystemTime::now().duration_since(start_time)? {
        duration if duration < break_duration => BreakTimeResult::OvertimeUsed,
        duration => BreakTimeResult::BreakComplete(duration),
    })
}

#[derive(Default, Clone, Copy)]
enum BreakTimerMode {
    #[default]
    Running,
    RunningWithOvertimeOption(Duration),
    AfterBreak,
}

struct BreakTimer {
    seconds: u64,
    long_break: bool,
    mode: BreakTimerMode,
    colors_config: ColorsConfig,
}
