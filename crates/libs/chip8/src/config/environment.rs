use std::env;
use std::str::FromStr;

use super::config::Config;
use crate::shared::config::error::ConfigError;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;

pub const DEFAULT_ENVIRONMENT: &str = "development";

impl From<String> for Environment {
    fn from(env: String) -> Self {
        Self::from_str(&env).unwrap_or(Self::Any(env))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Environment {
    #[serde(rename = "production")]
    Production,
    #[serde(rename = "development")]
    Development,
    #[serde(rename = "test")]
    Test,
    Any(String),
}

impl Environment {
    pub fn from_env() -> Self {
        dotenv().ok();
        let env_var = env::var("ENVIRONMENT").unwrap_or_else(|_| DEFAULT_ENVIRONMENT.to_string());
        Self::from(env_var)
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        Config::new(self)
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any(s) => s.fmt(f),
            _ => to_variant_name(self).expect("only enum supported").fmt(f),
        }
    }
}

impl FromStr for Environment {
    type Err = &'static str;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        match input {
            "production" => Ok(Self::Production),
            "development" => Ok(Self::Development),
            "test" => Ok(Self::Test),
            s => Ok(Self::Any(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!("production", Environment::Production.to_string());
        assert_eq!("custom", Environment::Any("custom".to_string()).to_string());
    }

    #[test]
    fn test_into() {
        let e: Environment = "production".to_string().into();
        assert_eq!(e, Environment::Production);
        let e: Environment = "custom".to_string().into();
        assert_eq!(e, Environment::Any("custom".to_string()));
    }

    #[test]
    fn test_from_env() {
        env::set_var("environment", "production");
        let e = Environment::from_env();
        assert_eq!(e, Environment::Production);

        env::set_var("environment", "custom");
        let e = Environment::from_env();
        assert_eq!(e, Environment::Any("custom".to_string()));

        env::remove_var("environment");
        let e = Environment::from_env();
        assert_eq!(e, Environment::Development);
    }
}
