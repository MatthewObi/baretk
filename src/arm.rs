use crate::{dis::{DisassemblySection}, prog::{Program, Section}};
use crate::util::BitExtr;

fn cond(x: u32) -> u32 {
    x.bextr(31, 28)
}

fn fstr(x: bool) -> &'static str {
    return if x { "s" } else { "" }
}

fn wbstr(x: bool) -> &'static str {
    return if x { "!" } else { "" }
}

const COND_EQ: u32 = 0b0000;
const COND_NE: u32 = 0b0001;
const COND_CS: u32 = 0b0010;
const COND_CC: u32 = 0b0011;
const COND_MI: u32 = 0b0100;
const COND_PL: u32 = 0b0101;
const COND_VS: u32 = 0b0110;
const COND_VC: u32 = 0b0111;
const COND_HI: u32 = 0b1000;
const COND_LS: u32 = 0b1001;
const COND_GE: u32 = 0b1010;
const COND_LT: u32 = 0b1011;
const COND_GT: u32 = 0b1100;
const COND_LE: u32 = 0b1101;
const COND_AL: u32 = 0b1110;

fn condstr(cond: u32) -> &'static str {
    match cond {
        COND_EQ => "eq",
        COND_NE => "ne",
        COND_CS => "cs",
        COND_CC => "cc",
        COND_MI => "mi",
        COND_PL => "pl",
        COND_VS => "vs",
        COND_VC => "vc",
        COND_HI => "hi",
        COND_LS => "ls",
        COND_GE => "ge",
        COND_LT => "lt",
        COND_GT => "gt",
        COND_LE => "le",
        COND_AL => "",
        _       => "??",
    }
}

fn opcode(x: u32) -> u32 {
    x.bextr(24, 21)
}

const ST_LSL: u8 = 0b00;
const ST_LSR: u8 = 0b01;
const ST_ASR: u8 = 0b10;
const ST_ROR: u8 = 0b11;

fn shtystr(x: u8) -> &'static str {
    match x {
        ST_LSL => "lsl",
        ST_LSR => "lsr",
        ST_ASR => "asr",
        ST_ROR => "ror",
        _ => "?"
    }
}

const BLTAM_DA: u8 = 0b00;
const BLTAM_IA: u8 = 0b01;
const BLTAM_DB: u8 = 0b10;
const BLTAM_IB: u8 = 0b11;

fn bltamstr(x: u8) -> &'static str {
    match x {
        BLTAM_DA => "da",
        BLTAM_IA => "ia",
        BLTAM_DB => "db",
        BLTAM_IB => "ib",
        _ => "?"
    }
}

const PSR_CPSR: u8 = 0;
const PSR_SPSR: u8 = 1;

const PSR_MODE_ALL: u8 = 0;
const PSR_MODE_FLAG: u8 = 1;
const PSR_MODE_C: u8 = 2;

const REG_SP: u8 = 13;
const REG_LR: u8 = 14;
const REG_PC: u8 = 15;

#[derive(Clone, Copy)]
enum Operand {
    Reg(u8, u8, u8),
    Imm(u32, u8, u8),
    RegList(u32),
    Psr(u8, u8),
    SImm(i32),
}

impl Operand {
    fn print(self) -> String {
        match self {
            Self::Reg(r, s, st) => {
                if s != 0 {
                    format!("r{} {} #{}", r, shtystr(st), s)
                }
                else {
                    if r == REG_SP {
                        format!("sp")
                    }
                    else if r == REG_LR {
                        format!("lr")
                    }
                    else if r == REG_PC {
                        format!("pc")
                    }
                    else {
                        format!("r{}", r)
                    }
                }
            },
            Self::RegList(mut x) => {
                let mut out = String::from("{");
                for i in 0..16 {
                    if (x & (1 << i)) != 0 {
                        if i == REG_LR {
                            out += "lr";
                        }
                        else {
                            out += format!("r{}", i).as_str();
                        }
                        x &= !(1 << i);
                        if x == 0 {
                            break;
                        }
                        out += ", ";
                    }
                }
                out += "}";
                out
            },
            Self::Imm(x, s, st) => {
                if s != 0 {
                    format!("#{} {} #{}", x, shtystr(st), s)
                }
                else {
                    format!("#{}", x)
                }
            },
            Self::SImm(x) => format!("#{}", x),
            Self::Psr(which, state) => {
                match which {
                    PSR_CPSR => match state {
                        PSR_MODE_FLAG => format!("CPSR_flg"),
                        PSR_MODE_C => format!("CPSR_c"),
                        _ => format!("CPSR"),
                    },
                    PSR_SPSR => match state {
                        PSR_MODE_FLAG => format!("SPSR_flg"),
                        PSR_MODE_C => format!("SPSR_c"),
                        _ => format!("SPSR"),
                    },
                    _ => format!("???"),
                }
            },
        }
    }

