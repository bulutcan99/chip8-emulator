use anyhow::{anyhow, Error};
use display::sdl::window::CustomWindow;
use sdl2::{rect::Point, AudioSubsystem, EventPump};
use shared::data::math_2d::Math2d;

use super::emulator::Emulator;

pub struct DisplayController<'a> {
    window: &'a mut CustomWindow<'a>,
}

impl<'a> DisplayController<'a> {
    pub fn new(window: &'a mut CustomWindow<'a>) -> Self {
        Self { window }
    }

    pub fn display_canvas(&mut self) {
        self.window.canvas.present();
    }

    pub fn get_audio_subsystem(&self) -> AudioSubsystem {
        self.window.sdl.audio().unwrap()
    }

    pub fn get_event_pump(&self) -> EventPump {
        self.window.sdl.event_pump().unwrap()
    }

    pub fn get_window(&self) -> &CustomWindow {
        self.window
    }

    pub fn set_canvas_scale(&mut self) {
        self.window
            .canvas
            .set_scale(self.window.scale as f32, self.window.scale as f32)
            .unwrap()
    }
}
