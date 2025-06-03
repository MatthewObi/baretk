use crate::dis::{self, DisassemblySection};
use crate::prog::{Section, Program};
use crate::util::i32_sign;

const AX: u8 = 0x0;
const CX: u8 = 0x1;
const DX: u8 = 0x2;
const BX: u8 = 0x3;
const SP: u8 = 0x4;
const BP: u8 = 0x5;
const SI: u8 = 0x6;
const DI: u8 = 0x7;
// const R8: u8 = 0x8;
// const R9: u8 = 0x9;
// const R10: u8 = 0xa;
// const R11: u8 = 0xb;
// const R12: u8 = 0xc;
// const R13: u8 = 0xd;
// const R14: u8 = 0xe;
// const R15: u8 = 0xf;

// const AH: u8 = 0x4;
// const CH: u8 = 0x5;
// const DH: u8 = 0x6;
// const BH: u8 = 0x7;

const OPCODE_ADD_BYTE_STR: u8 = 0x00;
const OPCODE_ADD_DWORD_STR: u8 = 0x01;
const OPCODE_ADD_BYTE_LD: u8 = 0x02;
const OPCODE_ADD_DWORD_LD: u8 = 0x03;
const OPCODE_ADD_AL_IMM8: u8 = 0x04;
const OPCODE_OR_BYTE_STR: u8 = 0x08;
const OPCODE_OR_DWORD_STR: u8 = 0x09;
const OPCODE_OR_BYTE_LD: u8 = 0x0a;
const OPCODE_OR_DWORD_LD: u8 = 0x0b;
const OPCODE_OR_AL_IMM8: u8 = 0x0c;
const OPCODE_ADC_BYTE_STR: u8 = 0x10;
const OPCODE_ADC_DWORD_STR: u8 = 0x11;
const OPCODE_ADC_BYTE_LD: u8 = 0x12;
const OPCODE_ADC_DWORD_LD: u8 = 0x13;
const OPCODE_ADC_AL_IMM8: u8 = 0x14;
const OPCODE_AND_BYTE_STR: u8 = 0x20;
const OPCODE_AND_DWORD_STR: u8 = 0x21;
const OPCODE_AND_BYTE_LD: u8 = 0x22;
const OPCODE_AND_DWORD_LD: u8 = 0x23;
const OPCODE_AND_AL_IMM8: u8 = 0x24;
const OPCODE_SUB_BYTE_STR: u8 = 0x28;
const OPCODE_SUB_DWORD_STR: u8 = 0x29;
const OPCODE_SUB_BYTE_LD: u8 = 0x2a;
const OPCODE_SUB_DWORD_LD: u8 = 0x2b;
const OPCODE_SUB_AL_IMM8: u8 = 0x2c;
const OPCODE_XOR_BYTE_STR: u8 = 0x30;
const OPCODE_XOR_DWORD_STR: u8 = 0x31;
const OPCODE_XOR_BYTE_LD: u8 = 0x32;
const OPCODE_XOR_DWORD_LD: u8 = 0x33;
const OPCODE_XOR_AL_IMM8: u8 = 0x34;
const OPCODE_CMP_BYTE_STR: u8 = 0x38;
const OPCODE_CMP_DWORD_STR: u8 = 0x39;
const OPCODE_CMP_BYTE_LD: u8 = 0x3a;
const OPCODE_CMP_DWORD_LD: u8 = 0x3b;
const OPCODE_CMP_AL_IMM8: u8 = 0x3c;
const OPCODE_REX_W: u8 = 0x48;
const OPCODE_PUSH_REG: u8 = 0x50;
const OPCODE_PUSH_RAX: u8 = OPCODE_PUSH_REG+AX;
const OPCODE_PUSH_RCX: u8 = OPCODE_PUSH_REG+CX;
const OPCODE_PUSH_RDX: u8 = OPCODE_PUSH_REG+DX;
const OPCODE_PUSH_RBX: u8 = OPCODE_PUSH_REG+BX;
const OPCODE_PUSH_RSP: u8 = OPCODE_PUSH_REG+SP;
const OPCODE_PUSH_RBP: u8 = OPCODE_PUSH_REG+BP;
const OPCODE_PUSH_RSI: u8 = OPCODE_PUSH_REG+SI;
const OPCODE_PUSH_RDI: u8 = OPCODE_PUSH_REG+DI;
const OPCODE_POP_REG: u8 = 0x58;
const OPCODE_POP_RAX: u8 = OPCODE_POP_REG+AX;
const OPCODE_POP_RCX: u8 = OPCODE_POP_REG+CX;
const OPCODE_POP_RDX: u8 = OPCODE_POP_REG+DX;
const OPCODE_POP_RBX: u8 = OPCODE_POP_REG+BX;
const OPCODE_POP_RSP: u8 = OPCODE_POP_REG+SP;
const OPCODE_POP_RBP: u8 = OPCODE_POP_REG+BP;
const OPCODE_POP_RSI: u8 = OPCODE_POP_REG+SI;
const OPCODE_POP_RDI: u8 = OPCODE_POP_REG+DI;
const OPCODE_OP_BYTE_IMM: u8 = 0x80;
const OPCODE_OP_DWORD_IMM: u8 = 0x83;
const OPCODE_TEST_BYTE_STR: u8 = 0x84;
const OPCODE_TEST_DWORD_STR: u8 = 0x85;
const OPCODE_MOV_BYTE_STR: u8 = 0x88;
const OPCODE_MOV_DWORD_STR: u8 = 0x89;
const OPCODE_MOV_BYTE_LD: u8 = 0x8a;
const OPCODE_MOV_DWORD_LD: u8 = 0x8b;
const OPCODE_NOP: u8 = 0x90;
const OPCODE_MOV_REG_IMM8: u8 = 0xb0;
const OPCODE_MOV_AL: u8 = OPCODE_MOV_REG_IMM8+AX;
const OPCODE_MOV_CL: u8 = OPCODE_MOV_REG_IMM8+CX;
const OPCODE_MOV_DL: u8 = OPCODE_MOV_REG_IMM8+DX;
const OPCODE_MOV_BL: u8 = OPCODE_MOV_REG_IMM8+BX;
const OPCODE_MOV_SP: u8 = OPCODE_MOV_REG_IMM8+SP;
const OPCODE_MOV_BP: u8 = OPCODE_MOV_REG_IMM8+BP;
const OPCODE_MOV_SIL: u8 = OPCODE_MOV_REG_IMM8+SI;
const OPCODE_MOV_DIL: u8 = OPCODE_MOV_REG_IMM8+DI;
const OPCODE_MOV_REG_IMM: u8 = 0xb8;
const OPCODE_MOV_RAX: u8 = OPCODE_MOV_REG_IMM+AX;
const OPCODE_MOV_RCX: u8 = OPCODE_MOV_REG_IMM+CX;
const OPCODE_MOV_RDX: u8 = OPCODE_MOV_REG_IMM+DX;
const OPCODE_MOV_RBX: u8 = OPCODE_MOV_REG_IMM+BX;
const OPCODE_MOV_RSP: u8 = OPCODE_MOV_REG_IMM+SP;
const OPCODE_MOV_RBP: u8 = OPCODE_MOV_REG_IMM+BP;
const OPCODE_MOV_RSI: u8 = OPCODE_MOV_REG_IMM+SI;
const OPCODE_MOV_RDI: u8 = OPCODE_MOV_REG_IMM+DI;
const OPCODE_RET: u8 = 0xc3;
const OPCODE_CALL: u8 = 0xe8;

