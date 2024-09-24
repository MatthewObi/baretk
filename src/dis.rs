use crate::query::{get_file_type, FileType};
use crate::{elf, prog};
use crate::arm;

pub fn disassemble(bytes: &Vec<u8>) -> String {
    let program = prog::load_program_from_bytes(bytes);
    match program.machine_type.as_str() {
        "ARM" => arm::disassemble_arm(&program.section_table[".text"], &program),
        _ => format!("Can't disassemble this. Not enough info or not able to disassemble architecture yet."),
    }
}