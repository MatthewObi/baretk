use std::collections::HashMap;

use crate::dis::{self, Disassembly, Instruction};

#[derive(Clone, Copy)]
pub enum Language {
    Pseudocode, 
    C, // TODO: Add C decompilation target
}

pub const OP_ADD: u8 = 0x0;
pub const OP_SUB: u8 = 0x1;
pub const OP_MUL: u8 = 0x2;
pub const OP_AND: u8 = 0x3;
pub const OP_OR: u8 = 0x4;
pub const OP_XOR: u8 = 0x5;
pub const OP_LSL: u8 = 0x6;
pub const OP_LSR: u8 = 0x7;
pub const OP_ASR: u8 = 0x8;
pub const OP_CMP: u8 = 0x9;
pub const OP_LT: u8 = 0xa;
pub const OP_GT: u8 = 0xb;
pub const OP_LTE: u8 = 0xc;
pub const OP_GTE: u8 = 0xd;
pub const OP_EQ: u8 = 0xe;
pub const OP_NEQ: u8 = 0xf;
// pub const OP_ROL: u8 = 0x10;
pub const OP_ROR: u8 = 0x11;
pub const OP_ANDAND: u8 = 0x12;
pub const OP_OROR: u8 = 0x13;

#[derive(Clone)]
pub enum Expr {
    Constant(i64),
    // Memory(i64),
    Label(String),
    Register(String),
    Special(String, Vec<Box<Expr>>),
    Dereference(u8, Box<Expr>),
    Binary(u8, Box<Expr>, Box<Expr>),
    // Unary(u8, Box<Expr>),
    Call(Box<Expr>),
    Goto(Box<Expr>),
    Store(Box<Expr>, Box<Expr>),
    Group(Vec<Box<Expr>>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Nop,
    Return
}

impl Expr {
    fn print(&self, depth: i32, symbols: &Vec<(u64, String)>, lang: Language) -> String {
        let mut out = String::new();
        for _ in 0..depth {
            out += "    ";
        }
        out += (match self {
            Self::Constant(i) => format!("{}", i),
            Self::Register(r) => format!("{}", r),
            Self::Label(r) => format!("{}", r),
            Self::Dereference(s, rhs) => {
                match lang {
                    Language::Pseudocode => match s {
                        1 => format!("*u8({})", (*rhs).print(0, symbols, lang)),
                        2 => format!("*u16({})", (*rhs).print(0, symbols, lang)),
                        4 => format!("*u32({})", (*rhs).print(0, symbols, lang)),
                        8 => format!("*u64({})", (*rhs).print(0, symbols, lang)),
                        _ => format!("*({})", (*rhs).print(0, symbols, lang))
                    },
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Binary(op, lhs, rhs) => {
                match lang {
                    Language::Pseudocode => match *op {
                        OP_ADD => format!("({} + {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_SUB => format!("({} - {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_MUL => format!("({} * {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_AND => format!("({} & {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_OR => format!("({} | {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_XOR => format!("({} ^ {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_LSL => format!("({} << {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_LSR => format!("({} >> {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_ASR => format!("({} >>> {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_CMP => format!("cmp({}, {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_LT  => format!("({} < {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_GT  => format!("({} > {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_LTE => format!("({} <= {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_GTE => format!("({} >= {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_EQ  => format!("({} == {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_NEQ => format!("({} != {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_ANDAND => format!("({} && {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        OP_OROR   => format!("({} || {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang)),
                        _ => format!("({} ? {})", (*lhs).print(0, symbols, lang), (*rhs).print(0, symbols, lang))
                    }
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Call(op) => {
                match lang {
                    Language::Pseudocode => {
                        if let Self::Constant(c) = **op {
                            for symbol in symbols {
                                if symbol.0 == c as u64 {
                                    return format!("{}()", symbol.1);
                                }
                            }
                        }
                        format!("({})()", (*op).print(0, symbols, lang))
                    },
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Return => {
                match lang {
                    Language::Pseudocode => format!("return"),
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Goto(op) => {
                match lang {
                    Language::Pseudocode => {
                        if let Self::Constant(c) = **op {
                            for symbol in symbols {
                                if symbol.0 == c as u64 {
                                    return format!("goto {}", symbol.1);
                                }
                            }
                        }
                        format!("goto ({})", (*op).print(0, symbols, lang))
                    },
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::If(cond, then, el) => {
                match lang {
                    Language::Pseudocode => {
                        let mut out = String::new();
                        out += format!("if ({}) {}\n", (*cond).print(0, symbols, lang), (*then).print(0, symbols, lang)).as_str();
                        if let Some(el) = el {
                            out += format!("else {}\n", (*el).print(0, symbols, lang)).as_str();
                        }
                        out.strip_suffix("\n").unwrap_or(out.as_str()).to_string()
                    },
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Store(dest, src) => {
                match lang {
                    Language::Pseudocode => format!("{} = {}", (*dest).print(0, symbols, lang), (*src).print(0, symbols, lang)),
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Nop => format!("nop"),
            Self::Group(group) => {
                let mut out = String::new();
                out += "do:\n";
                for expr in group {
                    out += format!("    {}\n", (*expr).print(depth + 1, symbols, lang)).as_str();
                }
                out.strip_suffix("\n").unwrap_or(out.as_str()).to_string()
            },
            Self::Special(name, args) => {
                let mut out = String::new();
                out += format!("${}(", name).as_str();
                for expr in args {
                    out += format!("{}, ", (*expr).print(0, symbols, lang)).as_str();
                }
                out = out.strip_suffix(", ").unwrap_or(out.as_str()).to_string();
                out += ")";
                out
            },
            // _ => todo!("Finish expression printing")
        }).as_str();
        out
    }
}

#[allow(dead_code)] // TODO: Use this struct.
struct ExprList {
    exprs: Vec<Expr>,
}

pub struct Decomp {
    pub disassembly: Disassembly,
    dest_lang: Language,
    expr_list: Vec<Expr>,
}

impl Decomp {
    pub fn print(&self) -> String {
        let addr = if let Some(section) = self.disassembly.program().section_table.get(&self.disassembly.section().section_name) {
            section.addr
        } else {
            0
        };
        let section = self.disassembly.program().section_table.get(&self.disassembly.section().section_name).unwrap();
        let symbols = self.disassembly.program().get_symbols_in_section(section.addr, section.addr + section.bytes.len() as u64);
        let mut out = format!("fn sub_{:08x}:\n", addr);
        for expr in self.expr_list.as_slice() {
            if let Expr::Label(lbl) = expr {
                out += format!("{}:\n", lbl).as_str();
            }
            else {
                out += format!("    {}\n", expr.print(0, &symbols, self.dest_lang)).as_str();
            }
        }
        out
    }
}

struct ChangeList {
    uses: Vec<u64>,
    stores: Vec<u64>,
    loads: Vec<u64>,
    last_store: u64,
    last_load: u64,
}

impl ChangeList {
    fn add_store(&mut self, id: u64) {
        self.stores.push(id);
        self.last_store = id;
        self.add_use(id);
    }

    fn add_load(&mut self, id: u64) {
        self.loads.push(id);
        self.last_load = id;
        self.add_use(id);
    }

    fn add_use(&mut self, id: u64) {
        self.uses.push(id);
    }
}

pub fn expr_register(r: String) -> Box<Expr> {
    Box::new(Expr::Register(r))
}

pub fn expr_constant(i: i64) -> Box<Expr> {
    Box::new(Expr::Constant(i))
}

pub fn expr_binary(op: u8, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Binary(op, lhs, rhs))
}

pub fn expr_dereference(size: u8, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Dereference(size, rhs))
}

pub fn expr_store(dest: Box<Expr>, src: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Store(dest, src))
}

pub fn expr_group(group: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr::Group(group))
}

pub fn expr_call(callee: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Call(callee))
}

pub fn expr_goto(jmp: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Goto(jmp))
}

pub fn expr_nop() -> Box<Expr> {
    Box::new(Expr::Nop)
}

pub fn expr_ret() -> Box<Expr> {
    Box::new(Expr::Return)
}

pub fn expr_if(cond: Box<Expr>, then: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::If(cond, then, None))
}

// pub fn expr_if_else(cond: Box<Expr>, then: Box<Expr>, el: Box<Expr>) -> Box<Expr> {
//     Box::new(Expr::If(cond, then, Some(el)))
// }

pub fn expr_special(name: &'static str, args: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr::Special(String::from(name), args))
}

pub struct ExprBuilder {
    next_id: u64,
    change_lists: HashMap<String, ChangeList>,
}

impl ExprBuilder {
    fn add_change_list_if_not_created(&mut self, s: &String) {
        if !self.change_lists.contains_key(s) {
            self.change_lists.insert(s.clone(), ChangeList { uses: vec![], stores: vec![], loads: vec![], last_store: 0, last_load: 0 });
        }
    }

    fn add_register_store(&mut self, s: &String) {
        self.add_change_list_if_not_created(s);
        self.change_lists.get_mut(s).expect("").add_store(self.next_id);
    }

    fn add_register_load(&mut self, s: &String) {
        self.add_change_list_if_not_created(s);
        self.change_lists.get_mut(s).expect("").add_load(self.next_id);
    }

    fn create_uses_in_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Store(dest, src) => {
                match &**dest {
                    Expr::Register(r) => self.add_register_store(&r),
                    _ => (),
                };
                match &**src {
                    Expr::Register(r) => self.add_register_load(&r),
                    _ => (),
                };
            },
            Expr::Group(group) => {
                for expr in group {
                    self.create_uses_in_expr(expr);
                }
            },
            _ => (),
        }
    }

    fn decomp_instruction(&mut self, ins: &Instruction) -> Expr {
        let expr = match ins {
            Instruction::Arm(arm) => *arm.into_expr(),
            Instruction::X86(x86) => *x86.into_expr(),
            Instruction::Rv(rv) => *rv.into_expr(),
        };
        self.create_uses_in_expr(&expr);
        expr
    }
}

fn decomp_disassembly(dis: &Disassembly) -> Vec<Expr> {
    let instrs = dis.section().instructions.instruction_vec();
    let mut expr_list = Vec::<Expr>::new();
    let mut expr_builder = ExprBuilder { change_lists: HashMap::<String, ChangeList>::new(), next_id: 1 };
    let section = dis.program().section_table.get(&dis.section().section_name).unwrap();
    let symbols = dis.program().get_symbols_in_section(section.addr, section.addr + section.bytes.len() as u64);
    'instr_loop: for instr in instrs {
        for symbol in symbols.as_slice() {
            if symbol.0 == instr.offset() as u64 {
                expr_list.push(Expr::Label(symbol.1.clone()));
                expr_builder.next_id += 1;
                continue 'instr_loop;
            }
        }
        let expr = expr_builder.decomp_instruction(&instr);
        // println!("{} // {}", expr.print(0, lang), instr.print());
        expr_list.push(expr);
        expr_builder.next_id += 1;
    }
    expr_list
}

pub fn decomp_program_from_bytes(bytes: &[u8], dest_lang: Language) -> Decomp {
    let dis = dis::disassemble(bytes);
    decomp_program(dis, dest_lang)
}

pub fn decomp_program(dis: Disassembly, dest_lang: Language) -> Decomp {
    let expr_list = decomp_disassembly(&dis);
    Decomp { disassembly: dis, dest_lang, expr_list }
}
