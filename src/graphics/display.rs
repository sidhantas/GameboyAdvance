#![allow(unused)]
use std::{sync::{Arc, Mutex}, time::Duration};

use sdl2::{event::Event, pixels::Color};

use crate::memory::memory::GBAMemory;

#[repr(u32)]
enum DisplayAddresses {
    DISPCNT = 0x4000_0000,
    DISPSTAT = 0x4000_0004,
    VCOUNT = 0x4000_0006,
    BG0CNT = 0x4000_0008
}

pub fn start_display(memory: Arc<Mutex<GBAMemory>>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Gameboy Advance", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
