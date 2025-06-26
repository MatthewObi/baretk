use crate::decomp;
use crate::dis::{DisassemblySection};
use crate::prog::{Section, Program};
use crate::util::{i32_sign, BitExtr};

#[derive(PartialEq)]
#[derive(Copy, Clone)]
struct Register(u8);

impl Register {
    const ZERO: Register = Register(0x0);
    const RA: Register = Register(0x1);
    const SP: Register = Register(0x2);
    // const GP: Register = Register(0x3);
    // const TP: Register = Register(0x4);
    // const T0: Register = Register(0x5);
    // const T1: Register = Register(0x6);
    // const T2: Register = Register(0x7);
    const S0: Register = Register(0x8);
    // const S1: Register = Register(0x9);
    // const A0: Register = Register(0xa);
    // const A1: Register = Register(0xb);
    // const A2: Register = Register(0xc);
    // const A3: Register = Register(0xd);
    // const A4: Register = Register(0xe);
    // const A5: Register = Register(0xf);
    // const A6: Register = Register(0x10);
    // const A7: Register = Register(0x11);
    // const S2: Register = Register(0x12);
    // const S3: Register = Register(0x13);
    // const S4: Register = Register(0x14);
    // const S5: Register = Register(0x15);
    // const S6: Register = Register(0x16);
    // const S7: Register = Register(0x17);
    // const S8: Register = Register(0x18);
    // const S9: Register = Register(0x19);
    // const S10: Register = Register(0x1a);
    // const S11: Register = Register(0x1b);
    // const T3: Register = Register(0x1c);
    // const T4: Register = Register(0x1d);
    // const T5: Register = Register(0x1e);
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
    Sltu,
    Sll,
    Srl,
    Sra,
    Mul,
    Addi,
    Addiw,
    Andi,
    Ori,
    Xori,
    Slti,
    Sltui,
    Slli,
    Slliw,
    Srli,
    Srliw,
    Srai,
    Sraiw,
    Addw,
    Subw,
    Sllw,
    Srlw,
    Sraw,
    Mulw,
    Auipc,
    Lui,
    Li,
    Jal,
    Jalr,
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
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
    Unknown,
}

#[derive(Clone, Copy)]
enum Operand {
    Nothing,
    Reg(u8),
    // ImmU8(u8),
    ImmU16(u16),
    ImmU32(u32),
    // ImmU64(u64),
    // ImmS8(i8),
    ImmS16(i16),
    ImmS32(i32),
    // ImmS64(i64),
}

impl Operand {
    fn is_zero(self) -> bool {
        match self {
            Self::Reg(r) => r == 0,
            // Self::ImmU8(x) => x == 0,
            Self::ImmU16(x) => x == 0,
            Self::ImmU32(x) => x == 0,
            // Self::ImmU64(x) => x == 0,
            // Self::ImmS8(x) => x == 0,
            Self::ImmS16(x) => x == 0,
            Self::ImmS32(x) => x == 0,
            // Self::ImmS64(x) => x == 0,
            _ => false,
        }
    }

    fn is_register(self, reg: Register) -> bool {
        match self {
            Self::Reg(r) => r == reg.0,
            _ => false,
        }
    }

    fn value(self) -> i64 {
        match self {
            // Self::ImmS8(x) => x.into(),
            Self::ImmS16(x) => x.into(),
            Self::ImmS32(x) => x.into(),
            // Self::ImmS64(x) => x,
            _ => 0,
        }
    }

    fn print(self) -> String {
        match self {
            Self::Reg(r) => Register(r).name().to_string(),
            // Self::ImmU8(x) => x.to_string(),
            Self::ImmU16(x) => x.to_string(),
            Self::ImmU32(x) => x.to_string(),
            // Self::ImmU64(x) => x.to_string(),
            // Self::ImmS8(x) => x.to_string(),
            Self::ImmS16(x) => x.to_string(),
            Self::ImmS32(x) => x.to_string(),
            // Self::ImmS64(x) => x.to_string(),
            _ => "???".to_string(),
        }
    }

