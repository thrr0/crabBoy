use crate::{cpu::CPU, ppu::PPU};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

mod cpu;
mod hardware;
mod memory;
mod ppu;

const COLORS: [u32; 4] = [0x9BBC0F, 0x8BAC0F, 0x306230, 0x0F380F]; // DMG original const
// const COLORS: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000]; // Black & white
// const COLORS: [u32; 4] = [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F]; // GB Pocket
// const COLORS: [u32; 4] = [0xE8F8E0, 0xB0E018, 0x509000, 0x202850]; // GB Light
// const COLORS: [u32; 4] = [0xFFFFFF, 0x666666, 0x333333, 0x000000]; // High contrast

fn main() {
    let mut cpu = CPU::new();
    let mut ppu = PPU::new();

    let mut window = Window::new("CrabBoy", 160 * 3, 144 * 3, WindowOptions::default()).unwrap();

    let path = "roms/";

    let filename;
    //TEST ROMS
    // filename = "cpu_instrs.gb";
    // filename = "individual/01-special.gb";
    // filename = "individual/02-interrupts.gb";
    // filename = "individual/03-op sp,hl.gb";
    // filename = "individual/05-op rp.gb";
    // filename = "individual/06-ld r,r.gb";
    // filename = "individual/07-jr,jp,call,ret,rst.gb";
    // filename = "individual/08-misc instrs.gb";
    // filename = "individual/09-op r,r.gb";
    // filename = "individual/10-bit ops.gb";
    // filename = "individual/11-op a,(hl).gb";
    //
    //GAMES
    // filename = "games/tetris.gb";
    // filename = "games/dr mario.gb";
    // filename = "dmg-acid2.gb";
    filename = "games/super mario land.gb";
    // filename = "games/zelda.gb";
    // filename = "games/donkey kong 3.gb";
    // filename = "games/metroid 2.gb";
    // filename = "games/pokemon yellow.gb";
    // filename = "games/kirby.gb";

    let full_path = format!("{}{}", path, filename);
    let save_path = full_path.replace(".gb", ".sav");

    cpu.memory_bus.load_rom(&full_path);
    cpu.memory_bus.load_ram(&save_path);

    let frame_duration = Duration::from_nanos(16_666_667);
    // let frame_duration = Duration::from_nanos(8_333_332);
    // let frame_duration = Duration::from_nanos(512_222_223);
    //
    let mut last_frame = Instant::now();

    let mut count = 0;
    let mut was_boot_active = cpu.memory_bus.boot_rom_active;
    loop {
        let cycles = cpu.step();
        ppu.step(&mut cpu.memory_bus, cycles);

        if was_boot_active != cpu.memory_bus.boot_rom_active {
            eprintln!(
                "A={:#04x} F={:#04x} B={:#04x}",
                cpu.registers.a,
                cpu.registers.f.get_register(),
                cpu.registers.b
            );
            // eprintln!("boot bgp: {}", cpu.memory_bus.memory[0xFF47]);
            was_boot_active = false;
        }
        // if count >= 10_000_000 {
        //     eprintln!("ciclos: {}", count);
        //     eprintln!(
        //         "pc: {:#04x} op: {:#04x}",
        //         cpu.registers.pc,
        //         cpu.memory_bus.read(cpu.registers.pc)
        //     );
        //     eprintln!("loop B={:#04x}", cpu.registers.b);
        //     if cpu.registers.pc == 0x1E7E {
        //         eprintln!("0x1E7F = {:#04x}", cpu.memory_bus.read(0x1E7F));
        //         eprintln!("HL = {:#08x}", cpu.registers.get_hl());
        //     }
        //     if cpu.registers.pc == 0x1E80 {
        //         eprintln!("f_zero={}", cpu.registers.f.zero);
        //     }
        //     // if cpu.registers.pc == 338 {
        //     //     eprintln!(
        //     //         "pc + 1: {:#04x} a: {:#04x}",
        //     //         cpu.memory_bus.read(cpu.registers.pc + 1),
        //     //         cpu.registers.a
        //     //     )
        //     // }
        //     // eprintln!("0xFF40: {:#04x}", cpu.memory_bus.read(0xFF40));
        // }

        if count == 0 {
            eprintln!("bgp: {}", cpu.memory_bus.memory[0xFF47]);
        }
        if ppu.frame_ready {
            // count = 0;
            let buffer: Vec<u32> = ppu
                .framebuffer
                .iter()
                .map(|&c| COLORS[c as usize])
                .collect();
            window.update_with_buffer(&buffer, 160, 144).unwrap();
            ppu.frame_ready = false;

            if cpu.memory_bus.ram_dirty {
                cpu.memory_bus.save_ram(&save_path);
                cpu.memory_bus.ram_dirty = false;
            }
            let elapsed = last_frame.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
            last_frame = Instant::now();
        }

        cpu.memory_bus.joypad.up = window.is_key_down(Key::W);
        cpu.memory_bus.joypad.down = window.is_key_down(Key::S);
        cpu.memory_bus.joypad.left = window.is_key_down(Key::A);
        cpu.memory_bus.joypad.right = window.is_key_down(Key::D);
        cpu.memory_bus.joypad.a_button = window.is_key_down(Key::J);
        cpu.memory_bus.joypad.b_button = window.is_key_down(Key::K);
        cpu.memory_bus.joypad.select = window.is_key_down(Key::Comma);
        cpu.memory_bus.joypad.start = window.is_key_down(Key::Period);
        count += 1;
    }
}