const OPSIZE_BYTE: u8 = 0x0;
const OPSIZE_WORD: u8 = 0x1;
const OPSIZE_DWORD: u8 = 0x2;
const OPSIZE_QWORD: u8 = 0x3;

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Adc,
    Sub,
    Sbb,
    And,
    Or,
    Xor,
    Cmp,
    Test,
    Mov,
    Nop,
    Push,
    Pop,
    Ret,
    Call,
    Unknown,
}

const PREFIX_REX_W: u8 = 1;

#[derive(Clone, Copy)]
enum Operand {
    Nothing,
    ImmU8(u8),
    // ImmU16(u16),
    ImmU32(u32),
    ImmS8(i8),
    // ImmS32(i32),
    Reg8(u8),
    Reg8H(u8),
    Reg16(u8),
    Reg32(u8),
    Reg64(u8),
    PtrRegByte(u8, i32),
    PtrRegRegByte(u8, u8, u8),
    PtrRegRegWord(u8, u8, u8),
    PtrRegRegDword(u8, u8, u8),
    PtrRegRegQword(u8, u8, u8),
    PtrRegWord(u8, i32),
    PtrRegDword(u8, i32),
    PtrRegQword(u8, i32),
    PtrRelByte(u32),
    PtrRelWord(u32),
    PtrRelDword(u32),
    PtrRelQword(u32),
}

static REG_NAMES: [[&'static str; 5]; 16] = [
    ["al",   "ax",   "eax",  "rax", "al"],
    ["cl",   "cx",   "ecx",  "rcx", "cl"],
    ["dl",   "dx",   "edx",  "rdx", "dl"],
    ["bl",   "bx",   "ebx",  "rbx", "bl"],
    ["spl",  "sp",   "esp",  "rsp", "ah"],
    ["bpl",  "bp",   "ebp",  "rbp", "ch"],
    ["sil",  "si",   "esi",  "rsi", "dh"],
    ["dil",  "di",   "edi",  "rdi", "bh"],
    ["r8l",  "r8w",  "r8d",  "r8",  "r8l"],
    ["r9l",  "r9w",  "r9d",  "r9",  "r9l"],
    ["r10l", "r10w", "r10d", "r10", "r10l"],
    ["r11l", "r11w", "r11d", "r11", "r11l"],
    ["r12l", "r12w", "r12d", "r12", "r12l"],
    ["r13l", "r13w", "r13d", "r13", "r13l"],
    ["r14l", "r14w", "r14d", "r14", "r14l"],
    ["r15l", "r15w", "r15d", "r15", "r15l"],
];

fn print_reg(s: usize, x: u8) -> &'static str {
    REG_NAMES[x as usize][s]
}

