use crate::utils::bits::Bits;

use super::memory::{CPUCallbacks, GBAMemory, MemoryError};

pub const IO_BASE: usize = 0x4000000;
pub const DISPCNT: usize = 0x000;
pub const DISPSTAT: usize = 0x004;
pub const VCOUNT: usize = 0x006;
pub const BG0CNT: usize = 0x008;
pub const BG1CNT: usize = 0x00A;
pub const BG2CNT: usize = 0x00C;
pub const BG3CNT: usize = 0x00E;
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
pub const BLDCNT: usize = 0x050;
pub const BLDALPHA: usize = 0x052;
pub const BLDY: usize = 0x054;

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
pub const TM0CNT_L: usize = 0x100;
pub const TM0CNT_H: usize = 0x102;
pub const TM1CNT_L: usize = 0x104;
pub const TM1CNT_H: usize = 0x106;
pub const TM2CNT_L: usize = 0x108;
pub const TM2CNT_H: usize = 0x10A;
pub const TM3CNT_L: usize = 0x10C;
pub const TM3CNT_H: usize = 0x10E;
pub const KEYINPUT: usize = 0x130;
const KEYCNT: usize = 0x132;

const SOUNDBIAS: usize = 0x088;

pub const IME: usize = 0x208;
pub const IE: usize = 0x200;
pub const IF: usize = 0x202;
const WAITCNT: usize = 0x204;
const POSTFLG: usize = 0x300;
const HALTCNT: usize = 0x301;

#[derive(Copy, Clone)]
struct IORegisterDefinition {
    pub mask: BitMask,
    needs_special_handling: bool,
    pub callback: Option<fn(&mut GBAMemory, u16, u16) -> ()>,
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
        Self {
            mask,
            needs_special_handling,
            callback: None,
        }
    }

    pub const fn with_callback(mut self, callback: fn(&mut GBAMemory, u16, u16) -> ()) -> Self {
        self.callback = Some(callback);
        self
    }

    #[inline(always)]
    fn requires_special_handling(&self) -> bool {
        self.needs_special_handling
    }
}

