use crate::{cpu::IF_ADDRESS, memory::MemoryBus};

const SCREEN_SIZE: usize = 160 * 144;

pub struct PPU {
    pub mode: u8, // The PPU cycles through 4 modes each scanline (see step())
    pub cycles: u32,
    // Each pixel is stored as a color index 0-3 (2 bits per pixel = "2bpp").
    // The actual color (green shades) is resolved later when rendering to screen.
    pub framebuffer: [u8; SCREEN_SIZE],
    pub ly: u8, // Current scanline being drawn (0-153)
    pub frame_ready: bool,
    pub w_counter: u8,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            mode: 2, // Start in OAM scan
            cycles: 0,
            framebuffer: [0; SCREEN_SIZE],
            ly: 0,
            frame_ready: false,
            w_counter: 0,
        }
    }

    // The PPU is driven by the same clock as the CPU.
    // `cycles` is how many t-states the last CPU instruction consumed.
    // The PPU uses that to know how much time has passed and advance its state machine.
    pub fn step(&mut self, memory_bus: &mut MemoryBus, cycles: u32) {
        self.cycles += cycles;

        // LCDC (0xFF40) is the main LCD control register
        //   bit 7: LCD on/off,  if 0, PPU does nothing
        //   bit 6: window tile map select (0=0x9800, 1=0x9C00)
        //   bit 5: window enable
        //   bit 4: tile data addressing mode (0=0x9000 signed, 1=0x8000 unsigned)
        //   bit 3: background tile map select (0=0x9800, 1=0x9C00)
        //   bit 2: sprite size (0=8x8, 1=8x16)
        //   bit 1: sprites enable
        //   bit 0: background enable
        let lcdc = memory_bus.read(0xFF40);

        // The PPU draws one scanline at a time. For each of the 144 visible lines,
        // it goes through 3 modes in order:
        //   Mode 2 - OAM Scan  (80 cycles):  PPU looks at OAM to find sprites on this line
        //   Mode 3 - Drawing   (172 cycles): PPU renders the line into the framebuffer
        //   Mode 0 - H-Blank   (204 cycles): PPU is idle, CPU can access VRAM freely
        // After all 144 lines:
        //   Mode 1 - V-Blank   (4560 cycles): entire frame is done, CPU can access OAM/VRAM
        // Total per frame: (80+172+204)*144 + 4560 = 70224 cycles
        match self.mode {
            2 => {
                // OAM scan — sprite selection happens here in hardware,
                // but we do it directly in draw_sprites() for simplicity
                if self.cycles > 80 {
                    self.mode = 3;
                    self.cycles = 0;
                }
            }
            3 => {
                if self.cycles > 172 {
                    if lcdc & 0x80 != 0 {
                        self.draw_line(memory_bus, lcdc);
                    }
                    self.mode = 0;
                    self.cycles = 0;
                }
            }
            0 => {
                if self.cycles > 204 {
                    self.cycles = 0;
                    self.ly += 1;
                    // LY (0xFF44) is a read-only register the game reads to know
                    // which line is currently being drawn
                    memory_bus.write(0xFF44, self.ly);
                    if self.ly < 144 {
                        self.mode = 2;
                    } else if self.ly == 144 {
                        self.mode = 1;
                        // Request V-Blank interrupt so the game knows the frame is done.
                        // Games use this moment to update graphics, input, game logic, etc.
                        let if_flag = memory_bus.read(IF_ADDRESS);
                        memory_bus.write(IF_ADDRESS, if_flag | 0x01);
                    }
                }
            }
            1 => {
                if self.cycles > 456 {
                    self.cycles = 0;
                    self.ly += 1;
                    memory_bus.write(0xFF44, self.ly);
                    if self.ly == 153 {
                        self.ly = 0;
                        self.w_counter = 0;
                        memory_bus.write(0xFF44, self.ly);
                        self.mode = 2;
                        self.cycles = 0;
                        self.frame_ready = true;
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn draw_line(&mut self, memory_bus: &MemoryBus, lcdc: u8) {
        // eprintln!("draw_line ly={} lcdc={:#04x}", self.ly, lcdc);
        // LCDC bit 0: if 0, background is disabled — nothing to draw
        if lcdc & 0x01 == 0 {
            return;
        }

        // SCY/SCX (0xFF42/0xFF43): scroll offsets. The background is a 256x256 pixel map
        // that wraps around. SCX/SCY shift the viewport into that map.
        let scy = memory_bus.read(0xFF42);
        let scx = memory_bus.read(0xFF43);

        // Which row of tiles does this scanline fall in?
        // Each tile is 8px tall, so divide by 8.
        let tile_y = (self.ly.wrapping_add(scy)) / 8;

        // The screen is 160px wide = 20 tiles of 8px each
        for x in 0..20u8 {
            let tile_x: u8 = (x * 8 + scx) / 8;

            // The tile map is a 32x32 grid of tile IDs stored in VRAM.
            // It wraps around with % 32 to handle scroll going past the edge.
            let tile_map_index = (tile_y as u16 % 32) * 32 + (tile_x as u16 % 32);

            // LCDC bit 3: selects which tile map to read IDs from.
            // There are two maps: 0x9800 and 0x9C00. Games pick one via LCDC.
            let tile_id: u8 = if lcdc & 0x08 != 0 {
                memory_bus.read(0x9C00 + tile_map_index)
            } else {
                memory_bus.read(0x9800 + tile_map_index)
            };

            // LCDC bit 4: selects the tile data addressing mode.
            // Tile data (the actual pixel graphics) lives in VRAM starting at 0x8000 or 0x9000.
            // Each tile is 16 bytes (8 rows × 2 bytes per row).
            //
            // bit4=1 → 0x8000 base, tile_id is unsigned (0-255)
            // bit4=0 → 0x9000 base, tile_id is signed (-128 to 127)
            //   This means tile 0 is at 0x9000, tile 1 at 0x9010, tile -1 at 0x8FF0, etc.
            //
            // Within the tile, we only want the row that corresponds to the current scanline.
            // (ly + scy) % 8 gives which of the 8 rows within the tile we're on.
            // × 2 because each row is 2 bytes.
            let tile_data_address: u16 = if lcdc & 0x10 != 0 {
                0x8000 + tile_id as u16 * 16 + self.ly.overflowing_add(scy).0 as u16 % 8 * 2
            } else {
                0x9000u16.wrapping_add((tile_id as i8 as i16 as u16).wrapping_mul(16))
                    + self.ly.overflowing_add(scy).0 as u16 % 8 * 2
            };

            let low = memory_bus.read(tile_data_address);
            let high = memory_bus.read(tile_data_address + 1);

            // Each pixel is encoded across 2 bytes using 2bpp format:
            // bit N of `low` is the LSB of pixel N's color, bit N of `high` is the MSB.
            // So color = (high_bit << 1) | low_bit → value 0-3.
            // Pixel 0 is the leftmost, stored in bit 7.
            for pixel in 0..8 {
                let color = (high >> (7 - pixel) & 1) << 1 | (low >> (7 - pixel) & 1);
                self.framebuffer[self.ly as usize * 160 + x as usize * 8 + pixel as usize] = color;
            }
        }

        if lcdc & 0x20 != 0 {
            let (wx, wy) = (memory_bus.read(0xFF4B), memory_bus.read(0xFF4A));

            if self.ly >= wy {}
        }
        if lcdc & 0x02 != 0 {
            self.draw_sprites(memory_bus, lcdc);
        }
    }

    fn draw_sprites(&mut self, memory_bus: &MemoryBus, _lcdc: u8) {
        // OAM (Object Attribute Memory) at 0xFE00–0xFE9F stores data for up to 40 sprites.
        // Each sprite is 4 bytes: [Y position, X position, tile index, attributes]
        //
        // Y and X are offset by 16 and 8 respectively:
        //   OAM Y=16 → screen Y=0,  OAM Y=0 → sprite is 16px above the screen (hidden)
        //   OAM X=8  → screen X=0,  OAM X=0 → sprite is 8px left of the screen (hidden)
        // This allows sprites to enter the screen partially from any edge.

        let mut count = 0;
        for sprite_index in 0..40u16 {
            // Hardware limitation: only 10 sprites can appear on the same scanline.
            // If more than 10 intersect, the ones with higher OAM index are dropped.
            if count == 10 {
                break;
            }

            let base = 0xFE00 + sprite_index * 4;
            let sprite_y = memory_bus.read(base);
            let sprite_x = memory_bus.read(base + 1);
            let tile_id = memory_bus.read(base + 2);
            let attributes = memory_bus.read(base + 3);

            // Check if this sprite overlaps the current scanline.
            // With the Y offset, the sprite covers screen rows (sprite_y - 16) to (sprite_y - 9).
            if sprite_y <= self.ly + 16 && self.ly + 16 < sprite_y + 8 {
                count += 1;

                // Which row within the tile are we drawing?
                // (ly + 16 - sprite_y) gives the offset from the top of the sprite (0-7).
                let mut row = (self.ly + 16 - sprite_y) % 8;
                if attributes & 0x40 != 0 {
                    //y flip
                    row = 7 - row;
                }
                let addr = 0x8000 + tile_id as u16 * 16 + row as u16 * 2;
                let low = memory_bus.read(addr);
                let high = memory_bus.read(addr + 1);

                for pixel in 0..8u8 {
                    let color = if attributes & 0x20 != 0 {
                        (high >> (pixel) & 1) << 1 | (low >> (pixel) & 1)
                    } else {
                        (high >> (7 - pixel) & 1) << 1 | (low >> (7 - pixel) & 1)
                    };

                    // Color 0 is always transparent for sprites (unlike background where it's a real color)
                    if color != 0 {
                        let screen_x = sprite_x as usize + pixel as usize - 8;
                        self.framebuffer[self.ly as usize * 160 + screen_x] = color;
                    }
                }
            }
        }
    }
}
