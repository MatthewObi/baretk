use crate::query::{get_file_type, FileType};
use crate::elf;
use crate::prog::build_program_from_binary;
use crate::arm;

pub fn disassemble(bytes: &Vec<u8>) -> String {
    let file_type = get_file_type(bytes);
    let program = match file_type {
        FileType::Elf => elf::load_program_from_bytes(bytes),
        _ => build_program_from_binary(bytes, None, None)
    };
    match program.machine_type.as_str() {
        "ARM" => arm::disassemble_arm(&program.section_table[".text"], &program),
        _ => format!("Can't disassemble this. Not enough info or not able to disassemble architecture yet."),
    }
}