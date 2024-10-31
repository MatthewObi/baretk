use crate::prog;
use crate::arm;
use crate::x86;
use crate::riscv;

pub enum InstructionListing {
    Rv(Vec<riscv::Instruction>),
    X86(Vec<x86::Instruction>),
    Unknown,
}

impl InstructionListing {
    pub fn print(&self, addr: u64, bytes: Option<&[u8]>) -> String {
        let mut out = String::new();
        match self {
            Self::Rv(instrs) => {
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
            _ => out += "unknown\n",
        };
        out
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
    pub fn print(&self, show_bytes: bool) -> String {
        let mut out = String::new();
        out += format!(".section {}\n", self.section.section_name).as_str();
        if let Some(section) = self.program.section_table.get(&self.section.section_name) {
            out += format!(".org {:#010x}\n", section.addr).as_str();
            let bytes = match show_bytes {
                true => Some(section.bytes.as_slice()),
                _ => None,
            };
            out += self.section.instructions.print(section.addr, bytes).as_str();
        }
        else {
            out += self.section.instructions.print(0x0, None).as_str();
        }
        out
    }
}

pub fn disassemble(bytes: &Vec<u8>) -> Disassembly {
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
