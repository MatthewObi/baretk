use crate::prog::{Section, Program};
use crate::util::BitExtr;

#[derive(PartialEq)]
#[derive(Copy, Clone)]
struct Register(u8);

impl Register {
    const ZERO: Register = Register(0x0);
    const RA: Register = Register(0x1);
    const SP: Register = Register(0x2);
    const GP: Register = Register(0x3);
    const TP: Register = Register(0x4);
    const T0: Register = Register(0x5);
    const T1: Register = Register(0x6);
    const T2: Register = Register(0x7);
    const S0: Register = Register(0x8);
    const S1: Register = Register(0x9);
    const A0: Register = Register(0xa);
    const A1: Register = Register(0xb);
    const A2: Register = Register(0xc);
    const A3: Register = Register(0xd);
    const A4: Register = Register(0xe);
    const A5: Register = Register(0xf);
    const A6: Register = Register(0x10);
    const A7: Register = Register(0x11);
    const S2: Register = Register(0x12);
    const S3: Register = Register(0x13);
    const S4: Register = Register(0x14);
    const S5: Register = Register(0x15);
    const S6: Register = Register(0x16);
    const S7: Register = Register(0x17);
    const S8: Register = Register(0x18);
    const S9: Register = Register(0x19);
    const S10: Register = Register(0x1a);
    const S11: Register = Register(0x1b);
    const T3: Register = Register(0x1c);
    const T4: Register = Register(0x1d);
    const T5: Register = Register(0x1e);
    const T6: Register = Register(0x1f);
    const COUNT: usize = Self::T6.0 as usize + 1;

    const REG_NAMES: [&'static str; Self::COUNT] = [
        "Zero",
        "ra",
        "sp",
        "gp",
        "tp",
        "t0",
        "t1",
        "t2",
        "s0",
        "s1",
        "a0",
        "a1",
        "a2",
        "a3",
        "a4",
        "a5",
        "a6",
        "a7",
        "s2",
        "s3",
        "s4",
        "s5",
        "s6",
        "s7",
        "s8",
        "s9",
        "s10",
        "s11",
        "t3",
        "t4",
        "t5",
        "t6",
    ];

    fn name(self) -> &'static str {
        if (self.0 as usize) < Self::REG_NAMES.len() {
            return Self::REG_NAMES[self.0 as usize]
        }
        "?"
    }
}

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Slt,
    Sge,
    Sltu,
    Sgeu,
    Sll,
    Srl,
    Sra,
    Auipc,
    Lui,
    Li,
    Jal,
    Jalr,
    Lbu,
    Lb,
    Lhu,
    Lh,
    Lwu,
    Lw,
    Ld,
    Sb,
    Sh,
    Sw,
    Sd,
    Ret,
}

#[derive(Clone, Copy)]
enum Operand {
    Nothing,
    Reg(u8),
    ImmU8(u8),
    ImmU16(u16),
    ImmU32(u32),
    ImmU64(u64),
    ImmS8(i8),
    ImmS16(i16),
    ImmS32(i32),
    ImmS64(i64),
}

impl Operand {
    fn is_zero(self) -> bool {
        match self {
            Self::Reg(r) => r == 0,
            Self::ImmU8(x) => x == 0,
            Self::ImmU16(x) => x == 0,
            Self::ImmU32(x) => x == 0,
            Self::ImmU64(x) => x == 0,
            Self::ImmS8(x) => x == 0,
            Self::ImmS16(x) => x == 0,
            Self::ImmS32(x) => x == 0,
            Self::ImmS64(x) => x == 0,
            _ => false,
        }
    }

