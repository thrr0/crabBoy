use std::io::LineWriter;

use crate::{apu::APU, cpu::CPU, ppu::PPU};

pub struct GameBoy {
    cpu: CPU,
    ppu: PPU,
    apu: APU,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            cpu: CPU::new(),
            ppu: PPU::new(),
            apu: APU::new(),
        }
    }
}