    fn into_expr(&self) -> Box<decomp::Expr> {
        match self {
            Self::Reg(r) => decomp::expr_register(Register(*r).name().to_string()),
            Self::ImmU16(x) => decomp::expr_constant(*x as i64),
            Self::ImmU32(x) => decomp::expr_constant(*x as i64),
            Self::ImmS16(x) => decomp::expr_constant(*x as i64),
            Self::ImmS32(x) => decomp::expr_constant(*x as i64),
            _ => decomp::expr_nop(),
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)] // TODO: Remove this and actually use unused fields
pub struct Instruction {
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
    pub fn print(self) -> String {
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
            Operation::Mul   => format!("mul {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Addi  => format!("addi {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Xori  => format!("xori {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Ori   => format!("ori {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Andi  => format!("andi {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Slti  => format!("slti {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Sltui => format!("sltui {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Addiw => if self.imm.is_zero() {
                format!("sext.w {}, {}", self.rd.print(), self.rs1.print())
            } else {
                format!("addiw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print())
            },
            Operation::Slli  => format!("slli {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Srli  => format!("srli {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Srai  => format!("srai {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Slliw => format!("slliw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Srliw => format!("srliw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Sraiw => format!("sraiw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Addw  => format!("addw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Subw  => format!("subw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sllw  => format!("sllw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Srlw  => format!("srlw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sraw  => format!("sraw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Mulw  => format!("mulw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Lbu   => format!("lbu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lhu   => format!("lhu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lwu   => format!("lwu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lb    => format!("lb {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lh    => format!("lh {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lw    => if self.imm.is_zero() { 
                format!("lw {}, [{}]", self.rd.print(), self.rs1.print()) 
            } else {
                format!("lw {}, [{} {} {}]", self.rd.print(), self.rs1.print(), i32_sign(self.imm.value() as i32), self.imm.print())
            },
            Operation::Ld    => format!("ld {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Sb    => if self.imm.is_zero() { 
                format!("sb {}, [{}]", self.rs1.print(), self.rs2.print())
            } else {
                format!("sb {}, [{} {} {}]", self.rs1.print(), self.rs2.print(), i32_sign(self.imm.value() as i32), self.imm.print())
            },
            Operation::Sh    => if self.imm.is_zero() { 
                format!("sh {}, [{}]", self.rs1.print(), self.rs2.print())
            } else {
                format!("sh {}, [{} {} {}]", self.rs1.print(), self.rs2.print(), i32_sign(self.imm.value() as i32), self.imm.print())
            },
            Operation::Sw    => if self.imm.is_zero() { 
                format!("sw {}, [{}]", self.rs1.print(), self.rs2.print())
            } else {
                format!("sw {}, [{} {} {}]", self.rs1.print(), self.rs2.print(), i32_sign(self.imm.value() as i32), self.imm.print())
            },
            Operation::Sd    => if self.imm.is_zero() { 
                format!("sd {}, [{}]", self.rs1.print(), self.rs2.print())
            } else {
                format!("sd {}, [{} {} {}]", self.rs1.print(), self.rs2.print(), i32_sign(self.imm.value() as i32), self.imm.print())
            },
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
                    if self.rs1.is_register(Register::RA) {
                        return format!("ret");
                    }
                    format!("jr {}", self.rs1.print())
                } else {
                    format!("jalr {}, {}", self.rd.print(), self.rs1.print())
                }
            },
            Operation::Beq   => {
                if self.rs2.is_zero() {
                    format!("beqz {}, {}", self.rs1.print(), self.imm.print())
                } else {
                    format!("beq {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print())
                }
            },
            Operation::Bne   => {
                if self.rs2.is_zero() {
                    format!("bnez {}, {}", self.rs1.print(), self.imm.print())
                } else {
                    format!("bne {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print())
                }
            },
            Operation::Blt   => format!("blt {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Bge   => format!("bge {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Bltu  => format!("bltu {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Bgeu  => format!("bgeu {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print()),
            Operation::Unknown => format!("???"),
            // _ => format!("unknown")
        }
    }

    pub fn offset(self) -> usize {
        self.offset
    }

    pub fn size(self) -> usize {
        self.ins_size as usize
    }

    pub fn into_expr(&self) -> Box<decomp::Expr> {
        match self.operation {
            Operation::Add   => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_ADD, self.rs1.into_expr(), self.rs2.into_expr())),
            Operation::Sub   => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_SUB, self.rs1.into_expr(), self.rs2.into_expr())),
            Operation::Xor   => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_XOR, self.rs1.into_expr(), self.rs2.into_expr())),
            Operation::And   => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_AND, self.rs1.into_expr(), self.rs2.into_expr())),
            Operation::Or    => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_OR, self.rs1.into_expr(), self.rs2.into_expr())),
            Operation::Slt   => decomp::expr_nop(), //format!("slt {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sltu  => decomp::expr_nop(), //format!("sltu {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sll   => decomp::expr_nop(), //format!("sll {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Srl   => decomp::expr_nop(), //format!("srl {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sra   => decomp::expr_nop(), //format!("sra {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Mul   => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_MUL, self.rs1.into_expr(), self.rs2.into_expr())),
            Operation::Addi  => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_ADD, self.rs1.into_expr(), self.imm.into_expr())),
            Operation::Xori  => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_XOR, self.rs1.into_expr(), self.imm.into_expr())),
            Operation::Ori   => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_OR, self.rs1.into_expr(), self.imm.into_expr())),
            Operation::Andi  => decomp::expr_store(self.rd.into_expr(), decomp::expr_binary(decomp::OP_AND, self.rs1.into_expr(), self.imm.into_expr())),
            Operation::Slti  => decomp::expr_nop(), // format!("slti {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Sltui => decomp::expr_nop(), // format!("sltui {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            // Operation::Addiw => if self.imm.is_zero() {
            //     format!("sext.w {}, {}", self.rd.print(), self.rs1.print())
            // } else {
            //     format!("addiw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print())
            // },
            Operation::Slli  => decomp::expr_nop(), //format!("slli {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Srli  => decomp::expr_nop(), //format!("srli {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Srai  => decomp::expr_nop(), //format!("srai {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Slliw => decomp::expr_nop(), //format!("slliw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Srliw => decomp::expr_nop(), //format!("srliw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Sraiw => decomp::expr_nop(), //format!("sraiw {}, {}, {}", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Addw  => decomp::expr_nop(), //format!("addw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Subw  => decomp::expr_nop(), //format!("subw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sllw  => decomp::expr_nop(), //format!("sllw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Srlw  => decomp::expr_nop(), //format!("srlw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Sraw  => decomp::expr_nop(), //format!("sraw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Mulw  => decomp::expr_nop(), //format!("mulw {}, {}, {}", self.rd.print(), self.rs1.print(), self.rs2.print()),
            Operation::Lbu   => decomp::expr_nop(), //format!("lbu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lhu   => decomp::expr_nop(), //format!("lhu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lwu   => decomp::expr_nop(), //format!("lwu {}, [{}{:+}]", self.rd.print(), self.rs1.print(), self.imm.print()),
            Operation::Lb    => if self.imm.is_zero() { 
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(1, self.rs2.into_expr()))
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs1.into_expr(), self.imm.into_expr());
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(1, rhs))
            },
            Operation::Lh    => if self.imm.is_zero() { 
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(2, self.rs1.into_expr()))
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs1.into_expr(), self.imm.into_expr());
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(2, rhs))
            },
            Operation::Lw    => if self.imm.is_zero() { 
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(4, self.rs1.into_expr()))
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs1.into_expr(), self.imm.into_expr());
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(4, rhs))
            },
            Operation::Ld    => if self.imm.is_zero() { 
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(8, self.rs1.into_expr()))
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs1.into_expr(), self.imm.into_expr());
                decomp::expr_store(self.rd.into_expr(), decomp::expr_dereference(8, rhs))
            },
            Operation::Sb    => if self.imm.is_zero() { 
                decomp::expr_store(decomp::expr_dereference(1, self.rs2.into_expr()), self.rs1.into_expr())
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs2.into_expr(), self.imm.into_expr());
                decomp::expr_store(decomp::expr_dereference(1, rhs), self.rs1.into_expr())
            },
            Operation::Sh    => if self.imm.is_zero() { 
                decomp::expr_store(decomp::expr_dereference(2, self.rs2.into_expr()), self.rs1.into_expr())
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs2.into_expr(), self.imm.into_expr());
                decomp::expr_store(decomp::expr_dereference(2, rhs), self.rs1.into_expr())
            },
            Operation::Sw    => if self.imm.is_zero() { 
                decomp::expr_store(decomp::expr_dereference(4, self.rs2.into_expr()), self.rs1.into_expr())
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs2.into_expr(), self.imm.into_expr());
                decomp::expr_store(decomp::expr_dereference(4, rhs), self.rs1.into_expr())
            },
            Operation::Sd    => if self.imm.is_zero() { 
                decomp::expr_store(decomp::expr_dereference(8, self.rs2.into_expr()), self.rs1.into_expr()) // format!("sd {}, [{}]", self.rs1.print(), self.rs2.print())
            } else {
                let rhs = decomp::expr_binary(decomp::OP_ADD, self.rs2.into_expr(), self.imm.into_expr());
                decomp::expr_store(decomp::expr_dereference(8, rhs), self.rs1.into_expr()) // format!("sd {}, [{} {} {}]", self.rs1.print(), self.rs2.print(), i32_sign(self.imm.value() as i32), self.imm.print())
            },
            Operation::Li    => decomp::expr_store(self.rd.into_expr(), self.imm.into_expr()), // format!("li {}, {}", self.rd.print(), self.imm.print()),
            Operation::Lui   => decomp::expr_store(self.rd.into_expr(), self.imm.into_expr()), // format!("lui {}, {}", self.rd.print(), self.imm.print()),
            Operation::Auipc => decomp::expr_store(self.rd.into_expr(), 
                decomp::expr_binary(decomp::OP_ADD,
                    decomp::expr_binary(decomp::OP_AND, 
                        decomp::expr_register(String::from("pc")), 
                        decomp::expr_constant(0xfffffffffff00000u64 as i64)), 
                    self.imm.into_expr())), // format!("auipc {}, {}", self.rd.print(), self.imm.print()),
            Operation::Jal   => {
                if self.rd.is_zero() {
                     decomp::expr_goto(decomp::expr_binary(decomp::OP_ADD, 
                        decomp::expr_register(String::from("pc")),
                        self.imm.into_expr()))
                } else {
                    if self.rd.is_register(Register::RA) {
                        return decomp::expr_call(decomp::expr_binary(decomp::OP_ADD, 
                            decomp::expr_register(String::from("pc")),
                            self.imm.into_expr()));
                    }
                    decomp::expr_special("jal", vec![
                        self.rd.into_expr(), 
                        decomp::expr_binary(decomp::OP_ADD, 
                            decomp::expr_register(String::from("pc")),
                            self.imm.into_expr())])
                }
            },
            Operation::Jalr  => {
                if self.rd.is_zero() {
                    if self.rs1.is_register(Register::RA) {
                        return decomp::expr_ret();
                    }
                    return decomp::expr_goto(self.rs1.into_expr())
                } else {
                    if self.rd.is_register(Register::RA) {
                        return decomp::expr_call(self.rs1.into_expr());
                    }
                    return decomp::expr_special("jal", vec![self.rd.into_expr(), self.rs1.into_expr()])
                }
            },
            Operation::Beq   => {
                if self.rs2.is_zero() {
                    decomp::expr_if(
                        decomp::expr_binary(decomp::OP_EQ, 
                            self.rs1.into_expr(),
                            decomp::expr_constant(0)),
                        decomp::expr_goto(decomp::expr_binary(decomp::OP_ADD, 
                            decomp::expr_register(String::from("pc")),
                            self.imm.into_expr())))
                } else {
                    decomp::expr_if(
                        decomp::expr_binary(decomp::OP_EQ, 
                            self.rs1.into_expr(),
                            self.rs2.into_expr()),
                        decomp::expr_goto(decomp::expr_binary(decomp::OP_ADD, 
                            decomp::expr_register(String::from("pc")),
                            self.imm.into_expr())))
                }
            },
            Operation::Bne   => {
                if self.rs2.is_zero() {
                    decomp::expr_if(
                        decomp::expr_binary(decomp::OP_NEQ, 
                            self.rs1.into_expr(),
                            decomp::expr_constant(0)),
                        decomp::expr_goto(decomp::expr_binary(decomp::OP_ADD, 
                            decomp::expr_register(String::from("pc")),
                            self.imm.into_expr())))
                } else {
                    decomp::expr_if(
                        decomp::expr_binary(decomp::OP_NEQ, 
                            self.rs1.into_expr(),
                            self.rs2.into_expr()),
                        decomp::expr_goto(decomp::expr_binary(decomp::OP_ADD, 
                            decomp::expr_register(String::from("pc")),
                            self.imm.into_expr())))
                }
            },
            Operation::Blt   => decomp::expr_if(
                decomp::expr_binary(decomp::OP_LT, 
                    self.rs1.into_expr(),
                    self.rs2.into_expr()),
                decomp::expr_goto(decomp::expr_binary(decomp::OP_ADD, 
                    decomp::expr_register(String::from("pc")),
                    self.imm.into_expr()))),
            Operation::Bge   => decomp::expr_if(
                decomp::expr_binary(decomp::OP_GTE, 
                    self.rs1.into_expr(),
                    self.rs2.into_expr()),
                decomp::expr_goto(decomp::expr_binary(decomp::OP_ADD, 
                    decomp::expr_register(String::from("pc")),
                    self.imm.into_expr()))),
            // Operation::Bltu  => format!("bltu {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print()),
            // Operation::Bgeu  => format!("bgeu {}, {}, {}", self.rs1.print(), self.rs2.print(), self.imm.print()),
            // Operation::Unknown => format!("???"),
            _ => decomp::expr_nop(), // format!("unknown")
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

