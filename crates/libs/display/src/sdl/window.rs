use sdl2::image::{InitFlag, LoadSurface}; // LoadSurface için gerekli modül
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;
use sdl2::Sdl;

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
        win_title: &str,
        win_w: u32,
        win_h: u32,
        scale: u32,
        bg_color: Color,
        pixel_color: Color,
    ) -> Self {
        let win_w_scaled = win_w * scale;
        let win_h_scaled = win_h * scale;

        let mut canvas = sdl
            .video()
            .unwrap()
            .window(win_title, win_w_scaled, win_h_scaled)
            .position_centered()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        sdl2::image::init(InitFlag::PNG).unwrap();
        if let Ok(win_icon) = Surface::from_file(".\\assets\\img\\icon-65x64.png") {
            canvas.window_mut().set_icon(win_icon);
        }

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
