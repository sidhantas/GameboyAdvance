//use crate::{
//    memory::wrappers::blending::{BldAlpha, BldCnt, BldY, BlendMode},
//    utils::bits::Bits,
//};
//
//use super::PixelPriority;
//
//
//struct BGLayers {
//    layers: [Option<PixelPriority>; 4],
//}
//
//impl BGLayers {
//    fn new(
//        bg3_pixel: Option<PixelPriority>,
//        bg2_pixel: Option<PixelPriority>,
//        bg1_pixel: Option<PixelPriority>,
//        bg0_pixel: Option<PixelPriority>,
//    ) -> Self {
//        Self {
//            layers: [bg0_pixel, bg1_pixel, bg2_pixel, bg3_pixel],
//        }
//    }
//
//    fn get_highest_priority_pixel(&self) -> u32 {
//        let mut current_pixel: Option<PixelPriority> = None;
//
//        for pixel in self.layers.iter().rev() {
//            if current_pixel.is_none() {
//                current_pixel = *pixel;
//            }
//            if let Some(pixel) = pixel {
//                if pixel.priority >= current_pixel.unwrap().priority {
//                    current_pixel = Some(*pixel);
//                }
//            }
//        }
//
//        if let Some(pixel) = current_pixel {
//            pixel.pixel
//        } else {
//            0x0
//        }
//    }
//
//}
//
//fn color_effects(layers: Layers, bldcnt: &BldCnt, bldalpha: &BldAlpha, bldy: &BldY) -> u32 {
//    match bldcnt.bld_mode() {
//        BlendMode::BldOff => return layers.get_highest_priority_pixel(),
//        BlendMode::BldAlpha => {}
//        BlendMode::BldWhite => todo!(),
//        BlendMode::BldBlack => todo!(),
//    }
//}
//
//fn alpha_blending(top_pixel: PixelPriority, bottom_pixel: PixelPriority) -> u32 {}
