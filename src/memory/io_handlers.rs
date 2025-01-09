use super::memory::{GBAMemory, MemoryError};

const DISPCNT: usize = 0x000;
const IME: usize = 0x208;
const IE: usize = 0x200;
const IF: usize = 0x202;
const WAITCNT: usize = 0x204;
const POSTFLG: usize = 0x300;
const HALTCNT: usize = 0x301;

#[derive(Copy, Clone)]
struct IORegisterDefinition {
    pub mask: BitMask,
    flags: u8,
}

#[derive(Clone, Copy)]
enum BitMask {
    EIGHT(u8),
    SIXTEEN(u16),
    THIRTYTWO(u32),
}

const R: u8 = 1;
const W: u8 = 1 << 1;
const SPECIAL_HANDLING: u8 = 1 << 2;

impl IORegisterDefinition {
    pub const fn new(mask: BitMask, flags: u8) -> Self {
        Self { mask, flags }
    }

    #[inline(always)]
    fn is_readable(&self) -> bool {
        self.flags & R > 0
    }

    #[inline(always)]
    fn is_writeable(&self) -> bool {
        self.flags & W > 0
    }

    #[inline(always)]
    fn requires_special_handling(&self) -> bool {
        self.flags & SPECIAL_HANDLING > 0
    }
}

const IO_REGISTER_DEFINITIONS: [Option<IORegisterDefinition>; 0x412] = {
    let mut definitions = [None; 0x412];

    definitions[DISPCNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF), R | W));
    definitions[IME] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0001), R | W));
    definitions[IE] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x3FFF), R | W));
    definitions[IF] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x3FFF),
        R | W | SPECIAL_HANDLING,
    ));
    definitions[WAITCNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF), R | W));
    definitions[POSTFLG] = Some(IORegisterDefinition::new(BitMask::EIGHT(0x01), R | W));
    definitions[HALTCNT] = Some(IORegisterDefinition::new(BitMask::EIGHT(0x80), R | W));
    definitions[0x410] = Some(IORegisterDefinition::new(BitMask::EIGHT(0xFF), W));

    //bit_mask[DISPCNT] = Some(0xFFFF);
    //bit_mask[IME] = Some(0x1);
    //bit_mask[IE] = Some(0x3FFF);
    //bit_mask[WAITCNT] = Some(0xFFFF);
    //bit_mask[WAITCNT + 2] = Some(0);
    //bit_mask[POSTFLG] = Some(0x8001);
    //bit_mask[HALTCNT] = None;

    definitions
};

#[inline(always)]
fn io_load(region: &Vec<u16>, address: usize) -> u16 {
    *region.get(address >> 1).unwrap_or(&0)
}

#[inline(always)]
fn io_store(region: &mut Vec<u16>, address: usize, value: u16) {
    let store_address = address >> 1;
    if store_address < region.len() {
        region[store_address] = value;
    }
}

#[inline(always)]
fn get_io_read_definition(offset: usize) -> Result<IORegisterDefinition, MemoryError> {
    let Some(io_definition) = IO_REGISTER_DEFINITIONS[offset] else {
        return Err(MemoryError::NoIODefinition(offset));
    };

    if !io_definition.is_readable() {
        return Err(MemoryError::ReadError(offset));
    }

    Ok(io_definition)
}

impl GBAMemory {
    pub(super) fn io_readu8(&self, address: usize) -> Result<u8, MemoryError> {
        let offset = address & 0xFFE;
        let io_definition = get_io_read_definition(offset)?;

        if !io_definition.requires_special_handling() {
            return Ok(match io_definition.mask {
                BitMask::EIGHT(mask) => {
                    let data = (io_load(&self.ioram, offset) >> (8 * address & 0b1)) as u8;
                    data & mask
                }
                BitMask::SIXTEEN(mask) => {
                    let data = io_load(&self.ioram, offset) & mask;
                    (data >> (8 * address & 0b1)) as u8
                }
                BitMask::THIRTYTWO(mask) => {
                    let word_aligned_address = offset & !0x3;
                    let data = (io_load(&self.ioram, word_aligned_address + 2) as u32) << 16 
                        | io_load(&self.ioram, word_aligned_address) as u32;
                    let data = data & mask;

                    (data >> (8 * address & 0b11)) as u8
                }
            });
        }
        todo!()
    }

