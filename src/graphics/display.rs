use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect, render::Texture, surface::Surface};

use super::ppu::{HDRAW, VDRAW};

const DISPLAY_SCALE: u32 = 3;
pub const CANVAS_AREA: usize = (HDRAW * VDRAW) as usize;

pub fn start_display(pixel_buffer: Arc<Mutex<[u32; CANVAS_AREA]>>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Gameboy Advance",
            HDRAW * DISPLAY_SCALE,
            VDRAW * DISPLAY_SCALE,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let surface = Surface::new(HDRAW, VDRAW, PixelFormatEnum::RGB24).unwrap();
    let mut texture = Texture::from_surface(&surface, &texture_creator).unwrap();
    canvas.set_logical_size(HDRAW, VDRAW).unwrap();

    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.clear();
        let pixel_data: MutexGuard<'_, [u8; CANVAS_AREA * 4]> =
            unsafe { std::mem::transmute(pixel_buffer.lock().unwrap()) };
        texture
            .update(
                Rect::new(0, 0, HDRAW, VDRAW),
                &*pixel_data,
                HDRAW as usize * size_of::<u32>(),
            )
            .unwrap();
        canvas.copy(&texture, None, None).unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }
        canvas.present();
        drop(pixel_data);
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
