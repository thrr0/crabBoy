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
        self.decode_execute(op);
    }
    fn fetch(&mut self) -> u8 {
        let instruction = self.memory_bus.read(self.registers.pc);
        self.registers.pc += 1;
        instruction
    }

    fn decode_execute(&mut self, opcode: u8) {
        match opcode {
            //NOP
            0x00 => {}
            //DEC L
            0x2D => self.registers.l = self.dec(self.registers.l),
            //LD (HL-), A
            0x32 => {
                self.memory_bus
                    .write(self.registers.get_hl(), self.registers.a);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
            }
            //INC
            0x3C => self.registers.a = self.inc(self.registers.a),
            // ADD A
            0x80 => self.add(self.registers.b),
            0x81 => self.add(self.registers.c),
            0x82 => self.add(self.registers.d),
            0x83 => self.add(self.registers.e),
            0x84 => self.add(self.registers.h),
            0x85 => self.add(self.registers.l),
            0x86 => self.add(self.memory_bus.read(self.registers.get_hl())),
            0x87 => self.add(self.registers.a),
            //SBC A
            0x9A => self.registers.a = self.sbc(self.registers.d),
            0x9E => self.registers.a = self.sbc(self.memory_bus.read(self.registers.get_hl())),
            //OR A, E
            0xB3 => {
                self.registers.a = self.or(self.registers.e);
            }
            //JUMP
            0xC3 => {
                let low_byte = self.fetch();
                let high_byte = self.fetch();
                self.registers.pc = (high_byte as u16) << 8 | low_byte as u16;
            }
            //RET
            0xC9 => {
                let low_byte = self.memory_bus.read(self.registers.sp);
                self.registers.sp = self.registers.sp.wrapping_add(1);
                let high_byte = self.memory_bus.read(self.registers.sp);
                self.registers.pc = (high_byte as u16) << 8 | low_byte as u16;
                self.registers.sp = self.registers.sp.wrapping_add(1);
            }
            _ => {
                panic!(
                    "opcode no implementado: {:#04x} en PC: {:#06x}",
                    opcode, self.registers.pc
                );
            }
        }
    }

    fn add(&mut self, value: u8) {
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

        return result;
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0x0F) == 0x00;

        return result;
    }
    fn or(&mut self, value: u8) -> u8 {
        let result = (self.registers.a) | (value);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;

        return result;
    }

    fn sbc(&mut self, value: u8) -> u8 {
        let (result1, carry1) = self.registers.a.overflowing_sub(value);
        let (result2, carry2) = result1.overflowing_sub(self.registers.f.carry as u8);

        self.registers.f.zero = result2 == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry =
            (self.registers.a & 0x0F) < (value & 0x0F) + self.registers.f.carry as u8;
        self.registers.f.carry = carry1 || carry2;

        return result2;
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
            pc: 0,
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
