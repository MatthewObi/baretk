use crate::prog;
use crate::arm;
use crate::x86;
use crate::riscv;

pub enum Operand {
    Nothing,
    Register(&'static str),
    Memory(&'static str, &'static str, i64, u8),
    Immediate(i64),
}

impl Operand {
    pub fn print(&self) -> String {
        match *self {
            Operand::Register(name) => format!("{}", name),
            Operand::Memory(base, index, offset, size) => {
                let word_name = match size {
                    1 => "BYTE",
                    2 => "WORD",
                    4 => "DWORD",
                    8 => "QWORD",
                    _ => "?"
                };
                if base == "" {
                    format!("{} [{}]", word_name, offset)
                }
                else if base == "." {
                    format!("{} [pc+{}]", word_name, offset)
                }
                else if index != "" {
                    format!("{} [{}+{}*{}]", word_name, base, index, offset)
                }
                else {
                    format!("{} [{}+{}]", word_name, base, offset)
                }
            },
            Operand::Immediate(i) => format!("{}", i),
            _ => format!("()")
        }
    }
}

// Common instruction struct for all architectures
pub struct Instruction {
    pub opcode: &'static str,
    pub operands: Vec<Operand>,
    pub flags: u64,
}

impl Instruction {
    pub fn print(&self) -> String {
        let mut str = format!("{}", self.opcode);
        for operand in self.operands.as_slice() {
            str += format!(" {},", operand.print()).as_str()
        }
        str.strip_suffix(",").unwrap_or(str.as_str()).to_string()
    }
}

pub enum InstructionListing {
    Rv(Vec<riscv::Instruction>),
    X86(Vec<x86::Instruction>),
    Arm(Vec<arm::Instruction>),
    Unknown,
}

impl InstructionListing {
    pub fn print(&self, addr: u64, bytes: Option<&[u8]>, symbols: Vec<(u64, String)>) -> String {
        let mut out = String::new();
        match self {
            Self::Rv(instrs) => {
                for ins in instrs {
                    for sym in &symbols {
                        if sym.0 == addr + ins.offset() as u64 {
                            out += format!("{}::\n", sym.1).as_str();
                        }
                    }
                    out += format!("    {:32}", ins.print()).as_str();
                    if let Some(b) = bytes {
                        out += format!("({:02x}", b[ins.offset()]).as_str();
                        for i in 1..ins.size() {
                            out += format!(" {:02x}", b[ins.offset() + i]).as_str();
                        }
                        out += ")\n";
                    }
                }
            },
            Self::X86(instrs) => {
                for ins in instrs {
                    out += format!("    {:32}", ins.print()).as_str();
                    if let Some(b) = bytes {
                        out += format!("({:02x}", b[ins.offset()]).as_str();
                        for i in 1..ins.size() {
                            out += format!(" {:02x}", b[ins.offset() + i]).as_str();
                        }
                        out += ")\n";
                    }
                }
            },
            Self::Arm(instrs) => {
                for ins in instrs {
                    for sym in &symbols {
                        if sym.0 == addr + ins.offset() as u64 {
                            out += format!("{}::\n", sym.1).as_str();
                        }
                    }
                    out += format!("    {:32}", ins.print()).as_str();
                    if let Some(b) = bytes {
                        out += format!("({:02x}", b[ins.offset()]).as_str();
                        for i in 1..ins.size() {
                            out += format!(" {:02x}", b[ins.offset() + i]).as_str();
                        }
                        out += ")\n";
                    }
                }
            },
            _ => out += "unknown\n",
        };
        out
    }

    pub fn instruction_vec(&self) -> Vec<Instruction> {
        let mut out = Vec::<Instruction>::new();
        match self {
            Self::Rv(rv) => { 
                let iter = rv.into_iter();
                for it in iter {
                    out.push(it.into());
                }
                out
            },
            Self::X86(rv) => { 
                let iter = rv.into_iter();
                for it in iter {
                    out.push(it.into());
                }
                out
            },
            _ => out
        }
    }
}

pub struct DisassemblySection {
    pub section_name: String,
    pub instructions: InstructionListing,
}

pub struct Disassembly {
    program: prog::Program,
    section: DisassemblySection,
}

impl Disassembly {
    pub fn program(&self) -> &prog::Program {
        &self.program
    }

    pub fn section(&self) -> &DisassemblySection {
        &self.section
    }

    pub fn print(&self, show_bytes: bool) -> String {
        let mut out = String::new();
        out += format!(".section {}\n", self.section.section_name).as_str();
        if let Some(section) = self.program.section_table.get(&self.section.section_name) {
            out += format!(".org {:#010x}\n", section.addr).as_str();
            let symbols = self.program.get_symbols_in_section(section.addr, section.addr + section.bytes.len() as u64);
            let bytes = match show_bytes {
                true => Some(section.bytes.as_slice()),
                _ => None,
            };
            out += self.section.instructions.print(section.addr, bytes, symbols).as_str();
        }
        else {
            out += self.section.instructions.print(0x0, None, Vec::<(u64, String)>::new()).as_str();
        }
        out
    }
}

pub fn disassemble(bytes: &[u8]) -> Disassembly {
    let program = prog::load_program_from_bytes(bytes);
    disassemble_program(program)
}

pub fn disassemble_program(program: prog::Program) -> Disassembly {
    let default_section = if program.section_table.contains_key(".text") { ".text" } else { "file" };
    let section_name = String::from(default_section);
    let section = match program.machine_type.as_str() {
        "arm" => arm::disassemble_arm(&program.section_table[default_section], &section_name, &program),
        "x86" => x86::disassemble_x86(&program.section_table[default_section], &section_name, &program),
        "amd64" => x86::disassemble_x86(&program.section_table[default_section], &section_name, &program), // TODO: Maybe separate amd64 and x86 disassembly code?
        "riscv" => riscv::disassemble_riscv(&program.section_table[default_section], &section_name, &program),
        _ => {
            eprintln!("Can't disassemble this. Not enough info or not able to disassemble architecture yet.\nArch: {}", program.machine_type);
            DisassemblySection { section_name: section_name.clone(), instructions: InstructionListing::Unknown }
        }
    };
    Disassembly {
        program,
        section,
    }
}
