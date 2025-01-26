use super::memory::{GBAMemory, MemoryError};

pub const IO_BASE: usize = 0x4000000;
const DISPCNT: usize = 0x000;
const DISPSTAT: usize = 0x004;
pub const VCOUNT: usize = 0x006;
const BG0CNT: usize = 0x008;
const BG1CNT: usize = 0x00A;
const BG2CNT: usize = 0x00C;
const BG3CNT: usize = 0x00E;
const BG0HOFS: usize = 0x010;
const BG0VOFS: usize = 0x012;
const BG1HOFS: usize = 0x014;
const BG1VOFS: usize = 0x016;
const BG2HOFS: usize = 0x018;
const BG2VOFS: usize = 0x01A;
const BG3HOFS: usize = 0x01C;
const BG3VOFS: usize = 0x01E;
const DX: usize = 0x020;
const DMX: usize = 0x022;
const DY: usize = 0x024;
const DMY: usize = 0x026;
const BG2X_L: usize = 0x028;
const BG2X_H: usize = 0x02A;
const BG2Y_L: usize = 0x02C;
const BG2Y_H: usize = 0x02E;
const BG3_DX: usize = 0x030;
const BG3_DMX: usize = 0x032;
const BG3_DY: usize = 0x034;
const BG3_DMY: usize = 0x036;
const BG3X_L: usize = 0x038;
const BG3X_H: usize = 0x03A;
const BG3Y_L: usize = 0x03C;
const BG3Y_H: usize = 0x03E;
const WIN0H: usize = 0x040;
const WIN1H: usize = 0x042;
const WIN0V: usize = 0x044;
const WIN1V: usize = 0x046;
const WININ: usize = 0x048;
const WINOUT: usize = 0x04A;
const MOSAIC: usize = 0x04C;
const BLDCNT: usize = 0x050;
const BLDALPHA: usize = 0x052;
const BLDY: usize = 0x054;

const DMA0SAD: usize = 0x0B0;
const DMA0DAD: usize = 0x0B4;
const DMA0CNT_L: usize = 0x0B8;
const DMA0CNT_H: usize = 0x0BA;
const DMA1SAD: usize = 0x0BC;
const DMA1DAD: usize = 0x0C0;
const DMA1CNT_L: usize = 0x0C4;
const DMA1CNT_H: usize = 0x0C6;
const DMA2SAD: usize = 0x0C8;
const DMA2DAD: usize = 0x0CC;
const DMA2CNT_L: usize = 0x0D0;
const DMA2CNT_H: usize = 0x0D2;
const DMA3SAD: usize = 0x0D4;
const DMA3DAD: usize = 0x0D8;
const DMA3CNT_L: usize = 0x0DC;
const DMA3CNT_H: usize = 0x0DE;
const TM0CNT_L: usize = 0x100;
const TM0CNT_H: usize = 0x102;
const TM1CNT_L: usize = 0x104;
const TM1CNT_H: usize = 0x106;
const TM2CNT_L: usize = 0x108;
const TM2CNT_H: usize = 0x10A;
const TM3CNT_L: usize = 0x10C;
const TM3CNT_H: usize = 0x10E;
const KEYINPUT: usize = 0x130;
const KEYCNT: usize = 0x132;


const SOUNDBIAS: usize = 0x088;

const IME: usize = 0x208;
const IE: usize = 0x200;
const IF: usize = 0x202;
const WAITCNT: usize = 0x204;
const POSTFLG: usize = 0x300;
const HALTCNT: usize = 0x301;

#[derive(Copy, Clone)]
struct IORegisterDefinition {
    pub mask: BitMask,
    needs_special_handling: bool,
}

#[derive(Clone, Copy)]
enum BitMask {
    // first is read mask, second is write mask
    EIGHT(u8, u8),
    SIXTEEN(u16, u16),
    THIRTYTWO(u32, u32),
}

impl IORegisterDefinition {
    pub const fn new(mask: BitMask, needs_special_handling: bool) -> Self {
        Self { mask, needs_special_handling }
    }


    #[inline(always)]
    fn requires_special_handling(&self) -> bool {
        self.needs_special_handling
    }
}

