use std::{mem::size_of, usize};

pub trait MemoryBlock {
    fn writeu8(&mut self, address: usize, value: u8);
    fn writeu16(&mut self, address: usize, value: u16);
    fn writeu32(&mut self, address: usize, value: u32);
    fn readu8(&self, address: usize) -> u8;
    fn readu16(&self, address: usize) -> u16;
    fn readu32(&self, address: usize) -> u32;
}

pub struct SimpleMemoryBlock {
    pub memory: Vec<u8>,
    memory_mask: usize,
}

impl SimpleMemoryBlock {
    pub fn new(size: usize, mask: usize) -> Self {
        SimpleMemoryBlock {
            memory: vec![0; size],
            memory_mask: mask,
        }
    }
    fn get_memory_slice_mut<const SIZE: usize>(
        &mut self,
        address: usize,
    ) -> Option<&mut [u8; SIZE]> {
        let address = address & Self::get_slice_alignment(SIZE);
        let mirror_masked_address = address & self.memory_mask;

        let slice: &mut Vec<u8> = self.memory.as_mut();

        if mirror_masked_address + SIZE > slice.len() {
            return None;
        };

        Some(
            slice[mirror_masked_address..][..SIZE]
                .as_mut()
                .try_into()
                .unwrap(),
        )
    }

    fn get_memory_slice<const SIZE: usize>(&self, address: usize) -> Option<[u8; SIZE]> {
        let address = address & Self::get_slice_alignment(SIZE);
        let mirror_masked_address = address & self.memory_mask;

        let slice: &Vec<u8> = self.memory.as_ref();

        if mirror_masked_address + SIZE > slice.len() {
            return None;
        };

        Some(
            slice[mirror_masked_address..][..SIZE]
                .as_ref()
                .try_into()
                .unwrap(),
        )
    }

    const fn get_slice_alignment(size: usize) -> usize {
        match size {
            1 => !0x0,
            2 => !0x1,
            4 => !0x3,
            _ => unreachable!(),
        }
    }
}

impl MemoryBlock for SimpleMemoryBlock {
    fn writeu8(&mut self, address: usize, value: u8) {
        let Some(slice) = self.get_memory_slice_mut::<{ size_of::<u8>() }>(address) else {
            return;
        };
        slice.copy_from_slice(&value.to_le_bytes())
    }

    fn writeu16(&mut self, address: usize, value: u16) {
        let Some(slice) = self.get_memory_slice_mut::<{ size_of::<u16>() }>(address) else {
            return;
        };
        slice.copy_from_slice(&value.to_le_bytes())
    }

    fn writeu32(&mut self, address: usize, value: u32) {
        let Some(slice) = self.get_memory_slice_mut::<{ size_of::<u32>() }>(address) else {
            return;
        };
        slice.copy_from_slice(&value.to_le_bytes())
    }

    fn readu8(&self, address: usize) -> u8 {
        let Some(slice) = self.get_memory_slice::<{ size_of::<u8>() }>(address) else {
            return 0;
        };

        slice[0]
    }

    fn readu16(&self, address: usize) -> u16 {
        let Some(slice) = self.get_memory_slice::<{ size_of::<u16>() }>(address) else {
            return 0;
        };

        u16::from_le_bytes(slice)
    }

    fn readu32(&self, address: usize) -> u32 {
        let Some(slice) = self.get_memory_slice::<{ size_of::<u32>() }>(address) else {
            return 0;
        };

        u32::from_le_bytes(slice)
    }
}
