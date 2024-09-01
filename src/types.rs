pub type WORD = u32;
pub type HWORD = u16;
pub type BYTE = u8;

pub type REGISTER = u32;

pub trait ARMBITS {}
impl ARMBITS for WORD {}
impl ARMBITS for HWORD {}
impl ARMBITS for BYTE {}

pub type ARMByteCode = WORD;
pub type THUMBInstruction = HWORD;
