use crate::memory::MemoryBus;

const SCREEN_SIZE: usize = 160 * 144;

pub struct PPU {
    pub mode: u8, // (0-3)
    pub cycles: u32,
    pub framebuffer: [u8; SCREEN_SIZE],
    pub ly: u8,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            mode: 2, // oam scan
            cycles: 0,
            framebuffer: [0; SCREEN_SIZE],
            ly: 0,
        }
    }

    pub fn step(&mut self, memory_bus: &mut MemoryBus, cycles: u32) {
        self.cycles += cycles;
        match self.mode {
            2 => {
                if self.cycles > 80 {
                    self.mode = 3;
                    self.cycles = 0;
                }
            }
            3 => {
                if self.cycles > 172 {
                    self.mode = 0;
                    self.cycles = 0;
                }
            }
            0 => {
                if self.cycles > 204 {
                    self.cycles = 0;
                    self.ly += 1;
                    memory_bus.write(0xFF44, self.ly);
                    if self.ly < 144 {
                        self.mode = 2;
                    } else if self.ly == 144 {
                        self.mode = 1;
                    }
                }
            }
            1 => {
                if self.cycles > 4560 {
                    self.ly = 0;
                    memory_bus.write(0xFF44, self.ly);
                    self.mode = 2;
                    self.cycles = 0;
                }
            }
            _ => unreachable!(),
        }
    }
}
