use crate::memory::{
    io_handlers::{BG0CNT, BG1CNT, BG2CNT, BG3CNT},
    memory::GBAMemory,
    oam::OBJMode,
    wrappers::{bgcnt::BGCnt, dispcnt::Dispcnt},
};

use super::{pallete::BGPalleteData, ppu_modes::hdraw::RGBComponents, wrappers::tile::Tile};

pub struct DisplayContext {
    pub bg0_enabled: bool,
    pub bg1_enabled: bool,
    pub bg2_enabled: bool,
    pub bg3_enabled: bool,
    pub obj_enabled: bool,
    pub color_special_effects_enabled: bool,
}

#[derive(Clone)]
pub struct Layers {
    pub bg0: Option<BGPixel>,
    pub bg1: Option<BGPixel>,
    pub bg2: Option<BGPixel>,
    pub bg3: Option<BGPixel>,
    pub obj: Option<OBJPixel>,
    pub bd: BGPixel,
}

impl Default for Layers {
    fn default() -> Self {
        Self {
            bg0: None,
            bg1: None,
            bg2: None,
            bg3: None,
            obj: None,
            bd: BGPixel::backdrop(),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct OBJPixel {
    pub priority: u16,
    pub pixel: RGBComponents,
    pub mode: OBJMode,
}

#[derive(Clone, Copy)]
pub struct BGPixel {
    pub priority: u16,
    pub pixel: Option<RGBComponents>,
}

#[derive(Clone, Copy)]
pub enum LayerPixel {
    OBJ(OBJPixel),
    BG(BGPixel),
}

impl LayerPixel {
    pub fn pixel(&self) -> Option<RGBComponents> {
        match self {
            LayerPixel::OBJ(obj) => Some(obj.pixel),
            LayerPixel::BG(bg) => bg.pixel,
        }
    }

    pub const fn priority(&self) -> u16 {
        match self {
            LayerPixel::OBJ(obj) => obj.priority,
            LayerPixel::BG(bg) => bg.priority,
        }
    }
}

impl BGPixel {
    pub const fn backdrop() -> Self {
        Self {
            priority: 3,
            pixel: Some(RGBComponents {
                r: 0x1f,
                g: 0x1f,
                b: 0x1f,
            }),
        }
    }
}

impl Layers {
    pub fn get_top_layer(&self) -> LayerPixel {
        let pixels = [
            self.bg3.map(|pixel| LayerPixel::BG(pixel)),
            self.bg2.map(|pixel| LayerPixel::BG(pixel)),
            self.bg1.map(|pixel| LayerPixel::BG(pixel)),
            self.bg0.map(|pixel| LayerPixel::BG(pixel)),
            self.obj.map(|pixel| LayerPixel::OBJ(pixel)),
        ];
        pixels.into_iter().fold(
            LayerPixel::BG(self.bd),
            |top_pixel: LayerPixel, new_pixel: Option<LayerPixel>| {
                let Some(pixel) = new_pixel else {
                    return top_pixel;
                };
                if pixel.priority() <= top_pixel.priority() && pixel.pixel().is_some(){
                    pixel
                } else {
                    top_pixel
                }
            },
        )
    }

    pub fn get_top_bg_pixel(&self) -> BGPixel {
        let pixels = [self.bg3, self.bg2, self.bg1, self.bg0];
        pixels
            .into_iter()
            .fold(self.bd, |top_pixel: BGPixel, new_pixel: Option<BGPixel>| {
                let Some(pixel) = new_pixel else {
                    return top_pixel;
                };
                if pixel.priority <= top_pixel.priority {
                    pixel
                } else {
                    top_pixel
                }
            })
    }

    pub fn get_enabled_layers(
        x: i32,
        y: i32,
        memory: &GBAMemory,
        obj: Option<OBJPixel>,
        bg_mode: u16,
        context: &DisplayContext,
    ) -> Self {
        let mut layers = Layers::default();

        if context.bg0_enabled {
            let bgcnt = BGCnt(memory.io_load(BG0CNT));
            layers.bg0 = Some(Self::get_background_pixel(x, y, bgcnt, memory, bg_mode));
        }

        if context.bg1_enabled {
            let bgcnt = BGCnt(memory.io_load(BG1CNT));
            layers.bg1 = Some(Self::get_background_pixel(x, y, bgcnt, memory, bg_mode));
        }

        if context.bg2_enabled {
            let bgcnt = BGCnt(memory.io_load(BG2CNT));
            layers.bg2 = Some(Self::get_background_pixel(x, y, bgcnt, memory, bg_mode));
        }

        if context.bg3_enabled {
            let bgcnt = BGCnt(memory.io_load(BG3CNT));
            layers.bg3 = Some(Self::get_background_pixel(x, y, bgcnt, memory, bg_mode));
        }

        if context.obj_enabled {
            layers.obj = obj;
        }

        layers
    }

    fn get_background_pixel(
        x: i32,
        y: i32,
        bgcnt: BGCnt,
        memory: &GBAMemory,
        bg_mode: u16,
    ) -> BGPixel {
        let tile_x = x / 8;
        let tile_y = y / 8;
        let pixel_x = x % 8;
        let pixel_y = y % 8;

        let pallete_region = &memory.pallete_ram.memory[0x00..][..0x200]
            .try_into()
            .unwrap();
        let pallete = BGPalleteData(pallete_region);

        let tile = match bg_mode {
            0x2 => Tile::get_tile_relative_bg(memory, &bgcnt, tile_y as usize, tile_x as usize),
            _ => {
                return BGPixel {
                    priority: 3,
                    pixel: Default::default(),
                }
            }
        };

        BGPixel {
            priority: bgcnt.priority(),
            pixel: pallete.get_pixel_from_tile(&tile, pixel_x as usize, pixel_y as usize),
        }
    }
}
