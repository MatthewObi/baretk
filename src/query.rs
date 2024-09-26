use crate::pe;

pub enum FileType {
    RawBinary,
    Elf,
    PE,
}

pub fn get_file_type(bytes: &Vec<u8>) -> FileType {
    if bytes.starts_with(&[0x7fu8, 0x45u8, 0x4cu8, 0x46u8]) {
        return FileType::Elf
    }
    else if pe::check_is_pe_executable(bytes) {
        return FileType::PE
    }
    FileType::RawBinary
}