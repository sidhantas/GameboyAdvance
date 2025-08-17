use std::{mem::size_of, usize};

pub trait MemoryBlock {
    fn writeu8(&mut self, address: usize, value: u8);
    fn writeu16(&mut self, address: usize, value: u16);
    fn writeu32(&mut self, address: usize, value: u32);
    fn readu8(&self, address: usize) -> u8;
    fn readu16(&self, address: usize) -> u16;
    fn readu32(&self, address: usize) -> u32;
}

pub struct SimpleMemoryBlock<const MASK: usize> {
    pub memory: Vec<u8>,
}

impl<const MASK: usize> SimpleMemoryBlock<MASK> {
    pub fn new(size: usize) -> Self {
        SimpleMemoryBlock::<MASK> {
            memory: vec![0; size + 4],
        }
    }
    fn get_memory_slice_mut<const SIZE: usize>(&mut self, address: usize) -> &mut [u8; SIZE] {
        let address = address & Self::get_slice_alignment(SIZE);
        let mirror_masked_address = address & MASK;

        let slice: &mut Vec<u8> = self.memory.as_mut();

        slice[mirror_masked_address..][..SIZE]
            .as_mut()
            .try_into()
            .unwrap()
    }

    fn get_memory_slice<const SIZE: usize>(&self, address: usize) -> [u8; SIZE] {
        let address = address & Self::get_slice_alignment(SIZE);
        let mirror_masked_address = address & MASK;

        let slice: &Vec<u8> = self.memory.as_ref();

        slice[mirror_masked_address..][..SIZE].try_into().unwrap()
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

impl<const MASK: usize> MemoryBlock for SimpleMemoryBlock<MASK> {
    fn writeu8(&mut self, address: usize, value: u8) {
        let slice = self.get_memory_slice_mut::<{ size_of::<u8>() }>(address);
        slice.copy_from_slice(&value.to_le_bytes())
    }

    fn writeu16(&mut self, address: usize, value: u16) {
        let slice = self.get_memory_slice_mut::<{ size_of::<u16>() }>(address);
        slice.copy_from_slice(&value.to_le_bytes())
    }

    fn writeu32(&mut self, address: usize, value: u32) {
        let slice = self.get_memory_slice_mut::<{ size_of::<u32>() }>(address);
        slice.copy_from_slice(&value.to_le_bytes())
    }

    fn readu8(&self, address: usize) -> u8 {
        let slice = self.get_memory_slice::<{ size_of::<u8>() }>(address);

        slice[0]
    }

    fn readu16(&self, address: usize) -> u16 {
        let slice = self.get_memory_slice::<{ size_of::<u16>() }>(address);

        u16::from_le_bytes(slice)
    }

    fn readu32(&self, address: usize) -> u32 {
        let slice = self.get_memory_slice::<{ size_of::<u32>() }>(address);

        u32::from_le_bytes(slice)
    }
}