    fn print(self) -> String {
        match self {
            Self::Reg(r) => Register(r).name().to_string(),
            Self::ImmU8(x) => x.to_string(),
            Self::ImmU16(x) => x.to_string(),
            Self::ImmU32(x) => x.to_string(),
            Self::ImmU64(x) => x.to_string(),
            Self::ImmS8(x) => x.to_string(),
            Self::ImmS16(x) => x.to_string(),
            Self::ImmS32(x) => x.to_string(),
            Self::ImmS64(x) => x.to_string(),
            _ => "???".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
struct Instruction {
    operation: Operation,
    rd: Operand,
    rs1: Operand,
    rs2: Operand,
    rs3: Operand,
    imm: Operand,
    offset: usize,
    ins_size: u8,
}

impl Instruction {
    fn print(self) -> String {
        match self.operation {
            Operation::Add   => format!("add {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sub   => format!("sub {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Xor   => format!("xor {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::And   => format!("and {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Or    => format!("or {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Slt   => format!("slt {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sltu  => format!("sltu {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sll   => format!("sll {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Srl   => format!("srl {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sra   => format!("sra {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Lbu   => format!("lbu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lhu   => format!("lhu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lwu   => format!("lwu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lb    => format!("lb {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lh    => format!("lh {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lw    => format!("lw {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Ld    => format!("ld {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Sb    => format!("sb {}, [{}{:+}]", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Sh    => format!("sh {}, [{}{:+}]", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Sw    => format!("sw {}, [{}{:+}]", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Sd    => format!("sd {}, [{}{:+}]", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Li    => format!("li {}, {}", self.rd.print(), self.imm.print()),
            Operation::Lui   => format!("lui {}, {}", self.rd.print(), self.imm.print()),
            Operation::Auipc => format!("auipc {}, {}", self.rd.print(), self.imm.print()),
            Operation::Jal   => {
                if self.rd.is_zero() {
                    format!("j {}", self.imm.print())
                } else {
                    format!("jal {}, {}", self.rd.print(), self.imm.print())
                }
            },
            Operation::Jalr  => {
                if self.rd.is_zero() {
                    format!("jr {}", self.rs1.print())
                } else {
                    format!("jalr {}, {}", self.rd.print(), self.rs1.print())
                }
            },
            Operation::Ret   => format!("ret"),
            _ => format!("unknown")
        }
    }
}

fn instr_op_rd_rs1_rs2(op: Operation, ins: u32, offset: usize, ins_size: u8) -> Instruction {
    let rd = rd(ins) as u8;
    let rs1 = rs1(ins) as u8;
    let rs2 = rs2(ins) as u8;
    Instruction { operation: op, rd: Operand::Reg(rd), rs1: Operand::Reg(rs1), rs2: Operand::Reg(rs2), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size }
}

fn instr_op_rd_rs1_imm12(op: Operation, ins: u32, offset: usize, ins_size: u8) -> Instruction {
    let rd = rd(ins) as u8;
    let rs1 = rs1(ins) as u8;
    let imm = imm12(ins);
    Instruction { operation: op, rd: Operand::Reg(rd), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS32(imm), offset, ins_size }
}

fn instr_op_rs1_rs2_imm12_s(op: Operation, ins: u32, offset: usize, ins_size: u8) -> Instruction {
    let rs1 = rs1(ins) as u8;
    let rs2 = rs2(ins) as u8;
    let imm = imm12_s(ins);
    Instruction { operation: op, rd: Operand::Nothing, rs1: Operand::Reg(rs1), rs2: Operand::Reg(rs2), rs3: Operand::Nothing, imm: Operand::ImmS32(imm), offset, ins_size }
}

fn instr_op_rd_rs(op: Operation, rd: Register, rs: Register, offset: usize, ins_size: u8) -> Instruction {
    Instruction { operation: op, rd: Operand::Reg(rd.0), rs1: Operand::Reg(rs.0), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size }
}

fn opcode(ins: u32) -> u32 {
    ins & 0x7f
}

fn rd(ins: u32) -> u32 {
    (ins >> 7) & 0b11111
}

fn rs1(ins: u32) -> u32 {
    (ins >> 15) & 0b11111
}

fn rs2(ins: u32) -> u32 {
    (ins >> 20) & 0b11111
}

fn funct3(ins: u32) -> u32 {
    (ins >> 12) & 0b111
}

fn funct7(ins: u32) -> u32 {
    ins >> 25
}

fn imm20(ins: u32) -> i32 {
    (ins as i32) >> 12
}

fn imm12(ins: u32) -> i32 {
    (ins as i32) >> 20
}

fn imm12_s(ins: u32) -> i32 {
    (((ins as i32) >> 20) << 5) | (ins as i32 & 0b11111)
}

fn instr_op_rd_imm20(op: Operation, ins: u32, offset: usize) -> Instruction {
    let rd = rd(ins) as u8;
    let imm = imm20(ins);
    Instruction { operation: op, rd: Operand::Reg(rd), rs1: Operand::Nothing, rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS32(imm), offset: offset, ins_size: 4 }
}

fn disassemble_lui(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_imm20(Operation::Lui, ins, offset)
}

fn disassemble_auipc(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_imm20(Operation::Auipc, ins, offset)
}

fn disassemble_add(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Add, ins, offset, 4)
}

fn disassemble_sub(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Sub, ins, offset, 4)
}

fn disassemble_xor(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Xor, ins, offset, 4)
}

fn disassemble_and(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::And, ins, offset, 4)
}

fn disassemble_or(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Or, ins, offset, 4)
}

fn disassemble_slt(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Slt, ins, offset, 4)
}

fn disassemble_sltu(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Sltu, ins, offset, 4)
}

fn disassemble_sll(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Sll, ins, offset, 4)
}

fn disassemble_srl(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Srl, ins, offset, 4)
}

fn disassemble_sra(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Sra, ins, offset, 4)
}

fn disassemble_lb(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Lb, ins, offset, 4)
}

fn disassemble_lbu(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Lbu, ins, offset, 4)
}

fn disassemble_lh(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Lh, ins, offset, 4)
}

fn disassemble_lhu(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Lhu, ins, offset, 4)
}

fn disassemble_lw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Lw, ins, offset, 4)
}

fn disassemble_lwu(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Lwu, ins, offset, 4)
}

fn disassemble_ld(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Ld, ins, offset, 4)
}

fn disassemble_sb(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_imm12_s(Operation::Sb, ins, offset, 4)
}

fn disassemble_sh(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_imm12_s(Operation::Sh, ins, offset, 4)
}

fn disassemble_sw(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_imm12_s(Operation::Sw, ins, offset, 4)
}

fn disassemble_sd(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_imm12_s(Operation::Sd, ins, offset, 4)
}

fn disassemble_32(ins: u32, offset: usize) -> Option<Instruction> {
    let opcode = opcode(ins);
    let funct3 = funct3(ins);
    match opcode {
        0b0110111 => Some(disassemble_lui(ins, offset)),
        0b0010111 => Some(disassemble_auipc(ins, offset)),
        0b0000011 => {
            match funct3 {
                0b000 => Some(disassemble_lb(ins, offset)),
                0b001 => Some(disassemble_lh(ins, offset)),
                0b010 => Some(disassemble_lw(ins, offset)),
                0b011 => Some(disassemble_ld(ins, offset)),
                0b100 => Some(disassemble_lbu(ins, offset)),
                0b101 => Some(disassemble_lhu(ins, offset)),
                0b110 => Some(disassemble_lwu(ins, offset)),
                _ => None
            }
        },
        0b0100011 => {
            match funct3 {
                0b000 => Some(disassemble_sb(ins, offset)),
                0b001 => Some(disassemble_sh(ins, offset)),
                0b010 => Some(disassemble_sw(ins, offset)),
                0b011 => Some(disassemble_sd(ins, offset)),
                _ => None
            }
        },
        0b0110011 => {
            match funct3 {
                0b000 => match funct7(ins) {
                    0b0000000 => Some(disassemble_add(ins, offset)),
                    0b0100000 => Some(disassemble_sub(ins, offset)),
                    _ => None
                },
                0b001 => Some(disassemble_sll(ins, offset)),
                0b010 => Some(disassemble_slt(ins, offset)),
                0b011 => Some(disassemble_sltu(ins, offset)),
                0b100 => Some(disassemble_xor(ins, offset)),
                0b101 => match funct7(ins) {
                    0b0000000 => Some(disassemble_srl(ins, offset)),
                    0b0100000 => Some(disassemble_sra(ins, offset)),
                    _ => None
                },
                0b110 => Some(disassemble_or(ins, offset)),
                0b111 => Some(disassemble_and(ins, offset)),
                _ => None
            }
        }
        _ => None
    }
}

// rd' = ins[2:4]
fn rd_rs2_p(ins: u16) -> u16 {
    (ins >> 2) & 0b111
}

// rs1' = ins[7:9]
fn rs1_p(ins: u16) -> u16 {
    (ins >> 7) & 0b111
}

// ins[6,10:12,5]
fn c_uimm7(ins: u16) -> u16 {
    (ins.bextr(6, 1) << 2) | (ins.bextr(12, 3) << 3) | (ins.bextr(5, 1) << 6)
}

fn c_imm6(ins: u16) -> i16 {
    let sins = ins as i16;
    (ins.bextr(6, 4) as i16) | (sins.bextr(12, 1) << 4)
}

fn disassemble_c_lw(ins: u16, offset: usize) -> Instruction {
    let rd = rd_rs2_p(ins) as u8 + Register::S0.0;
    let rs1 = rs1_p(ins) as u8 + Register::S0.0;
    let imm = c_uimm7(ins);
    Instruction { operation: Operation::Lw, rd: Operand::Reg(rd), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmU16(imm), offset, ins_size: 2 }
}

fn disassemble_c_li(ins: u16, offset: usize) -> Instruction {
    let rd = rd(ins as u32) as u8;
    let imm = c_imm6(ins);
    Instruction { operation: Operation::Li, rd: Operand::Reg(rd), rs1: Operand::Nothing, rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS16(imm), offset, ins_size: 2 }
}

fn disassemble_c_jr(ins: u16, offset: usize) -> Instruction {
    let rs1 = rd(ins as u32) as u8;
    Instruction { operation: Operation::Jalr, rd: Operand::Reg(Register::ZERO.0), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS16(0), offset, ins_size: 2 }
}

fn disassemble_c_jalr(ins: u16, offset: usize) -> Instruction {
    let rs1 = rd(ins as u32) as u8;
    Instruction { operation: Operation::Jalr, rd: Operand::Reg(Register::RA.0), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS16(0), offset, ins_size: 2 }
}

fn disassemble_16(ins: u16, offset: usize) -> Option<Instruction> {
    let op = ins & 3;
    let funct = (ins >> 13) & 7;
    match op {
        0b00 => match funct {
            0b010 => Some(disassemble_c_lw(ins, offset)),
            _ => None,
        },
        0b01 => match funct {
            0b010 => Some(disassemble_c_li(ins, offset)),
            _ => None,
        },
        0b10 => match funct {
            0b100 => {
                match ins.bextr(12, 1) {
                    0 => Some(disassemble_c_jr(ins, offset)),
                    1 => Some(disassemble_c_jalr(ins, offset)),
                    _ => None
                }
            },
            _ => None,
        },
        _ => None,
    }
}

fn disassemble_instruction(bytes: &[u8], offset: usize) -> Option<Instruction> {
    let ins = u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap());
    if (ins & 3) == 3 {
        return disassemble_32(ins, offset)
    }
    disassemble_16(u16::from_le_bytes(bytes[offset..offset+2].try_into().unwrap()), offset)
}

pub fn disassemble_riscv(section: &Section, program: &Program) -> String {
    let mut out = String::new();
    let mut offset: usize = 0;
    let bytes = section.bytes.as_slice();
    while offset + 4 < 16 {
        let instr = disassemble_instruction(bytes, offset);
        if instr.is_some() {
            let ins = instr.unwrap();
            offset += ins.ins_size as usize;
            out += (ins.print() + "\n").as_str();
        }
        else {
            out += format!("??? ({:02X} {:02X})\n", bytes[offset], bytes[offset + 1]).as_str();
            offset += 2;
        }
    }
    println!("{}", out);
    format!("TODO: RISC-V stuff")
}
