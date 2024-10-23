use sdl2::pixels::Color;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::environment::Environment;
use super::error::ConfigError;
use crate::logger::logger;

/// SerializableColor is a custom struct to allow serialization and deserialization of `sdl2::pixels::Color`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        SerializableColor {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

impl From<SerializableColor> for Color {
    fn from(serializable_color: SerializableColor) -> Self {
        Color::RGBA(
            serializable_color.r,
            serializable_color.g,
            serializable_color.b,
            serializable_color.a,
        )
    }
}

/// Main application configuration structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub app: App,
    pub logger: Logger,
    pub chip8: EmuSettings,
}

/// App configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct App {
    pub name: String,
}

/// Logger configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Logger {
    pub enable: bool,
    #[serde(default)]
    pub pretty_backtrace: bool,
    pub level: logger::LogLevel,
    pub format: logger::Format,
    pub override_filter: Option<String>,
    pub file_appender: Option<LoggerFileAppender>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LoggerFileAppender {
    pub enable: bool,
    #[serde(default)]
    pub non_blocking: bool,
    pub level: logger::LogLevel,
    pub format: logger::Format,
    pub rotation: logger::Rotation,
    pub dir: Option<String>,
    pub filename_prefix: Option<String>,
    pub filename_suffix: Option<String>,
    pub max_log_files: usize,
}

/// EmuSettings configuration with `SerializableColor`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmuSettings {
    pub scale: u32,
    pub cycles_per_frame: u32,
    pub bg_color: SerializableColor,
    pub pixel_color: SerializableColor,
    pub default_ch8_folder: String,
    pub st_equals_buzzer: bool,
    pub bit_shift_instructions_use_vy: bool,
    pub store_read_instructions_change_i: bool,
}

/// Fetch config and initialize it from folder, for YAML loading
static CONFIG: OnceLock<Config> = OnceLock::new();
impl Config {
    pub fn new(env: &Environment) -> Result<Self, ConfigError> {
        let config = Self::from_folder(env, Path::new("configs"))?;
        CONFIG
            .set(config.clone())
            .map_err(|_| ConfigError::SettingsAlreadyInitialized)?;
        Ok(config)
    }

    pub fn get() -> &'static Config {
        CONFIG.get().expect("SETTINGS has not been initialized!")
    }

    pub fn from_folder(env: &Environment, path: &Path) -> Result<Self, ConfigError> {
        let files = [
            path.join(format!("{env}.local.yaml")),
            path.join(format!("{env}.yaml")),
        ];

        let selected_path = files
            .iter()
            .find(|p| p.exists())
            .ok_or(ConfigError::NoConfigFileFound)?;

        let content = fs::read_to_string(selected_path)
            .map_err(|e| ConfigError::FileReadError(e.to_string()))?;

        serde_yaml::from_str(&content).map_err(|e| ConfigError::YamlParseError(e.to_string()))
    }
}
