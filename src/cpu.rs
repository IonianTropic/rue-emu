use anyhow::{Result, Error};

use shared::cpu::*;

use crate::mem::Mem;

pub struct Cpu {
    pub registers: [Xlen; 32],
    pub pc: Xlen,
    pub mem: Mem,
}

impl Cpu {
    pub fn from_mem(mem: Mem) -> Self {
        Self {
            registers: [0; 32],
            pc: 0,
            mem,
        }
    }

    pub fn from_buf(buf: Vec<u8>) -> Self {
        Self::from_mem(Mem::from_buf(buf))
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            // 1. Fetch
            let instr = match self.fetch() {
                Ok(instr) => instr,
                Err(_) => break,
            };
            self.inc_pc();
            dbg!(format!("{instr:x}, {}", self.pc));
            // 2. Decode
            // 3. Execute
            // 4. Memory access
            // 5. Write back
            match self.execute(instr) {
                Ok(_) => (),
                Err(_) => break,
            }
        } 
        self.dump_registers();
        Ok(())
    }

    fn fetch(&self) -> Result<Ilen> {
        if self.pc % IALIGN != 0 {
            return Err(Error::msg(""));
        }
        let addr = self.pc as usize;
        Ok(self.mem.load_word(addr))
    }

    fn execute(&mut self, instr: Ilen) -> Result<()> {
        let opcode = instr & MASK_OPCODE;
        let rd = ((instr & MASK_RD) >> SHIFT_RD) as usize;
        let rs1 = ((instr & MASK_RS1) >> SHIFT_RS1) as usize;
        let rs2 = ((instr & MASK_RS2) >> SHIFT_RS2) as usize;
        let funct3 = (instr & MASK_FUNCT3) >> SHIFT_FUNCT3;
        let funct7 = (instr & MASK_FUNCT7) >> SHIFT_FUNCT7;
        match opcode {
            OPCODE_OP => {
                match (funct3, funct7) {
                    (0x0, 0x00) => self.add(rd, rs1, rs2),
                    (0x0, 0x20) => self.sub(rd, rs1, rs2),
                    (0x4, 0x00) => self.xor(rd, rs1, rs2),
                    (0x6, 0x00) => self.or(rd, rs1, rs2),
                    (0x7, 0x00) => self.and(rd, rs1, rs2),
                    (0x1, 0x00) => self.sll(rd, rs1, rs2),
                    (0x5, 0x00) => self.srl(rd, rs1, rs2),
                    (0x5, 0x20) => self.sra(rd, rs1, rs2),
                    (0x2, 0x00) => self.slt(rd, rs1, rs2),
                    (0x2, 0x00) => self.sltu(rd, rs1, rs2),
                    _ => return Err(Error::msg("Decode error: Op funct not implemented")),
                }
            }
            OPCODE_OP_IMM => {
                let imm = ((instr & MASK_I_IMM) as Ixlen >> SHIFT_RS2) as Xlen;
                let immu = (instr & MASK_I_IMM) >> SHIFT_RS2;
                match funct3 {
                    0x0 => self.addi(rd, rs1, imm),
                    0x4 => self.xori(rd, rs1, imm),
                    0x6 => self.ori(rd, rs1, imm),
                    0x7 => self.andi(rd, rs1, imm),
                    0x1 => {
                        if funct7 != 0x00 {
                            return Err(Error::msg("Decode error: Op-imm funct not implemented"));
                        }
                        self.slli(rd, rs1, imm);
                    }
                    0x5 => match funct7 {
                        0x00 => self.srli(rd, rs1, imm),
                        0x20 => self.srai(rd, rs1, imm),
                    }
                    0x2 => self.slti(rd, rs1, imm),
                    0x3 => self.sltui(rd, rs1, immu),
                    _ => return Err(Error::msg("Decode error: Op-imm funct not implemented")),
                }
            }
            OPCODE_LOAD => {
                let imm = ((instr & MASK_I_IMM) as Ixlen >> SHIFT_RS2) as Xlen;
                let addr = self.registers[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => self.lb(rd, addr),
                    0x1 => self.lh(rd, addr),
                    0x2 => self.lw(rd, addr),
                    0x4 => self.lbu(rd, addr),
                    0x5 => self.lhu(rd, addr),
                    _ => return Err(Error::msg("Decode error: Load funct not implemented")),
                };
            }
            OPCODE_STORE => {
                let imm = Self::get_s_imm(instr);
                let addr = self.registers[rs1].wrapping_add(imm);
                let value = self.registers[rs2];
                match funct3 {
                    0x0 => self.sb(addr, value),
                    0x1 => self.sh(addr, value),
                    0x2 => self.sw(addr, value),
                    _ => return Err(Error::msg("Decode error: Store funct not implemented")),
                }
            }
            OPCODE_BRANCH => {
                let imm;
                let immu;
                match funct3 {
                    0x0 => self.beq(rs1, rs2, imm),
                    0x1 => self.bne(rs1, rs2, imm),
                    0x4 => self.blt(rs1, rs2, imm),
                    0x5 => self.bge(rs1, rs2, imm),
                    0x6 => self.bltu(rs1, rs2, immu),
                    0x7 => self.begu(rs1, rs2, immu),
                    _ => return Err(Error::msg("Decode error: Branch funct not implemented")),
                }
            }
            OPCODE_JAL => (),
            OPCODE_JALR => (),
            OPCODE_LUI => (),
            OPCODE_AUIPC => (),
            OPCODE_FENCE => (),
            OPCODE_SYSTEM => (),
            _ => return Err(Error::msg(format!("Decode error: Opcode {:06b} not implemented", opcode))),
        };
        Ok(())
    }

    fn get_s_imm(instr: Ilen) -> Xlen {
        (((instr & MASK_FUNCT7) as Ixlen >> SHIFT_RS2) as Xlen) | ((instr & MASK_RD) >> SHIFT_RD)
    }

    fn dump_registers(&self) -> () {
        for (i, register) in self.registers.iter().enumerate() {
            println!("x{i}={register}");
        }
    }

    fn inc_pc(&mut self) -> () {
        self.pc = self.pc.wrapping_add(4);
        self.pc %= self.mem.len() as Xlen;
    }

    fn add(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = self.registers[rs1].wrapping_add(self.registers[rs2]);
    }

    fn sub(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = self.registers[rs1].wrapping_sub(self.registers[rs2]);
    }

    fn xor(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = self.registers[rs1] ^ self.registers[rs2];
    }

    fn or(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = self.registers[rs1] | self.registers[rs2];
    }

    fn and(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = self.registers[rs1] & self.registers[rs2];
    }

    fn sll(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = self.registers[rs1] << self.registers[rs2];
    }

    fn srl(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = self.registers[rs1] >> self.registers[rs2];
    }

    fn sra(&mut self, rd: usize, rs1: usize, rs2: usize) -> () {
        self.registers[rd] = (self.registers[rs1] as Ixlen >> self.registers[rs2] as Ixlen) as Xlen;
    }

    fn addi(&mut self, rd: usize, rs1: usize, imm: Xlen) -> () {
        self.registers[rd] = self.registers[rs1].wrapping_add(imm);
    }

    fn xori(&mut self, rd: usize, rs1: usize, imm: Xlen) -> () {
        self.registers[rd] = self.registers[rs1] ^ imm;
    }

    fn ori(&mut self, rd: usize, rs1: usize, imm: Xlen) -> () {
        self.registers[rd] = self.registers[rs1] | imm;
    }

    fn andi(&mut self, rd: usize, rs1: usize, imm: Xlen) -> () {
        self.registers[rd] = self.registers[rs1] & imm;
    }

    fn lb(&mut self, rd: usize, addr: Xlen) {
        let value = self.mem.load_byte(addr as usize);
        self.registers[rd] = value as i8 as Ixlen as Xlen;
    }

    fn lh(&mut self, rd: usize, addr: Xlen) {
        let value = self.mem.load_half(addr as usize);
        self.registers[rd] = value as i16 as Ixlen as Xlen;
    }

    fn lw(&mut self, rd: usize, addr: Xlen) {
        let value = self.mem.load_word(addr as usize);
        self.registers[rd] = value as Ixlen as Xlen;
    }

    fn lbu(&mut self, rd: usize, addr: Xlen) {
        let value = self.mem.load_byte(addr as usize);
        self.registers[rd] = value as Xlen;
    }

    fn lhu(&mut self, rd: usize, addr: Xlen) {
        let value = self.mem.load_half(addr as usize);
        self.registers[rd] = value as Xlen;
    }

    fn sb(&mut self, addr: Xlen, value: Xlen) {
        self.mem.store_byte(addr as usize, value as u8)
    }

    fn sh(&mut self, addr: Xlen, value: Xlen) {
        self.mem.store_half(addr as usize, value as u16)
    }

    fn sw(&mut self, addr: Xlen, value: Xlen) {
        self.mem.store_word(addr as usize, value as u32)
    }

    fn beq(&self, rs1: usize, rs2: usize, imm: Xlen) {
        todo!()
    }

    fn bne(&self, rs1: usize, rs2: usize, imm: u32) {
        todo!()
    }
}
