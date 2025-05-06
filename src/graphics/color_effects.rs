use crate::memory::{
    io_handlers::{BLDALPHA, BLDCNT},
    memory::GBAMemory,
    wrappers::blending::{BldAlpha, BldCnt, BlendMode},
};

use super::{
    layers::{BGPixel, LayerPixel, Layers, OBJPixel},
    ppu_modes::hdraw::RGBComponents,
};

pub fn color_effects_pipeline(memory: &GBAMemory, layers: Layers) -> RGBComponents {
    let binding = memory.io_load(BLDCNT);
    let bldcnt = BldCnt(&binding);

    let blend_mode = if let Some(obj_pixel) = &layers.obj {
        if obj_pixel.is_semi_transparent {
            BlendMode::BldAlpha
        } else {
            bldcnt.bld_mode()
        }
    } else {
        bldcnt.bld_mode()
    };

    match blend_mode {
        BlendMode::BldAlpha => {
            let (target_layers_a, target_layers_b) =
                TargetLayer::get_enabled_layers(&layers, &bldcnt);

            let target_pixel_a = if let Some(obj_pixel) = target_layers_a.obj {
                if obj_pixel.is_semi_transparent {
                    LayerPixel::OBJ(obj_pixel)
                } else {
                    let Some(target_pixel_a) = target_layers_a.get_top_pixel(false) else {
                        return layers.get_top_layer().pixel();
                    };
                    target_pixel_a
                }
            } else {
                let Some(target_pixel_a) = target_layers_a.get_top_pixel(false) else {
                    return layers.get_top_layer().pixel();
                };
                target_pixel_a
            };

            let Some(target_pixel_b) =
                target_layers_b.get_top_pixel(matches!(target_pixel_a, LayerPixel::OBJ(_)))
            else {
                return layers.get_top_layer().pixel();
            };

            if target_pixel_b.priority() < target_pixel_a.priority() {
                return target_pixel_b.pixel();
            }

            let bldalpha = BldAlpha(memory.io_load(BLDALPHA));
            let eva = bldalpha.eva();
            let evb = bldalpha.evb();

            let pixel_a = RGBComponents {
                r: ((target_pixel_a.pixel().r as u16 * eva as u16) / 16) as u16,
                g: ((target_pixel_a.pixel().g as u16 * eva as u16) / 16) as u16,
                b: ((target_pixel_a.pixel().b as u16 * eva as u16) / 16) as u16,
            };

            let pixel_b = RGBComponents {
                r: ((target_pixel_b.pixel().r as u16 * evb as u16) / 16) as u16,
                g: ((target_pixel_b.pixel().g as u16 * evb as u16) / 16) as u16,
                b: ((target_pixel_b.pixel().b as u16 * evb as u16) / 16) as u16,
            };

            RGBComponents {
                r: pixel_a.r + pixel_b.r,
                g: pixel_a.g + pixel_b.g,
                b: pixel_a.b + pixel_b.b,
            }
        }
        _ => return layers.get_top_layer().pixel(),
    }
}

#[derive(Default)]
struct TargetLayer {
    pub bg0: Option<BGPixel>,
    pub bg1: Option<BGPixel>,
    pub bg2: Option<BGPixel>,
    pub bg3: Option<BGPixel>,
    pub obj: Option<OBJPixel>,
    pub bd: Option<BGPixel>,
}

impl TargetLayer {
    pub fn get_enabled_layers(layers: &Layers, bldcnt: &BldCnt) -> (Self, Self) {
        let mut target_layers_a = TargetLayer::default();
        let mut target_layers_b = TargetLayer::default();

        if bldcnt.target_a_bg0_enabled() {
            target_layers_a.bg0 = layers.bg0;
        }
        if bldcnt.target_a_bg1_enabled() {
            target_layers_a.bg1 = layers.bg1;
        }
        if bldcnt.target_a_bg2_enabled() {
            target_layers_a.bg2 = layers.bg2;
        }
        if bldcnt.target_a_bg3_enabled() {
            target_layers_a.bg3 = layers.bg3;
        }
        if bldcnt.target_a_obj_enabled() {
            target_layers_a.obj = layers.obj;
        }

        if bldcnt.target_a_bd_enabled() {
            target_layers_a.bd = Some(layers.bd);
        }

        if bldcnt.target_b_bg0_enabled() {
            target_layers_b.bg0 = layers.bg0;
        }
        if bldcnt.target_b_bg1_enabled() {
            target_layers_b.bg1 = layers.bg1;
        }
        if bldcnt.target_b_bg2_enabled() {
            target_layers_b.bg2 = layers.bg2;
        }
        if bldcnt.target_b_bg3_enabled() {
            target_layers_b.bg3 = layers.bg3;
        }
        if bldcnt.target_b_obj_enabled() {
            target_layers_b.obj = layers.obj;
        }
        if bldcnt.target_b_bd_enabled() {
            target_layers_b.bd = Some(layers.bd);
        }

        (target_layers_a, target_layers_b)
    }

    pub fn get_top_pixel(&self, exclude_obj: bool) -> Option<LayerPixel> {
        let background_pixels = [self.bd, self.bg3, self.bg2, self.bg1, self.bg0];

        let mut highest_priority_bg_pixel: Option<BGPixel> = None;

        for bg_pixel in background_pixels {
            if let Some(pixel) = highest_priority_bg_pixel {
                if let bg_pix @ Some(bg_pixel) = bg_pixel {
                    if bg_pixel.priority <= pixel.priority {
                        highest_priority_bg_pixel = bg_pix;
                    }
                }
            } else {
                highest_priority_bg_pixel = bg_pixel;
            }
        }

        if !exclude_obj {
            if let Some(bg_pixel) = highest_priority_bg_pixel {
                if let Some(obj_pixel) = self.obj {
                    if obj_pixel.priority <= bg_pixel.priority {
                        return Some(LayerPixel::OBJ(obj_pixel));
                    }
                }
                return Some(LayerPixel::BG(bg_pixel));
            } else {
                return self.obj.map(|obj| LayerPixel::OBJ(obj));
            }
        }

        return highest_priority_bg_pixel.map(|bg_pixel| LayerPixel::BG(bg_pixel));
    }
}

#[cfg(test)]
mod color_effects_tests {
    #[test]
    fn alpha_blends_obj_and_bg() {}
}