    pub(super) fn io_readu16(&self, address: usize) -> Result<u16, MemoryError> {
        let offset = address & 0xFFF;
        let io_definition = get_io_read_definition(offset)?;

        if !io_definition.requires_special_handling() {
            return Ok(match io_definition.mask {
                BitMask::EIGHT(lower_mask) => {
                    let upper_mask = if let Ok(def) = get_io_read_definition(offset + 1) {
                        if let BitMask::EIGHT(mask) = def.mask {
                            mask
                        } else {
                            panic!("Upper mask isn't 8 bit")
                        }
                    } else {
                        0 as u8
                    };

                    let data = io_load(&self.ioram, offset);
                    let full_mask: u16 =
                        Into::<u16>::into(upper_mask) << 8 | Into::<u16>::into(lower_mask);

                    data & full_mask
                }
                BitMask::SIXTEEN(mask) => {
                    let data = io_load(&self.ioram, offset & !1);
                    return Ok(data & mask);
                }
                BitMask::THIRTYTWO(mask) => {
                    let word_aligned_address = offset & !0x3;
                    let data = (io_load(&self.ioram, word_aligned_address + 2)as u32) << 16 
                        | io_load(&self.ioram, word_aligned_address) as u32;
                    let data = data & mask;
                    (data >> (8 * address & 0b10)) as u16
                }
            });
        }
        todo!()
    }

    pub(super) fn io_readu32(&self, address: usize) -> Result<u32, MemoryError> {
        let offset = address & !0x3;
        let io_definition = get_io_read_definition(offset)?;

        if !io_definition.requires_special_handling() {
            return Ok(match io_definition.mask {
                BitMask::THIRTYTWO(mask) => {
                    let data = (io_load(&self.ioram, offset + 2)as u32) << 16 
                        | io_load(&self.ioram, offset) as u32;

                    data & mask
                }
                _ => {
                    (self.io_readu16(address + 2)?as u32) << 16  | self.io_readu16(address)? as u32
                }
            });
        }
        todo!()
    }

    pub(super) fn io_writeu8(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        let offset = address & 0xFFE;
        let mut current_value = io_load(&self.ioram, offset);
        current_value &= (!(0xFF << 8 * (address & 1))) as u16;

        let new_hword = (value as u16) << (8 * (address & 1));
        let new_hword = new_hword | current_value;

        let store_value = match offset {
            DISPCNT => new_hword,
            IME => new_hword & 0x1,
            IE => new_hword & 0x3FFF,
            _ => return Err(MemoryError::WriteError(address, value as u32)),
        };
        io_store(&mut self.ioram, offset, store_value);

        Ok(())
    }

    pub(super) fn io_writeu16(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        let offset = address & 0xFFE;
        let store_value = match offset {
            DISPCNT | 0x02 => value,
            IME => value & 0x1,
            IE => value & 0x3FFF,
            0x206 => return Ok(()),
            _ => return Err(MemoryError::WriteError(address, value as u32)),
        };
        io_store(&mut self.ioram, offset, store_value);
        Ok(())
    }

    pub(super) fn io_writeu32(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
        let address = address & !0x3;
        let upper: u16 = (value >> 16) as u16;
        let lower: u16 = (value & 0xFFFF) as u16;
        let offset = address & 0xFFC;
        match offset {
            IME => {
                io_store(&mut self.ioram, address, value as u16 & 0x1);
                return Ok(());
            }
            _ => {}
        };

        self.io_writeu16(address, lower)?;
        self.io_writeu16(address + 2, upper)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::memory::{
        io_handlers::{io_store, DISPCNT, IME},
        memory::GBAMemory,
    };


    #[rstest]
    #[case(DISPCNT, 0xAB, 0xAB)]
    #[case(DISPCNT, 0xFFFF, 0xFFFF)]
    #[case(IME, 0xFFFF, 0x1)]
    #[case(IME, 0xFFFE, 0x0)]
    fn test_regular_read_io_16(#[case] address: usize, #[case] write_value: u16, #[case] expected_value: u16) {

        let mut memory = GBAMemory::new();
        io_store(&mut memory.ioram, address, write_value);
        assert_eq!(memory.io_readu16(address).unwrap(), expected_value);
    }
    
}