impl Operand {
    fn print(self) -> String {
        match self {
            Self::ImmU8(x)  => format!("0x{:x}", x),
            // Self::ImmU16(x)  => format!("0x{:x}", x),
            Self::ImmU32(x)  => format!("0x{:x}", x),
            Self::ImmS8(x)  => format!("{}", x),
            // Self::ImmS32(x)  => format!("{}", x),
            Self::Reg8(x)  => format!("{}", print_reg(0x0, x)),
            Self::Reg8H(x) => format!("{}", print_reg(0x4, x)),
            Self::Reg16(x) => format!("{}", print_reg(0x1, x)),
            Self::Reg32(x) => format!("{}", print_reg(0x2, x)),
            Self::Reg64(x) => format!("{}", print_reg(0x3, x)),
            Self::PtrRegByte(reg, offset) => {
                if offset == 0x0 {
                    format!("BYTE PTR [{}]", print_reg(0x3, reg))
                } else {
                    format!("BYTE PTR [{}{}0x{:02x}]", print_reg(0x3, reg), i32_sign(offset), offset.abs())
                }
            },
            Self::PtrRegWord(reg, offset) => {
                if offset == 0x0 {
                    format!("WORD PTR [{}]", print_reg(0x3, reg))
                } else {
                    format!("WORD PTR [{}{}0x{:02x}]", print_reg(0x3, reg), i32_sign(offset), offset.abs())
                }
            },
            Self::PtrRegDword(reg, offset) => {
                if offset == 0x0 {
                    format!("DWORD PTR [{}]", print_reg(0x3, reg))
                } else {
                    format!("DWORD PTR [{}{}0x{:02x}]", print_reg(0x3, reg), i32_sign(offset), offset.abs())
                }
            },
            Self::PtrRegQword(reg, offset) => {
                if offset == 0x0 {
                    format!("QWORD PTR [{}]", print_reg(0x3, reg))
                } else {
                    format!("QWORD PTR [{}{}0x{:04x}]", print_reg(0x3, reg), i32_sign(offset), offset.abs())
                }
            },
            Self::PtrRelByte(rel) => format!("BYTE PTR [rip+0x{:08x}]", rel),
            Self::PtrRelWord(rel) => format!("WORD PTR [rip+0x{:08x}]", rel),
            Self::PtrRelDword(rel) => format!("DWORD PTR [rip+0x{:08x}]", rel),
            Self::PtrRelQword(rel) => format!("QWORD PTR [rip+0x{:08x}]", rel),
            Self::PtrRegRegByte(base, offset, mul) => {
                if mul == 0x1 {
                    format!("BYTE PTR [{}+{}]", print_reg(0x3, base), print_reg(0x3, offset))
                } else {
                    format!("BYTE PTR [{}+{}*{}]", print_reg(0x3, base), print_reg(0x3, offset), mul)
                }
            },
            Self::PtrRegRegWord(base, offset, mul) => {
                if mul == 0x1 {
                    format!("WORD PTR [{}+{}]", print_reg(0x3, base), print_reg(0x3, offset))
                } else {
                    format!("WORD PTR [{}+{}*{}]", print_reg(0x3, base), print_reg(0x3, offset), mul)
                }
            },
            Self::PtrRegRegDword(base, offset, mul) => {
                if mul == 0x1 {
                    format!("DWORD PTR [{}+{}]", print_reg(0x3, base), print_reg(0x3, offset))
                } else {
                    format!("DWORD PTR [{}+{}*{}]", print_reg(0x3, base), print_reg(0x3, offset), mul)
                }
            },
            Self::PtrRegRegQword(base, offset, mul) => {
                if mul == 0x1 {
                    format!("QWORD PTR [{}+{}]", print_reg(0x3, base), print_reg(0x3, offset))
                } else {
                    format!("QWORD PTR [{}+{}*{}]", print_reg(0x3, base), print_reg(0x3, offset), mul)
                }
            },
            _ => format!("???"),
        }
    }

