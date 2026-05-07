use crate::{cpu::CPU, ppu::PPU};

mod cpu;
mod memory;
mod ppu;

fn main() {
    let mut cpu = CPU::new();
    let mut ppu = PPU::new();

    cpu.memory_bus.load_rom("roms/cpu_instrs.gb");
    // cpu.memory_bus.load_rom("roms/01-special.gb");
    // cpu.memory_bus.load_rom("roms/02-interrupts.gb");

    loop {
        let cycles = cpu.step();
        ppu.step(&mut cpu.memory_bus, cycles);
    }
}
