use std::{
    mem::size_of,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Condvar, Mutex, MutexGuard,
    },
    time::Duration,
};

use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect, render::Texture, surface::Surface};

use super::ppu::{HDRAW, VDRAW};

const DISPLAY_SCALE: u32 = 3;
pub const CANVAS_AREA: usize = (HDRAW * VDRAW) as usize;

pub struct DisplayBuffer {
    pub buffer: Mutex<[u32; CANVAS_AREA]>,
    pub ready_to_render: AtomicBool,
}

impl DisplayBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new([0; CANVAS_AREA]),
            ready_to_render: true.into(),
        }
    }
}

pub fn start_display(pixel_buffer: Arc<DisplayBuffer>) {
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
        while pixel_buffer.ready_to_render.load(Relaxed) == false {
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
            std::thread::sleep(Duration::new(0, 1_000_000_000 / 60));
        }
        pixel_buffer.ready_to_render.store(false, Relaxed);
        canvas.clear();
        let pixel_data: [u8; CANVAS_AREA * size_of::<u32>()] =
            unsafe { std::mem::transmute(*pixel_buffer.buffer.lock().unwrap()) };
        texture
            .update(None, &pixel_data, HDRAW as usize * size_of::<u32>())
            .unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}
