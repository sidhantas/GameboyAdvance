use super::PixelPriority;

pub(super) struct Background<'a>([&'a Option<PixelPriority>; 4]);


impl<'a> Background<'a> {
    pub fn new(
        bg3_pixel: &'a Option<PixelPriority>,
        bg2_pixel: &'a Option<PixelPriority>,
        bg1_pixel: &'a Option<PixelPriority>,
        bg0_pixel: &'a Option<PixelPriority>,
    ) -> Self {
        Self ([bg0_pixel, bg1_pixel, bg2_pixel, bg3_pixel])
    }

    pub fn get_highest_priority_pixel(&self) -> u32 {
        let mut current_pixel: PixelPriority = PixelPriority {
           pixel: 0x0,
           priority: 5
        };

        for pixel in self.0.iter().rev() {
            if let Some(pixel) = pixel {
                if pixel.priority <= current_pixel.priority {
                    current_pixel = *pixel;
                }
            }
        }

        current_pixel.pixel
    }
}
