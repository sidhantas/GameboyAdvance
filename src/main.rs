use std::env;
pub mod memory;
mod hardware;
use memory::Memory;


fn main() {
    let args: Vec<String> = env::args().collect();
    let memory = Memory::initialize();
}
