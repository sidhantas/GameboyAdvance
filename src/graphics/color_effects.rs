use crate::memory::{
    io_handlers::{BLDCNT, BLDY},
    memory::GBAMemory,
    wrappers::blending::{BldCnt, BldY, BlendMode},
};

use super::ppu_modes::hdraw::{BGPixel, OBJPixel, RGBComponents};

pub enum TargetPixel {
    Obj(OBJPixel),
    BG(BGPixel),
}

pub fn color_effects_pipeline(
    memory: &GBAMemory,
    obj_pixel: Option<OBJPixel>,
    bg_pixel: BGPixel,
) -> RGBComponents {
    let binding = memory.io_load(BLDCNT);
    let bldcnt = BldCnt(&binding);

    match bldcnt.bld_mode() {
        BlendMode::BldOff => {
            if let Some(OBJPixel {
                priority,
                pixel,
                is_semi_transparent: _,
            }) = obj_pixel
            {
                if priority <= bg_pixel.priority {
                    return pixel;
                }
            } else {
                return bg_pixel.pixel;
            }
        }
        BlendMode::BldAlpha => {}
        BlendMode::BldWhite => {
            let bldy = BldY(&memory.io_load(BLDY));
            if let Some(OBJPixel {
                priority,
                pixel,
                is_semi_transparent: _,
            }) = obj_pixel
            {
                if priority <= bg_pixel.priority {
                    return pixel;
                }
            } else {
                return bg_pixel.pixel;
            }
        }
        BlendMode::BldBlack => {}
    }

    RGBComponents::default()
}
