use crate::{cpu::IF_ADDRESS, memory::MemoryBus};

const SCREEN_SIZE: usize = 160 * 144;

pub struct PPU {
    pub mode: u8, // (0-3)
    pub cycles: u32,
    pub framebuffer: [u8; SCREEN_SIZE],
    pub ly: u8, //current line index
    pub frame_ready: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            mode: 2, // oam scan
            cycles: 0,
            framebuffer: [0; SCREEN_SIZE],
            ly: 0,
            frame_ready: false,
        }
    }

    pub fn step(&mut self, memory_bus: &mut MemoryBus, cycles: u32) {
        self.cycles += cycles;

        let lcdc = memory_bus.read(0xFF40);
        match self.mode {
            2 => {
                //OAM scan
                if self.cycles > 80 {
                    self.mode = 3;
                    self.cycles = 0;
                }
            }
            3 => {
                //drawing
                if self.cycles > 172 {
                    if lcdc & 0x80 != 0 {
                        // bit 7 - lcd on/off
                        self.draw_line(memory_bus, lcdc);
                    }
                    self.mode = 0;
                    self.cycles = 0;
                }
            }
            0 => {
                //h-blank
                if self.cycles > 204 {
                    self.cycles = 0;
                    self.ly += 1;
                    memory_bus.write(0xFF44, self.ly);
                    if self.ly < 144 {
                        self.mode = 2;
                    } else if self.ly == 144 {
                        self.mode = 1;
                        let if_flag = memory_bus.read(IF_ADDRESS);
                        memory_bus.write(IF_ADDRESS, if_flag | 0x01)
                    }
                }
            }
            1 => {
                //v-blank
                if self.cycles > 4560 {
                    self.ly = 0;
                    memory_bus.write(0xFF44, self.ly);
                    self.mode = 2;

                    self.cycles = 0;
                    self.frame_ready = true;
                }
            }
            _ => unreachable!(),
        }
    }

    fn draw_line(&mut self, memory_bus: &MemoryBus, lcdc: u8) {
        // each screen tile is 8px. 160/8 = 20 tiles per horizontal line.
        if lcdc & 0x01 == 0 {
            return;
        }
        if lcdc & 0x02 != 0 {
            self.draw_sprites(memory_bus, lcdc);
        }

        let scy = memory_bus.read(0xFF42);
        let scx = memory_bus.read(0xFF43);

        let tile_y = (self.ly.wrapping_add(scy)) / 8;

        for x in 0..20u8 {
            let tile_x: u8 = (x * 8 + scx) / 8;
            let tile_map_index = (tile_y as u16 % 32) * 32 + (tile_x as u16 % 32);

            let tile_id: u8 = if lcdc & 0x08 != 0 {
                memory_bus.read(0x9C00 + tile_map_index as u16)
            } else {
                memory_bus.read(0x9800 + tile_map_index as u16)
            };

            let tile_data_address: u16 = if lcdc & 0x10 != 0 {
                0x8000 + tile_id as u16 * 16 + self.ly.overflowing_add(scy).0 as u16 % 8 * 2
            } else {
                0x9000u16.wrapping_add((tile_id as i8 as i16 as u16).wrapping_mul(16))
                    + self.ly.overflowing_add(scy).0 as u16 % 8 * 2
            };

            let tile_data_low_byte: u8 = memory_bus.read(tile_data_address);
            let tile_data_high_byte: u8 = memory_bus.read(tile_data_address + 1);

            for pixel in 0..8 {
                let color = (tile_data_high_byte >> (7 - pixel) & 1) << 1
                    | (tile_data_low_byte >> (7 - pixel) & 1);
                self.framebuffer[self.ly as usize * 160 + x as usize * 8 + pixel as usize] = color;
            }
        }
    }

    fn draw_sprites(&mut self, memory_bus: &MemoryBus, lcdc: u8) {
        //each sprite is stored in 4 bytes, 40 sprites can be stored
        //sprite 0 = 0xFE00: [Y][X][tile][attrs]
        // sprite 1 = 0xFE04: [Y][X][tile][attrs]
        // sprite 2 = 0xFE08: [Y][X][tile][attrs]
        // ...
        // sprite 39 = 0xFE9C: [Y][X][tile][attrs]

        for sprite_index in 0..39 {
            let base = 0xFE00 + sprite_index * 4;
            let sprite_y = memory_bus.read(base);
            let sprite_x = memory_bus.read(base + 1);
            let tile_id = memory_bus.read(base + 2);
            let attributes = memory_bus.read(base + 3);

            if sprite_y <= self.ly + 16 && self.ly + 16 < sprite_y + 8 {}
        }
    }
}