    fn into(self) -> dis::Operand {
        match self {
            Self::Reg8(x)  => dis::Operand::Register(print_reg(0x0, x)),
            Self::Reg8H(x) => dis::Operand::Register(print_reg(0x4, x)),
            Self::Reg16(x) => dis::Operand::Register(print_reg(0x1, x)),
            Self::Reg32(x) => dis::Operand::Register(print_reg(0x2, x)),
            Self::Reg64(x) => dis::Operand::Register(print_reg(0x3, x)),
            Self::ImmU8(x) => dis::Operand::Immediate(x.into()),
            // Self::ImmU16(x) => dis::Operand::Immediate(x.into()),
            Self::ImmU32(x) => dis::Operand::Immediate(x.into()),
            Self::ImmS8(x) => dis::Operand::Immediate(x.into()),
            // Self::ImmS32(x) => dis::Operand::Immediate(x.into()),
            Self::PtrRegByte(reg, offset) => dis::Operand::Memory(print_reg(0x3, reg), "", offset.into(), 1),
            Self::PtrRegWord(reg, offset) => dis::Operand::Memory(print_reg(0x3, reg), "", offset.into(), 2),
            Self::PtrRegDword(reg, offset) => dis::Operand::Memory(print_reg(0x3, reg), "", offset.into(), 4),
            Self::PtrRegQword(reg, offset) => dis::Operand::Memory(print_reg(0x3, reg), "", offset.into(), 8),
            Self::PtrRelByte(rel) => dis::Operand::Memory(".", "", rel.into(), 1),
            Self::PtrRelWord(rel) => dis::Operand::Memory(".", "", rel.into(), 2),
            Self::PtrRelDword(rel) => dis::Operand::Memory(".", "", rel.into(), 4),
            Self::PtrRelQword(rel) => dis::Operand::Memory(".", "", rel.into(), 8),
            Self::PtrRegRegByte(base, offset, _mul) => dis::Operand::Memory(print_reg(0x3, base), print_reg(0x0, offset), 0x0, 1),
            Self::PtrRegRegWord(base, offset, _mul) => dis::Operand::Memory(print_reg(0x3, base), print_reg(0x1, offset), 0x0, 2),
            Self::PtrRegRegDword(base, offset, _mul) => dis::Operand::Memory(print_reg(0x3, base), print_reg(0x2, offset), 0x0, 4),
            Self::PtrRegRegQword(base, offset, _mul) => dis::Operand::Memory(print_reg(0x3, base), print_reg(0x3, offset), 0x0, 8),
            Self::Nothing => dis::Operand::Nothing,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Instruction {
    operation: Operation,
    reg1: Operand,
    reg2: Operand,
    offset: usize,
    ins_size: u8,
}

impl Instruction {
    pub fn print(self) -> String {
        match self.operation {
            Operation::Add  => format!("add {}, {}", self.reg1.print(), self.reg2.print()),
            Operation::Adc  => format!("adc {}, {}", self.reg1.print(), self.reg2.print()),
            Operation::Sub  => format!("sub {}, {}", self.reg1.print(), self.reg2.print()),
            Operation::Or   => format!("or {}, {}",  self.reg1.print(), self.reg2.print()),
            Operation::And  => format!("and {}, {}",  self.reg1.print(), self.reg2.print()),
            Operation::Xor  => format!("xor {}, {}",  self.reg1.print(), self.reg2.print()),
            Operation::Test => format!("test {}, {}",  self.reg1.print(), self.reg2.print()),
            Operation::Cmp  => format!("cmp {}, {}",  self.reg1.print(), self.reg2.print()),
            Operation::Mov  => format!("mov {}, {}",  self.reg1.print(), self.reg2.print()),
            Operation::Push => format!("push {}",    self.reg1.print()),
            Operation::Pop  => format!("pop {}",     self.reg1.print()),
            Operation::Nop  => format!("nop"),
            Operation::Ret  => format!("ret"),
            Operation::Call => format!("call {}", self.reg1.print()),
            Operation::Unknown => format!("(bad)"),
            _ => format!("unknown")
        }
    }

    pub fn offset(self) -> usize {
        self.offset
    }

    pub fn size(self) -> usize {
        self.ins_size as usize
    }

    pub fn into(&self) -> dis::Instruction {
        match self.operation {
            Operation::Add   => dis::Instruction { opcode: "add", operands: vec![self.reg1.into(), self.reg1.into(), self.reg2.into()], flags: 0 },
            Operation::Sub   => dis::Instruction { opcode: "sub", operands: vec![self.reg1.into(), self.reg1.into(), self.reg2.into()], flags: 0 },
            Operation::And   => dis::Instruction { opcode: "and", operands: vec![self.reg1.into(), self.reg1.into(), self.reg2.into()], flags: 0 },
            Operation::Or    => dis::Instruction { opcode: "or", operands: vec![self.reg1.into(), self.reg1.into(), self.reg2.into()], flags: 0 },
            Operation::Xor   => dis::Instruction { opcode: "xor", operands: vec![self.reg1.into(), self.reg1.into(), self.reg2.into()], flags: 0 },
            Operation::Mov   => dis::Instruction { opcode: "mov", operands: vec![self.reg1.into(), self.reg2.into()], flags: 0 },
            Operation::Call  => dis::Instruction { opcode: "call", operands: vec![self.reg1.into()], flags: 0 },
            Operation::Push  => dis::Instruction { opcode: "push", operands: vec![self.reg1.into()], flags: 0 },
            Operation::Pop   => dis::Instruction { opcode: "pop", operands: vec![self.reg1.into()], flags: 0 },
            Operation::Nop   => dis::Instruction { opcode: "nop", operands: vec![], flags: 0 },
            Operation::Ret   => dis::Instruction { opcode: "ret", operands: vec![], flags: 0 },
            _ => panic!(""),
        }
    }
}

fn ins_dest_src(foffset: usize, ins_size: u8, operation: Operation, dest: Operand, source: Operand) -> Instruction {
    Instruction { operation, reg1: dest, reg2: source, offset: foffset, ins_size }
}

fn ins_single_op(foffset: usize, ins_size: u8, operation: Operation, op: Operand) -> Instruction {
    Instruction { operation, reg1: op, reg2: Operand::Nothing, offset: foffset, ins_size }
}

// op dest:r8, source:r8
fn ins_regh_regh(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, dest: u8, source: u8) -> Instruction {
    match op_size {
        OPSIZE_BYTE  => ins_dest_src(foffset, ins_size, operation, Operand::Reg8H(dest), Operand::Reg8H(source)),
        OPSIZE_WORD  => ins_dest_src(foffset, ins_size, operation, Operand::Reg16(dest), Operand::Reg16(source)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg32(dest),Operand::Reg32(source)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg64(dest),Operand::Reg64(source)),
        _ => panic!("Invalid op size")
    }
}

// op dest:r8, source:imm8
fn ins_regh_imm8(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, dest: u8, source: i8) -> Instruction {
    match op_size {
        OPSIZE_BYTE  => ins_dest_src(foffset, ins_size, operation, Operand::Reg8H(dest), Operand::ImmS8(source)),
        OPSIZE_WORD  => ins_dest_src(foffset, ins_size, operation, Operand::Reg16(dest), Operand::ImmS8(source)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg32(dest),Operand::ImmS8(source)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg64(dest),Operand::ImmS8(source)),
        _ => panic!("Invalid op size")
    }
}

// op SIZE PTR [dest:r+offset:i], source:r
fn ins_preg_regh(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, dest: u8, offset: i32, source: u8) -> Instruction {
    match op_size {
        OPSIZE_BYTE  => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegByte(dest, offset), Operand::Reg8H(source)),
        OPSIZE_WORD  => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegWord(dest, offset), Operand::Reg16(source)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegDword(dest, offset), Operand::Reg32(source)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegQword(dest, offset), Operand::Reg64(source)),
        _ => panic!("Invalid op size")
    }
}

// op dest:r, SIZE PTR [source:r+offset:i]
fn ins_regh_preg(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, dest: u8, source: u8, offset: i32) -> Instruction {
    match op_size {
        OPSIZE_BYTE  => ins_dest_src(foffset, ins_size, operation, Operand::Reg8H(dest), Operand::PtrRegByte(source, offset)),
        OPSIZE_WORD  => ins_dest_src(foffset, ins_size, operation, Operand::Reg16(dest), Operand::PtrRegWord(source, offset)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg32(dest), Operand::PtrRegDword(source, offset)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg64(dest), Operand::PtrRegQword(source, offset)),
        _ => panic!("Invalid op size")
    }
}

// op dest:r, SIZE PTR [base:r+offset:r*mul:i]
fn ins_regh_pregreg(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, dest: u8, source: u8, offset: u8, mul: u8) -> Instruction {
    match op_size {
        OPSIZE_BYTE  => ins_dest_src(foffset, ins_size, operation, Operand::Reg8H(dest), Operand::PtrRegRegByte(source, offset, mul)),
        OPSIZE_WORD  => ins_dest_src(foffset, ins_size, operation, Operand::Reg16(dest), Operand::PtrRegRegWord(source, offset, mul)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg32(dest), Operand::PtrRegRegDword(source, offset, mul)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg64(dest), Operand::PtrRegRegQword(source, offset, mul)),
        _ => panic!("Invalid op size")
    }
}

// op SIZE PTR [base:r+offset:r*mul:i], source:r
fn ins_pregreg_regh(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, source: u8, dest: u8, offset: u8, mul: u8) -> Instruction {
    match op_size {
        OPSIZE_BYTE  => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegRegByte(source, offset, mul), Operand::Reg8H(source)),
        OPSIZE_WORD  => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegRegWord(source, offset, mul), Operand::Reg16(source)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegRegDword(source, offset, mul), Operand::Reg32(dest)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::PtrRegRegQword(source, offset, mul), Operand::Reg64(dest)),
        _ => panic!("Invalid op size")
    }
}

