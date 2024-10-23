use anyhow::Error;
use log::info;
use shared::{config::environment::Environment, logger::logger};

#[tokio::main]
async fn main() -> Result<(), Error> {
    info!("Chip-8 Emulator Desktop");
    info!("2024 Bulut Can Gocer <gocerbulutcan@gmail.com>");
    Environment::from_env().load()?;
    logger::init()?;
    info!("Environment loaded successfully");
    Ok(())
}
