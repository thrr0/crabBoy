# CrabBoy

A working Game Boy (DMG) emulator written in Rust. Passes all blargg `cpu_instrs` tests and dmg-acid2. Runs most commercial GB games including Tetris, Super Mario Land, Kirby's Dream Land, Zelda: Link's Awakening, Pokémon Yellow, and more.

---

## Features

- **CPU** — Full SM83 instruction set, interrupts (V-Blank, LCD STAT, Timer, Joypad), IME, HALT, timer
- **PPU** — Background/window/sprite rendering, scroll, palettes, OAM DMA, LCD STAT
- **APU** — All 4 channels (square wave, wave table, noise), envelope, length timer, DIV-APU sequencer, stereo routing
- **MBC** — MBC1, MBC2, MBC3, MBC5 with external RAM and save states (.sav)
- **Boot ROM** — DMG boot ROM support with Nintendo logo animation
- **Input** — Full joypad support

## Project Structure

```
src/
  main.rs       — window, game loop, ROM loading
  cpu.rs        — SM83 CPU, registers, interrupts, timer
  memory.rs     — memory bus, OAM DMA, joypad, MBC
  ppu.rs        — PPU state machine, background, sprites, window
  apu.rs        — APU, audio channels, sample generation
  hardware.rs   — hardware mode enum (DMG/GBC)
roms/
  boot.gb       — DMG boot ROM
  individual/   — blargg cpu_instrs test ROMs
  games/
```

## Running

```bash
cargo run
```

Change the `load_rom` call in `main.rs` to switch ROMs.

## References

| Resource | Purpose |
|----------|---------|
| [Pan Docs](https://gbdev.io/pandocs/) | Hardware reference |
| [GBCPUman](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf) | CPU manual |
| [Imran Nazar's blog](https://imrannazar.com/series/gameboy-emulation-in-javascript) | Step-by-step guide |
| [awesome-gbdev](https://github.com/gbdev/awesome-gbdev) | Curated resources |
| [blargg test ROMs](https://github.com/retrio/gb-test-roms) | CPU validation |
| [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) | PPU validation |
