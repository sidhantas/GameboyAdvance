use crate::{memory::memory::GBAMemory, utils::bits::fixed88_point_to_floating_point};

use super::oam::Oam;

pub struct AffineParameters([[f32; 2]; 2]);

impl AffineParameters {
    pub fn create_parameters(memory: &GBAMemory, oam: &Oam) -> Option<Self> {
        let Some(group) = oam.rotation_scaling_parameter() else {
            return None;
        };
        Some(Self([
            [
                fixed88_point_to_floating_point(u16::from_le_bytes(memory.oam[group * 0x20 + 0x6..][..2].try_into().unwrap())),
                fixed88_point_to_floating_point(u16::from_le_bytes(memory.oam[group * 0x20 + 0xe..][..2].try_into().unwrap())),
            ],
            [
                fixed88_point_to_floating_point(u16::from_le_bytes(memory.oam[group * 0x20 + 0x16..][..2].try_into().unwrap())),
                fixed88_point_to_floating_point(u16::from_le_bytes(memory.oam[group * 0x20 + 0x1e..][..2].try_into().unwrap())),
            ],
        ]))
    }

    pub fn transform_coordinates(&self, x: i32, y: i32) -> (i32, i32) {
        let transform_x = x as f32 * self.0[0][0] + x as f32 * self.0[1][0];
        let transform_y = y as f32 * self.0[0][1] + x as f32 * self.0[1][1];

        (transform_x as i32, transform_y as i32)
    }
}
