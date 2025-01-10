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

fn masked_io_load(region: &Vec<u16>, address: usize) -> Result<u16, MemoryError> {
    let def = get_io_read_definition(address)?;
    if !def.is_readable() {
        panic!();
    }
    if def.requires_special_handling() {
        match address {
            IF => {}
            _ => todo!()
        }
    }
    let data = io_load(region, address);
    Ok(match def.mask {
        BitMask::EIGHT(lower_mask) => {
            let upper_mask = get_io_read_definition(address + 1).map_or(0, |def| {
                let BitMask::EIGHT(mask) = def.mask else {
                    panic!("Upper mask isn't 8 bit")
                };
                mask
            });
            let full_mask: u16 = (upper_mask as u16) << 8 | lower_mask as u16;

            data & full_mask
        }
        BitMask::SIXTEEN(mask) => mask & data,
        BitMask::THIRTYTWO(mask) => {
            let shifted_mask = (mask >> (8 * address & 0b10)) as u16;
            data & shifted_mask
        }
    })
}

#[inline(always)]
fn io_store(region: &mut Vec<u16>, address: usize, value: u16) {
    let store_address = address >> 1;
    if store_address < region.len() {
        region[store_address] = value;
    }
}

fn masked_io_store(region: &mut Vec<u16>, address: usize, value: u16) -> Result<(), MemoryError>{
    let def = get_io_write_definition(address)?;
    if !def.is_readable() {
        panic!();
    }
    if def.requires_special_handling() {
        panic!()
    }
    let store_value = match def.mask {
        BitMask::EIGHT(lower_mask) => {
            let upper_mask = get_io_write_definition(address + 1).map_or(0, |def| {
                let BitMask::EIGHT(mask) = def.mask else {
                    panic!("Upper mask isn't 8 bit")
                };
                mask
            });
            let full_mask: u16 = (upper_mask as u16) << 8 | lower_mask as u16;

            value & full_mask
        }
        BitMask::SIXTEEN(mask) => value & mask,
        BitMask::THIRTYTWO(mask) => {
            let shifted_mask = (mask >> (8 * address & 0b10)) as u16;
            value & shifted_mask
        }
    };

    io_store(region, address, store_value);
    Ok(())
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

#[inline(always)]
fn get_io_write_definition(offset: usize) -> Result<IORegisterDefinition, MemoryError> {
    let Some(io_definition) = IO_REGISTER_DEFINITIONS[offset] else {
        return Err(MemoryError::NoIODefinition(offset));
    };

    if !io_definition.is_writeable() {
        return Err(MemoryError::ReadError(offset));
    }

    Ok(io_definition)
}

impl GBAMemory {
    pub(super) fn io_readu8(&self, address: usize) -> Result<u8, MemoryError> {
        let load_value = masked_io_load(&self.ioram, address & 0xFFE)?;
        Ok((load_value >> (8 * (address & 0b1))) as u8)

    }

    pub(super) fn io_readu16(&self, address: usize) -> Result<u16, MemoryError> {
        Ok(masked_io_load(&self.ioram, address)?)
    }

    pub(super) fn io_readu32(&self, address: usize) -> Result<u32, MemoryError> {
        let word_aligned_offset = address & 0xFFC;
        let lower = masked_io_load(&self.ioram, word_aligned_offset).unwrap_or(0) as u32;
        let upper = masked_io_load(&self.ioram, word_aligned_offset + 2).unwrap_or(0) as u32;

        Ok(upper << 16 | lower)
    }

    pub(super) fn io_writeu8(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        let mut current_value = io_load(&self.ioram, address & 0xFFE);
        current_value &= 0xFFFF << !(address & 0b1);
        masked_io_store(&mut self.ioram, address & 0xFFF, current_value | (value as u16) << (8 * (address & 0b1)))
    }

    pub(super) fn io_writeu16(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        masked_io_store(&mut self.ioram, address & 0xFFE, value)
    }

    pub(super) fn io_writeu32(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
        let offset = address & 0xFFC;
        let io_definition = get_io_write_definition(offset)?;

        match io_definition.mask {
            BitMask::THIRTYTWO(mask) => {
                if io_definition.requires_special_handling() {
                    todo!();
                }
                let store_value = mask & value;
                io_store(&mut self.ioram, offset, (store_value >> 16) as u16);
                io_store(&mut self.ioram, offset, (store_value & 0xFFFF) as u16);
            }
            _ => {
                self.io_writeu16(offset + 2, (value >> 16) as u16)?;
                self.io_writeu16(offset, (value & 0xFFFF) as u16)?;

                return Ok(());
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::memory::{
        io_handlers::{io_store, DISPCNT, HALTCNT, IE, IME, POSTFLG},
        memory::GBAMemory,
    };

    #[rstest]
    #[case(DISPCNT, 0xAB, 0xAB)]
    #[case(DISPCNT + 1, 0xFFAB, 0xFF)]
    #[case(IME, 0xFFFF, 0x1)]
    #[case(IME, 0xFFFE, 0x0)]
    #[case(POSTFLG, 0xFFFF, 0x01)]
    #[case(HALTCNT, 0xFFFF, 0x80)]
    #[case(IE, 0xFFFE, 0xFE)]
    #[case(IE + 1, 0xCDFE, 0x0D)]
    fn test_regular_read_io_8(
        #[case] address: usize,
        #[case] write_value: u16,
        #[case] expected_value: u8,
    ) {
        let mut memory = GBAMemory::new();
        io_store(&mut memory.ioram, address, write_value);
        assert_eq!(memory.io_readu8(address).unwrap(), expected_value);
    }

    #[rstest]
    #[case(DISPCNT, 0xAB, 0xAB)]
    #[case(DISPCNT, 0xFFFF, 0xFFFF)]
    #[case(IME, 0xFFFF, 0x1)]
    #[case(IME, 0xFFFE, 0x0)]
    #[case(POSTFLG, 0xFFFF, 0x8001)]
    #[case(HALTCNT, 0xFFFF, 0x0080)]
    #[case(IE, 0xFFFE, 0x3FFE)]
    fn test_regular_read_io_16(
        #[case] address: usize,
        #[case] write_value: u16,
        #[case] expected_value: u16,
    ) {
        let mut memory = GBAMemory::new();
        io_store(&mut memory.ioram, address, write_value);
        assert_eq!(memory.io_readu16(address).unwrap(), expected_value);
    }

    #[rstest]
    #[case(DISPCNT, 0xABCDEFAB, 0xEFAB)]
    #[case(DISPCNT, 0xFFFF, 0xFFFF)]
    #[case(IME, 0xFFFF, 0x1)]
    #[case(IME, 0xFFFE, 0x0)]
    #[case(POSTFLG, 0xFFFF, 0x8001)]
    #[case(HALTCNT, 0xFFFF, 0x8001)] // Word aligns
    #[case(IE, 0xABCDFFFE, 0x2BCD3FFE)]
    fn test_regular_read_io_32(
        #[case] address: usize,
        #[case] write_value: u32,
        #[case] expected_value: u32,
    ) {
        let mut memory = GBAMemory::new();
        io_store(&mut memory.ioram, address, (write_value & 0xFFFF) as u16);
        io_store(
            &mut memory.ioram,
            address + 2,
            ((write_value >> 16) & 0xFFFF) as u16,
        );
        assert_eq!(memory.io_readu32(address).unwrap(), expected_value);
    }

    #[rstest]
    #[case(DISPCNT, 0xFFFF, 0xFFFF)]
    fn test_regular_write_io16(
        #[case] address: usize,
        #[case] write_value: u16,
        #[case] expected_value: u16,
    ) {
        use crate::memory::io_handlers::io_load;

        let mut memory = GBAMemory::new();
        memory.io_writeu16(address, write_value).unwrap();

        assert_eq!(io_load(&memory.ioram, address), expected_value);
    }
}
