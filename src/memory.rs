use std::fs;

use crate::hardware::HardwareMode;

pub const MEMORY_BUS_SIZE: usize = 65536;

pub struct MemoryBus {
    pub memory: [u8; MEMORY_BUS_SIZE],
    pub joypad: JoyPad,
    pub rom: Vec<u8>,
    pub rom_bank: u16,
    pub mbc_type: u8,
    pub ram_enabled: bool,
    pub ram: Vec<u8>,
    pub ram_bank: u8,
    pub ram_dirty: bool,
    pub hardware_mode: HardwareMode,
    pub boot_rom: Option<[u8; 256]>,
    pub boot_rom_active: bool,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; MEMORY_BUS_SIZE],
            joypad: JoyPad::new(),
            rom: Vec::new(),
            rom_bank: 1,
            mbc_type: 0,
            ram_enabled: false,
            ram: Vec::new(),
            ram_bank: 0,
            ram_dirty: false,
            hardware_mode: HardwareMode::DMG,
            boot_rom: None,
            boot_rom_active: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if self.boot_rom_active && address < 0x0100 {
            if let Some(boot) = &self.boot_rom {
                return boot[address as usize];
            }
        }

        return match address {
            0x4000..=0x7FFF => {
                let index = self.rom_bank as usize * 0x4000 + (address as usize - 0x4000);
                if index < self.rom.len() {
                    self.rom[index]
                } else {
                    0xFF
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if (self.mbc_type == 0x05 || self.mbc_type == 0x06) && address <= 0xA1FF {
                        //mbc2
                        self.ram[(address as usize - 0xA000) % 512] | 0xF0
                    } else {
                        //RAM read
                        let real_address =
                            self.ram_bank as usize * 0x2000 + (address as usize - 0xA000);
                        self.ram[real_address]
                    }
                } else {
                    0xFF
                }
            }
            0xFF00 => self.handle_joypad(self.memory[0xFF00]),
            _ => self.memory[address as usize],
        };
    }

    fn handle_joypad(&self, value: u8) -> u8 {
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

        value & 0x30 | (bit_3 as u8) << 3 | (bit_2 as u8) << 2 | (bit_1 as u8) << 1 | bit_0 as u8
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.handle_ram_enable(address, value);
            }
            0x2000..=0x3FFF => {
                self.handle_mbc_write(address, value);
            }
            0x4000..=0x5FFF => {
                //RAM banking
                if (0x00..=0x03).contains(&value) {
                    self.ram_bank = value;
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if (self.mbc_type == 0x05 || self.mbc_type == 0x06) && address <= 0xA1FF {
                        self.ram[(address as usize - 0xA000) % 512] = value & 0xF;
                    } else {
                        //RAM write
                        let real_address =
                            self.ram_bank as usize * 0x2000 + (address as usize - 0xA000);
                        self.ram[real_address] = value;
                        self.ram_dirty = true;
                    }
                }
            }
            0xFF04 => {
                // DIV is read-only; any write resets it to 0
                self.memory[address as usize] = 0;
            }
            0xFF50 => {
                //     eprintln!("0xFF50 write: {:#04x}", value);
                eprintln!("boot rom disabled");
                self.boot_rom_active = false;
                self.memory[address as usize] = value;
            }
            0xFF46 => {
                // OAM DMA: copying 160 bytes from source address to OAM (0xFE00-0xFE9F)
                let source = (value as u16) << 8;
                for i in 0..160u16 {
                    let byte = self.memory[source as usize + i as usize];
                    self.memory[0xFE00 + i as usize] = byte;
                }
            }
            _ => {
                //mode 3 locks vram
                //NOT STRICTLY NECESSARY
                // if (0x8000..=0x9FFF).contains(&address) {
                //     if (self.memory[0xFF41] & 0x03) == 3 {
                //         return;
                //     }
                // }
                self.memory[address as usize] = value;
            }
        }
    }

    fn handle_ram_enable(&mut self, address: u16, value: u8) {
        // Writing to ROM space (0x0000-0x1FFF) is intercepted by the mbc as a ram enable command
        // Value 0x0A in lower nibble enables external RAM; any other value disables it
        let is_mbc2 = matches!(self.mbc_type, 0x05 | 0x06);
        let bit8 = address & 0x0100 != 0;
        let enables_ram = value & 0x0F == 0x0A;

        if is_mbc2 {
            if !bit8 {
                self.ram_enabled = enables_ram;
            }
        } else if self.mbc_type != 0 {
            self.ram_enabled = enables_ram;
        }
    }

    fn handle_mbc_write(&mut self, address: u16, value: u8) {
        match self.mbc_type {
            0x00 => { /* Rom only */ }
            0x01..=0x03 => {
                //mbc1: 5-bit bank number, bank 0 is remapped to 1 (bank 0 is always at 0x0000-0x3FFF)

                let bank = value as u16 & 0x1F;
                self.rom_bank = match bank {
                    0x00 => 1,
                    _ => bank,
                };
                // eprintln!("rom bank write= {:#04x}", self.rom_bank);
            }
            0x05..=0x06 => {
                //mbc2 4-bit bank number, 0 is remapped to 1. writing is only allowed if bit 8 from address == 1
                let bit8: bool = address & 0x0100 != 0;

                if bit8 {
                    let new_bank = match (value & 0x0F) as u16 {
                        0x00 => 0x01,
                        _ => (value & 0x0F) as u16,
                    };
                    self.rom_bank = new_bank;
                }
            }
            0x0F..=0x13 => {
                // mbc3: 7-bit bank number, bank 0 is valid unlike MBC1

                self.rom_bank = value as u16 & 0x7F;
            }
            0x19..=0x1E => {
                //mbc5: 9-bit bank number split across two registers
                //0x2000-0x2FFF: low 8 bits, 0x3000-0x3FFF: bit 8
                if address <= 0x2FFF {
                    self.rom_bank = (self.rom_bank & 0x100) | value as u16;
                } else {
                    self.rom_bank = (self.rom_bank & 0xFF) | ((value as u16 & 0x01) << 8);
                }
            }
            _ => panic!("wrong mbc"),
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        self.rom = fs::read(path).unwrap();

        eprintln!("mbc type;: {:#04x}", self.rom[0x0147]);
        self.mbc_type = self.rom[0x0147];

        let cartridge_ram_size = match self.rom[0x0149] {
            0x01 => 0x800,
            0x02 => 0x2000,
            0x03 => 0x8000,
            0x04 => 0x20000,
            0x05 => 0x10000,
            _ => 0,
        };

        if matches!(self.mbc_type, 0x05 | 0x06) {
            self.ram = vec![0; 512];
        } else {
            self.ram = vec![0; cartridge_ram_size];
        }

        self.hardware_mode = match self.rom[0x147] {
            // 0x80 => //dmg & gbc
            0xC0 => HardwareMode::GBC,
            _ => HardwareMode::DMG,
        };

        if matches!(self.hardware_mode, HardwareMode::DMG) {
            if let Ok(bytes) = fs::read("roms/boot.gb") {
                let mut arr = [0u8; 256];
                arr.copy_from_slice(&bytes[..256]);
                self.boot_rom = Some(arr);
                self.boot_rom_active = true;
                eprintln!("boot rom loaded");
            } else {
                panic!("boot rom missing");
            }
        }
        // TO DO: gbc and both modes handle

        self.memory[0x0000..0x4000].copy_from_slice(&self.rom[0x0000..0x4000]);
    }

    pub fn load_ram(&mut self, path: &str) {
        if let Ok(bytes) = fs::read(path) {
            eprint!(".sav loaded");

            if bytes.len() == self.ram.len() {
                eprint!(".sav length ok");
                self.ram = bytes;
            } else {
                eprintln!(".sav length is wrong!!");
            }
        } else {
            eprint!(".sav not loaded");
        }
    }

    pub fn save_ram(&mut self, path: &str) -> bool {
        fs::write(path, &self.ram).is_ok()
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
