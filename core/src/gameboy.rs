use crate::{apu::APU, cpu::CPU, ppu::PPU};

/// Main Game Boy (DMG) emulator. Encapsulates the CPU, PPU, and APU.
///
/// ```no_run
/// let mut gb = GameBoy::new();
/// gb.load_rom("roms/tetris.gb".to_string());
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

    /// Loads a ROM from the given path and its corresponding save file if present.
    /// The save file is expected at the same path with a `.sav` extension;
    /// for example, `roms/pokemon.gb` loads `roms/pokemon.sav`.
    /// If no save file exists, the emulator starts with an empty RAM state.
    pub fn load_rom(&mut self, game_path: String) {
        self.game_path = game_path;
        self.cpu.memory_bus.load_rom(&self.get_game_path());
        self.cpu.memory_bus.load_ram(&self.get_save_path());
    }

    /// Advances the emulator by one CPU instruction.
    /// Returns `true` when a full frame (160x144 pixels) has been rendered and is ready to display.
    /// When `true` is returned, call `framebuffer()` and `drain_audio()` to retrieve the output.
    /// If external RAM was modified since the last call, the save file is written to disk automatically.
    pub fn step(&mut self) -> bool {
        if self.cpu.memory_bus.ram_dirty {
            if !self.cpu.memory_bus.save_ram(&self.get_save_path()) {
                eprintln!(".sav not written");
            }
            self.cpu.memory_bus.ram_dirty = false;
        }
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

    fn get_game_path(&self) -> String {
        self.game_path.clone()
    }

    fn get_save_path(&self) -> String {
        self.game_path.replace(".gb", ".sav")
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
