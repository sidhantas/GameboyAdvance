use std::{
    mem::size_of,
    ops::ControlFlow,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        mpsc::Receiver,
        Arc, Condvar, Mutex, MutexGuard,
    },
    time::Duration,
};

use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect, render::Texture, surface::Surface};

use crate::{debugger::terminal_commands::PPUToDisplayCommands, gba::KILL_SIGNAL};

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

pub struct Border {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn start_display(
    pixel_buffer: Arc<DisplayBuffer>,
    ppu_to_display_recv: Receiver<PPUToDisplayCommands>,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Gameboy Advance",
            HDRAW as u32 * DISPLAY_SCALE,
            VDRAW as u32 * DISPLAY_SCALE,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let surface = Surface::new(HDRAW as u32, VDRAW as u32, PixelFormatEnum::RGB24).unwrap();
    let mut texture = Texture::from_surface(&surface, &texture_creator).unwrap();
    canvas.set_logical_size(HDRAW as u32, VDRAW as u32).unwrap();

    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        while let Ok(command) = ppu_to_display_recv.try_recv() {
            match command {
                PPUToDisplayCommands::Render => {
                    render_frame(&pixel_buffer, &mut canvas, &mut texture);
                }
                PPUToDisplayCommands::RenderWithBorders(borders) => {
                    draw_object_borders(&pixel_buffer, borders);
                    render_frame(&pixel_buffer, &mut canvas, &mut texture);
                }
            }
        }
        if let ControlFlow::Break(_) = handle_events(&mut event_pump) {
            break 'running;
        }
    }
}

fn draw_object_borders(pixel_buffer: &Arc<DisplayBuffer>, borders: Vec<Border>) {
    let mut buff = pixel_buffer.buffer.lock().unwrap();
    for Border {
        x,
        y,
        width,
        height,
    } in borders
    {
        for i in x..x + width {
            if i < 0 {
                continue;
            }
            if i >= HDRAW {
                break;
            }
            if y < 0 {
                break;
            }
            buff[(y * HDRAW + i) as usize] = 0x00FF0000;
        }
        for j in y..y + height {
            if x < 0 {
                break;
            }
            if j >= VDRAW {
                break;
            }
            if j < 0 {
                continue;
            }
            buff[(j * HDRAW + x) as usize] = 0x00FF0000;
            if x + width < HDRAW {
                buff[(j * HDRAW + x + width) as usize] = 0x00FF0000;
            }
        }
        for i in x..x + width {
            if y < 0 {
                break;
            }
            if (y + height) >= VDRAW {
                break;
            }
            if i > HDRAW {
                break;
            }
            if i < 0b0 {
                continue;
            }
            buff[((y + height) * HDRAW + i) as usize] = 0x00FF0000;
        }
    }
}

fn handle_events(event_pump: &mut sdl2::EventPump) -> ControlFlow<()> {
    if KILL_SIGNAL.killed() {
        return ControlFlow::Break(());
    }
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Escape),
                ..
            } => {
                KILL_SIGNAL.kill();
            }
            _ => {}
        }
    }
    std::thread::sleep(Duration::new(0, 1_000_000_000 / 60));
    ControlFlow::Continue(())
}

fn render_frame(
    pixel_buffer: &Arc<DisplayBuffer>,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &mut Texture<'_>,
) {
    canvas.clear();
    let pixel_data: [u8; CANVAS_AREA * size_of::<u32>()] =
        unsafe { std::mem::transmute(*pixel_buffer.buffer.lock().unwrap()) };
    texture
        .update(None, &pixel_data, HDRAW as usize * size_of::<u32>())
        .unwrap();
    canvas.copy(&*texture, None, None).unwrap();
    canvas.present();
}
