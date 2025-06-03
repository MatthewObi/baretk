use std::collections::HashMap;
use crate::query;
use crate::elf;
use crate::pe;
use crate::util;

#[derive(Clone)]
pub struct Section {
    pub addr: u64,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Segment {
    pub perm: u8,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub size: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Symbol {
    pub addr: u64,
    pub size: u64,
}

#[derive(Clone)]
#[allow(dead_code)] // TODO: Remove this and actually use the unused fields
pub struct Program {
    pub bits: u8,
    pub endianess: u8,
    pub machine_type: String,
    pub entry_point: u64,
    pub program_table: Vec<Segment>,
    pub section_table: HashMap<String, Section>,
    pub symbol_table: HashMap<String, Symbol>
}

impl Program {
    #[allow(dead_code)] // TODO: Remove this and actually use the method
    pub fn find_section_and_segment(&self, addr: u64) -> (Option<&Section>, Option<&Segment>) {
        let mut section = Option::<&Section>::None;
        let mut segment = Option::<&Segment>::None;
        for (key, value) in &self.section_table {
            if addr >= value.addr && addr < value.addr + value.bytes.len() as u64 {
                section = Some(&self.section_table[key]);
                break;
            }
        }

        for seg in &self.program_table {
            if addr >= seg.vaddr && addr < seg.vaddr + seg.size as u64 {
                segment = Some(seg);
                break;
            }
        }
        (section, segment)
    }

    pub fn get_symbols_in_section(&self, start: u64, stop: u64) -> Vec<(u64, String)> {
        let mut symbols = Vec::<(u64, String)>::new();
        for (k, v) in &self.symbol_table {
            if v.addr >= start && v.addr < stop {
                symbols.push((v.addr, k.clone()));
            }
        }
        symbols
    }
}

pub fn build_program_from_binary(bytes: &[u8], bits: Option<u8>, endianess: Option<u8>, machine_type: Option<String>) -> Program {
    let mut section_table = HashMap::<String, Section>::new();
    section_table.insert(String::from("file"), Section {
        addr: 0x0,
        bytes: bytes.to_vec().clone()
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
        entry_point: 0,
        program_table,
        section_table,
        symbol_table: HashMap::new()
    }
}

pub fn load_program_from_file(path: &String) -> Result<Program, ()> {
    match util::try_read_file_contents(path) {
        Err(()) => Err(()),
        Ok(contents) => Ok(load_program_from_bytes(&contents)),
    }
}

pub fn load_program_from_bytes(bytes: &[u8]) -> Program {
    let file_type = query::get_file_type(bytes);
    match file_type {
        query::FileType::Elf => elf::load_program_from_bytes(bytes),
        query::FileType::PE  => pe::load_program_from_bytes(bytes),
        _ => build_program_from_binary(bytes, None, None, None)
    }
}