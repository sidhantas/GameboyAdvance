use std::mem::size_of;

use crate::memory::oam::Oam;

use super::{
    affine_parameters::{AffineParameters, NUM_OAM_AFFINE_PARAMETERS},
    memory_block::{MemoryBlock, SimpleMemoryBlock},
    oam::NUM_OAM_ENTRIES,
};
pub struct OAMBlock {
    pub memory: SimpleMemoryBlock,
    affine_parameters: Vec<AffineParameters>,
    pub is_dirty: bool,
    active_objects: Vec<Oam>,
}
const OAM_SIZE: usize = 0x400;
const OAM_MIRROR_MASK: usize = 0x3FF;

impl OAMBlock {
    pub fn new() -> Self {
        let oam = SimpleMemoryBlock::new(OAM_SIZE, OAM_MIRROR_MASK);
        let mut affine_parameters = Vec::with_capacity(NUM_OAM_AFFINE_PARAMETERS);

        for i in 0..NUM_OAM_AFFINE_PARAMETERS {
            affine_parameters.push(AffineParameters::create_parameters(&oam, i))
        }

        Self {
            memory: oam,
            affine_parameters,
            is_dirty: true,
            active_objects: Vec::new(),
        }
    }

    fn update_affine_paramters(&mut self, group: usize) {
        self.affine_parameters[group] = AffineParameters::create_parameters(&self.memory, group);
    }

    pub fn get_affine_paramters(&self, group: usize) -> AffineParameters {
        self.affine_parameters[group].clone()
    }

    pub fn oam_read(&self, oam_num: usize) -> Oam {
        let oam_slice: [u8; 6] = self.memory.memory[oam_num * 0x08..][..6]
            .try_into()
            .unwrap();
        let oam_slice: [u16; 3] = unsafe { oam_slice.align_to::<u16>().1.try_into().unwrap() };

        return Oam::new(oam_slice);
    }

    fn on_write_updates<const SIZE: usize>(&mut self, address: usize) {
        for i in 0..SIZE {
            // Check if affine parameters changed
            if (address + i) & 0x6 == 0x6 {
                let group = (address >> 5) & 0x1F;
                self.update_affine_paramters(group);
            }
        }
        self.is_dirty = true;
    }
}

impl MemoryBlock for OAMBlock {
    fn writeu8(&mut self, address: usize, value: u8) {
        self.memory.writeu8(address, value);
        self.on_write_updates::<{ size_of::<u8>() }>(address);
    }

    fn writeu16(&mut self, address: usize, value: u16) {
        self.memory.writeu16(address, value);
        self.on_write_updates::<{ size_of::<u16>() }>(address);
    }

    fn writeu32(&mut self, address: usize, value: u32) {
        self.memory.writeu32(address, value);
        self.on_write_updates::<{ size_of::<u32>() }>(address);
    }

    fn readu8(&self, address: usize) -> u8 {
        self.memory.readu8(address)
    }

    fn readu16(&self, address: usize) -> u16 {
        self.memory.readu16(address)
    }

    fn readu32(&self, address: usize) -> u32 {
        self.memory.readu32(address)
    }
}
