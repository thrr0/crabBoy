# crabboy-core

Game Boy (DMG) emulation core written in Rust. Designed to be frontend-agnostic; embed it in any application: desktop, web (WASM), or mobile.

Passes all blargg `cpu_instrs` tests and dmg-acid2.

---

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
crabboy-core = "0.1.0"
```

Basic game loop:

```rust
use crabboy_core::gameboy::{GameBoy, Buttons};
use std::fs;

let mut gb = GameBoy::new();

// The frontend is responsible for reading files from disk.
let rom = fs::read("roms/tetris.gb").unwrap();
gb.load_rom(rom);

// Load save data if it exists (optional).
if let Ok(sav) = fs::read("roms/tetris.sav") {
    gb.load_save(sav);
}

loop {
    if gb.step() {
        // framebuffer: 160x144 array of color indices 0-3
        // index 0 = lightest, index 3 = darkest
        // map to actual colors in your frontend
        let framebuffer = gb.framebuffer();

        // audio: interleaved stereo f32 at 44100 Hz [L, R, L, R, ...]
        let audio = gb.drain_audio();

        // persist save data when modified
        if gb.is_save_dirty() {
            fs::write("roms/tetris.sav", gb.get_save_data()).ok();
            gb.clear_save_dirty();
        }
    }

    gb.set_button(Buttons::A, true);
    gb.set_button(Buttons::A, false);
}
```

---

## API

### `GameBoy::new() -> GameBoy`
Creates a new emulator instance.

### `GameBoy::load_rom(rom: Vec<u8>)`
Loads a ROM from a byte vector. The frontend is responsible for reading the file from disk (or any other source). The DMG boot ROM is embedded in the core and loaded automatically.

### `GameBoy::load_save(sav: Vec<u8>)`
Loads external RAM (save data) from a byte vector. Call this after `load_rom` if a save file exists. If the size does not match the cartridge RAM size, the save is rejected.

### `GameBoy::step() -> bool`
Advances the emulator by one CPU instruction. Returns `true` when a full frame (160x144 pixels) is ready to display.

### `GameBoy::framebuffer() -> &[u8; 160 * 144]`
Returns the current frame as color indices 0-3. Index 0 is the lightest color; index 3 is the darkest. The frontend is responsible for mapping these to actual colors. Marks the frame as consumed.

### `GameBoy::drain_audio() -> Vec<f32>`
Returns all pending audio samples as interleaved stereo f32 at 44100 Hz. Call every frame to avoid buffer overflow.

### `GameBoy::set_button(button: Buttons, is_pressed: bool)`
Sets the pressed state of a button. Call with `true` on press and `false` on release.

### `GameBoy::is_save_dirty() -> bool`
Returns `true` if external RAM was modified since the last `clear_save_dirty()` call. Use this to decide when to persist save data.

### `GameBoy::get_save_data() -> Vec<u8>`
Returns a copy of the current external RAM contents. Pass the result to your storage backend (filesystem, localStorage, etc.).

### `GameBoy::clear_save_dirty()`
Resets the dirty flag after save data has been persisted.

### `Buttons`
```rust
pub enum Buttons { Up, Down, Left, Right, A, B, Start, Select }
```

---

## Features

- Full SM83 CPU (all opcodes, interrupts, timer)
- PPU (background, window, sprites, palettes, OAM DMA, LCD STAT)
- APU (all 4 channels, envelope, length timer, DIV-APU sequencer, stereo output)
- MBC1, MBC2, MBC3, MBC5 with external RAM
- DMG boot ROM embedded at compile time

## License

MIT
