use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use lazy_static::lazy_static;

use super::environment::Environment;
use super::error::ConfigError;
use crate::helper::renderer::render_string;
use crate::logger::logger;

lazy_static! {
    static ref DEFAULT_FOLDER: PathBuf = PathBuf::from("config");
}

/// Main application configuration structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub app: App,
    pub logger: Logger,
    pub chip8: ChipSettings,
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

/// ChipSettings configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChipSettings {
    pub scale: u32,
    pub cycles_per_frame: u32,
    pub default_ch8_folder: String,
    pub st_equals_buzzer: bool,
    pub bit_shift_instructions_use_vy: bool,
    pub store_read_instructions_change_i: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
impl Config {
    pub fn new(env: &Environment) -> Result<Self, ConfigError> {
        let config = Self::from_folder(env, DEFAULT_FOLDER.as_path())?;
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
        let rendered = render_string(&content, &serde_json::json!({}))
            .map_err(|e| ConfigError::TemplateRenderError(e.to_string()))?;

        serde_yaml::from_str(&rendered).map_err(|e| ConfigError::YamlParseError(e.to_string()))
    }
}
