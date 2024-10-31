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

fn try_ascii_string(index: usize, bytes: &Vec<u8>, min_len: usize, printable: bool) -> (Option<String>, usize) {
    let mut len = 0usize;
    while index + len < bytes.len() {
        if (printable && bytes[index + len] < 0x20u8) || bytes[index + len] == 0 {
            break;
        }
        else if bytes[index + len] <= 0x7fu8 {
            len += 1;
            continue;
        }
        else {
            return (None, len + 1);
        }
    }
    if len >= min_len {
        return (Some(String::from_utf8_lossy(&bytes[index..index + len]).into_owned()), len + 1);
    }
    else {
        return (None, len + 1)
    }
}

pub fn get_strings(bytes: &Vec<u8>, min_len: usize, printable: bool) -> Vec<String> {
    let mut index = 0usize;
    let mut strings = Vec::<String>::new();
    while index < bytes.len() {
        let (str, size) = try_ascii_string(index, bytes, min_len, printable);
        if let Some(s) = str {
            strings.push(s);
        }
        index += size;
    }
    strings
}