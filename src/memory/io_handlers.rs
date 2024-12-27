use super::memory::Memory;

struct IORegisterOffsets;
impl IORegisterOffsets {
    pub const DISPCNT: usize = 0x000;
    pub const IE: usize = 0x200;
    pub const IF: usize = 0x202;
    pub const IME: usize = 0x208;
}

impl Memory {
    pub(super) fn io_writeu16(&mut self, address: usize, hword: u16) {
        assert!(address % 2 == 0);
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
}
