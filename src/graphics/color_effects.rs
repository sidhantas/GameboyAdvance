use crate::memory::{
    io_handlers::{BLDALPHA, BLDCNT},
    memory::GBAMemory,
    oam::OBJMode,
    wrappers::blending::{BldAlpha, BldCnt, BlendMode},
};

use super::{
    layers::{BGPixel, LayerPixel, Layers, OBJPixel},
    ppu_modes::hdraw::RGBComponents,
};

pub fn color_effects_pipeline(memory: &GBAMemory, layers: Layers) -> RGBComponents {
    let binding = memory.io_load(BLDCNT);
    let bldcnt = BldCnt(&binding);

    let blend_mode = get_blend_mode(&layers, &bldcnt);

    match blend_mode {
        BlendMode::BldAlpha => {
            let (target_layers_a, target_layers_b) =
                TargetLayer::get_enabled_layers(&layers, &bldcnt);

            let Some(target_pixel_a) = target_layers_a.get_target_pixel_a() else {
                return layers.get_top_layer().pixel().unwrap_or(RGBComponents::backdrop());
            };

            let Some(target_pixel_b) = target_layers_b.get_target_pixel_b(target_pixel_a) else {
                return layers.get_top_layer().pixel().unwrap_or(RGBComponents::backdrop());
            };

            let Some(target_pixel_a) = target_pixel_a.pixel() else {
                return layers.get_top_layer().pixel().unwrap_or(RGBComponents::backdrop());
            };
            let Some(target_pixel_b) = target_pixel_b.pixel() else {
                return layers.get_top_layer().pixel().unwrap_or(RGBComponents::backdrop());
            };

            let bldalpha = BldAlpha(memory.io_load(BLDALPHA));
            let eva = bldalpha.eva();
            let evb = bldalpha.evb();

            let pixel_a = RGBComponents {
                r: ((target_pixel_a.r as f32 * eva as f32) / 16.) as u16,
                g: ((target_pixel_a.g as f32 * eva as f32) / 16.) as u16,
                b: ((target_pixel_a.b as f32 * eva as f32) / 16.) as u16,
            };

            let pixel_b = RGBComponents {
                r: ((target_pixel_b.r as f32 * evb as f32) / 16.) as u16,
                g: ((target_pixel_b.g as f32 * evb as f32) / 16.) as u16,
                b: ((target_pixel_b.b as f32 * evb as f32) / 16.) as u16,
            };

            RGBComponents {
                r: pixel_a.r + pixel_b.r,
                g: pixel_a.g + pixel_b.g,
                b: pixel_a.b + pixel_b.b,
            }
        }
        _ => return layers.get_top_layer().pixel().unwrap_or(RGBComponents::backdrop()),
    }
}

fn get_blend_mode(layers: &Layers, bldcnt: &BldCnt<'_>) -> BlendMode {
    if let Some(obj_pixel) = &layers.obj {
        if obj_pixel.mode == OBJMode::SemiTransparent {
            return BlendMode::BldAlpha;
        }
    }
    bldcnt.bld_mode()
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
    pub fn get_target_pixel_a(&self) -> Option<LayerPixel> {
        if let Some(obj) = self.obj {
            if obj.mode == OBJMode::SemiTransparent {
                return Some(LayerPixel::OBJ(obj));
            }
        }

        let pixels = [
            self.bd.map(|pixel| LayerPixel::BG(pixel)),
            self.bg3.map(|pixel| LayerPixel::BG(pixel)),
            self.bg2.map(|pixel| LayerPixel::BG(pixel)),
            self.bg1.map(|pixel| LayerPixel::BG(pixel)),
            self.bg0.map(|pixel| LayerPixel::BG(pixel)),
            self.obj.map(|pixel| LayerPixel::OBJ(pixel)),
        ];

        pixels.into_iter().fold(
            None,
            |top_pixel: Option<LayerPixel>, new_pixel: Option<LayerPixel>| {
                let Some(current_top_pixel) = top_pixel else {
                    return new_pixel;
                };
                let Some(pixel) = new_pixel else {
                    return top_pixel;
                };
                if pixel.priority() <= current_top_pixel.priority() {
                    new_pixel
                } else {
                    top_pixel
                }
            },
        )
    }

    pub fn get_target_pixel_b(&self, target_pixel_a: LayerPixel) -> Option<LayerPixel> {
        let target_pixel_a_priority = target_pixel_a.priority();
        let pixels = [
            self.bd.map(|pixel| LayerPixel::BG(pixel)),
            self.bg3.map(|pixel| LayerPixel::BG(pixel)),
            self.bg2.map(|pixel| LayerPixel::BG(pixel)),
            self.bg1.map(|pixel| LayerPixel::BG(pixel)),
            self.bg0.map(|pixel| LayerPixel::BG(pixel)),
            self.obj.map(|pixel| LayerPixel::OBJ(pixel)),
        ];

        pixels.into_iter().fold(
            None,
            |top_pixel: Option<LayerPixel>, new_pixel: Option<LayerPixel>| {
                let Some(current_top_pixel) = top_pixel else {
                    if let Some(pixel) = new_pixel {
                        if pixel.priority() > target_pixel_a_priority {
                            return new_pixel;
                        }
                    }
                    return None;
                };
                let Some(pixel) = new_pixel else {
                    return top_pixel;
                };
                if pixel.priority() <= current_top_pixel.priority()
                    && pixel.priority() > target_pixel_a_priority
                {
                    new_pixel
                } else {
                    top_pixel
                }
            },
        )
    }

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
}

#[cfg(test)]
mod color_effects_tests {
    #[test]
    fn alpha_blends_obj_and_bg() {}
}
