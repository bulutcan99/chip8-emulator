use anyhow::Error;
use log::info;
use shared::{config::environment::Environment, logger::logger};

#[tokio::main]
async fn main() -> Result<(), Error> {
    Environment::from_env().load()?;
    logger::init();
    info!("Environment loaded successfully");
    Ok(())
}
