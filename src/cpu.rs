use crate::memory::MemoryBus;

pub struct CPU {
    pub registers: Registers,
    pub memory_bus: MemoryBus,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            memory_bus: MemoryBus::new(),
        }
    }
    pub fn step(&mut self) {
        let op: u8 = self.fetch();
        println!("PC: {:#06x} OP: {:#04x}", self.registers.pc - 1, op);
        self.decode_execute(op);
    }
    fn fetch(&mut self) -> u8 {
        let instruction = self.memory_bus.read(self.registers.pc);
        self.registers.pc += 1;
        instruction
    }

    fn fetch_u16(&mut self) -> u16 {
        let low_byte = self.fetch();
        let high_byte = self.fetch();
        (high_byte as u16) << 8 | low_byte as u16
    }

    fn decode_execute(&mut self, opcode: u8) {
        match opcode {
            //NOP
            0x00 => {}
            //LD BC, n16
            0x01 => {
                let value = self.fetch_u16();
                self.registers.set_bc(value);
            }
            //INC BC
            0x03 => {
                let value = self.inc_u16(self.registers.get_bc());
                self.registers.set_bc(value);
            }
            //LD B, n8
            0x06 => self.registers.b = self.fetch(),
            //LD C, n8
            0x0E => self.registers.c = self.fetch(),
            //LD D, n8
            0x16 => self.registers.d = self.fetch(),
            //LD E, n8
            0x1E => self.registers.e = self.fetch(),
            //JR i8
            0x18 => {
                let offset = self.fetch() as i16;
                self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
            }
            //JR nz, i8
            0x20 => {
                let offset = self.fetch() as i8;
                if !self.registers.f.zero {
                    self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                }
            }
            //INC HL
            0x23 => {
                let value = self.inc_u16(self.registers.get_hl());
                self.registers.set_hl(value);
            }
            //LD HL, n16
            0x21 => {
                let value = self.fetch_u16();
                self.registers.set_hl(value);
            }
            //LD H, n8
            0x26 => self.registers.h = self.fetch(),
            //JR Z, i8
            0x28 => {
                let offset = self.fetch() as i8;
                if self.registers.f.zero {
                    self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                }
            }
            //LD A, (HL+)
            0x2A => {
                self.registers.a = self.memory_bus.read(self.registers.get_hl());
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
            }
            //DEC L
            0x2D => self.registers.l = self.dec(self.registers.l),
            //LD L, n8
            0x2E => self.registers.l = self.fetch(),
            //JR NC, i8
            0x30 => {
                let offset = self.fetch() as i8;
                if !self.registers.f.carry {
                    self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                }
            }
            //LD SP, n16
            0x31 => self.registers.sp = self.fetch_u16(),
            //LD (HL-), A
            0x32 => {
                self.memory_bus
                    .write(self.registers.get_hl(), self.registers.a);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
            }
            //LD [HL], n8
            0x36 => {
                let data = self.fetch();
                self.memory_bus.write(self.registers.get_hl(), data);
            }
            //JR C, i8
            0x38 => {
                let offset = self.fetch() as i8;
                if self.registers.f.carry {
                    self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                }
            }
            //INC A
            0x3C => self.registers.a = self.inc(self.registers.a),
            //LD A, n8
            0x3E => self.registers.a = self.fetch(),
            0x76 => { /* TODO: HALT*/ }
            // LD r, r
            0x40..=0x7f => {
                // opcode: 01_xxx_yyy → xxx = destination, yyy = source
                let src = match opcode & 0x07 {
                    0 => self.registers.b,
                    1 => self.registers.c,
                    2 => self.registers.d,
                    3 => self.registers.e,
                    4 => self.registers.h,
                    5 => self.registers.l,
                    6 => self.memory_bus.read(self.registers.get_hl()), //memory[HL]
                    7 => self.registers.a,
                    _ => panic!("QUE PASO EN MATCH OPCODE AYUDA"),
                };
                let target_adress = (opcode >> 3) & 0x07;

                if target_adress == 6 {
                    //LD r, (HL)
                    self.memory_bus.write(self.registers.get_hl(), src);
                } else {
                    let target = match target_adress {
                        0 => &mut self.registers.b,
                        1 => &mut self.registers.c,
                        2 => &mut self.registers.d,
                        3 => &mut self.registers.e,
                        4 => &mut self.registers.h,
                        5 => &mut self.registers.l,
                        7 => &mut self.registers.a,
                        _ => panic!("QUE PASO EN MATCH OPCODE AYUDA"),
                    };

                    *target = src;
                }
            }
            // ADD A
            0x80 => self.add_to_a(self.registers.b),
            0x81 => self.add_to_a(self.registers.c),
            0x82 => self.add_to_a(self.registers.d),
            0x83 => self.add_to_a(self.registers.e),
            0x84 => self.add_to_a(self.registers.h),
            0x85 => self.add_to_a(self.registers.l),
            0x86 => self.add_to_a(self.memory_bus.read(self.registers.get_hl())),
            0x87 => self.add_to_a(self.registers.a),
            //SBC A
            0x9A => self.registers.a = self.sbc(self.registers.d),
            0x9E => self.registers.a = self.sbc(self.memory_bus.read(self.registers.get_hl())),
            //OR A, C
            0xB1 => self.registers.a = self.or(self.registers.c),
            //OR A, E
            0xB3 => {
                self.registers.a = self.or(self.registers.e);
            }
            //JP NZ, n16
            0xC2 => {
                let address = self.fetch_u16();
                if !self.registers.f.zero {
                    self.registers.pc = address;
                }
            }
            //JUMP
            0xC3 => {
                let address = self.fetch_u16();
                self.registers.pc = address;
            }
            //POP BC
            0xC1 => {
                let address = self.stack_pop();
                self.registers.set_bc(address);
            }
            //PUSH BC
            0xC5 => {
                self.stack_push(self.registers.get_bc());
            }
            //RET
            0xC9 => {
                let address = self.stack_pop();
                self.registers.pc = address;
            }
            //JP Z, n16
            0xCA => {
                let address = self.fetch_u16();
                if self.registers.f.zero {
                    self.registers.pc = address;
                }
            }
            //CALL
            0xCD => {
                //operand
                let address = self.fetch_u16();

                //saving return adress to stack
                self.stack_push(self.registers.pc);

                //jump to operand
                self.registers.pc = address;
            }
            //POP DE
            0xD1 => {
                let address = self.stack_pop();
                self.registers.set_de(address);
            }
            //JP NC, n16
            0xD2 => {
                let address = self.fetch_u16();
                if !self.registers.f.carry {
                    self.registers.pc = address;
                }
            }
            //PUSH DE
            0xD5 => {
                self.stack_push(self.registers.get_de());
            }
            //JP C, n16
            0xDA => {
                let address = self.fetch_u16();
                if self.registers.f.carry {
                    self.registers.pc = address;
                }
            }
            //LD (0xFF00 + n8), A
            0xE0 => {
                let offset = self.fetch();
                self.memory_bus
                    .write(0xFF00 + offset as u16, self.registers.a);
            }
            //POP HL
            0xE1 => {
                let address = self.stack_pop();
                self.registers.set_hl(address);
            }
            //PUSH HL
            0xE5 => {
                self.stack_push(self.registers.get_hl());
            }
            //LD (n16), A
            0xEA => {
                let address = self.fetch_u16();
                self.memory_bus.write(address, self.registers.a);
            }
            //LD A, (0xFF00 + n8)
            0xF0 => {
                let offset = self.fetch();
                self.registers.a = self.memory_bus.read(0xFF00 + offset as u16);
            }
            //POP AF
            0xF1 => {
                let address = self.stack_pop();
                self.registers.set_af(address);
            }
            0xF3 => { /* TODO: Disable interrupts */ }
            //PUSH AF
            0xF5 => {
                self.stack_push(self.registers.get_af());
            }
            //LD A, (a16)
            0xFA => {
                let address = self.fetch_u16();
                self.registers.a = self.memory_bus.read(address);
            }
            //CP A, n8
            0xFE => {
                let operand = self.fetch();
                self.compare(self.registers.a, operand);
            }
            _ => {
                panic!(
                    "opcode no implementado: {:#04x} en PC: {:#06x}",
                    opcode, self.registers.pc
                );
            }
        }
    }

    fn add_to_a(&mut self, value: u8) {
        let (result_a, hubo_carry) = self.registers.a.overflowing_add(value);

        //flags
        self.registers.f.zero = result_a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = hubo_carry;
        self.registers.f.half_carry = (self.registers.a & 0x0F) + (value & 0x0F) > 0x0F;

        //a register
        self.registers.a = result_a;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0x0F) == 0x0F;

        result
    }

    fn inc_u16(&mut self, value: u16) -> u16 {
        value.wrapping_add(1)
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0x0F) == 0x00;

        result
    }

    fn compare(&mut self, value: u8, operand: u8) {
        let (result, carry) = value.overflowing_sub(operand);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (value & 0x0F) < (result & 0x0F);
    }
    fn or(&mut self, value: u8) -> u8 {
        let result = (self.registers.a) | (value);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;

        result
    }

    fn sbc(&mut self, value: u8) -> u8 {
        let (result1, carry1) = self.registers.a.overflowing_sub(value);
        let (result2, carry2) = result1.overflowing_sub(self.registers.f.carry as u8);

        self.registers.f.zero = result2 == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry =
            (self.registers.a & 0x0F) < (value & 0x0F) + self.registers.f.carry as u8;
        self.registers.f.carry = carry1 || carry2;

        result2
    }

    fn stack_push(&mut self, value: u16) {
        let high_byte = (value >> 8) as u8;
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory_bus.write(self.registers.sp, high_byte);
        let low_byte = value as u8;
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory_bus.write(self.registers.sp, low_byte);
    }

    fn stack_pop(&mut self) -> u16 {
        let low_byte = self.memory_bus.read(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let high_byte = self.memory_bus.read(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);

        let adress = (high_byte as u16) << 8 | low_byte as u16;

        adress
    }
}

