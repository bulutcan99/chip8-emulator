use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    FileReadError(String),
    #[error("Failed to render template: {0}")]
    TemplateRenderError(String),
    #[error("Failed to parse YAML configuration: {0}")]
    YamlParseError(String),
    #[error("No configuration file found")]
    NoConfigFileFound,
    #[error("Settings already initialized")]
    SettingsAlreadyInitialized,
}