const IO_REGISTER_DEFINITIONS: [Option<IORegisterDefinition>; 0x412] = {
    let mut definitions = [None; 0x412];

    definitions[DISPCNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[DISPSTAT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFF3F, 0xFF38), false));
    definitions[VCOUNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x00FF, 0x0000), false));
    definitions[BG0CNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[BG1CNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[BG2CNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[BG3CNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[BG0HOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[BG0VOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[BG1HOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[BG1VOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[BG2HOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[BG2VOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[BG3HOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[BG3VOFS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x01FF), false));
    definitions[DX] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[DMX] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[DY] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[DMY] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG2X_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG2X_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0FFF), false));
    definitions[BG2Y_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG2Y_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0FFF), false));
    definitions[BG3_DX] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG3_DMX] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG3_DY] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG3_DMY] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG3X_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG3X_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0FFF), false));
    definitions[BG3Y_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[BG3Y_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0FFF), false));
    definitions[WIN0H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[WIN1H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[WIN0V] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[WIN1V] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0xFFFF), false));
    definitions[WININ] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x3F3F, 0x3F3F), false));
    definitions[WINOUT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x3F3F, 0x3F3F), false));
    definitions[MOSAIC] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[BLDCNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x3FFF, 0x3FFF), false));
    definitions[BLDALPHA] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x3FFF, 0x3FFF), false));
    definitions[BLDY] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x001F, 0x001F), false));
    definitions[DMA0SAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x07FFFFFF), false));
    definitions[DMA0DAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x07FFFFFF), false));
    definitions[DMA0CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0, 0x3FFF), false));
    definitions[DMA0CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[DMA1SAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x07FFFFFF), false));
    definitions[DMA1DAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x07FFFFFF), false));
    definitions[DMA1CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0, 0x3FFF), false));
    definitions[DMA1CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[DMA2SAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x07FFFFFF), false));
    definitions[DMA2DAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x07FFFFFF), false));
    definitions[DMA2CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0, 0x3FFF), false));
    definitions[DMA2CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[DMA3SAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x0FFFFFFF), false));
    definitions[DMA3DAD] = Some(IORegisterDefinition::new(BitMask::THIRTYTWO(0, 0x0FFFFFFF), false));
    definitions[DMA3CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0, 0xFFFF), false));
    definitions[DMA3CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[TM0CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[TM0CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x00FB, 0x00FB), false));
    definitions[TM1CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[TM1CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x00FF, 0x00FF), false));
    definitions[TM2CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[TM2CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x00FF, 0x00FF), false));
    definitions[TM3CNT_L] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[TM3CNT_H] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x00FF, 0x00FF), false));
    definitions[KEYINPUT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x03FF, 0x03FF), false));
    definitions[KEYCNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xFFFF, 0xFFFF), false));
    definitions[SOUNDBIAS] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xC3FE, 0xC3FE), false));
    definitions[IME] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0001, 0x0001), false));
    definitions[IE] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x3FFF, 0x3FFF), false ));
    definitions[IF] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x3FFF, 0x3FFF), true));
    definitions[WAITCNT] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0xDFFF, 0xDFFF), false));
    definitions[POSTFLG] = Some(IORegisterDefinition::new(BitMask::EIGHT(0x01, 0x01), false));
    definitions[HALTCNT] = Some(IORegisterDefinition::new(BitMask::EIGHT(0x80, 0x80), false));
    definitions[0x110] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x112] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x114] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x410] = Some(IORegisterDefinition::new(BitMask::EIGHT(0x00, 0xFF), false));
    definitions[0x206] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x20A] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x20C] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x20E] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x210] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x212] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x214] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x216] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x218] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x21A] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x21C] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x21E] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x04E] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x056] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x058] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x05A] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x05C] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    definitions[0x05E] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
    let mut i = 0x0E0;
    while i != 0x100 {
        definitions[i] = Some(IORegisterDefinition::new(BitMask::SIXTEEN(0x0000, 0x0000), false));
        i += 2;
    }

    definitions
};

#[inline(always)]
pub fn io_load(region: &Vec<u16>, address: usize) -> u16 {
    *region.get(address >> 1).unwrap_or(&0)
}

