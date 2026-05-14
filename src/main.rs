use crate::{cpu::CPU, ppu::PPU};
use minifb::{Window, WindowOptions};

mod cpu;
mod memory;
mod ppu;

const COLORS: [u32; 4] = [0x9BBC0F, 0x8BAC0F, 0x306230, 0x0F380F];

fn main() {
    let mut cpu = CPU::new();
    let mut ppu = PPU::new();

    let mut window = Window::new("CrabBoy", 160 * 3, 144 * 3, WindowOptions::default()).unwrap();
    //cpu.memory_bus.load_rom("roms/cpu_instrs.gb");
    // cpu.memory_bus.load_rom("roms/individual/01-special.gb");
    // cpu.memory_bus.load_rom("roms/individual/02-interrupts.gb");
    // cpu.memory_bus.load_rom("roms/individual/03-op sp,hl.gb");
    // cpu.memory_bus.load_rom("roms/individual/05-op rp.gb");
    // cpu.memory_bus.load_rom("roms/individual/06-ld r,r.gb");
    // cpu.memory_bus.load_rom("roms/individual/07-jr,jp,call,ret,rst.gb");
    // cpu.memory_bus.load_rom("roms/individual/08-misc instrs.gb");
    // cpu.memory_bus.load_rom("roms/individual/09-op r,r.gb");
    // cpu.memory_bus.load_rom("roms/individual/10-bit ops.gb");
    cpu.memory_bus.load_rom("roms/individual/11-op a,(hl).gb");

    loop {
        let cycles = cpu.step();
        ppu.step(&mut cpu.memory_bus, cycles);

        if ppu.frame_ready {
            let buffer: Vec<u32> = ppu
                .framebuffer
                .iter()
                .map(|&c| COLORS[c as usize])
                .collect();
            window.update_with_buffer(&buffer, 160, 144).unwrap();
            ppu.frame_ready = false;
        }
    }
}
