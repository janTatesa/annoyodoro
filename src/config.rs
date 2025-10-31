use std::{fs, num::NonZero, path::PathBuf, time::Duration};

#[cfg(not(debug_assertions))]
use color_eyre::eyre::ContextCompat;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use figment::{
    Figment,
    providers::{Data, Toml}
};
use iced::{Color, Font, Theme, theme::Palette};
use serde::{Deserialize, Deserializer, de::Error};

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_font")]
    pub font: Font,
    pub pomodoro: PomodoroConfig,
    colors: ColorsConfig
}

fn deserialize_font<'de, D>(deserializer: D) -> Result<Font, D::Error>
where
    D: Deserializer<'de>
{
    let name = String::deserialize(deserializer)?.leak();
    Ok(Font::with_name(name))
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PomodoroConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub break_duration: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub work_duration: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub long_break_duration: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub notification_duration: Duration,
    pub long_break_each: NonZero<u16>
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecsAndMins {
    #[serde(default)]
    mins: u64,
    #[serde(default)]
    secs: u64
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>
{
    let secs_and_mins = SecsAndMins::deserialize(deserializer)?;
    let secs = secs_and_mins.secs + secs_and_mins.mins * 60;
    Ok(Duration::from_secs(secs))
}

#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct ColorsConfig {
    #[serde(deserialize_with = "deserialize_color")]
    background: Color,
    #[serde(deserialize_with = "deserialize_color")]
    text: Color,
    #[serde(deserialize_with = "deserialize_color")]
    accent: Color,
    #[serde(deserialize_with = "deserialize_color")]
    danger: Color
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>
{
    let csscolorparser::Color { r, g, b, a } = String::deserialize(deserializer)?
        .parse()
        .map_err(Error::custom)?;

    Ok(Color { r, g, b, a })
}

impl Config {
    const DEFAULT: &str = include_str!("./default_config.toml");

    #[cfg(not(debug_assertions))]
    fn path() -> Result<PathBuf> {
        let mut path_buf = dirs::config_dir().wrap_err("Cannot get config dir")?;
        path_buf.push("annoyodoro");
        fs::create_dir_all(&path_buf)?;
        path_buf.push("config.toml");
        Ok(path_buf)
    }

    #[cfg(debug_assertions)]
    fn path() -> Result<PathBuf> {
        let mut path_buf = PathBuf::new();
        path_buf.push("testing-files");
        fs::create_dir_all(&path_buf)?;
        path_buf.push("config.toml");
        Ok(path_buf)
    }

    pub fn print_default() {
        println!("{}", Self::DEFAULT);
    }

    pub fn write_default() -> Result<()> {
        let path = Self::path()?;
        fs::write(&path, Self::DEFAULT)?;
        println!("Wrote default config to {}", path.display().green());
        Ok(())
    }

    pub fn new() -> Result<Self> {
        let path = Self::path()?;
        let config = Figment::new()
            .merge(Data::<Toml>::string(Self::DEFAULT))
            .merge(Data::<Toml>::file(path))
            .extract()?;
        Ok(config)
    }

    pub fn theme(&self) -> Theme {
        let ColorsConfig {
            background,
            text,
            accent,
            danger
        } = self.colors;

        Theme::custom(
            String::new(),
            Palette {
                background,
                text,
                primary: accent,
                success: accent,
                danger,
                warning: danger
            }
        )
    }
}
