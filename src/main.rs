use crate::cpu::CPU;

mod cpu;

fn main() {
    let mut cpu = CPU::new();

    cpu.memory_bus.load_rom("roms/cpu_instrs.gb");

    loop {
        cpu.step();
    }
}
