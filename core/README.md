crabboy-core

Game Boy (DMG) emulation core written in Rust. Designed to be frontend-agnostic; embed it in any application: desktop, web (WASM), or mobile.

Passes all blargg `cpu_instrs` tests and dmg-acid2.

---

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
crabboy-core = { git = "https://github.com/thrr0/crabboy" }
```

Basic game loop:

```rust
use crabboy_core::gameboy::{GameBoy, Buttons};

let mut gb = GameBoy::new();
gb.load_rom("roms/tetris.gb".to_string());

loop {
    if gb.step() {
        // framebuffer: 160x144 array of color indices 0-3
        // index 0 = lightest, index 3 = darkest
        let framebuffer = gb.framebuffer();

        // audio: interleaved stereo f32 at 44100 Hz [L, R, L, R, ...]
        let audio = gb.drain_audio();
    }

    gb.set_button(Buttons::A, true);
    gb.set_button(Buttons::A, false);
}
```

---

## API

### `GameBoy::new() -> GameBoy`
Creates a new emulator instance.

### `GameBoy::load_rom(path: String)`
Loads a ROM from disk. The save file is expected at the same path with a `.sav` extension; for example, `roms/pokemon.gb` loads `roms/pokemon.sav`. If no save file exists, the emulator starts fresh.

### `GameBoy::step() -> bool`
Advances the emulator by one CPU instruction. Returns `true` when a full frame is ready. When RAM is modified, the save file is written to disk automatically.

### `GameBoy::framebuffer() -> &[u8; 160 * 144]`
Returns the current frame as color indices 0-3. The frontend maps these to actual colors. Marks the frame as consumed.

### `GameBoy::drain_audio() -> Vec<f32>`
Returns all pending audio samples as interleaved stereo f32 at 44100 Hz. Call every frame to avoid buffer overflow.

### `GameBoy::set_button(button: Buttons, is_pressed: bool)`
Sets the state of a button. Call with `true` on press and `false` on release.

### `Buttons`
```rust
pub enum Buttons { Up, Down, Left, Right, A, B, Start, Select }
```

---

## Features

- Full SM83 CPU (all opcodes, interrupts, timer)
- PPU (background, window, sprites, palettes, OAM DMA, LCD STAT)
- APU (all 4 channels, envelope, length timer, DIV-APU sequencer)
- MBC1, MBC2, MBC3, MBC5 with external RAM and save states
- DMG boot ROM support

## License

MIT
