# CrabBoy

Game Boy (DMG) emulator written in Rust. Passes all blargg `cpu_instrs` tests. 

---

## Status

### CPU 
- Full SM83 instruction set (all opcodes + CB prefix)
- Registers, flags, fetch/decode/execute loop
- Interrupts (V-Blank, LCD STAT, Timer, Serial, Joypad), IME, HALT
- Timer (DIV, TIMA, TMA, TAC)

### PPU (in progress)
- [x] Background rendering (SCX/SCY scroll, LCDC bits 0/3/4/7)
- [x] Sprites 8x8 (OAM, X/Y flip, 10-per-line limit)
- [x] OAM DMA (0xFF46)
- [ ] Window layer (LCDC bits 5/6, WX/WY)
- [ ] Sprite palette (OBP0/OBP1, attribute bit 4)
- [ ] Sprite priority (attribute bit 7)
- [ ] 8x16 sprites (LCDC bit 2)
- [ ] Background palette (BGP)
- [ ] LCD STAT interrupt (0xFF41, LYC)

### Input
- [ ] Joypad (0xFF00)

### MBC
- [ ] MBC1
- [ ] MBC2
- [ ] MBC3 (+ RTC)
- [ ] MBC5

### Audio
- [ ] APU (channels 1-4)

---

## Project Structure

```
src/
  main.rs       — window, game loop, ROM loading
  cpu.rs        — SM83 CPU, registers, interrupts, timer
  memory.rs     — flat 64KB memory bus, OAM DMA
  ppu.rs        — PPU state machine, background, sprites
roms/
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
