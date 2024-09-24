use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use crate::query;
use crate::elf;

pub struct Section {
    pub name: String,
    pub addr: u64,
    pub bytes: Vec<u8>,
}

pub struct Program {
    pub flags: u8,
    pub machine_type: String,
    pub section_table: HashMap<String, Section>
}

impl Program {
    pub fn has_section(&self, name: &'static str) -> bool {
        self.section_table.contains_key(&String::from(name))
    }
}

pub fn build_program_from_binary(bytes: &Vec<u8>, flags: Option<u8>, machine_type: Option<String>) -> Program {
    let mut section_table = HashMap::<String, Section>::new();
    section_table.insert(String::from("file"), Section {
        name: String::from("file"),
        addr: 0x0,
        bytes: bytes.clone()
    });
    Program {
        flags: flags.unwrap_or_default(),
        machine_type: machine_type.unwrap_or("unknown".to_string()),
        section_table: section_table,
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
        _ => build_program_from_binary(bytes, None, None)
    }
}