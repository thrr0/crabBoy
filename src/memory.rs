use std::fs;

pub struct MemoryBus {
    memory: [u8; 65536],
}

impl MemoryBus {
    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, adress: u16, value: u8) {
        self.memory[adress as usize] = value;
    }

    fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).unwrap();
        self.memory[..bytes.len()].copy_from_slice(&bytes);
    }
}