// op dest:r, SIZE PTR [ip+offset:i]
fn ins_regh_prel(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, dest: u8, offset: u32) -> Instruction {
    match op_size {
        OPSIZE_BYTE =>  ins_dest_src(foffset, ins_size, operation, Operand::Reg8H(dest), Operand::PtrRelByte(offset)),
        OPSIZE_WORD =>  ins_dest_src(foffset, ins_size, operation, Operand::Reg16(dest), Operand::PtrRelWord(offset)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg32(dest), Operand::PtrRelDword(offset)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::Reg64(dest), Operand::PtrRelQword(offset)),
        _ => panic!("Invalid op size")
    }
}

// op SIZE PTR [ip+offset:i], source:r
fn ins_prel_regh(foffset: usize, ins_size: u8, operation: Operation, op_size: u8, source: u8, offset: u32) -> Instruction {
    match op_size {
        OPSIZE_BYTE =>  ins_dest_src(foffset, ins_size, operation, Operand::PtrRelByte(offset), Operand::Reg8H(source)),
        OPSIZE_WORD =>  ins_dest_src(foffset, ins_size, operation, Operand::PtrRelWord(offset), Operand::Reg16(source)),
        OPSIZE_DWORD => ins_dest_src(foffset, ins_size, operation, Operand::PtrRelDword(offset),Operand::Reg32(source)),
        OPSIZE_QWORD => ins_dest_src(foffset, ins_size, operation, Operand::PtrRelQword(offset),Operand::Reg64(source)),
        _ => panic!("Invalid op size")
    }
}