fn instr_op_rd_rs1_shamt(op: Operation, ins: u32, offset: usize, ins_size: u8) -> Instruction {
    let rd = rd(ins) as u8;
    let rs1 = rs1(ins) as u8;
    let imm = shamt(ins);
    Instruction { operation: op, rd: Operand::Reg(rd), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmU32(imm), offset, ins_size }
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

fn shamt(ins: u32) -> u32 {
    (ins >> 20) & 0b11111
}

fn jimm20(ins: u32) -> i32 {
    ((((ins.bextr(31, 31) as i32) << 20) as u32) | (ins.bextr(30, 21) << 1)
    | (ins.bextr(20, 20) << 11) | (ins.bextr(19, 12) << 12)) as i32
}

fn branch(ins: u32) -> i32 {
    ((((ins.bextr(31, 31) as i32) << 12) as u32) | (ins.bextr(30, 25) << 5)
    | (ins.bextr(11, 8) << 1) | (ins.bextr(7, 7) << 11)) as i32
}

fn imm12(ins: u32) -> i32 {
    (ins as i32) >> 20
}

fn imm12_s(ins: u32) -> i32 {
    ((((ins as i32).bextr(31, 25) << 11) as u32) | (ins.bextr(11, 7))) as i32
}

fn csr(ins: u32) -> u32 {
    ins.bextr(31, 20)
}

fn instr_op_rd_imm20(op: Operation, ins: u32, offset: usize) -> Instruction {
    let rd = rd(ins) as u8;
    let imm = imm20(ins);
    Instruction { operation: op, rd: Operand::Reg(rd), rs1: Operand::Nothing, rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS32(imm), offset, ins_size: 4 }
}

fn instr_op_rd_jimm20(op: Operation, ins: u32, offset: usize) -> Instruction {
    let rd = rd(ins) as u8;
    let imm = jimm20(ins);
    Instruction { operation: op, rd: Operand::Reg(rd), rs1: Operand::Nothing, rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS32(imm), offset, ins_size: 4 }
}

fn instr_op_rs1_rs2_branch(op: Operation, ins: u32, offset: usize) -> Instruction {
    let rs1 = rs1(ins) as u8;
    let rs2 = rs2(ins) as u8;
    let imm = branch(ins);
    Instruction { operation: op, rd: Operand::Nothing, rs1: Operand::Reg(rs1), rs2: Operand::Reg(rs2), rs3: Operand::Nothing, imm: Operand::ImmS32(imm), offset, ins_size: 4 }
}

fn instr_op_rs1_csr(op: Operation, ins: u32, offset: usize) -> Instruction {
    let rd = rd(ins) as u8;
    let rs1 = rs1(ins) as u8;
    let imm = csr(ins);
    Instruction { operation: op, rd: Operand::Reg(rd), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmU32(imm), offset, ins_size: 4 }
}

fn disassemble_lui(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_imm20(Operation::Lui, ins, offset)
}

fn disassemble_auipc(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_imm20(Operation::Auipc, ins, offset)
}

fn disassemble_jal(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_jimm20(Operation::Jal, ins, offset)
}

fn disassemble_jalr(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Jalr, ins, offset, 4)
}

