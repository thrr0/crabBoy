use std::fs;

pub const MEMORY_BUS_SIZE: usize = 65536;

pub struct MemoryBus {
    pub memory: [u8; MEMORY_BUS_SIZE],
    pub joypad: JoyPad,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; MEMORY_BUS_SIZE],
            joypad: JoyPad::new(),
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        let mut value = self.memory[address as usize];

        if address == 0xff00 {
            eprintln!("joypad read, value={:#04x}", value);
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
        if address == 0xFF40 {
            // eprintln!("LCDC write: {:#04x}", value);
        }
        if address == 0xFF46 {
            let source = (value as u16) << 8;
            for i in 0..160u16 {
                let byte = self.memory[source as usize + i as usize];
                self.memory[0xFE00 + i as usize] = byte;
            }
        }
        //cpu_instr.gb
        // if address == 0xFF02 && value == 0x81 {
        //     if self.memory[0xFF01] == 0x0A {
        //         //line jump
        //         println!();
        //     } else {
        //         print!("{}", self.memory[0xFF01] as char);
        //     }
        // }
        if address == 0xFF04 {
            value = 0
        }
        self.memory[address as usize] = value;
    }

    pub fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).unwrap();
        self.memory[..bytes.len()].copy_from_slice(&bytes);
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
