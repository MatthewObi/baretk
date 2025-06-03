use std::collections::HashMap;

use crate::dis::{self, Disassembly, Instruction};

#[derive(Clone, Copy)]
pub enum Language {
    Pseudocode, 
    C, // TODO: Add C decompilation target
}

const OP_ADD: u8 = 0x0;
const OP_SUB: u8 = 0x1;
const OP_MUL: u8 = 0x2;
const OP_AND: u8 = 0x3;
const OP_OR: u8 = 0x4;
const OP_XOR: u8 = 0x5;

enum Expr {
    Constant(i64),
    // Memory(i64),
    Register(&'static str),
    Dereference(u8, Box<Expr>),
    Binary(u8, Box<Expr>, Box<Expr>),
    // Unary(u8, Box<Expr>),
    Call(Box<Expr>),
    Store(Box<Expr>, Box<Expr>),
    Group(Vec<Box<Expr>>),
    Nop,
    Return
}

impl Expr {
    fn print(&self, depth: i32, lang: Language) -> String {
        let mut out = String::new();
        for _ in 0..depth {
            out += "    ";
        }
        out += (match self {
            Self::Constant(i) => format!("{}", i),
            Self::Register(r) => format!("{}", r),
            Self::Dereference(s, rhs) => {
                match lang {
                    Language::Pseudocode => match s {
                        1 => format!("*u8({})", (*rhs).print(0, lang)),
                        2 => format!("*u16({})", (*rhs).print(0, lang)),
                        4 => format!("*u32({})", (*rhs).print(0, lang)),
                        8 => format!("*u64({})", (*rhs).print(0, lang)),
                        _ => format!("*({})", (*rhs).print(0, lang))
                    },
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Binary(op, lhs, rhs) => {
                match lang {
                    Language::Pseudocode => match *op {
                        OP_ADD => format!("({} + {})", (*lhs).print(0, lang), (*rhs).print(0, lang)),
                        OP_SUB => format!("({} - {})", (*lhs).print(0, lang), (*rhs).print(0, lang)),
                        OP_MUL => format!("({} * {})", (*lhs).print(0, lang), (*rhs).print(0, lang)),
                        OP_XOR => format!("({} ^ {})", (*lhs).print(0, lang), (*rhs).print(0, lang)),
                        _ => format!("({} ? {})", (*lhs).print(0, lang), (*rhs).print(0, lang))
                    }
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Call(op) => {
                match lang {
                    Language::Pseudocode => format!("({})()", (*op).print(0, lang)),
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Return => {
                match lang {
                    Language::Pseudocode => format!("return"),
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Store(dest, src) => {
                match lang {
                    Language::Pseudocode => format!("{} = {}", (*dest).print(0, lang), (*src).print(0, lang)),
                    _ => todo!("Other languages besides the pseudocode")
                }
            },
            Self::Nop => format!("nop"),
            Self::Group(group) => {
                let mut out = String::new();
                out += "do:\n";
                for expr in group {
                    out += format!("    {}\n", (*expr).print(depth + 1, lang)).as_str();
                }
                out.strip_suffix("\n").unwrap_or(out.as_str()).to_string()
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
    disassembly: Disassembly,
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
        let mut out = format!("fn sub_{:08x}:\n", addr);
        for expr in self.expr_list.as_slice() {
            out += format!("    {}\n", expr.print(0, self.dest_lang)).as_str();
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

fn expr_register(r: &'static str) -> Box<Expr> {
    Box::new(Expr::Register(r))
}

fn expr_constant(i: i64) -> Box<Expr> {
    Box::new(Expr::Constant(i))
}

fn expr_binary(op: u8, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Binary(op, lhs, rhs))
}

fn expr_dereference(size: u8, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Dereference(size, rhs))
}

fn expr_store(dest: Box<Expr>, src: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Store(dest, src))
}

fn expr_group(group: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr::Group(group))
}

fn expr_call(callee: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Call(callee))
}

fn expr_nop() -> Box<Expr> {
    Box::new(Expr::Nop)
}

fn expr_ret() -> Box<Expr> {
    Box::new(Expr::Return)
}

fn operand_to_expr(op: &dis::Operand) -> Box<Expr> {
    match *op {
        dis::Operand::Memory(r1, r2, offset, size) => {
            if r1 == "" {
                expr_dereference(size, 
                    expr_binary(OP_ADD, expr_register(r1), expr_constant(offset))
                )
            }
            else if r1 == "." {
                expr_dereference( size, 
                    expr_binary(OP_ADD, expr_register("pc"), expr_constant(offset))
                )
            }
            else if r2 != "" {
                expr_dereference(size, 
                    expr_binary(OP_ADD, 
                        expr_binary(OP_ADD,
                            expr_register(r1), 
                            expr_binary( OP_MUL, 
                                expr_register(r2), 
                                expr_constant(size.into())
                            )
                        ),
                        expr_constant(offset)
                    )
                )
            }
            else {
                expr_dereference(size, 
                    expr_binary(OP_ADD, expr_register(r1), expr_constant(offset))
                )
            }
        },
        dis::Operand::Register(r) => expr_register(r),
        dis::Operand::Immediate(i) => expr_constant(i),
        _ => todo!("finish this")
    }
}

struct ExprBuilder {
    next_id: u64,
    change_lists: HashMap<&'static str, ChangeList>,
}

impl ExprBuilder {
    fn add_change_list_if_not_created(&mut self, s: &'static str) {
        if !self.change_lists.contains_key(&s) {
            self.change_lists.insert(s, ChangeList { uses: vec![], stores: vec![], loads: vec![], last_store: 0, last_load: 0 });
        }
    }

    fn add_register_store(&mut self, s: &'static str) {
        self.add_change_list_if_not_created(s);
        self.change_lists.get_mut(s).expect("").add_store(self.next_id);
    }

    fn add_register_load(&mut self, s: &'static str) {
        self.add_change_list_if_not_created(s);
        self.change_lists.get_mut(s).expect("").add_load(self.next_id);
    }

    fn create_uses_in_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Store(dest, src) => {
                match **dest {
                    Expr::Register(r) => self.add_register_store(r),
                    _ => (),
                };
                match **src {
                    Expr::Register(r) => self.add_register_load(r),
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

    fn decomp_instruction(&mut self, ins: &Instruction, _expr_list: &Vec<Expr>) -> Expr {
        match ins.opcode {
            "add" => { // op0 = op1 + op2
                let dest = &ins.operands[0];
                let src1 = &ins.operands[1];
                let src2 = &ins.operands[2];
                let expr = expr_binary(OP_ADD, 
                    operand_to_expr(src1), operand_to_expr(src2));
                *expr_store(operand_to_expr(dest), expr)
            },
            "sub" => { // op0 = op1 - op2
                let dest = &ins.operands[0];
                let src1 = &ins.operands[1];
                let src2 = &ins.operands[2];
                let expr = expr_binary(OP_SUB, 
                    operand_to_expr(src1), operand_to_expr(src2));
                *expr_store(operand_to_expr(dest), expr)
            },
            "and" => { // op0 = op1 & op2
                let dest = &ins.operands[0];
                let src1 = &ins.operands[1];
                let src2 = &ins.operands[2];
                let expr = expr_binary(OP_AND, 
                    operand_to_expr(src1), operand_to_expr(src2));
                *expr_store(operand_to_expr(dest), expr)
            },
            "or" => { // op0 = op1 | op2
                let dest = &ins.operands[0];
                let src1 = &ins.operands[1];
                let src2 = &ins.operands[2];
                let expr = expr_binary(OP_OR, 
                    operand_to_expr(src1), operand_to_expr(src2));
                *expr_store(operand_to_expr(dest), expr)
            },
            "xor" => { // op0 = op1 ^ op2
                let dest = &ins.operands[0];
                let src1 = &ins.operands[1];
                let src2 = &ins.operands[2];
                let expr = expr_binary(OP_XOR, 
                    operand_to_expr(src1), operand_to_expr(src2));
                *expr_store(operand_to_expr(dest), expr)
            },
            "mov" => { // op0 = op1
                let dest = &ins.operands[0];
                let src = &ins.operands[1];
                let out = expr_store(operand_to_expr(dest), operand_to_expr(src));
                self.create_uses_in_expr(&out);
                *out
            },
            "push" => { // sp -= size, *sp = op0
                let op0 = &ins.operands[0];
                let sp = dis::Operand::Register("rsp");
                let out = expr_group(vec![
                    expr_store(operand_to_expr(&sp), expr_binary(OP_SUB, operand_to_expr(&sp), expr_constant(8))),
                    expr_store(expr_dereference(8, operand_to_expr(&sp)), operand_to_expr(op0)),
                ]);
                self.create_uses_in_expr(&out);
                *out
            },
            "pop" => { // op0 = *sp, sp += size
                let op0 = &ins.operands[0];
                let sp = dis::Operand::Register("rsp");
                let out = expr_group(vec![
                    expr_store(operand_to_expr(op0), expr_dereference(8, operand_to_expr(&sp))),
                    expr_store(operand_to_expr(&sp), expr_binary(OP_ADD, operand_to_expr(&sp), expr_constant(8))),
                ]);
                self.create_uses_in_expr(&out);
                *out
            },
            "call" => { // TODO: Find call arguments and return value. Maybe use calling convention and function analysis to detect this?
                let callee = &ins.operands[0];
                let out = expr_call(operand_to_expr(callee));
                self.create_uses_in_expr(&out);
                *out
            },
            "nop" => *expr_nop(),
            "ret" => *expr_ret(),
            _ => todo!("need to implement {} decompilation", ins.opcode)
        }
    }
}

fn decomp_disassembly(dis: &Disassembly, out_lang: &str) -> Vec<Expr> {
    let instrs = dis.section().instructions.instruction_vec();
    let lang = match out_lang {
        "c"         => Language::C,
        _           => Language::Pseudocode,
    };
    let mut expr_list = Vec::<Expr>::new();
    let mut expr_builder = ExprBuilder { change_lists: HashMap::<&str, ChangeList>::new(), next_id: 1 };
    for instr in instrs {
        let expr = expr_builder.decomp_instruction(&instr, &expr_list);
        println!("{} // {}", expr.print(0, lang), instr.print());
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
    let expr_list = decomp_disassembly(&dis, "");
    Decomp { disassembly: dis, dest_lang, expr_list }
}
