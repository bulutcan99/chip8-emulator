use sdl2::image::{InitFlag, LoadSurface}; // LoadSurface için gerekli modül
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;
use sdl2::Sdl;

const TITLE: &str = "Chip-8 Emulator";

pub struct CustomWindow<'a> {
    sdl: &'a Sdl,
    win_w: u32,
    win_h: u32,
    scale: u32,
    canvas: Canvas<Window>,
    pixel_vec: Vec<u8>,
    bg_color: Color,
    pixel_color: Color,
}

impl<'a> CustomWindow<'a> {
    pub fn new(
        sdl: &'a Sdl,
        win_w: u32,
        win_h: u32,
        scale: u32,
        bg_color: Color,
        pixel_color: Color,
    ) -> Self {
        let win_w_scaled = win_w * scale;
        let win_h_scaled = win_h * scale;

        let window = sdl
            .video()
            .unwrap()
            .window(TITLE, win_w_scaled, win_h_scaled)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.clear();
        canvas.present();

        let pixel_vec = vec![0; win_w as usize * win_h as usize];

        Self {
            sdl,
            win_w,
            win_h,
            scale,
            canvas,
            pixel_vec,
            bg_color,
            pixel_color,
        }
    }
}
