use crate::memory::{
    io_handlers::{BG0CNT, BG1CNT, BG2CNT, BG3CNT},
    memory::GBAMemory,
    wrappers::{bgcnt::BGCnt, dispcnt::Dispcnt},
};

use super::{pallete::BGPalleteData, ppu_modes::hdraw::RGBComponents, wrappers::tile::Tile};

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

#[derive(Clone, Copy)]
pub struct OBJPixel {
    pub priority: u16,
    pub pixel: RGBComponents,
    pub is_semi_transparent: bool,
}

#[derive(Clone, Copy)]
pub struct BGPixel {
    pub priority: u16,
    pub pixel: Option<RGBComponents>,
}

pub enum LayerPixel {
    OBJ(OBJPixel),
    BG(BGPixel),
}

impl LayerPixel {
    pub fn pixel(&self) -> RGBComponents {
        match self {
            LayerPixel::OBJ(obj) => obj.pixel,
            LayerPixel::BG(bg) => bg.pixel.unwrap_or(RGBComponents::backdrop()),
        }
    }

    pub fn priority(&self) -> u16 {
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
        let highest_priority_background_pixel = self.get_highest_priority_bg_pixel();
        if let Some(obj_pixel) = self.obj {
            if obj_pixel.priority <= highest_priority_background_pixel.priority {
                return LayerPixel::OBJ(obj_pixel);
            }
        }

        LayerPixel::BG(highest_priority_background_pixel)
    }

    fn get_highest_priority_bg_pixel(&self) -> BGPixel {
        let bg_layers = [self.bg3, self.bg2, self.bg1, self.bg0];
        let mut highest_priority_pixel = self.bd;

        for layer in bg_layers {
            if let Some(bg_pixel) = layer {
                if bg_pixel.priority <= highest_priority_pixel.priority && bg_pixel.pixel.is_some()
                {
                    highest_priority_pixel = bg_pixel
                }
            }
        }

        highest_priority_pixel
    }

    pub fn get_enabled_layers(
        x: i32,
        y: i32,
        dispcnt: &Dispcnt,
        memory: &GBAMemory,
        obj: Option<OBJPixel>,
    ) -> Self {
        let mut layers = Layers::default();

        if dispcnt.bg0_enabled() {
            let bgcnt = BGCnt(memory.io_load(BG0CNT));
            layers.bg0 = Some(Self::get_background_pixel(x, y, bgcnt, memory, &dispcnt));
        }

        if dispcnt.bg1_enabled() {
            let bgcnt = BGCnt(memory.io_load(BG1CNT));
            layers.bg1 = Some(Self::get_background_pixel(x, y, bgcnt, memory, &dispcnt));
        }

        if dispcnt.bg2_enabled() {
            let bgcnt = BGCnt(memory.io_load(BG2CNT));
            layers.bg2 = Some(Self::get_background_pixel(x, y, bgcnt, memory, &dispcnt));
        }

        if dispcnt.bg3_enabled() {
            let bgcnt = BGCnt(memory.io_load(BG3CNT));
            layers.bg3 = Some(Self::get_background_pixel(x, y, bgcnt, memory, &dispcnt));
        }

        if dispcnt.obj_enabled() {
            layers.obj = obj;
        }

        layers
    }

    fn get_background_pixel(
        x: i32,
        y: i32,
        bgcnt: BGCnt,
        memory: &GBAMemory,
        dispcnt: &Dispcnt,
    ) -> BGPixel {
        let tile_x = x / 8;
        let tile_y = y / 8;
        let pixel_x = x % 8;
        let pixel_y = y % 8;

        let pallete_region = &memory.pallete_ram.memory[0x00..][..0x200].try_into().unwrap();
        let pallete = BGPalleteData(pallete_region);

        let tile = match dispcnt.get_bg_mode() {
            0x2 => Tile::get_tile_relative_bg(
                memory,
                &bgcnt,
                dispcnt,
                tile_y as usize,
                tile_x as usize,
            ),
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
