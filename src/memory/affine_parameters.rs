use crate::utils::bits::fixed88_point_to_floating_point;

use super::{memory_block::SimpleMemoryBlock, oam::Oam};

pub const NUM_OAM_AFFINE_PARAMETERS: usize = 32;
#[derive(Clone)]
pub struct AffineParameters([[f32; 2]; 2]);

impl AffineParameters {
    pub fn create_parameters(oam_memory: &SimpleMemoryBlock, group: usize) -> Self {
        Self([
            [
                fixed88_point_to_floating_point(u16::from_le_bytes(
                    oam_memory.memory[(group * 0x20 + 0x6)..][..2]
                        .try_into()
                        .unwrap(),
                )),
                fixed88_point_to_floating_point(u16::from_le_bytes(
                    oam_memory.memory[(group * 0x20 + 0xe)..][..2]
                        .try_into()
                        .unwrap(),
                )),
            ],
            [
                fixed88_point_to_floating_point(u16::from_le_bytes(
                    oam_memory.memory[(group * 0x20 + 0x16)..][..2]
                        .try_into()
                        .unwrap(),
                )),
                fixed88_point_to_floating_point(u16::from_le_bytes(
                    oam_memory.memory[(group * 0x20 + 0x1e)..][..2]
                        .try_into()
                        .unwrap(),
                )),
            ],
        ])
    }

    pub fn transform_coordinates(&self, x: i32, y: i32, oam: &Oam) -> (i32, i32) {
        let (view_center_x, view_center_y) = oam.view_center();
        let (relative_x, relative_y) = (x - view_center_x, y - view_center_y);
        let transform_x = relative_x as f32 * self.0[0][0] + relative_x as f32 * self.0[1][0];
        let transform_y = relative_y as f32 * self.0[0][1] + relative_y as f32 * self.0[1][1];
    
        let (center_x, center_y) = oam.center();

        (
            transform_x as i32 + center_x,
            transform_y as i32 + center_y,
        )
    }
}

#[cfg(test)]
mod affine_tests {
    use crate::{graphics::wrappers::oam::Oam, memory::memory::GBAMemory};

    use super::AffineParameters;

    #[test]
    fn try_transform() {
        let mut memory = GBAMemory::new();
        memory.oam.memory[0x26] = 0xb8;
        memory.oam.memory[0x3e] = 0xb8;
        let oam = Oam(&[0x27de, 0x0225, 0x08c0]);
        let affine_params = AffineParameters::create_parameters(&memory, &oam).unwrap();
        dbg!(affine_params.0);

        //        let (x, y) = affine_params.transform_coordinates(-64, -32, &oam);
        //        println!("{x} {y}");
    }
}