    fn value(self) -> i64 {
        match self {
            Self::Imm(x, s, st) => {
                if s != 0 {
                    match st {
                        ST_LSL => ((x as u64) << s) as i64,
                        ST_LSR => ((x as u64) >> s) as i64,
                        ST_ASR => ((x as i32) as i64) >> s,
                        ST_ROR => x.rotate_right(s.into()) as i64,
                        _ => x as i64
                    }
                }
                else {
                    x as i64
                }
            },
            Self::SImm(x) => x as i64,
            _ => 0,
        }
    }
}

fn op2(x: u32) -> Operand {
    if (x & (1 << 25)) == 0 {
        Operand::Reg(x.bextr(3, 0) as u8, x.bextr(11, 8) as u8, x.bextr(6, 5) as u8)
    }
    else {
        Operand::Imm(x.bextr(7, 0), x.bextr(11, 7) as u8, x.bextr(6, 5) as u8)
    }
}

fn rn(x: u32) -> Operand {
    Operand::Reg(x.bextr(19, 16) as u8, 0, 0)
}

fn rd(x: u32) -> Operand {
    Operand::Reg(x.bextr(15, 12) as u8, 0, 0)
}

fn bl_offset(x: u32) -> i32 {
    ((x as i32).bextr(23, 0)) << 2
}

enum Opcode {
    Unknown,
    Bx(Operand),
    B(Operand),
    Bl(Operand),
    Mul(Operand, Operand, Operand),
    MulA(Operand, Operand, Operand, Operand),
    Mrs(Operand, Operand),
    Msr(Operand, Operand),
    Ldr(Operand, Operand, Operand),
    Str(Operand, Operand, Operand),
    Ldm(Operand, Operand, bool, u8),
    Stm(Operand, Operand, bool, u8),
    And(Operand, Operand, Operand),
    Eor(Operand, Operand, Operand),
    Sub(Operand, Operand, Operand),
    Rsb(Operand, Operand, Operand),
    Add(Operand, Operand, Operand),
    Adc(Operand, Operand, Operand),
    Sbc(Operand, Operand, Operand),
    Rsc(Operand, Operand, Operand),
    Tst(Operand, Operand),
    Teq(Operand, Operand),
    Cmp(Operand, Operand),
    Cmn(Operand, Operand),
    Orr(Operand, Operand, Operand),
    Mov(Operand, Operand),
    Bic(Operand, Operand, Operand),
    Mvn(Operand, Operand),
    Swi,
}

pub struct Instruction {
    opcode: Opcode,
    offset: usize,
    cond: u32,
    set_flags: bool,
    ins_size: u8,
}