fn masked_io_load(region: &Vec<u16>, address: usize) -> Result<u16, MemoryError> {
    let Ok(def) = get_io_definition(address) else {
        return Ok(0)
    };
    if def.requires_special_handling() {
        match address {
            IF => {}
            _ => todo!(),
        }
    }
    let data = io_load(region, address);
    Ok(match def.mask {
        BitMask::EIGHT(lower_mask, _) => {
            let upper_mask = get_io_definition(address + 1).map_or(0, |def| {
                let BitMask::EIGHT(mask, _) = def.mask else {
                    panic!("Upper mask isn't 8 bit")
                };
                mask
            });
            let full_mask: u16 = ((upper_mask as u16) << 8) | lower_mask as u16;

            data & full_mask
        }
        BitMask::SIXTEEN(mask, _) => mask & data,
        BitMask::THIRTYTWO(mask, _) => {
            let shifted_mask = (mask >> (8 * address & 0b10)) as u16;
            data & shifted_mask
        }
    })
}

#[inline(always)]
pub fn io_store(region: &mut Vec<u16>, address: usize, value: u16) {
    let store_address = address >> 1;
    if store_address < region.len() {
        region[store_address] = value;
    }
}

fn masked_io_store(region: &mut Vec<u16>, address: usize, value: u16) -> Result<(), MemoryError> {
    let mut value = value;
    let Ok(def) = get_io_definition(address) else {
        return Ok(())
    };
    if def.requires_special_handling() {
        match address {
            IF => {
                value = io_load(region, address) ^ value;
            }
            _ => return Err(MemoryError::NoIODefinition(address)),
        }
    }
    let store_value = match def.mask {
        BitMask::EIGHT(_, lower_mask) => {
            let upper_mask = get_io_definition(address + 1).map_or(0, |def| {
                let BitMask::EIGHT(_, mask) = def.mask else {
                    panic!("Upper mask isn't 8 bit")
                };
                mask
            });
            let full_mask: u16 = (upper_mask as u16) << 8 | lower_mask as u16;

            value & full_mask
        }
        BitMask::SIXTEEN(_, mask) => value & mask,
        BitMask::THIRTYTWO(_, mask) => {
            let shifted_mask = (mask >> (8 * address & 0b10)) as u16;
            value & shifted_mask
        }
    };

    io_store(region, address, store_value);
    Ok(())
}

#[inline(always)]
fn get_io_definition(offset: usize) -> Result<IORegisterDefinition, MemoryError> {
    let Some(io_definition) = IO_REGISTER_DEFINITIONS[offset] else {
        return Err(MemoryError::NoIODefinition(offset));
    };

    Ok(io_definition)
}

impl GBAMemory {
    pub(super) fn io_readu8(&self, address: usize) -> Result<u8, MemoryError> {
        let load_value = masked_io_load(&self.ioram, address & 0xFFE)?;
        Ok((load_value >> (8 * (address & 0b1))) as u8)
    }

    pub(super) fn io_readu16(&self, address: usize) -> Result<u16, MemoryError> {
        Ok(masked_io_load(&self.ioram, address & 0xFFE)?)
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
        masked_io_store(
            &mut self.ioram,
            address & 0xFFF,
            current_value | (value as u16) << (8 * (address & 0b1)),
        )
    }

    pub(super) fn io_writeu16(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        masked_io_store(&mut self.ioram, address & 0xFFE, value)
    }

    pub(super) fn io_writeu32(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
        let offset = address & 0xFFC;
        let Ok(io_definition) = get_io_definition(offset) else {
            return Ok(())
        };

        match io_definition.mask {
            BitMask::THIRTYTWO(_, mask) => {
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
        io_handlers::*,
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
    #[case(HALTCNT, 0xFFFF, 0x8001)]
    #[case(IE, 0xFFFE, 0x3FFE)]
    #[case(DISPSTAT, 0xFFFF, 0xFF3F)]
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
    #[case(DISPSTAT, 0xFFFF, 0xFF38)]
    fn test_regular_write_io16(
        #[case] address: usize,
        #[case] write_value: u16,
        #[case] expected_value: u16,
    ) {
        let mut memory = GBAMemory::new();
        memory.io_writeu16(address, write_value).unwrap();

        assert_eq!(io_load(&memory.ioram, address), expected_value);
    }

    #[rstest]
    #[case(0x3FFF, 0x3FFF, 0)]
    #[case(0x3FF0, 0x0FF0, 0x3000)]
    fn test_1s_to_if_should_clear_interupts(
        #[case] if_val: u16,
        #[case] write_val: u16,
        #[case] expected_val: u16,
    ) {
        let mut memory = GBAMemory::new();
        io_store(&mut memory.ioram, IF, if_val);
        memory.io_writeu16(IF, write_val).unwrap();

        assert_eq!(io_load(&memory.ioram, IF), expected_val);
    }
}
