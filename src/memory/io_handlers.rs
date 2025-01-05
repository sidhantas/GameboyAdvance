use super::memory::GBAMemory;

const DISPCNT: usize = 0x000;
const IME: usize = 0x208;
const IE: usize = 0x200;

#[inline(always)]
fn io_load(region: &Vec<u16>, address: usize) -> u16 {
    *region
        .get(address >> 1)
        .unwrap_or(&0)
}

#[inline(always)]
fn io_store(region: &mut Vec<u16>, address: usize, value: u16) {
    let store_address = address >> 1;
    if store_address < region.len() {
        region[store_address] = value;
    }
}

impl GBAMemory {
    pub(super) fn io_writeu8(&mut self, address: usize, value: u8) {
        let offset = address & 0xFFE;
        let mut current_value = io_load(&self.ioram, offset);
        current_value &= (!(0xFF << 8 * (address & 1))) as u16;

        let new_hword = (value as u16) << (8 * (address & 1));
        let new_hword = new_hword | current_value; 

        let store_value = match offset {
            DISPCNT => new_hword,
            IME => (value & 0x1).into(),
            IE => new_hword,
            _ => panic!("Unimplemented address {:#x}", address)
        };
        io_store(&mut self.ioram, offset, store_value)
    }

    pub(super) fn io_writeu16(&mut self, address: usize, value: u16) {
        let offset = address & 0xFFE;
        let store_value = match offset {
            DISPCNT | 0x02 | IE => value,
            IME => value & 0x1,
            _ => panic!("Unimplemented address {:#x}", address)
        };
        io_store(&mut self.ioram, offset, store_value)
    }

    pub(super) fn io_writeu32(&mut self, address: usize, value: u32) {
        let address = address & !0x3;
        let upper: u16 = (value >> 16) as u16;
        let lower : u16 = (value & 0xFFFF) as u16;
        let offset = address & 0xFFC;
        match offset {
            IME => {
                io_store(&mut self.ioram, address, value as u16 & 0x1);
                return;
            },
            _ => {}
        };

        self.io_writeu16(address, lower);
        self.io_writeu16(address + 2, upper);
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::memory::{io_handlers::{DISPCNT, IE}, memory::GBAMemory};

    use super::IME;

    #[test]
    fn can_enable_and_disable_ime() {
        let mut memory = GBAMemory::new();
        memory.io_writeu8(IME, 0xFF);
        assert_eq!(memory.ioram[(IME & 0xFFE) >> 1], 1);
        memory.io_writeu8(IME, 0xF0);
        assert_eq!(memory.ioram[(IME & 0xFFE) >> 1], 0);
        memory.io_writeu16(IME, 0xFFFF);
        assert_eq!(memory.ioram[(IME & 0xFFE) >> 1], 1);
        memory.io_writeu16(IME, 0xFFF0);
        assert_eq!(memory.ioram[(IME & 0xFFE) >> 1], 0);
        memory.io_writeu32(IME, 0xFFFFFFF1);
        assert_eq!(memory.ioram[(IME & 0xFFE) >> 1], 1);
        memory.io_writeu32(IME, 0xFFFFFFF0);
        assert_eq!(memory.ioram[(IME & 0xFFE) >> 1], 0);
    }

    #[rstest]
    #[case (IE, false)]
    #[case (DISPCNT, true)]
    fn test_regular_write_io(#[case] address: usize, #[case] test_u32: bool) {
        let test_write_value: u32 = 0xABCDEF12;

        let adjusted_address = (address & 0xFFE) >> 1;
        let mut memory = GBAMemory::new();

        // u8 tests
        memory.io_writeu8(address, test_write_value as u8);
        assert_eq!(memory.ioram[adjusted_address], test_write_value as u8 as u16);
        memory.io_writeu8(address + 1, (test_write_value >> 8) as u8);
        assert_eq!(memory.ioram[adjusted_address], test_write_value as u16);

        memory.ioram[adjusted_address] = 0;

        //u16 tests
        memory.io_writeu16(address, test_write_value as u16);
        assert_eq!(memory.ioram[adjusted_address], test_write_value as u16);
        memory.io_writeu16(address + 1, (test_write_value >> 8) as u16); // should floor with
                                                                         // halfword
        assert_eq!(memory.ioram[adjusted_address], (test_write_value >> 8) as u16);

        if test_u32 {
            //u32 tests
            memory.ioram[adjusted_address] = 0;
            memory.ioram[adjusted_address + 1] = 0;

            memory.io_writeu32(address, test_write_value);
            assert_eq!(memory.ioram[adjusted_address], test_write_value as u16);
            assert_eq!(memory.ioram[adjusted_address + 1], (test_write_value >> 16) as u16);

        }
    }
}
