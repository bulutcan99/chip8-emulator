use anyhow::{anyhow, Error};
use display::sdl::window::CustomWindow;
use sdl2::{rect::Point, AudioSubsystem, EventPump};
use shared::data::math_2d::Math2d;

use super::emulator::Emulator;

pub struct Controller<'a> {
    window: &'a mut CustomWindow<'a>,
}

impl<'a> Controller<'a> {
    pub fn new(window: &'a mut CustomWindow<'a>) -> Self {
        Self { window }
    }

    pub fn pixel_at(&mut self, x: u8, y: u8, emu: &mut Emulator) -> Result<(), Error> {
        // Wrap the coordinates to fit within the window dimensions.
        let x = Math2d::wrap_coord(x, self.window.win_w);
        let y = Math2d::wrap_coord(y, self.window.win_h);
        let pixel_index = (y as u32 * self.window.win_w) + x as u32;

        // Determine if the pixel is OFF (0) or ON (1) and choose the color accordingly.
        let pixel_is_off = self.window.pixel_vec[pixel_index as usize] == 0;
        let draw_color = if pixel_is_off {
            self.window.pixel_color
        } else {
            // If pixel is ON, check and set the collision flag (VF).
            if emu.get_v(0xF)? == 0 {
                emu.set_v(0xF, 1)?;
            }
            self.window.bg_color
        };

        // Set the draw color and draw the point.
        self.window.canvas.set_draw_color(draw_color);
        self.window
            .canvas
            .draw_point(Point::new(x as i32, y as i32))
            .map_err(|e| anyhow!("Error drawing point: {:?}", e))?;
        // Toggle the pixel state (ON/OFF).
        self.window.pixel_vec[pixel_index as usize] ^= 1;

        Ok(())
    }
    pub fn clear_screen(&mut self) {
        self.window.pixel_vec.fill(0);
        self.window.canvas.set_draw_color(self.window.bg_color);
        self.window.canvas.clear();
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