fn disassemble_x86_op_op(operation: Operation, bytes: &[u8], offset: usize, op_size: u8, swap_operands: bool) -> Option<Instruction> {
    if offset + 1 >= bytes.len() {
        return None
    }
    let x = bytes[offset+1];
    if x & 0b11000000 == 0 {
        let source = (x >> 3) & 0b111;
        let op2 = x & 0b111;
        if op2 == 0x4 {
            let y = bytes[offset+2];
            let reg2 = (y >> 3) & 0b111;
            let reg1 = y & 0b111;
            let mul = (y >> 6) & 0b11;
            if swap_operands {
                return Some(ins_regh_pregreg(offset, 3, operation, op_size, source, reg1, reg2, mul))
            }
            else {
                return Some(ins_pregreg_regh(offset, 3, operation, op_size, source, reg1, reg2, mul))
            }
        }
        else if op2 == 0x5 {
            let rel = u32::from_le_bytes([bytes[offset+2], bytes[offset+3], bytes[offset+4], bytes[offset+5]]);
            if swap_operands {
                return Some(ins_regh_prel(offset, 6, operation, op_size, source, rel))
            }
            else {
                return Some(ins_prel_regh(offset, 6, operation, op_size, source, rel))
            }
        }
        else {
            let dest = match x & 0b111 {
                0x0 => AX,
                0x1 => CX,
                0x2 => DX,
                0x3 => BX,
                0x6 => SI,
                0x7 => DI,
                _ => DI,
            };
            if swap_operands {
                return Some(ins_regh_preg(offset, 2, operation, op_size, source, dest, 0x0))
            } else {
                return Some(ins_preg_regh(offset, 2, operation, op_size, dest, 0x0, source))
            }
        }
    }
    else if x & 0b11000000 == 0b01000000 {
        let source = (x >> 3) & 0b111;
        let op2 = x & 0b111;
        let o = if bytes[offset+2] & 0x80 != 0 { -(0x100 - bytes[offset+2] as i32) } else { bytes[offset+2] as i32 };
        if swap_operands {
            return Some(ins_regh_preg(offset, 3, operation, op_size, source, op2, o))
        } else {
            return Some(ins_preg_regh(offset, 3, operation, op_size, op2, o, source))
        }
    }
    else if x & 0b11000000 == 0b11000000 {
        let dest = (x >> 3) & 0b111;
        let source = x & 0b111;
        if swap_operands {
            return Some(ins_regh_regh(offset, 2, operation, op_size, source, dest))
        }
        else {
            return Some(ins_regh_regh(offset, 2, operation, op_size, dest, source))
        }
    }
    None
}

fn disassemble_x86_al_imm8(operation: Operation, bytes: &[u8], offset: usize) -> Option<Instruction> {
    let imm = bytes[offset+1];
    Some(ins_dest_src(offset, 2, operation, Operand::Reg8(AX), Operand::ImmU8(imm)))
}

fn disassemble_x86_op_imm(bytes: &[u8], offset: usize, op_size: u8, _swap_operands: bool) -> Option<Instruction> {
    if offset + 1 >= bytes.len() {
        return None
    }
    let x = bytes[offset+1];
    let operation = match (x >> 3) & 0b111 {
        0x0 => Operation::Add,
        0x1 => Operation::Or,
        0x2 => Operation::Adc,
        0x3 => Operation::Sbb,
        0x4 => Operation::And,
        0x5 => Operation::Sub,
        0x6 => Operation::Xor,
        0x7 => Operation::Cmp,
        _ => return None
    };
    if x & 0b11000000 == 0b11000000 {
        let source = bytes[offset+2] as i8;
        let dest = x & 0b111;
        return Some(ins_regh_imm8(offset, 3, operation, op_size, dest, source))
    }
    None
}

fn disassemble_x86_push_pop(operation: Operation, bytes: &[u8], offset: usize) -> Option<Instruction> {
    let imm = bytes[offset] - match operation { Operation::Push => OPCODE_PUSH_REG, Operation::Pop => OPCODE_POP_REG, _ => 0 };
    Some(ins_single_op(offset, 1, operation, Operand::Reg64(imm)))
}

