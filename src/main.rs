use crate::{cpu::CPU, ppu::PPU};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

mod cpu;
mod hardware;
mod memory;
mod ppu;

// const COLORS: [u32; 4] = [0x9BBC0F, 0x8BAC0F, 0x306230, 0x0F380F]; // DMG original const
// const COLORS: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000]; // Black & white
const COLORS: [u32; 4] = [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F]; // GB Pocket
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
    // filename = "games/dr mario.gb";
    // filename = "dmg-acid2.gb";
    // filename = "games/super mario land.gb";
    filename = "games/zelda.gb";
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

    loop {
        cpu.memory_bus.joypad.up = window.is_key_down(Key::W);
        cpu.memory_bus.joypad.down = window.is_key_down(Key::S);
        cpu.memory_bus.joypad.left = window.is_key_down(Key::A);
        cpu.memory_bus.joypad.right = window.is_key_down(Key::D);
        cpu.memory_bus.joypad.a_button = window.is_key_down(Key::J);
        cpu.memory_bus.joypad.b_button = window.is_key_down(Key::K);
        cpu.memory_bus.joypad.select = window.is_key_down(Key::Comma);
        cpu.memory_bus.joypad.start = window.is_key_down(Key::Period);

        if cpu.memory_bus.ram_dirty {
            if cpu.memory_bus.save_ram(&save_path) {
                eprintln!(".sav succesfully written");
            } else {
                eprintln!(".sav not written");
            }
            cpu.memory_bus.ram_dirty = false;
        }

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

            let elapsed = last_frame.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
            last_frame = Instant::now();
        }
    }
}