const IO_REGISTER_DEFINITIONS: [Option<IORegisterDefinition>; 0x412] = {
    let mut definitions = [None; 0x412];

    definitions[DISPCNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[DISPSTAT] = Some(
        IORegisterDefinition::new(BitMask::SIXTEEN(0xFF3F, 0xFF38), false)
            .with_callback(GBAMemory::dispstat_callback),
    );
    definitions[VCOUNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x00FF, 0x0000),
        false,
    ));
    definitions[BG0CNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[BG1CNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[BG2CNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[BG3CNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[BG0HOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[BG0VOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[BG1HOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[BG1VOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[BG2HOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[BG2VOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[BG3HOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[BG3VOFS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x01FF),
        false,
    ));
    definitions[DX] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[DMX] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[DY] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[DMY] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG2X_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG2X_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0FFF),
        false,
    ));
    definitions[BG2Y_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG2Y_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0FFF),
        false,
    ));
    definitions[BG3_DX] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG3_DMX] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG3_DY] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG3_DMY] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG3X_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG3X_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0FFF),
        false,
    ));
    definitions[BG3Y_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[BG3Y_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0FFF),
        false,
    ));
    definitions[WIN0H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[WIN1H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[WIN0V] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[WIN1V] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0xFFFF),
        false,
    ));
    definitions[WININ] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x3F3F, 0x3F3F),
        false,
    ));
    definitions[WINOUT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x3F3F, 0x3F3F),
        false,
    ));
    definitions[MOSAIC] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[BLDCNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x3FFF, 0x3FFF),
        false,
    ));
    definitions[BLDALPHA] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x3FFF, 0x3FFF),
        false,
    ));
    definitions[BLDY] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x001F, 0x001F),
        false,
    ));
    definitions[DMA0SAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x07FFFFFF),
        false,
    ));
    definitions[DMA0DAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x07FFFFFF),
        false,
    ));
    definitions[DMA0CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0, 0x3FFF),
        false,
    ));
    definitions[DMA0CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[DMA1SAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x07FFFFFF),
        false,
    ));
    definitions[DMA1DAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x07FFFFFF),
        false,
    ));
    definitions[DMA1CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0, 0x3FFF),
        false,
    ));
    definitions[DMA1CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[DMA2SAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x07FFFFFF),
        false,
    ));
    definitions[DMA2DAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x07FFFFFF),
        false,
    ));
    definitions[DMA2CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0, 0x3FFF),
        false,
    ));
    definitions[DMA2CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[DMA3SAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x0FFFFFFF),
        false,
    ));
    definitions[DMA3DAD] = Some(IORegisterDefinition::new(
        BitMask::THIRTYTWO(0, 0x0FFFFFFF),
        false,
    ));
    definitions[DMA3CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0, 0xFFFF),
        false,
    ));
    definitions[DMA3CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[TM0CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        true,
    ));
    definitions[TM0CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x00FB, 0x00FB),
        true,
    ));
    definitions[TM1CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        true,
    ));
    definitions[TM1CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x00FF, 0x00FF),
        true,
    ));
    definitions[TM2CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        true,
    ));
    definitions[TM2CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x00FF, 0x00FF),
        true,
    ));
    definitions[TM3CNT_L] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        true,
    ));
    definitions[TM3CNT_H] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x00FF, 0x00FF),
        true,
    ));
    definitions[KEYINPUT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x03FF, 0x0000),
        true,
    ));
    definitions[KEYCNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xFFFF, 0xFFFF),
        false,
    ));
    definitions[SOUNDBIAS] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xC3FE, 0xC3FE),
        false,
    ));
    definitions[IME] = Some(
        IORegisterDefinition::new(BitMask::SIXTEEN(0x0001, 0x0001), false)
            .with_callback(GBAMemory::check_interrupts),
    );
    definitions[IE] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x3FFF, 0x3FFF),
        false,
    ));
    definitions[IF] = Some(
        IORegisterDefinition::new(BitMask::SIXTEEN(0x3FFF, 0x3FFF), true)
            .with_callback(GBAMemory::check_interrupts),
    );
    definitions[WAITCNT] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0xDFFF, 0xDFFF),
        false,
    ));
    definitions[POSTFLG] = Some(IORegisterDefinition::new(BitMask::EIGHT(0x01, 0x01), false));
    definitions[HALTCNT] = Some(
        IORegisterDefinition::new(BitMask::EIGHT(0x80, 0x80), false).with_callback(GBAMemory::halt),
    );
    definitions[0x110] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x112] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x114] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x410] = Some(IORegisterDefinition::new(BitMask::EIGHT(0x00, 0xFF), false));
    definitions[0x206] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x20A] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x20C] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x20E] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x210] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x212] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x214] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x216] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x218] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x21A] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x21C] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x21E] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x04E] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x056] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x058] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x05A] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x05C] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    definitions[0x05E] = Some(IORegisterDefinition::new(
        BitMask::SIXTEEN(0x0000, 0x0000),
        false,
    ));
    let mut i = 0x0E0;
    while i != 0x100 {
        definitions[i] = Some(IORegisterDefinition::new(
            BitMask::SIXTEEN(0x0000, 0x0000),
            false,
        ));
        i += 2;
    }

    definitions
};

#[inline(always)]
fn get_io_definition(offset: usize) -> Result<IORegisterDefinition, MemoryError> {
    if let Some(io_definition) = IO_REGISTER_DEFINITIONS[offset] {
        return Ok(io_definition);
    };
    if let Some(io_definition) = IO_REGISTER_DEFINITIONS[offset & 0xFFE] {
        return Ok(io_definition);
    };
    return Err(MemoryError::NoIODefinition(offset));
}