fn disassemble_x86_branch_imm(operation: Operation, bytes: &[u8], offset: usize, op_size: u8) -> Option<Instruction> {
    match op_size {
        OPSIZE_BYTE => {
            let imm = bytes[offset+1] as i8;
            Some(ins_single_op(offset, 2, operation, Operand::ImmS8(imm + 2)))
        },
        OPSIZE_DWORD => {
            let imm = u32::from_le_bytes([bytes[offset+1], bytes[offset+2], bytes[offset+3], bytes[offset+4]]);
            Some(ins_single_op(offset, 5, operation, Operand::ImmU32(imm + 5)))
        },
        _ => None
    }
}

fn disassemble_x86_mov_imm(bytes: &[u8], offset: usize, op_size: u8) -> Option<Instruction> {
    let reg = bytes[offset] - match op_size { OPSIZE_BYTE => OPCODE_MOV_REG_IMM8, _ => OPCODE_MOV_REG_IMM };
    match op_size {
        OPSIZE_BYTE  => {
            let imm = bytes[offset+1];
            Some(ins_dest_src(offset, 2, Operation::Mov, Operand::Reg8(reg), Operand::ImmU8(imm)))
        },
        OPSIZE_DWORD => {
            let imm = u32::from_le_bytes([bytes[offset+1], bytes[offset+2], bytes[offset+3], bytes[offset+4]]);
            Some(ins_dest_src(offset, 5, Operation::Mov, Operand::Reg32(reg), Operand::ImmU32(imm)))
        },
        _ => None
    }
}

fn rex_w_qword_or_dword(prefix: u8) -> u8 {
    if (prefix & PREFIX_REX_W) != 0 { OPSIZE_QWORD } else { OPSIZE_DWORD }
}

