use crate::{apu::APU, cpu::CPU, ppu::PPU};

/// Main Game Boy (DMG) emulator. Encapsulates the CPU, PPU, and APU.
///
/// ```no_run
///use crabboy_core::gameboy::{GameBoy, Buttons};
/// let mut gb = GameBoy::new();
/// gb.load_rom(std::fs::read("roms/tetris.gb").unwrap());
///
/// loop {
///     if gb.step() {
///         let framebuffer = gb.framebuffer(); // 160x144 color indices 0-3
///         let audio = gb.drain_audio();       // interleaved stereo f32 samples
///     }
///     gb.set_button(Buttons::A, true);
/// }
/// ```
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

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.cpu.memory_bus.load_rom(rom);
    }

    pub fn load_save(&mut self, sav: Vec<u8>) {
        self.cpu.memory_bus.load_ram(sav);
    }

    pub fn is_save_dirty(&self) -> bool {
        self.cpu.memory_bus.ram_dirty
    }

    pub fn clear_save_dirty(&mut self) {
        self.cpu.memory_bus.ram_dirty = false;
    }

    pub fn get_save_data(&mut self) -> Vec<u8> {
        self.cpu.memory_bus.save_ram()
    }

    /// Advances the emulator by one CPU instruction.
    /// Returns `true` when a full frame (160x144 pixels) has been rendered and is ready to display.
    /// When `true` is returned, call `framebuffer()` and `drain_audio()` to retrieve the output.
    pub fn step(&mut self) -> bool {
        let cycles = self.cpu.step();
        self.ppu.step(&mut self.cpu.memory_bus, cycles);
        self.apu.step(&mut self.cpu.memory_bus, cycles);
        self.ppu.frame_ready
    }

    /// Returns the current framebuffer as a 160x144 array of color indices (0-3).
    /// Index 0 is the lightest color; index 3 is the darkest.
    /// The frontend is responsible for mapping these indices to actual colors.
    /// Marks the frame as consumed; subsequent calls return the same frame until the next one is ready.
    pub fn framebuffer(&mut self) -> &[u8; 160 * 144] {
        self.ppu.frame_ready = false;
        &self.ppu.framebuffer
    }

    /// Drains and returns all pending audio samples as interleaved stereo f32 values.
    /// Format is [left, right, left, right, ...] at 44100 Hz.
    /// Call this every frame and feed the result to your audio backend.
    /// If not called regularly, the internal buffer will grow unbounded.
    pub fn drain_audio(&mut self) -> Vec<f32> {
        self.apu.buffer.drain(..).collect()
    }

    /// Sets the pressed state of a Game Boy button.
    /// Call with `true` when the button is pressed and `false` when released.
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