fn disassemble_beq(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_branch(Operation::Beq, ins, offset)
}

fn disassemble_bne(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_branch(Operation::Bne, ins, offset)
}

fn disassemble_blt(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_branch(Operation::Blt, ins, offset)
}

fn disassemble_bge(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_branch(Operation::Bge, ins, offset)
}

fn disassemble_bltu(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_branch(Operation::Bltu, ins, offset)
}

fn disassemble_bgeu(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_rs2_branch(Operation::Bgeu, ins, offset)
}

fn disassemble_addi(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Addi, ins, offset, 4)
}

fn disassemble_addiw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Addiw, ins, offset, 4)
}

fn disassemble_xori(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Xori, ins, offset, 4)
}

fn disassemble_ori(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Ori, ins, offset, 4)
}

fn disassemble_slti(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Slti, ins, offset, 4)
}

fn disassemble_sltui(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Sltui, ins, offset, 4)
}

fn disassemble_andi(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_imm12(Operation::Andi, ins, offset, 4)
}

fn disassemble_slli(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_shamt(Operation::Slli, ins, offset, 4)
}

fn disassemble_slliw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_shamt(Operation::Slliw, ins, offset, 4)
}

fn disassemble_srli(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_shamt(Operation::Srli, ins, offset, 4)
}

