use anyhow::{anyhow, Error};
use sdl2::{Sdl};

pub struct SdlContext;

impl SdlContext {
    pub fn init() -> Result<Sdl, Error> {
        let sdl_context = sdl2::init().map_err(|e| anyhow!("Failed to initialize SDL: {:?}", e))?;
        Ok(sdl_context)
    }
}
