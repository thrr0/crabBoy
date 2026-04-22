use std::fs;
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
        // println!("PC: {:#06x} OP: {:#04x}", self.registers.pc - 1, op);
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
            //LD (BC), A
            0x02 => {
                self.memory_bus
                    .write(self.registers.get_bc(), self.registers.a);
            }
            //INC BC
            0x03 => {
                let value = self.inc_u16(self.registers.get_bc());
                self.registers.set_bc(value);
            }
            //INC B
            0x04 => self.registers.b = self.inc(self.registers.b),
            //DEC B
            0x05 => self.registers.b = self.dec(self.registers.b),
            //LD B, n8
            0x06 => self.registers.b = self.fetch(),
            //RLCA
            0x07 => self.registers.a = self.rlca(self.registers.a),
            //LD (nn), SP
            0x08 => {
                let address = self.fetch_u16();
                let sp_low_byte = self.registers.sp as u8;
                let sp_high_byte = (self.registers.sp >> 8) as u8;

                self.memory_bus.write(address, sp_low_byte);
                self.memory_bus.write(address + 1, sp_high_byte);
            }
            //ADD HL, u16
            0x09 | 0x19 | 0x29 | 0x39 => {
                let value = match opcode {
                    0x09 => self.registers.get_bc(),
                    0x19 => self.registers.get_de(),
                    0x29 => self.registers.get_hl(),
                    0x39 => self.registers.sp,
                    _ => panic!(""),
                };

                let result = self.add_u16(self.registers.get_hl(), value);

                self.registers.set_hl(result);
            }
            //DEC BC
            0x0B => {
                let value = self.dec_u16(self.registers.get_bc());

                self.registers.set_bc(value);
            }
            //INC C
            0x0C => self.registers.c = self.inc(self.registers.c),
            //DEC C
            0x0D => self.registers.c = self.dec(self.registers.c),
            //LD C, n8
            0x0E => self.registers.c = self.fetch(),
            //RRCA
            0x0F => self.registers.a = self.rrca(self.registers.a),
            0x10 => { /* TODO: STOP */ }
            //LD DE, n16
            0x11 => {
                let value = self.fetch_u16();
                self.registers.set_de(value);
            }
            //LD (DE), A
            0x12 => {
                self.memory_bus
                    .write(self.registers.get_de(), self.registers.a);
            }
            //INC DE
            0x13 => {
                let value = self.inc_u16(self.registers.get_de());

                self.registers.set_de(value);
            }
            //INC D
            0x14 => self.registers.d = self.inc(self.registers.d),
            //DEC D
            0x15 => self.registers.d = self.dec(self.registers.d),
            //LD D, n8
            0x16 => self.registers.d = self.fetch(),
            //RLA
            0x17 => self.registers.a = self.rla(self.registers.a),
            //JR i8
            0x18 => {
                let offset = self.fetch() as i8 as i16;
                self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
            }
            //LD A, (DE)
            0x1A => self.registers.a = self.memory_bus.read(self.registers.get_de()),
            //DEC DE
            0x1B => {
                let value = self.dec_u16(self.registers.get_de());

                self.registers.set_de(value);
            }
            //INC E
            0x1C => self.registers.e = self.inc(self.registers.e),
            //DEC E
            0x1D => self.registers.e = self.dec(self.registers.e),
            //LD E, n8
            0x1E => self.registers.e = self.fetch(),
            //RRA
            0x1F => self.registers.a = self.rra(self.registers.a),
            //JR nz, i8
            0x20 => {
                let offset = self.fetch() as i8;
                if !self.registers.f.zero {
                    self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                }
            }
            //LD HL, n16
            0x21 => {
                let value = self.fetch_u16();
                self.registers.set_hl(value);
            }
            //LD (HL+), A
            0x22 => {
                self.memory_bus
                    .write(self.registers.get_hl(), self.registers.a);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
            }
            //INC HL
            0x23 => {
                let value = self.inc_u16(self.registers.get_hl());
                self.registers.set_hl(value);
            }
            //INC H
            0x24 => self.registers.h = self.inc(self.registers.h),
            //DEC H
            0x25 => self.registers.h = self.dec(self.registers.h),
            //LD H, n8
            0x26 => self.registers.h = self.fetch(),
            //DAA
            0x27 => {
                if self.registers.f.subtract {
                    if self.registers.f.half_carry {
                        self.registers.a = self.registers.a.wrapping_sub(0x06);
                    }
                    if self.registers.f.carry {
                        self.registers.a = self.registers.a.wrapping_sub(0x60)
                    }
                } else {
                    if (self.registers.a & 0x0F > 0x09) || self.registers.f.half_carry {
                        self.registers.a = self.registers.a.wrapping_add(0x06);
                    }
                    if (self.registers.a >> 4) & 0x0F > 0x09 || self.registers.f.carry {
                        self.registers.a = self.registers.a.wrapping_add(0x60);
                        self.registers.f.carry = true;
                    }
                }

                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.half_carry = false;
            }
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
            //DEC HL
            0x2B => {
                let value = self.dec_u16(self.registers.get_hl());

                self.registers.set_hl(value);
            }
            //INC L
            0x2C => self.registers.l = self.inc(self.registers.l),
            //DEC L
            0x2D => self.registers.l = self.dec(self.registers.l),
            //LD L, n8
            0x2E => self.registers.l = self.fetch(),
            //CPL A
            0x2F => {
                self.registers.a = !self.registers.a;

                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;
            }
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
            //INC SP
            0x33 => {
                let value = self.inc_u16(self.registers.sp);

                self.registers.sp = value;
            }
            //INC (HL)
            0x34 => {
                let original_value = self.memory_bus.read(self.registers.get_hl());
                let incremented_value = self.inc(original_value);

                self.memory_bus
                    .write(self.registers.get_hl(), incremented_value);
            }
            //DEC (HL)
            0x35 => {
                let original_value = self.memory_bus.read(self.registers.get_hl());
                let decreased_value = self.dec(original_value);

                self.memory_bus
                    .write(self.registers.get_hl(), decreased_value);
            }
            //LD [HL], n8
            0x36 => {
                let data = self.fetch();
                self.memory_bus.write(self.registers.get_hl(), data);
            }
            //SCF
            0x37 => {
                self.registers.f.carry = true;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
            }
            //JR C, i8
            0x38 => {
                let offset = self.fetch() as i8;
                if self.registers.f.carry {
                    self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                }
            }
            //LD A, (HL-)
            0x3A => {
                self.registers.a = self.memory_bus.read(self.registers.get_hl());
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
            }
            //DEC SP
            0x3B => {
                let value = self.dec_u16(self.registers.sp);

                self.registers.sp = value;
            }
            //INC A
            0x3C => self.registers.a = self.inc(self.registers.a),
            //DEC A
            0x3D => self.registers.a = self.dec(self.registers.a),
            //LD A, n8
            0x3E => self.registers.a = self.fetch(),
            //CCF
            0x3F => {
                self.registers.f.carry = !self.registers.f.carry;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
            }
            0x76 => { /* TODO: HALT*/ }
            // LD n, n
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
                    _ => panic!("wrong source opcode match LD n, n"),
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
                        _ => panic!("wrong target opcode match LD n, n"),
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
            //ADC A
            0x88 => self.adc_to_a(self.registers.b),
            0x89 => self.adc_to_a(self.registers.c),
            0x8A => self.adc_to_a(self.registers.d),
            0x8B => self.adc_to_a(self.registers.e),
            0x8C => self.adc_to_a(self.registers.h),
            0x8D => self.adc_to_a(self.registers.l),
            0x8E => self.adc_to_a(self.memory_bus.read(self.registers.get_hl())),
            0x8F => self.adc_to_a(self.registers.a),
            //SUB A
            0x90 => self.sub_from_a(self.registers.b),
            0x91 => self.sub_from_a(self.registers.c),
            0x92 => self.sub_from_a(self.registers.d),
            0x93 => self.sub_from_a(self.registers.e),
            0x94 => self.sub_from_a(self.registers.h),
            0x95 => self.sub_from_a(self.registers.l),
            0x96 => self.sub_from_a(self.memory_bus.read(self.registers.get_hl())),
            0x97 => self.sub_from_a(self.registers.a),
            //SBC A
            0x98 => self.registers.a = self.sbc(self.registers.b),
            0x99 => self.registers.a = self.sbc(self.registers.c),
            0x9A => self.registers.a = self.sbc(self.registers.d),
            0x9B => self.registers.a = self.sbc(self.registers.e),
            0x9C => self.registers.a = self.sbc(self.registers.h),
            0x9D => self.registers.a = self.sbc(self.registers.l),
            0x9E => self.registers.a = self.sbc(self.memory_bus.read(self.registers.get_hl())),
            0x9F => self.registers.a = self.sbc(self.registers.a),
            //AND, XOR, OR, CMP n
            0xA0..=0xBF => {
                // opcode: 01_xxx_yyy → xxx = operaction, yyy = source
                let src = match opcode & 0x07 {
                    //operand changes every opecode, repeats after 7
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

                let result: u8 = match opcode {
                    //operation changes every 7 opcodes
                    0xA0..=0xA7 => self.a_and(src),
                    0xA8..=0xAF => self.a_xor(src),
                    0xB0..=0xB7 => self.a_or(src),
                    0xB8..=0xBF => {
                        self.compare(self.registers.a, src);
                        self.registers.a
                    }
                    _ => panic!("wrong opcode"),
                };

                self.registers.a = result;
            }
            //RET NZ
            0xC0 => {
                if !self.registers.f.zero {
                    self.ret()
                };
            }
            //POP BC
            0xC1 => {
                let address = self.stack_pop();
                self.registers.set_bc(address);
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

            //CALL NZ, nn
            0xC4 => {
                //operand
                let address = self.fetch_u16();
                if self.registers.f.zero == false {
                    //saving return adress to stack
                    self.stack_push(self.registers.pc);

                    //jump to operand
                    self.registers.pc = address;
                }
            }
            //PUSH BC
            0xC5 => {
                self.stack_push(self.registers.get_bc());
            }
            //ADD A, n8
            0xC6 => {
                let value = self.fetch();
                self.add_to_a(value);
            }
            //RST $00H
            0xC7 => self.rst(opcode),
            //RET Z
            0xC8 => {
                if self.registers.f.zero {
                    self.ret()
                }
            }
            //RET
            0xC9 => self.ret(),
            //JP Z, n16
            0xCA => {
                let address = self.fetch_u16();
                if self.registers.f.zero {
                    self.registers.pc = address;
                }
            }
            //PREFIX
            0xCB => {
                // opcode: 01_xxx_yyy → xxx = operaction, yyy = source
                let next_opcode = self.fetch();

                let src = match next_opcode & 0x07 {
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

                let result: u8;
                if (0x40..=0x7F).contains(&next_opcode) {
                    match next_opcode {
                        0x40..=0x47 => self.bit(src, 0),
                        0x48..=0x4F => self.bit(src, 1),
                        0x50..=0x57 => self.bit(src, 2),
                        0x58..=0x5F => self.bit(src, 3),
                        0x60..=0x67 => self.bit(src, 4),
                        0x68..=0x6F => self.bit(src, 5),
                        0x70..=0x77 => self.bit(src, 6),
                        0x78..=0x7F => self.bit(src, 7),
                        _ => panic!("wrong opcode PREFIX"),
                    }
                } else {
                    result = match next_opcode {
                        0x00..=0x07 => self.rlc(src),
                        0x08..=0x0F => self.rrc(src),
                        0x10..=0x17 => self.rl(src),
                        0x18..=0x1F => self.rr(src),
                        0x20..=0x27 => self.sla(src),
                        0x28..=0x2F => self.sra(src),
                        0x30..=0x37 => self.swap(src),
                        0x38..=0x3F => self.srl(src),
                        0x80..=0x87 => self.res(src, 0),
                        0x88..=0x8F => self.res(src, 1),
                        0x90..=0x97 => self.res(src, 2),
                        0x98..=0x9F => self.res(src, 3),
                        0xA0..=0xA7 => self.res(src, 4),
                        0xA8..=0xAF => self.res(src, 5),
                        0xB0..=0xB7 => self.res(src, 6),
                        0xB8..=0xBF => self.res(src, 7),
                        0xC0..=0xC7 => self.set(src, 0),
                        0xC8..=0xCF => self.set(src, 1),
                        0xD0..=0xD7 => self.set(src, 2),
                        0xD8..=0xDF => self.set(src, 3),
                        0xE0..=0xE7 => self.set(src, 4),
                        0xE8..=0xEF => self.set(src, 5),
                        0xF0..=0xF7 => self.set(src, 6),
                        0xF8..=0xFF => self.set(src, 7),
                        _ => panic!("wrong opcode PREFIX"),
                    };

                    let target_adress = next_opcode & 0x07;

                    if target_adress == 6 {
                        self.memory_bus.write(self.registers.get_hl(), result);
                    } else {
                        let target = match target_adress {
                            0 => &mut self.registers.b,
                            1 => &mut self.registers.c,
                            2 => &mut self.registers.d,
                            3 => &mut self.registers.e,
                            4 => &mut self.registers.h,
                            5 => &mut self.registers.l,
                            7 => &mut self.registers.a,
                            _ => panic!("wrong target adress prefix"),
                        };

                        *target = result;
                    }
                }
            }
            //CALL Z, nn
            0xCC => {
                let address = self.fetch_u16();

                if self.registers.f.zero {
                    self.stack_push(self.registers.pc);
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
            //ADC A, n
            0xCE => {
                let value = self.fetch();
                self.adc_to_a(value);
            }
            //RST $08
            0xCF => self.rst(opcode),
            //RET NC
            0xD0 => {
                if !self.registers.f.carry {
                    self.ret()
                }
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
            //CALL NC, nn
            0xD4 => {
                let address = self.fetch_u16();

                if !self.registers.f.carry {
                    self.stack_push(self.registers.pc);
                    self.registers.pc = address;
                }
            }
            //PUSH DE
            0xD5 => {
                self.stack_push(self.registers.get_de());
            }
            //SUB A, n8
            0xD6 => {
                let value = self.fetch();
                self.sub_from_a(value);
            }
            //RST $10
            0xD7 => self.rst(opcode),
            //RET C
            0xD8 => {
                if self.registers.f.carry {
                    self.ret();
                }
            }
            //RETI
            0xD9 => {
                self.ret();
                // TODO: enable interrupts
            }
            //JP C, n16
            0xDA => {
                let address = self.fetch_u16();
                if self.registers.f.carry {
                    self.registers.pc = address;
                }
            }
            //CALL C , nn
            0xDC => {
                let address = self.fetch_u16();

                if self.registers.f.carry {
                    self.stack_push(self.registers.pc);
                    self.registers.pc = address;
                }
            }
            //SBC A, n
            0xDE => {
                let value = self.fetch();
                self.registers.a = self.sbc(value);
            }
            //RST $18
            0xDF => self.rst(opcode),
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
            // LD ($FF00+C), A
            0xE2 => {
                self.memory_bus
                    .write(0xFF00 + self.registers.c as u16, self.registers.a);
            }
            //PUSH HL
            0xE5 => {
                self.stack_push(self.registers.get_hl());
            }
            //AND A, n8
            0xE6 => {
                let value = self.fetch();
                self.registers.a = self.a_and(value);
            }
            //RST $20
            0xE7 => self.rst(opcode),
            //ADD SP, n8
            0xE8 => {
                let value = self.fetch();

                let result = self.registers.sp.overflowing_add_signed(value as i8 as i16);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry =
                    (self.registers.sp as u8 & 0x0F) + (value & 0x0F) > 0x0F;
                self.registers.f.carry = (self.registers.sp & 0xFF) + (value as u16 & 0xFF) > 0xFF;

                self.registers.sp = result.0;
            }
            //JP HL
            0xE9 => self.registers.pc = self.registers.get_hl(),
            //LD (n16), A
            0xEA => {
                let address = self.fetch_u16();
                self.memory_bus.write(address, self.registers.a);
            }
            //XOR A, n8
            0xEE => {
                let value = self.fetch();
                self.registers.a = self.a_xor(value);
            }
            //RST $28
            0xEF => self.rst(opcode),
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
            //LD A, ($FF00 + C)
            0xF2 => self.registers.a = self.memory_bus.read(0xFF00 + self.registers.c as u16),
            0xF3 => { /* TODO: Disable interrupts */ }
            //PUSH AF
            0xF5 => {
                self.stack_push(self.registers.get_af());
            }
            //OR A, n8
            0xF6 => {
                let value = self.fetch();
                self.registers.a = self.a_or(value);
            }
            //RST $30
            0xF7 => self.rst(opcode),
            //LD HL, SP+n
            0xF8 => {
                let value = self.fetch();
                let sp_low_byte = self.registers.sp as u8;

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (sp_low_byte & 0x0F) + (value & 0x0F) > 0x0F;
                self.registers.f.carry = sp_low_byte.overflowing_add(value).1;

                self.registers
                    .set_hl(self.registers.sp.wrapping_add_signed(value as i8 as i16));
            }
            //LD SP, HL
            0xF9 => {
                self.registers.sp = self.registers.get_hl();
            }
            //LD A, (a16)
            0xFA => {
                let address = self.fetch_u16();
                self.registers.a = self.memory_bus.read(address);
            }
            0xFB => { /* TODO: ENABLE INTERRUPTS */ }
            //CP A, n8
            0xFE => {
                let operand = self.fetch();
                self.compare(self.registers.a, operand);
            }
            //RST $38
            0xFF => self.rst(opcode),
            _ => {
                panic!(
                    "opcode no implementado: {:#04x} en PC: {:#06x}",
                    opcode, self.registers.pc
                );
            }
        }
    }

    fn adc_to_a(&mut self, value: u8) {
        let previous_carry = self.registers.f.carry as u8;
        let (sum, carry1) = value.overflowing_add(previous_carry);
        let (result_a, carry2) = self.registers.a.overflowing_add(sum);

        self.registers.f.zero = result_a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry1 || carry2;
        self.registers.f.half_carry =
            (self.registers.a & 0x0F) + (value & 0x0F) + (previous_carry & 0x0F) > 0x0F;

        self.registers.a = result_a;
    }

    fn add_to_a(&mut self, value: u8) {
        let (result_a, carry) = self.registers.a.overflowing_add(value);

        //flags
        self.registers.f.zero = result_a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (self.registers.a & 0x0F) + (value & 0x0F) > 0x0F;

        //a register
        self.registers.a = result_a;
    }

    fn add_u16(&mut self, value_a: u16, value_b: u16) -> u16 {
        let (result, carry) = value_a.overflowing_add(value_b);

        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value_a & 0xFFF) + (value_b & 0xFFF) > 0xFFF;
        self.registers.f.carry = carry;

        result
    }

    fn sub_from_a(&mut self, value: u8) {
        let (result_a, carry) = self.registers.a.overflowing_sub(value);
        //flags
        self.registers.f.zero = result_a == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (self.registers.a & 0x0F) < (value & 0x0F);

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

    fn dec_u16(&mut self, value: u16) -> u16 {
        value.wrapping_sub(1)
    }

    fn compare(&mut self, value: u8, operand: u8) {
        let (result, carry) = value.overflowing_sub(operand);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (value & 0x0F) < (operand & 0x0F);
    }

    fn a_or(&mut self, value: u8) -> u8 {
        let result = (self.registers.a) | (value);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;

        result
    }

    fn a_and(&mut self, value: u8) -> u8 {
        let result = (self.registers.a) & (value);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = true;

        result
    }

    fn a_xor(&mut self, value: u8) -> u8 {
        let result = (self.registers.a) ^ (value);

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

    fn swap(&mut self, byte: u8) -> u8 {
        let high_nibble = byte & 0xF0;
        let low_nibble = byte & 0x0F;

        let result = low_nibble << 4 | high_nibble >> 4;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        result
    }

    // rorate left
    fn rlca(&mut self, byte: u8) -> u8 {
        let rotated = (byte << 1) | (byte >> 7);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x80 != 0;

        rotated
    }

    //rotate left through carry
    fn rla(&mut self, byte: u8) -> u8 {
        let rotated = (byte << 1) | self.registers.f.carry as u8;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x80 != 0;

        rotated
    }

    //rotate right
    fn rrca(&mut self, byte: u8) -> u8 {
        let rotated = (byte << 7) | (byte >> 1);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x01 != 0;

        rotated
    }

    //rotate right through carry
    fn rra(&mut self, byte: u8) -> u8 {
        let rotated = ((self.registers.f.carry as u8) << 7) | (byte >> 1);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x01 != 0;

        rotated
    }
    // CB: rotate left circular
    fn rlc(&mut self, byte: u8) -> u8 {
        let rotated = (byte << 1) | (byte >> 7);

        self.registers.f.zero = rotated == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x80 != 0;

        rotated
    }

    //CB: rotate right circular
    fn rrc(&mut self, byte: u8) -> u8 {
        let rotated = (byte << 7) | (byte >> 1);

        self.registers.f.zero = rotated == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x01 != 0;

        rotated
    }

    //CB: rotate left through carry
    fn rl(&mut self, byte: u8) -> u8 {
        let rotated = (byte << 1) | self.registers.f.carry as u8;

        self.registers.f.zero = rotated == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x80 != 0;

        rotated
    }

    //CB: rotate right through carry
    fn rr(&mut self, byte: u8) -> u8 {
        let rotated = ((self.registers.f.carry as u8) << 7) | (byte >> 1);

        self.registers.f.zero = rotated == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x01 != 0;

        rotated
    }

    //arithmetic left shift
    fn sla(&mut self, byte: u8) -> u8 {
        let shifted = byte << 1;

        self.registers.f.zero = shifted == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x80 != 0;

        shifted
    }

    //arithmetic right shift
    fn sra(&mut self, byte: u8) -> u8 {
        let shifted = byte & 0x80 | byte >> 1;

        self.registers.f.zero = shifted == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x01 != 0;

        shifted
    }

    //logic right shift
    fn srl(&mut self, byte: u8) -> u8 {
        let shifted = byte >> 1;

        self.registers.f.zero = shifted == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = byte & 0x01 != 0;

        shifted
    }

    // BIT b, r
    // test bit b in register r
    fn bit(&mut self, r: u8, bit: u8) {
        let result: bool = r & (1 << bit) == 0;

        self.registers.f.zero = result;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }

    fn set(&mut self, byte: u8, bit: u8) -> u8 {
        byte | (1 << bit)
    }

    fn res(&mut self, byte: u8, bit: u8) -> u8 {
        byte & !(1 << bit)
    }

    fn rst(&mut self, opcode: u8) {
        self.stack_push(self.registers.pc);
        self.registers.pc = opcode as u16 & 0x38;
    }
    //STACK

    fn ret(&mut self) {
        let address = self.stack_pop();
        self.registers.pc = address;
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

pub const MEMORY_BUS_SIZE: usize = 65536;

pub struct MemoryBus {
    pub memory: [u8; MEMORY_BUS_SIZE],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; MEMORY_BUS_SIZE],
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if address == 0xFF02 && value == 0x81 {
            print!("{}", self.memory[0xFF01] as char);
        }
        self.memory[address as usize] = value;
    }

    pub fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).unwrap();
        self.memory[..bytes.len()].copy_from_slice(&bytes);
    }
}
