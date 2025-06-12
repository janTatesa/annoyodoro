pub mod duration;

use color_eyre::{
    eyre::{ContextCompat, Result},
    owo_colors::OwoColorize,
};
use colors_transform::{Color as ColorsTransformColor, Rgb};
use figment::{
    Figment,
    providers::{Data, Toml},
};

use serde::{Deserialize, de};
use std::{fs, str::FromStr};
use tap::Tap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub font: FontConfig,
    pub colors: ColorsConfig,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FontConfig {
    pub size: f32,
    pub name: String,
}

const DEFAULT_CONFIG: &str = include_str!("./default_config.toml");

pub fn print_default_config() {
    println!("{DEFAULT_CONFIG}");
}

pub fn write_default_config() -> Result<()> {
    let path = get_config_path()?;
    fs::create_dir_all(&path)?;
    fs::write(&path, DEFAULT_CONFIG)?;
    println!("Wrote default config to {}", path.display().green());
    Ok(())
}

impl Config {
    pub fn parse() -> Result<Self> {
        let path = get_config_path()?;
        Ok(Figment::new()
            .merge(Data::<Toml>::string(DEFAULT_CONFIG))
            .merge(Data::<Toml>::file(path))
            .extract()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Figment::new()
            .merge(Data::<Toml>::string(DEFAULT_CONFIG))
            .extract()
            .unwrap()
    }
}

fn get_config_path() -> Result<std::path::PathBuf, color_eyre::eyre::Error> {
    Ok(dirs::config_dir()
        .wrap_err("Cannot get config dir")?
        .tap_mut(|p| p.extend(["annoyodoro", "config.toml"])))
}

#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct ColorsConfig {
    pub background: Color,
    pub text: Color,
    pub accent: Color,
    pub warning: Color,
}

#[derive(Clone, Copy)]
pub struct Color(pub f32, pub f32, pub f32);
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let rgb = Rgb::from_str(&string)
            .or_else(|_| Rgb::from_hex_str(&string))
            .map_err(|e| de::Error::custom(format!("Cannot parse color: {:?}", e)))?
            .as_tuple();
        Ok(Self(rgb.0, rgb.1, rgb.2))
    }
}