impl GBAMemory {
    pub fn ppu_io_write(&mut self, address: usize, value: u16) {
        let old_value = self.ioram[(address & 0xFFF) >> 1];
        self.ioram[(address & 0xFFF) >> 1] = value;
        let Ok(def) = get_io_definition(address & 0xFFF) else {
            return;
        };

        if let Some(callback) = def.callback {
            callback(self, old_value, value);
        }
    }
    pub(super) fn io_readu8(&self, address: usize) -> Result<u8, MemoryError> {
        let load_value = self.masked_io_load(address & 0xFFE)?;
        Ok((load_value >> (8 * (address & 0b1))) as u8)
    }

    pub(super) fn io_readu16(&self, address: usize) -> Result<u16, MemoryError> {
        Ok(self.masked_io_load(address & 0xFFE)?)
    }

    pub(super) fn io_readu32(&self, address: usize) -> Result<u32, MemoryError> {
        let word_aligned_offset = address & 0xFFC;
        let lower = self.masked_io_load(word_aligned_offset).unwrap_or(0) as u32;
        let upper = self.masked_io_load(word_aligned_offset + 2).unwrap_or(0) as u32;

        Ok(upper << 16 | lower)
    }

    pub(super) fn io_writeu8(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        let mut current_value = self.io_load(address & 0xFFE);
        current_value &= 0xFF << (8 * !(address & 0b1));
        current_value |= (value as u16) << (8 * (address & 0b1));
        self.masked_io_store(address & 0xFFF, current_value)
    }

    pub(super) fn io_writeu16(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        self.masked_io_store(address & 0xFFE, value)
    }

    pub(super) fn io_writeu32(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
        let offset = address & 0xFFC;
        let Ok(io_definition) = get_io_definition(offset) else {
            return Ok(());
        };

        match io_definition.mask {
            BitMask::THIRTYTWO(_, mask) => {
                if io_definition.requires_special_handling() {
                    todo!();
                }
                let store_value = mask & value;
                self.io_store(offset, (store_value >> 16) as u16);
                self.io_store(offset, (store_value & 0xFFFF) as u16);
            }
            _ => {
                self.io_writeu16(offset + 2, (value >> 16) as u16)?;
                self.io_writeu16(offset, (value & 0xFFFF) as u16)?;

                return Ok(());
            }
        };

        Ok(())
    }

    #[inline(always)]
    pub fn io_load(&self, address: usize) -> u16 {
        *self.ioram.get(address >> 1).unwrap_or(&0)
    }

