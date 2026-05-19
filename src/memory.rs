use std::fs;

pub const MEMORY_BUS_SIZE: usize = 65536;

pub struct MemoryBus {
    pub memory: [u8; MEMORY_BUS_SIZE],
    pub joypad: JoyPad,
    pub rom: Vec<u8>,
    pub mbc_type: u8,
    pub rom_bank: u8,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; MEMORY_BUS_SIZE],
            joypad: JoyPad::new(),
            rom: Vec::new(),
            mbc_type: 0,
            rom_bank: 1,
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        let mut value = if (0x4000..=0x7FFF).contains(&address) {
            self.rom[self.rom_bank as usize * 0x4000 + (address as usize - 0x4000)]
        } else {
            self.memory[address as usize]
        };

        if address == 0xff00 {
            // eprintln!("joypad read, value={:#04x}", value);
            // bit 5: select button group (0=active)
            // bit 4: select direction group (0=active)
            // bit 3: down  / start
            // bit 2: up    / select
            // bit 1: left  / b
            // bit 0: right / a
            let mut bit_0: bool = value & 0x1 != 0;
            let mut bit_1: bool = value & 0x2 != 0;
            let mut bit_2: bool = value & 0x4 != 0;
            let mut bit_3: bool = value & 0x8 != 0;
            if value & 0x20 == 0 {
                //button group
                bit_0 = !self.joypad.a_button;
                bit_1 = !self.joypad.b_button;
                bit_2 = !self.joypad.select;
                bit_3 = !self.joypad.start;
            }
            if value & 0x10 == 0 {
                //direction group
                bit_0 = !self.joypad.right;
                bit_1 = !self.joypad.left;
                bit_2 = !self.joypad.up;
                bit_3 = !self.joypad.down;
            }

            value = value & 0x30
                | (bit_3 as u8) << 3
                | (bit_2 as u8) << 2
                | (bit_1 as u8) << 1
                | bit_0 as u8;
        }
        value
    }

    pub fn write(&mut self, address: u16, mut value: u8) {
        if (0x2000..=0x3FFF).contains(&address) {
            self.rom_bank = value & 0x1F;
            eprintln!("rom bank= {:#04}", self.rom_bank);
            if self.rom_bank == 0 {
                self.rom_bank = 1;
            }
        } else {
            if address == 0xFF46 {
                let source = (value as u16) << 8;
                for i in 0..160u16 {
                    let byte = self.memory[source as usize + i as usize];
                    self.memory[0xFE00 + i as usize] = byte;
                }
            }

            // not accessed by the game
            if address == 0xff04 {
                value = 0
            }
            self.memory[address as usize] = value;
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).unwrap();
        self.rom = bytes;
        self.mbc_type = self.rom[0x0147];
        self.memory[0x0000..0x4000].copy_from_slice(&self.rom[0x0000..0x4000]);
    }
}

pub struct JoyPad {
    pub up: bool,
    pub down: bool,
    pub right: bool,
    pub left: bool,
    pub select: bool,
    pub start: bool,
    pub a_button: bool,
    pub b_button: bool,
}

impl JoyPad {
    pub fn new() -> JoyPad {
        JoyPad {
            up: false,
            down: false,
            right: false,
            left: false,
            select: false,
            start: false,
            a_button: false,
            b_button: false,
        }
    }
}
