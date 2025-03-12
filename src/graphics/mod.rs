pub mod display;
pub mod ppu;
pub mod pallete;
pub mod wrappers;
mod color_effects;
mod background;

#[derive(Clone, Copy)]
struct PixelPriority {
    pub pixel: u32,
    pub priority: u16,
}
