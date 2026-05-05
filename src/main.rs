use crate::cpu::CPU;

mod cpu;

fn main() {
    let mut cpu = CPU::new();

    cpu.memory_bus.load_rom("roms/cpu_instrs.gb");
    // cpu.memory_bus.load_rom("roms/01-special.gb");
    // cpu.memory_bus.load_rom("roms/02-interrupts.gb");

    loop {
        cpu.step();
    }
}
