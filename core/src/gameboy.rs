use crate::{apu::APU, cpu::CPU, ppu::PPU};

pub struct GameBoy {
    cpu: CPU,
    ppu: PPU,
    apu: APU,
    game_path: String,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            cpu: CPU::new(),
            ppu: PPU::new(),
            apu: APU::new(),
            game_path: String::new(),
        }
    }

    pub fn load_rom(&mut self, game_path: String) {
        self.game_path = game_path;

        self.cpu.memory_bus.load_rom(&self.get_game_path());
        self.cpu.memory_bus.load_ram(&self.get_save_path());
    }

    pub fn step(&mut self) -> bool {
        if self.cpu.memory_bus.ram_dirty {
            if self.cpu.memory_bus.save_ram(&self.get_save_path()) {
                // eprintln!(".sav succesfully written");
            } else {
                eprintln!(".sav not written");
            }
            self.cpu.memory_bus.ram_dirty = false;
        }

        let cycles = self.cpu.step();
        self.ppu.step(&mut self.cpu.memory_bus, cycles);
        self.apu.step(&mut self.cpu.memory_bus, cycles);

        self.ppu.frame_ready
    }

    pub fn framebuffer(&mut self) -> &[u8; 160 * 144] {
        self.ppu.frame_ready = false;

        &self.ppu.framebuffer
    }

    pub fn drain_audio(&mut self) -> Vec<f32> {
        self.apu.buffer.drain(..).collect()
    }

    pub fn set_button(&mut self, button: Buttons, is_pressed: bool) {
        use Buttons::*;

        match button {
            Up => self.cpu.memory_bus.joypad.up = is_pressed,
            Down => self.cpu.memory_bus.joypad.down = is_pressed,
            Left => self.cpu.memory_bus.joypad.left = is_pressed,
            Right => self.cpu.memory_bus.joypad.right = is_pressed,
            A => self.cpu.memory_bus.joypad.a_button = is_pressed,
            B => self.cpu.memory_bus.joypad.b_button = is_pressed,
            Select => self.cpu.memory_bus.joypad.select = is_pressed,
            Start => self.cpu.memory_bus.joypad.start = is_pressed,
        }
    }

    fn get_game_path(&self) -> String {
        self.game_path.clone()
    }

    fn get_save_path(&self) -> String {
        let save_path = self.game_path.replace(".gb", ".sav");
        save_path
    }
}

pub enum Buttons {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}