// 8-bit registers
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister, //flags 1111 0000: 1- zero 1- Subtraction 1- Half Carry 1- Carry 0000
    pub h: u8,
    pub l: u8,
    pub pc: u16, //program counter
    pub sp: u16, //stack pointer
}

impl Registers {
    fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister {
                zero: false,
                subtract: false,
                half_carry: false,
                carry: false,
            },
            h: 0,
            l: 0,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }
    //get 16-bit virtual regs
    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f.get_register() as u16 // bit manipulation
    }
    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16 // bit manipulation
    }
    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16 // bit manipulation
    }
    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16 // bit manipulation
    }

    //set 16-bit virtual regs
    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f.set_register((value & 0xFF) as u8);
    }
    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }
    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }
    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

pub struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POISITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl FlagsRegister {
    fn get_register(&self) -> u8 {
        (self.zero as u8) << ZERO_FLAG_BYTE_POSITION
            | (self.subtract as u8) << SUBTRACT_FLAG_BYTE_POISITION
            | (self.half_carry as u8) << HALF_CARRY_FLAG_BYTE_POSITION
            | (self.carry as u8) << CARRY_FLAG_BYTE_POSITION
    }

    fn set_register(&mut self, value: u8) {
        self.zero = ((value >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        self.subtract = ((value >> SUBTRACT_FLAG_BYTE_POISITION) & 0b1) != 0;
        self.half_carry = ((value >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        self.carry = ((value >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
    }
}
