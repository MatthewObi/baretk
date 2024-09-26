use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use crate::query;
use crate::elf;
use crate::pe;

pub struct Section {
    pub addr: u64,
    pub bytes: Vec<u8>,
}

pub struct Segment {
    pub perm: u8,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub size: usize,
}

pub struct Program {
    pub bits: u8,
    pub endianess: u8,
    pub machine_type: String,
    pub program_table: Vec<Segment>,
    pub section_table: HashMap<String, Section>
}

pub fn build_program_from_binary(bytes: &Vec<u8>, bits: Option<u8>, endianess: Option<u8>, machine_type: Option<String>) -> Program {
    let mut section_table = HashMap::<String, Section>::new();
    section_table.insert(String::from("file"), Section {
        addr: 0x0,
        bytes: bytes.clone()
    });
    let mut program_table = Vec::<Segment>::new();
    program_table.push(Segment {
        perm: 0x7,
        offset: 0x0,
        vaddr: 0x0,
        paddr: 0x0,
        size: bytes.len(),
    });
    Program {
        bits: bits.unwrap_or_default(),
        endianess: endianess.unwrap_or_default(),
        machine_type: machine_type.unwrap_or("unknown".to_string()),
        program_table,
        section_table,
    }
}

pub fn load_program_from_file(path: &String) -> Result<Program, ()> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error opening file {}: {}", path, error);
            return Err(());
        }
    };

    let mut contents: Vec<u8> = vec![];
    if let Err(error) = file.read_to_end(&mut contents) {
        eprintln!("Error reading file {}: {}", path, error);
        return Err(());
    }

    Ok(load_program_from_bytes(&contents))
}

pub fn load_program_from_bytes(bytes: &Vec<u8>) -> Program {
    let file_type = query::get_file_type(bytes);
    match file_type {
        query::FileType::Elf => elf::load_program_from_bytes(bytes),
        query::FileType::PE  => pe::load_program_from_bytes(bytes),
        _ => build_program_from_binary(bytes, None, None, None)
    }
}