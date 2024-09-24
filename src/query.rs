pub enum FileType {
    RawBinary,
    Elf,
}

pub fn get_file_type(bytes: &Vec<u8>) -> FileType {
    if bytes.starts_with(&[0x7fu8, 0x45u8, 0x4cu8, 0x46u8]) {
        return FileType::Elf
    }
    FileType::RawBinary
}