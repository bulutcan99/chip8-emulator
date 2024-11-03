use anyhow::Error;
use log::{debug, info, trace, warn};
use shared::{config::environment::Environment, logger::logger};

//TODO: LOG NOT WORKING
#[tokio::main]
async fn main() -> Result<(), Error> {
    Environment::from_env().load()?;
    logger::init();
    info!("Environment loaded successfully");
    println!("SA");
    info!("Starting the application");
    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    Ok(())
}