fn disassemble_x86_instruction(bytes: &[u8], offset: usize, prefix: u8) -> Option<Instruction> {
    if offset >= bytes.len() {
        return None
    }
    let opcode = bytes[offset];
    match opcode {
        OPCODE_REX_W => {
            let ins = disassemble_x86_instruction(bytes, offset + 1, prefix | PREFIX_REX_W);
            if ins.is_some() {
                let mut ins_ = ins.unwrap();
                ins_.ins_size += 1;
                ins_.offset = offset;
                return Some(ins_);
            }
            return None
        }
        _ => (),
    };
    match opcode {
        OPCODE_ADD_BYTE_STR  => disassemble_x86_op_op(Operation::Add, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_ADD_DWORD_STR => disassemble_x86_op_op(Operation::Add, bytes, offset, OPSIZE_DWORD, false),
        OPCODE_ADD_BYTE_LD   => disassemble_x86_op_op(Operation::Add, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_ADD_DWORD_LD  => disassemble_x86_op_op(Operation::Add, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_ADD_AL_IMM8   => disassemble_x86_al_imm8(Operation::Add, bytes, offset),
        OPCODE_OR_BYTE_STR   => disassemble_x86_op_op(Operation::Or, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_OR_DWORD_STR  => disassemble_x86_op_op(Operation::Or, bytes, offset, OPSIZE_DWORD, false),
        OPCODE_OR_BYTE_LD    => disassemble_x86_op_op(Operation::Or, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_OR_DWORD_LD   => disassemble_x86_op_op(Operation::Or, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_OR_AL_IMM8    => disassemble_x86_al_imm8(Operation::Or, bytes, offset),
        OPCODE_ADC_BYTE_STR  => disassemble_x86_op_op(Operation::Adc, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_ADC_DWORD_STR => disassemble_x86_op_op(Operation::Adc, bytes, offset, OPSIZE_DWORD, false),
        OPCODE_ADC_BYTE_LD   => disassemble_x86_op_op(Operation::Adc, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_ADC_DWORD_LD  => disassemble_x86_op_op(Operation::Adc, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_ADC_AL_IMM8   => disassemble_x86_al_imm8(Operation::Adc, bytes, offset),
        OPCODE_AND_BYTE_STR  => disassemble_x86_op_op(Operation::And, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_AND_DWORD_STR => disassemble_x86_op_op(Operation::And, bytes, offset, OPSIZE_DWORD, false),
        OPCODE_AND_BYTE_LD   => disassemble_x86_op_op(Operation::And, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_AND_DWORD_LD  => disassemble_x86_op_op(Operation::And, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_AND_AL_IMM8   => disassemble_x86_al_imm8(Operation::And, bytes, offset),
        OPCODE_SUB_BYTE_STR  => disassemble_x86_op_op(Operation::Sub, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_SUB_DWORD_STR => disassemble_x86_op_op(Operation::Sub, bytes, offset, OPSIZE_DWORD, false),
        OPCODE_SUB_BYTE_LD   => disassemble_x86_op_op(Operation::Sub, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_SUB_DWORD_LD  => disassemble_x86_op_op(Operation::Sub, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_SUB_AL_IMM8   => disassemble_x86_al_imm8(Operation::Sub, bytes, offset),
        OPCODE_XOR_BYTE_STR  => disassemble_x86_op_op(Operation::Xor, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_XOR_DWORD_STR => disassemble_x86_op_op(Operation::Xor, bytes, offset, OPSIZE_DWORD, false),
        OPCODE_XOR_BYTE_LD   => disassemble_x86_op_op(Operation::Xor, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_XOR_DWORD_LD  => disassemble_x86_op_op(Operation::Xor, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_XOR_AL_IMM8   => disassemble_x86_al_imm8(Operation::Xor, bytes, offset),
        OPCODE_CMP_BYTE_STR  => disassemble_x86_op_op(Operation::Cmp, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_CMP_DWORD_STR => disassemble_x86_op_op(Operation::Cmp, bytes, offset, OPSIZE_DWORD, false),
        OPCODE_CMP_BYTE_LD   => disassemble_x86_op_op(Operation::Cmp, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_CMP_DWORD_LD  => disassemble_x86_op_op(Operation::Cmp, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_CMP_AL_IMM8   => disassemble_x86_al_imm8(Operation::Cmp, bytes, offset),
        OPCODE_PUSH_RAX      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_PUSH_RCX      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_PUSH_RDX      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_PUSH_RBX      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_PUSH_RSP      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_PUSH_RBP      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_PUSH_RSI      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_PUSH_RDI      => disassemble_x86_push_pop(Operation::Push, bytes, offset),
        OPCODE_POP_RAX       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_POP_RCX       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_POP_RDX       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_POP_RBX       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_POP_RSP       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_POP_RBP       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_POP_RSI       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_POP_RDI       => disassemble_x86_push_pop(Operation::Pop, bytes, offset),
        OPCODE_OP_BYTE_IMM   => disassemble_x86_op_imm(bytes, offset, OPSIZE_BYTE, false),
        OPCODE_OP_DWORD_IMM   => disassemble_x86_op_imm(bytes, offset, rex_w_qword_or_dword(prefix), false),
        OPCODE_TEST_BYTE_STR  => disassemble_x86_op_op(Operation::Mov, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_TEST_DWORD_STR => disassemble_x86_op_op(Operation::Test, bytes, offset, rex_w_qword_or_dword(prefix), false),
        OPCODE_MOV_BYTE_STR  => disassemble_x86_op_op(Operation::Mov, bytes, offset, OPSIZE_BYTE, false),
        OPCODE_MOV_DWORD_STR => disassemble_x86_op_op(Operation::Mov, bytes, offset, rex_w_qword_or_dword(prefix), false),
        OPCODE_MOV_BYTE_LD   => disassemble_x86_op_op(Operation::Mov, bytes, offset, OPSIZE_BYTE, true),
        OPCODE_MOV_DWORD_LD  => disassemble_x86_op_op(Operation::Mov, bytes, offset, OPSIZE_DWORD, true),
        OPCODE_NOP           => Some(Instruction { operation: Operation::Nop, reg1: Operand::Nothing, reg2: Operand::Nothing, offset, ins_size: 1 }),
        OPCODE_MOV_AL        => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_CL        => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_DL        => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_BL        => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_SP        => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_BP        => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_SIL       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_DIL       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_BYTE),
        OPCODE_MOV_RAX       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_MOV_RCX       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_MOV_RDX       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_MOV_RBX       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_MOV_RSP       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_MOV_RBP       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_MOV_RSI       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_MOV_RDI       => disassemble_x86_mov_imm(bytes, offset, OPSIZE_DWORD),
        OPCODE_RET           => Some(Instruction { offset, ins_size: 1, operation: Operation::Ret, reg1: Operand::Nothing, reg2: Operand::Nothing }),
        OPCODE_CALL         => disassemble_x86_branch_imm(Operation::Call, bytes, offset, OPSIZE_DWORD),
        _ => None
    }
}

pub fn disassemble_x86(section: &Section, section_name: &String, _program: &Program) -> DisassemblySection {
    let mut offset = 0x0;
    // let bytes = &[
    //     0x50u8,
    //     0x31, 0xc0,
    //     0x89, 0x47, 0xf4,
    //     0x58,
    //     0x90,
    //     0xc3
    // ];
    let mut instrs = Vec::<Instruction>::new();
    let bytes = section.bytes.as_slice();
    while offset < bytes.len() { 
        let res = disassemble_x86_instruction(bytes, offset, 0);
        if res.is_some() {
            let ins = res.unwrap();
            offset += ins.ins_size as usize;
            instrs.push(ins);
        }
        else {
            instrs.push(Instruction {
                operation: Operation::Unknown, 
                reg1: Operand::Nothing, 
                reg2: Operand::Nothing, 
                offset, ins_size: 1});
            offset += 1;
        }
    }
    DisassemblySection {
        section_name: section_name.clone(),
        instructions: crate::dis::InstructionListing::X86(instrs)
    }
}
