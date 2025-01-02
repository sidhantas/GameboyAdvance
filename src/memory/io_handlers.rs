use super::memory3::GBAMemory;

struct IORegisterOffsets;
impl IORegisterOffsets {
    pub const DISPCNT: usize = 0x000;
    pub const IE: usize = 0x200;
    pub const IF: usize = 0x202;
    pub const IME: usize = 0x208;
}

impl GBAMemory {
    pub(super) fn io_writeu16(&mut self, address: usize, hword: u16) {
        let write_value = match address {
            IORegisterOffsets::DISPCNT => hword,
            // TODO: Figure out if write8 should only affect the address selected or the entire
            // byte MGBA affects the entire byte, but this seems wrong
            IORegisterOffsets::IE => hword & 0x3FFF,
            IORegisterOffsets::IF => {
                let present_value = u16::from_le_bytes(self.io_ram[address..address + 2].try_into().unwrap());
                present_value & !hword
            }
            IORegisterOffsets::IME => hword & 1,
            _ => panic!("Unimplimented address {:#x}", address)
        };

        self.io_ram[address..address + 2].copy_from_slice(&write_value.to_le_bytes());
    }

    pub(super) fn io_writeu8(&mut self, address: usize, byte: u8) {
        let hword: u16 = (byte as u16) << (8 * address & 1);
        let present_value: u16 = self.io_ram[address ^ 1].into();

        self.io_writeu16(address & !1, hword | present_value << (8 * !(address & 1)))
    }

    pub(super) fn io_writeu32(&mut self, address: usize, word: u32) {
        self.io_writeu16(address, word as u16);
        self.io_writeu16(address + 2, (word >> 16) as u16);
    }

    pub(super) fn io_readu16(&self, address: usize) -> u16 {
        match address {
            IORegisterOffsets::DISPCNT => u16::from_le_bytes(self.io_ram[address..address + 2].try_into().unwrap()),
            _ => panic!("Unimplimented read address {:#x}", address)
        }
    }

    pub(super) fn io_readu8(self, address: usize) -> u8 {
        let hword = self.io_readu16(address & !1);

        return (hword >> (8 * address & 1)) as u8;
    }

    pub(super) fn io_readu32(self, address: usize) -> u32 {
        let hi = self.io_readu16(address) as u32;
        let low = self.io_readu16(address + 2) as u32;

        return low << 16 | hi;
    }
}