    fn masked_io_load(&self, address: usize) -> Result<u16, MemoryError> {
        let Ok(def) = get_io_definition(address) else {
            return Ok(0);
        };
        if def.requires_special_handling() {
            match address {
                IF => {}
                KEYINPUT => {}
                TM0CNT_L | TM1CNT_L | TM2CNT_L | TM3CNT_L => {
                    let timer = (address & 0xF) / 4;
                    return Ok(self.timers.as_ref().unwrap().read_timer(timer) as u16);
                }
                TM0CNT_H | TM1CNT_H | TM2CNT_H | TM3CNT_H => {}
                _ => todo!(),
            }
        }
        let data = self.io_load(address);
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
    pub(super) fn io_store(&mut self, address: usize, value: u16) {
        let store_address = address >> 1;
        if store_address < self.ioram.len() {
            self.ioram[store_address] = value;
        }
    }

    fn masked_io_store(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        let mut value = value;
        let Ok(def) = get_io_definition(address) else {
            return Ok(());
        };
        if def.requires_special_handling() {
            match address {
                IF => {
                    value = self.io_load(address) & !value;
                }
                KEYINPUT => return Ok(()),
                TM0CNT_L | TM1CNT_L | TM2CNT_L | TM3CNT_L => {}
                TM0CNT_H | TM1CNT_H | TM2CNT_H | TM3CNT_H => {
                    let old_value = self.io_load(address);
                    let timer_enabled = !old_value & value;
                    let timer_num = ((address - 0x2) & 0xF) / 4;
                    if timer_enabled.bit_is_set(7) {
                        // The timer enabled bit has been set to 1
                        if let Some(mut timers) = self.timers.take() {
                            timers.reload_timer(timer_num, self.io_load(address - 0x2));
                            self.timers.replace(timers);
                        }
                    }
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
        let old_value = self.io_load(address);
        self.io_store(address, store_value);
        if let Some(callback) = def.callback {
            callback(self, old_value, store_value);
        }

        Ok(())
    }

    pub(super) fn halt(&mut self, _old_value: u16, value: u16) {
        if value & 0x8000 > 0 {
            self.cpu_commands.push(CPUCallbacks::Stop);
            return;
        }
        self.cpu_commands.push(CPUCallbacks::Halt);
    }

    pub(super) fn check_interrupts(&mut self, _old_value: u16, _value: u16) {
        if self.io_load(IME) == 0 {
            return;
        }
        if self.io_load(IF) & self.io_load(IE) > 0 {
            self.cpu_commands.push(CPUCallbacks::RaiseIrq);
        }
    }

    pub(super) fn dispstat_callback(&mut self, old_value: u16, value: u16) {
        let triggered_flags = (!old_value & value) & 0x7;
        if triggered_flags == 0 {
            return;
        }
        let toggled_interrupts = (value >> 3) & 0x7;
        let available_interrupts = triggered_flags & toggled_interrupts;
        let mut current_if = self.io_load(IF);
        current_if &= !0x7;
        current_if |= available_interrupts;
        self.io_store(IF, current_if);
        self.check_interrupts(old_value, value);
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        gba::GBA,
        memory::{io_handlers::*, memory::GBAMemory},
        utils::bits::Bits,
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
        memory.io_store(address, write_value);
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
    #[case(KEYINPUT, 0x3FF, 0x3FF)]
    fn test_regular_read_io_16(
        #[case] address: usize,
        #[case] write_value: u16,
        #[case] expected_value: u16,
    ) {
        let mut memory = GBAMemory::new();
        memory.io_store(address, write_value);
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
        memory.io_store(address, (write_value & 0xFFFF) as u16);
        memory.io_store(address + 2, ((write_value >> 16) & 0xFFFF) as u16);
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

        assert_eq!(memory.io_load(address), expected_value);
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
        memory.io_store(IF, if_val);
        memory.io_writeu16(IF, write_val).unwrap();

        assert_eq!(memory.io_load(IF), expected_val);
    }

    #[rstest]
    fn test_write_io8() {
        let mut memory = GBAMemory::new();
        memory.io_store(SOUNDBIAS, 0x200);
        memory.io_writeu8(SOUNDBIAS + 1, 0x42).unwrap();

        assert_eq!(memory.io_load(SOUNDBIAS), 0x4200);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn can_read_timers(#[case] timer_num: usize) {
        let mut gba = GBA::new_no_bios();
        let mut tmcnt = gba.memory.io_load(0x102 + 0x4 * timer_num);
        tmcnt.set_bit(7); // enables timer
        gba.memory.ppu_io_write(0x102 + 0x4 * timer_num, tmcnt);

        for _ in 0..5 {
            gba.step();
        }

        assert_eq!(gba.memory.io_readu16(0x100 + 0x4 * timer_num).unwrap(), 5);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn timers_reload_on_timer_enable(#[case] timer_num: usize) {
        let mut gba = GBA::new_no_bios();
        gba.memory.io_writeu16(0x100 + 0x4 * timer_num, 0xFF);
        let mut tmcnt = gba.memory.io_load(0x102 + 0x4 * timer_num);
        tmcnt.set_bit(7); // enables timer
        gba.memory
            .io_writeu16(0x102 + 0x4 * timer_num, tmcnt)
            .unwrap();

        assert_eq!(
            gba.memory.io_readu16(0x100 + 0x4 * timer_num).unwrap(),
            0xFF
        );
    }
}
