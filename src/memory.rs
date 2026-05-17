use std::fs;

pub const MEMORY_BUS_SIZE: usize = 65536;

pub struct MemoryBus {
    pub memory: [u8; MEMORY_BUS_SIZE],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; MEMORY_BUS_SIZE],
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, mut value: u8) {
        if address == 0xFF40 {
            eprintln!("LCDC write: {:#04x}", value);
        }
        if address == 0xFF46 {
            let source = (value as u16) << 8;
            for i in 0..160u16 {
                let byte = self.memory[source as usize + i as usize];
                self.memory[0xFE00 + i as usize] = byte;
            }
        }
        //cpu_instr.gb
        // if address == 0xFF02 && value == 0x81 {
        //     if self.memory[0xFF01] == 0x0A {
        //         //line jump
        //         println!();
        //     } else {
        //         print!("{}", self.memory[0xFF01] as char);
        //     }
        // }
        if address == 0xFF04 {
            value = 0
        }
        self.memory[address as usize] = value;
    }

    pub fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).unwrap();
        self.memory[..bytes.len()].copy_from_slice(&bytes);
    }
}