fn disassemble_srliw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_shamt(Operation::Srliw, ins, offset, 4)
}

fn disassemble_srai(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_shamt(Operation::Srai, ins, offset, 4)
}

fn disassemble_sraiw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_shamt(Operation::Sraiw, ins, offset, 4)
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

fn disassemble_mul(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Mul, ins, offset, 4)
}

fn disassemble_addw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Addw, ins, offset, 4)
}

fn disassemble_subw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Subw, ins, offset, 4)
}

fn disassemble_sllw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Sllw, ins, offset, 4)
}

fn disassemble_srlw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Srlw, ins, offset, 4)
}

fn disassemble_sraw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Sraw, ins, offset, 4)
}

fn disassemble_mulw(ins: u32, offset: usize) -> Instruction {
    instr_op_rd_rs1_rs2(Operation::Mulw, ins, offset, 4)
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

fn disassemble_csrrw(ins: u32, offset: usize) -> Instruction {
    instr_op_rs1_csr(Operation::Sd, ins, offset)
}

fn disassemble_32(ins: u32, offset: usize) -> Option<Instruction> {
    let opcode = opcode(ins);
    let funct3 = funct3(ins);
    match opcode {
        0b0110111 => Some(disassemble_lui(ins, offset)),
        0b0010111 => Some(disassemble_auipc(ins, offset)),
        0b1101111 => Some(disassemble_jal(ins, offset)),
        0b1100111 => Some(disassemble_jalr(ins, offset)),
        0b1100011 => {
            match funct3 {
                0b000 => Some(disassemble_beq(ins, offset)),
                0b001 => Some(disassemble_bne(ins, offset)),
                0b100 => Some(disassemble_blt(ins, offset)),
                0b101 => Some(disassemble_bge(ins, offset)),
                0b110 => Some(disassemble_bltu(ins, offset)),
                0b111 => Some(disassemble_bgeu(ins, offset)),
                _ => None
            }
        },
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
        0b0010011 => {
            match funct3 {
                0b000 => Some(disassemble_addi(ins, offset)),
                0b001 => Some(disassemble_slli(ins, offset)),
                0b010 => Some(disassemble_slti(ins, offset)),
                0b011 => Some(disassemble_sltui(ins, offset)),
                0b100 => Some(disassemble_xori(ins, offset)),
                0b101 => match funct7(ins) {
                    0b0000000 => Some(disassemble_srli(ins, offset)),
                    0b0100000 => Some(disassemble_srai(ins, offset)),
                    _ => None
                },
                0b110 => Some(disassemble_ori(ins, offset)),
                0b111 => Some(disassemble_andi(ins, offset)),
                _ => None
            }
        },
        0b0011011 => {
            match funct3 {
                0b000 => Some(disassemble_addiw(ins, offset)),
                0b001 => Some(disassemble_slliw(ins, offset)),
                0b101 => match funct7(ins) {
                    0b0000000 => Some(disassemble_srliw(ins, offset)),
                    0b0100000 => Some(disassemble_sraiw(ins, offset)),
                    _ => None
                },
                _ => None
            }
        },
        0b0110011 => {
            match funct3 {
                0b000 => match funct7(ins) {
                    0b0000000 => Some(disassemble_add(ins, offset)),
                    0b0000001 => Some(disassemble_mul(ins, offset)),
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
        },
        0b0111011 => {
            match funct3 {
                0b000 => match funct7(ins) {
                    0b0000000 => Some(disassemble_addw(ins, offset)),
                    0b0000001 => Some(disassemble_mulw(ins, offset)),
                    0b0100000 => Some(disassemble_subw(ins, offset)),
                    _ => None
                },
                0b001 => Some(disassemble_sllw(ins, offset)),
                0b101 => match funct7(ins) {
                    0b0000000 => Some(disassemble_srlw(ins, offset)),
                    0b0100000 => Some(disassemble_sraw(ins, offset)),
                    _ => None
                },
                _ => None
            }
        },
        0b1110011 => {
            match funct3 {
                0b001 => Some(disassemble_csrrw(ins, offset)),
                _ => None
            }
        },
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

fn c_rs2(ins: u16) -> u16 {
    (ins >> 2) & 0b11111
}

// ins[6,10:12,5]
fn c_uimm7(ins: u16) -> u16 {
    (ins.bextr(6, 5) << 2) | (ins.bextr(12, 10) << 3) | (ins.bextr(5, 4) << 6)
}

fn c_imm6(ins: u16) -> i16 {
    let sins = ins as i16;
    (ins.bextr(6, 2) as i16) | (sins.bextr(12, 11) << 4)
}

fn c_uimm8sp(ins: u16) -> u16 {
    (ins.bextr(6, 4) << 2) | (ins.bextr(3, 2) << 6) | (ins.bextr(12, 12) << 5)
}

fn c_uimm8sp_s(ins: u16) -> u16 {
    (ins.bextr(12, 9) << 2) | (ins.bextr(8, 7) << 6)
}

fn c_bimm9(ins: u16) -> i16 {
    ((((ins.bextr(12, 12) as i16) << 8) as u16) | (ins.bextr(11, 10) << 3)
    | (ins.bextr(6, 5) << 6) | (ins.bextr(4, 3) << 1) | (ins.bextr(2, 2) << 5)) as i16
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

fn disassemble_c_lui(ins: u16, offset: usize) -> Instruction {
    let rd = rd(ins as u32) as u8;
    let imm = ((ins.bextr(12, 11) as u32) << 17) | ((ins.bextr(6, 2) << 12) as u32);
    Instruction { operation: Operation::Lui, rd: Operand::Reg(rd), rs1: Operand::Nothing, rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmU32(imm), offset, ins_size: 2 }
}

fn disassemble_c_sub(ins: u16, offset: usize) -> Instruction {
    let rd = rd_rs2_p(ins) as u8 + Register::S0.0;
    let rs = rs1_p(ins) as u8 + Register::S0.0;
    Instruction { operation: Operation::Sub, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Reg(rs), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_xor(ins: u16, offset: usize) -> Instruction {
    let rd = rd_rs2_p(ins) as u8 + Register::S0.0;
    let rs = rs1_p(ins) as u8 + Register::S0.0;
    Instruction { operation: Operation::Xor, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Reg(rs), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_or(ins: u16, offset: usize) -> Instruction {
    let rd = rd_rs2_p(ins) as u8 + Register::S0.0;
    let rs = rs1_p(ins) as u8 + Register::S0.0;
    Instruction { operation: Operation::Or, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Reg(rs), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_and(ins: u16, offset: usize) -> Instruction {
    let rd = rd_rs2_p(ins) as u8 + Register::S0.0;
    let rs = rs1_p(ins) as u8 + Register::S0.0;
    Instruction { operation: Operation::And, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Reg(rs), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_addi(ins: u16, offset: usize) -> Instruction {
    let rd = rd(ins as u32) as u8;
    let imm = c_imm6(ins);
    Instruction { operation: Operation::Addi, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS16(imm), offset, ins_size: 2 }
}

fn disassemble_c_subw(ins: u16, offset: usize) -> Instruction {
    let rd = rd_rs2_p(ins) as u8 + Register::S0.0;
    let rs = rs1_p(ins) as u8 + Register::S0.0;
    Instruction { operation: Operation::Subw, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Reg(rs), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_addw(ins: u16, offset: usize) -> Instruction {
    let rd = rd_rs2_p(ins) as u8 + Register::S0.0;
    let rs = rs1_p(ins) as u8 + Register::S0.0;
    Instruction { operation: Operation::Addw, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Reg(rs), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_jr(ins: u16, offset: usize) -> Instruction {
    let rs1 = rd(ins as u32) as u8;
    Instruction { operation: Operation::Jalr, rd: Operand::Reg(Register::ZERO.0), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS16(0), offset, ins_size: 2 }
}

fn disassemble_c_jalr(ins: u16, offset: usize) -> Instruction {
    let rs1 = rd(ins as u32) as u8;
    Instruction { operation: Operation::Jalr, rd: Operand::Reg(Register::RA.0), rs1: Operand::Reg(rs1), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS16(0), offset, ins_size: 2 }
}

fn disassemble_c_mv(ins: u16, offset: usize) -> Instruction {
    let rd = rd(ins as u32) as u8;
    let rs2 = c_rs2(ins) as u8;
    Instruction { operation: Operation::Add, rd: Operand::Reg(rd), rs1: Operand::Reg(Register::ZERO.0), rs2: Operand::Reg(rs2), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_add(ins: u16, offset: usize) -> Instruction {
    let rd = rd(ins as u32) as u8;
    let rs2 = c_rs2(ins) as u8;
    Instruction { operation: Operation::Add, rd: Operand::Reg(rd), rs1: Operand::Reg(rd), rs2: Operand::Reg(rs2), rs3: Operand::Nothing, imm: Operand::Nothing, offset, ins_size: 2 }
}

fn disassemble_c_j(ins: u16, offset: usize) -> Instruction {
    let i = ins as i16;
    let imm = ((i.bextr(12, 12) << 11) as u16) | (ins.bextr(11, 11) << 4)
        | (ins.bextr(10, 9) << 8) | (ins.bextr(8, 8) << 10) | (ins.bextr(7, 7) << 6)
        | (ins.bextr(6, 6) << 7) | (ins.bextr(5, 3) << 1) | (ins.bextr(2, 2) << 5);
    Instruction { operation: Operation::Jal, rd: Operand::Reg(Register::ZERO.0), rs1: Operand::Nothing, rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmS16(imm as i16), offset, ins_size: 2 }
}

fn disassemble_c_lwsp(ins: u16, offset: usize) -> Instruction {
    let rd = rd(ins as u32) as u8;
    let imm = c_uimm8sp(ins);
    Instruction { operation: Operation::Lw, rd: Operand::Reg(rd), rs1: Operand::Reg(Register::SP.0), rs2: Operand::Nothing, rs3: Operand::Nothing, imm: Operand::ImmU16(imm), offset, ins_size: 2 }
}

fn disassemble_c_swsp(ins: u16, offset: usize) -> Instruction {
    let rs2 = c_rs2(ins) as u8;
    let imm = c_uimm8sp_s(ins);
    Instruction { operation: Operation::Sw, rd: Operand::Nothing, rs1: Operand::Reg(rs2), rs2: Operand::Reg(Register::SP.0), rs3: Operand::Nothing, imm: Operand::ImmU16(imm), offset, ins_size: 2 }
}

fn disassemble_c_beqz(ins: u16, offset: usize) -> Instruction {
    let rs1 = rs1_p(ins) as u8 + Register::S0.0;
    let imm = c_bimm9(ins);
    Instruction { operation: Operation::Beq, rd: Operand::Nothing, rs1: Operand::Reg(rs1), rs2: Operand::Reg(Register::ZERO.0), rs3: Operand::Nothing, imm: Operand::ImmS16(imm), offset, ins_size: 2 }
}

fn disassemble_c_bnez(ins: u16, offset: usize) -> Instruction {
    let rs1 = rs1_p(ins) as u8 + Register::S0.0;
    let imm = c_bimm9(ins);
    Instruction { operation: Operation::Bne, rd: Operand::Nothing, rs1: Operand::Reg(rs1), rs2: Operand::Reg(Register::ZERO.0), rs3: Operand::Nothing, imm: Operand::ImmS16(imm), offset, ins_size: 2 }
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
            0b000 => Some(disassemble_c_addi(ins, offset)),
            0b010 => Some(disassemble_c_li(ins, offset)),
            0b011 => match rd(ins.into()) {
                _ => Some(disassemble_c_lui(ins, offset)),
            },
            0b100 => match ins.bextr(11, 10) {
                0b11 => match ins.bextr(12, 12) {
                    0b0 => match ins.bextr(6, 5) {
                        0b00 => Some(disassemble_c_sub(ins, offset)),
                        0b01 => Some(disassemble_c_xor(ins, offset)),
                        0b10 => Some(disassemble_c_or(ins, offset)),
                        0b11 => Some(disassemble_c_and(ins, offset)),
                        _ => None,
                    },
                    0b1 => match ins.bextr(6, 5) {
                        0b00 => Some(disassemble_c_subw(ins, offset)),
                        0b01 => Some(disassemble_c_addw(ins, offset)),
                        _ => None,
                    },
                    _ => None
                },
                _ => None,
            },
            0b101 => Some(disassemble_c_j(ins, offset)),
            0b110 => Some(disassemble_c_beqz(ins, offset)),
            0b111 => Some(disassemble_c_bnez(ins, offset)),
            _ => None,
        },
        0b10 => match funct {
            0b010 => Some(disassemble_c_lwsp(ins, offset)),
            0b100 => match ins.bextr(12, 11) {
                0x0 => if c_rs2(ins) == 0 { 
                    Some(disassemble_c_jr(ins, offset))
                } else {
                    Some(disassemble_c_mv(ins, offset))
                },
                0x1 => if c_rs2(ins) == 0 {
                    Some(disassemble_c_jalr(ins, offset))
                } else {
                    Some(disassemble_c_add(ins, offset))
                },
                _ => None
            },
            0b110 => Some(disassemble_c_swsp(ins, offset)),
            _ => None,
        },
        _ => None,
    }
}

fn disassemble_instruction(bytes: &[u8], offset: usize) -> Option<Instruction> {
    let ins = u16::from_le_bytes(bytes[offset..offset+2].try_into().unwrap());
    if (ins & 3) == 3 {
        return disassemble_32(u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap()), offset)
    }
    disassemble_16(ins, offset)
}

pub fn disassemble_riscv(section: &Section, section_name: &String, _program: &Program) -> DisassemblySection {
    let mut instrs = Vec::<Instruction>::new();
    let mut offset: usize = 0;
    let bytes = section.bytes.as_slice();
    while offset + 2 <= bytes.len() {
        let instr = disassemble_instruction(bytes, offset);
        if instr.is_some() {
            let ins = instr.unwrap();
            offset += ins.ins_size as usize;
            instrs.push(ins);
        }
        else if offset + 4 <= bytes.len() && (u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap()) & 3) == 3 {
            instrs.push(Instruction { operation: Operation::Unknown,
                rd: Operand::Nothing,
                rs1: Operand::Nothing,
                rs2: Operand::Nothing,
                rs3: Operand::Nothing,
                imm: Operand::Nothing,
                offset,
                ins_size: 4});
            offset += 4;
        }
        else {
            instrs.push(Instruction { operation: Operation::Unknown,
                rd: Operand::Nothing,
                rs1: Operand::Nothing,
                rs2: Operand::Nothing,
                rs3: Operand::Nothing,
                imm: Operand::Nothing,
                offset,
                ins_size: 2});
            offset += 2;
        }
    }
    DisassemblySection {
        section_name: section_name.clone(),
        instructions: crate::dis::InstructionListing::Rv(instrs),
    }
}
