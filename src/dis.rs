use crate::prog;
use crate::arm;
use crate::x86;

pub fn disassemble(bytes: &Vec<u8>) -> String {
    let program = prog::load_program_from_bytes(bytes);
    match program.machine_type.as_str() {
        "arm" => arm::disassemble_arm(&program.section_table[".text"], &program),
        "x86" => x86::disassemble_x86(&program.section_table[".text"], &program),
        "amd64" => x86::disassemble_x86(&program.section_table[".text"], &program), // TODO: Maybe separate amd64 and x86 disassembly code?
        _ => {
            eprintln!("Can't disassemble this. Not enough info or not able to disassemble architecture yet.\nArch: {}", program.machine_type);
            format!("")
        }
    }
}