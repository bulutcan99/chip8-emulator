use crate::shared::data;
use crate::shared::logger::logger;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::environment::Environment;
use super::error::ConfigError;

lazy_static! {
    static ref DEFAULT_FOLDER: PathBuf = PathBuf::from("configs");
}

/// Main application configuration structure.
///
/// This struct encapsulates various configuration settings. The configuration
/// can be customized through YAML files for different environments.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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

/// EmuSettings configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EmuSettings {
    pub scale: u32,
    pub cycles_per_frame: u32,
    pub bg_color: Color,
    pub pixel_color: Color,
    pub default_ch8_folder: String,
    pub st_equals_buzzer: bool,
    pub bit_shift_instructions_use_vy: bool,
    pub store_read_instructions_change_i: bool,
}

impl EmuSettings {
    /// Function to create a new `EmuSettings` from default or YAML configuration.
    pub fn new_from_yaml() -> Self {
        let config = Config::get().chip8.clone();
        config
    }

    pub fn get_bg_color(&self) -> Color {
        self.bg_color
    }

    pub fn get_cycles_per_frame(&self) -> u32 {
        self.cycles_per_frame
    }

    pub fn get_default_ch8_folder(&self) -> &str {
        &self.default_ch8_folder
    }

    pub fn get_pixel_color(&self) -> Color {
        self.pixel_color
    }

    pub fn get_scale(&self) -> u32 {
        self.scale
    }

    pub fn get_st_equals_buzzer(&self) -> bool {
        self.st_equals_buzzer
    }

    pub fn get_bit_shift_instructions_use_vy(&self) -> bool {
        self.bit_shift_instructions_use_vy
    }

    pub fn get_store_read_instructions_change_i(&self) -> bool {
        self.store_read_instructions_change_i
    }
}

/// Fetch config and initialize it from folder, for YAML loading
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
        let rendered = data::render_string(&content, &serde_json::json!({}))
            .map_err(|e| ConfigError::TemplateRenderError(e.to_string()))?;

        serde_yaml::from_str(&rendered).map_err(|e| ConfigError::YamlParseError(e.to_string()))
    }
}