impl Instruction {
    pub fn print(&self) -> String {
        match self.opcode {
            Opcode::Swi                                           => format!("swi{}", condstr(self.cond)),
            Opcode::Bx(rn)                               => format!("bx{} {}", condstr(self.cond), rn.print()),
            Opcode::B(rn)                                => format!("b{} _{:08x}", condstr(self.cond), self.offset as i64 + 0x8 + rn.value()),
            Opcode::Bl(rn)                               => format!("bl{} _{:08x}", condstr(self.cond), self.offset as i64 + 0x8 + rn.value()),
            Opcode::Mrs(rm, psr)                => format!("mrs{} {}, {}", condstr(self.cond), rm.print(), psr.print()),
            Opcode::Msr(psr, rm)                => format!("msr{} {}, {}", condstr(self.cond), psr.print(), rm.print()),
            Opcode::Mul(rd, rm, rs)    => format!("mul{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), rm.print(), rs.print()),
            Opcode::MulA(rd, rm, rs, rn)    => format!("mla{}{} {}, {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), rm.print(), rs.print(), rn.print()),
            Opcode::Str(rn, op1, op2)  => format!("str{} {}, [{}, {}]", condstr(self.cond), rn.print(), op1.print(), op2.print()),
            Opcode::Ldr(rn, op1, op2)  => format!("ldr{} {}, [{}, {}]", condstr(self.cond), rn.print(), op1.print(), op2.print()),
            Opcode::And(rd, op1, op2)  => format!("and{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Eor(rd, op1, op2)  => format!("eor{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Sub(rd, op1, op2)  => format!("sub{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Rsb(rd, op1, op2)  => format!("rsb{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Add(rd, op1, op2)  => format!("add{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Adc(rd, op1, op2)  => format!("adc{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Sbc(rd, op1, op2)  => format!("sbc{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Rsc(rd, op1, op2)  => format!("rsc{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Tst(op1, op2)               => format!("tst{} {}, {}", condstr(self.cond), op1.print(), op2.print()),
            Opcode::Teq(op1, op2)               => format!("teq{} {}, {}", condstr(self.cond), op1.print(), op2.print()),
            Opcode::Cmp(op1, op2)               => format!("cmp{} {}, {}", condstr(self.cond), op1.print(), op2.print()),
            Opcode::Cmn(op1, op2)               => format!("cmn{} {}, {}", condstr(self.cond), op1.print(), op2.print()),
            Opcode::Orr(rd, op1, op2)  => format!("orr{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Mov(rd, op)  =>  {
                let dst = match rd { Operand::Reg(r, _, _) => Some(r), _ => None };
                let src = match op { Operand::Reg(r, _, _) => Some(r), _ => None };
                if src.is_some() && dst.is_some() && src == dst {
                    format!("nop")
                }
                else {
                    format!("mov{} {}, {}", condstr(self.cond), rd.print(), op.print())
                }
            },
            Opcode::Bic(rd, op1, op2)  => format!("bic{}{} {}, {}, {}", condstr(self.cond), fstr(self.set_flags), rd.print(), op1.print(), op2.print()),
            Opcode::Mvn(rd, op)  => format!("mvn{} {}, {}", condstr(self.cond), rd.print(), op.print()),
            Opcode::Stm(rn, op, wb, am)  => {
                let base = match rn { Operand::Reg(r, _, _) => r, _ => 0 };
                if base == REG_SP && wb && am == BLTAM_DB && self.cond == COND_AL {
                    format!("push {}", op.print())
                } else {
                    format!("stm{}{} {}{}, {}", condstr(self.cond), bltamstr(am), rn.print(), wbstr(wb), op.print())
                }
            }
            Opcode::Ldm(rn, op, wb, am)  => {
                let base = match rn { Operand::Reg(r, _, _) => r, _ => 0 };
                if base == REG_SP && wb && am == BLTAM_IA && self.cond == COND_AL {
                    format!("pop {}", op.print())
                } else {
                    format!("ldm{}{} {}{}, {}", condstr(self.cond), bltamstr(am), rn.print(), wbstr(wb), op.print())
                }
            }
            Opcode::Unknown     => format!("???"),
            // _ => format!("unknown")
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn size(&self) -> usize {
        self.ins_size as usize
    }
}

const OPCODE_AND: u32 = 0b0000;
const OPCODE_EOR: u32 = 0b0001;
const OPCODE_SUB: u32 = 0b0010;
const OPCODE_RSB: u32 = 0b0011;
const OPCODE_ADD: u32 = 0b0100;
const OPCODE_ADC: u32 = 0b0101;
const OPCODE_SBC: u32 = 0b0110;
const OPCODE_RSC: u32 = 0b0111;
const OPCODE_TST: u32 = 0b1000;
const OPCODE_TEQ: u32 = 0b1001;
const OPCODE_CMP: u32 = 0b1010;
const OPCODE_CMN: u32 = 0b1011;
const OPCODE_ORR: u32 = 0b1100;
const OPCODE_MOV: u32 = 0b1101;
const OPCODE_BIC: u32 = 0b1110;
const OPCODE_MVN: u32 = 0b1111;

fn disassemble_arm_ins(ins: u32, offset: usize) -> Option<Instruction> {
    let cond = cond(ins);
    let opcode = opcode(ins);
    if ins.bextr(27, 4) == 0b000100101111111111110001 {
        return Some(Instruction {opcode: Opcode::Bx(Operand::Reg(ins.bextr(3, 0) as u8, 0, 0)), offset, cond, set_flags: false, ins_size: 4})
    }
    if ins.bextr(27, 25) == 0b101 {
        if (ins & (1 << 24)) == 0 {
            return Some(Instruction {opcode: Opcode::B(Operand::SImm(bl_offset(ins))), offset, cond, set_flags: false, ins_size: 4})
        } else {
            return Some(Instruction {opcode: Opcode::Bl(Operand::SImm(bl_offset(ins))), offset, cond, set_flags: false, ins_size: 4})
        }
    }
    if ins.bextr(27, 25) == 0b100 {
        let base = rn(ins);
        let wb = (ins & (1 << 21)) != 0;
        let am = ins.bextr(24, 23) as u8;
        let reg_list = Operand::RegList(ins.bextr(15, 0));
        if (ins & (1 << 20)) == 0 {
            return Some(Instruction {opcode: Opcode::Stm(base, reg_list, wb, am), offset, cond, set_flags: false, ins_size: 4})
        } else {
            return Some(Instruction {opcode: Opcode::Ldm(base, reg_list, wb, am), offset, cond, set_flags: false, ins_size: 4})
        }
    }
    if ins.bextr(27, 22) == 0b000000 {
        let accumulate = (ins & (1 << 21)) != 0;
        let set_flags = (ins & (1 << 20)) != 0;
        let rd = Operand::Reg(ins.bextr(19, 16) as u8, 0, 0);
        let rn = Operand::Reg(ins.bextr(15, 12) as u8, 0, 0);
        let rs = Operand::Reg(ins.bextr(11, 8) as u8, 0, 0);
        let rm = Operand::Reg(ins.bextr(3, 0) as u8, 0, 0);
        if accumulate {
            return Some(Instruction {opcode: Opcode::MulA(rd, rn, rs, rm), offset, cond, set_flags, ins_size: 4})
        } else {
            return Some(Instruction {opcode: Opcode::Mul(rd, rn, rs), offset, cond, set_flags, ins_size: 4})
        }
    }
    if ins.bextr(27, 23) == 0b00010 && ins.bextr(21, 16) == 0b001111 && ins.bextr(11, 0) == 0 {
        let psr = Operand::Psr(if (ins & (1 << 22)) != 0 { PSR_SPSR } else { PSR_CPSR }, PSR_MODE_ALL);
        let rm = Operand::Reg(ins.bextr(15, 12) as u8, 0, 0);
        return Some(Instruction {opcode: Opcode::Mrs(rm, psr), offset, cond, set_flags: false, ins_size: 4})
    }
    if ins.bextr(27, 23) == 0b00010 && ins.bextr(21, 12) == 0b1010011111 && ins.bextr(11, 4) == 0 {
        let psr = Operand::Psr(if (ins & (1 << 22)) != 0 { PSR_SPSR } else { PSR_CPSR }, PSR_MODE_ALL);
        let rm = Operand::Reg(ins.bextr(3, 0) as u8, 0, 0);
        return Some(Instruction {opcode: Opcode::Msr(psr, rm), offset, cond, set_flags: false, ins_size: 4})
    }
    if ins.bextr(27, 23) == 0b00010 && ins.bextr(21, 12) == 0b1000011111 && ins.bextr(11, 4) == 0 {
        let psr = Operand::Psr(if (ins & (1 << 22)) != 0 { PSR_SPSR } else { PSR_CPSR }, PSR_MODE_C);
        let rm = Operand::Reg(ins.bextr(3, 0) as u8, 0, 0);
        return Some(Instruction {opcode: Opcode::Msr(psr, rm), offset, cond, set_flags: false, ins_size: 4})
    }
    if ins.bextr(27, 23) == 0b00010 && ins.bextr(21, 12) == 0b1010001111 {
        let psr = Operand::Psr(if (ins & (1 << 22)) != 0 { PSR_SPSR } else { PSR_CPSR }, PSR_MODE_FLAG);
        let rm = Operand::Reg(ins.bextr(3, 0) as u8, 0, 0);
        return Some(Instruction {opcode: Opcode::Msr(psr, rm), offset, cond, set_flags: false, ins_size: 4})
    }
    if ins.bextr(27, 23) == 0b00110 && ins.bextr(21, 12) == 0b1010001111 {
        let psr = Operand::Psr(if (ins & (1 << 22)) != 0 { PSR_SPSR } else { PSR_CPSR }, PSR_MODE_FLAG);
        let rm = Operand::Imm(ins.bextr(7, 0) as u32, ins.bextr(11, 8) as u8, ST_LSL);
        return Some(Instruction {opcode: Opcode::Msr(psr, rm), offset, cond, set_flags: false, ins_size: 4})
    }
    if ins.bextr(27, 24) == 0b1111 {
        return Some(Instruction {opcode: Opcode::Swi, offset, cond, set_flags: false, ins_size: 4})
    }
    if ins.bextr(27, 26) == 0b01 {
        let store = (ins & (1 << 20)) == 0;
        let offset2 = if (ins & (1 << 25)) == 0 { 
            Operand::Imm(ins.bextr(11, 0), 0, 0) 
        } else {
            Operand::Reg(ins.bextr(3, 0) as u8, ins.bextr(11, 4) as u8, 0)
        };
        if store {
            return Some(Instruction {opcode: Opcode::Str(rd(ins), rn(ins), offset2), offset, cond, set_flags: false, ins_size: 4})
        } else {
            return Some(Instruction {opcode: Opcode::Ldr(rd(ins), rn(ins), offset2), offset, cond, set_flags: false, ins_size: 4})
        }
    }
    if ins.bextr(27, 26) == 0b00 {
        let set_flags = (ins & (1 << 20)) != 0;
        return match opcode {
            OPCODE_AND => Some(Instruction { opcode: Opcode::And(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_EOR => Some(Instruction { opcode: Opcode::Eor(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_SUB => Some(Instruction { opcode: Opcode::Sub(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_RSB => Some(Instruction { opcode: Opcode::Rsb(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_ADD => Some(Instruction { opcode: Opcode::Add(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_ADC => Some(Instruction { opcode: Opcode::Adc(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_SBC => Some(Instruction { opcode: Opcode::Sbc(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_RSC => Some(Instruction { opcode: Opcode::Rsc(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            OPCODE_TST => Some(Instruction { opcode: Opcode::Tst(rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4 }),
            OPCODE_TEQ => Some(Instruction { opcode: Opcode::Teq(rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4 }),
            OPCODE_CMP => Some(Instruction { opcode: Opcode::Cmp(rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4 }),
            OPCODE_CMN => Some(Instruction { opcode: Opcode::Cmn(rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4 }),
            OPCODE_ORR => Some(Instruction { opcode: Opcode::Orr(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4 }),
            OPCODE_MOV => Some(Instruction { opcode: Opcode::Mov(rd(ins), op2(ins)), offset, cond, set_flags, ins_size: 4 }),
            OPCODE_BIC => Some(Instruction { opcode: Opcode::Bic(rd(ins), rn(ins), op2(ins)), offset, cond, set_flags, ins_size: 4 }),
            OPCODE_MVN => Some(Instruction { opcode: Opcode::Mvn(rd(ins), op2(ins)), offset, cond, set_flags, ins_size: 4}),
            _ => None,
        }
    }
    None
}

// const OPCODE_THUMB_AND: u16 = 0b0000;
// const OPCODE_THUMB_ORR: u16 = 0b1100;

// fn rm_th(x: u16) -> Operand {
//     Operand::Reg(x.bextr(5, 3) as u8, 0, 0)
// }

// fn rdn_th(x: u16) -> Operand {
//     Operand::Reg(x.bextr(2, 0) as u8, 0, 0)
// }

// fn disassemble_thumb_ins(ins1: u16, ins2: u16, offset: usize) -> Option<Instruction> {
//     let opcode = ins1.bextr(9, 6);
//     match opcode {
//         OPCODE_THUMB_AND => Some(Instruction { opcode: Opcode::And(rdn_th(ins1), rdn_th(ins1), rm_th(ins1)), offset, cond: COND_AL, set_flags: false, ins_size: 2 }),
//         OPCODE_THUMB_ORR => Some(Instruction { opcode: Opcode::Orr(rdn_th(ins1), rdn_th(ins1), rm_th(ins1)), offset, cond: COND_AL, set_flags: false, ins_size: 2 }),
//         _ => None
//     }
// }

fn disassemble_ins(bytes: &[u8], offset: usize, address: u64) -> Option<Instruction> {
    let ins = u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap());
    disassemble_arm_ins(ins, address as usize + offset)
}

pub fn disassemble_arm(section: &Section, section_name: &String, _program: &Program) -> DisassemblySection {
    let mut offset = 0x0;
    let address = section.addr;
    // let bytes = &[
    //     0xE1u8, 0xA0, 0x10, 0x00,
    //     0xE1, 0x80, 0x00, 0x00,
    //     0xE3, 0xa0, 0x20, 0x04,
    //     0x52, 0x4d, 0xd0, 0x0a,
    //     0xE2, 0x4d, 0xd0, 0x0a,
    //     0xE1, 0x2f, 0xff, 0x1e,
    //     0xE5, 0x81, 0x00, 0x00,
    //     0xE7, 0x80, 0x00, 0x04,
    //     0xEA, 0xff, 0xff, 0xf9
    // ];
    let mut instrs = Vec::<Instruction>::new();
    let bytes = section.bytes.as_slice();
    while offset < bytes.len() { 
        let res = disassemble_ins(bytes, offset, address);
        if res.is_some() {
            let ins = res.unwrap();
            offset += ins.ins_size as usize;
            instrs.push(ins);
        }
        else {
            instrs.push(Instruction {
                opcode: Opcode::Unknown, 
                offset, 
                cond: 0,
                set_flags: false,
                ins_size: 4});
            offset += 4;
        }
    }
    DisassemblySection {
        section_name: section_name.clone(),
        instructions: crate::dis::InstructionListing::Arm(instrs)
    }
}
