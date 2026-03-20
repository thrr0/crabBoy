use crate::memory::MemoryBus;

// 8-bit registers
struct CPU {
    registers: Registers,
    memory_bus: MemoryBus,
}

impl CPU {
    fn step(&mut self) {
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
            // ADD
            0x80 => {
                let (resultado_a, hubo_carry) = self.registers.a.overflowing_add(self.registers.b);

                //flags
                self.registers.f.zero = resultado_a == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = hubo_carry;
                self.registers.f.half_carry =
                    (self.registers.a & 0x0F) + (self.registers.b & 0x0F) > 0x0F;

                //a register
                self.registers.a = resultado_a;
            }
            _ => {
                panic!("opcode no implementado: {:#04x}", opcode);
            }
        }
    }
}
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister, //flags 1111 0000: 1- zero 1- Subtraction 1- Half Carry 1- Carry 0000
    h: u8,
    l: u8,
    pc: u16,
}

impl Registers {
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

struct FlagsRegister {
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
