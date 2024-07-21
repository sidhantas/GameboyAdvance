pub mod memory;
use memory::Memory;


fn main() {
    let mut memory = Memory::initialize();
    memory.write(0x00, 15).expect("Write Error");
    let written_val = memory.read(0x00).expect("Read Error");
    dbg!(written_val);
    memory.write(0x05000000, 15).expect("Write Error");
    
}
