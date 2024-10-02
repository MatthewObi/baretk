use std::{collections::HashMap, usize};
use crate::prog::{Program, Section, Segment};
use crate::dump::rwx_string;
use crate::util::{read_u16_from_u8_vec, read_u32_from_u8_vec, read_u32_to_u64_from_u8_vec, read_u64_from_u8_vec, BIG_ENDIAN, LITTLE_ENDIAN};

struct Header {
    class: u8,
    data: u8,
    version: u8,
    abi: u8,
    abi_version: u8,
}

fn read_header(bytes: &Vec<u8>) -> Header {
    Header{
        class: bytes[0x04],
        data: bytes[0x05],
        version: bytes[0x06],
        abi: bytes[0x07],
        abi_version: bytes[0x08],
    }
}

#[derive(Debug)]
struct HeaderCommon {
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}
#[derive(PartialEq)]
struct ElfType(u16);
impl ElfType {
    const NONE: ElfType = ElfType(0x0);
    const REL : ElfType = ElfType(0x1);
    const EXEC: ElfType = ElfType(0x2);
    const DYN : ElfType = ElfType(0x3);
    const CORE: ElfType = ElfType(0x4);
}

fn elf_file_type_string(t: u16) -> &'static str {
    let et = ElfType(t);
    match et {
        ElfType::NONE => "none",
        ElfType::REL  => "Relocatable",
        ElfType::EXEC => "Executable",
        ElfType::DYN  => "Shared object",
        ElfType::CORE => "Core",
        _ => "unknown",
    }
}

#[derive(PartialEq)]
struct MachineType(u16);
impl MachineType {
    const UNKNOWN   : MachineType = MachineType(0x0);
    const X86       : MachineType = MachineType(0x3);
    const ARM       : MachineType = MachineType(0x28);
    const AMD64     : MachineType = MachineType(0x3e);
}

fn machine_type_string(t: u16) -> &'static str {
    let mt = MachineType(t);
    match mt {
        MachineType::UNKNOWN => "unknown",
        MachineType::X86     => "x86",
        MachineType::AMD64   => "amd64",
        MachineType::ARM     => "arm",
        _ => "unknown",
    }
}

#[derive(Debug)]
struct ProgramHeaderEntry {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[derive(Debug)]
struct SectionHeaderEntry {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

fn read_common_header_32(bytes: &Vec<u8>, endianness: u8) -> HeaderCommon {
    HeaderCommon {
        e_type: read_u16_from_u8_vec(bytes, 0x10, endianness),
        e_machine: read_u16_from_u8_vec(bytes, 0x12,endianness),
        e_version: read_u32_from_u8_vec(bytes, 0x14,endianness),
        e_entry: read_u32_to_u64_from_u8_vec(bytes, 0x18, endianness),
        e_phoff: read_u32_to_u64_from_u8_vec(bytes, 0x1c, endianness),
        e_shoff: read_u32_to_u64_from_u8_vec(bytes, 0x20, endianness),
        e_flags: read_u32_from_u8_vec(bytes, 0x24, endianness),
        e_ehsize: read_u16_from_u8_vec(bytes, 0x28, endianness),
        e_phentsize: read_u16_from_u8_vec(bytes, 0x2a, endianness),
        e_phnum: read_u16_from_u8_vec(bytes, 0x2c, endianness),
        e_shentsize: read_u16_from_u8_vec(bytes, 0x2e, endianness),
        e_shnum: read_u16_from_u8_vec(bytes, 0x30, endianness),
        e_shstrndx: read_u16_from_u8_vec(bytes, 0x32, endianness),
    }
}

fn read_common_header_64(bytes: &Vec<u8>, endianness: u8) -> HeaderCommon {
    HeaderCommon {
        e_type: read_u16_from_u8_vec(bytes, 0x10, endianness),
        e_machine: read_u16_from_u8_vec(bytes, 0x10, endianness),
        e_version: read_u32_from_u8_vec(bytes, 0x14,endianness),
        e_entry: read_u64_from_u8_vec(bytes, 0x18, endianness),
        e_phoff: read_u64_from_u8_vec(bytes, 0x20, endianness),
        e_shoff: read_u64_from_u8_vec(bytes, 0x28, endianness),
        e_flags: read_u32_from_u8_vec(bytes, 0x30, endianness),
        e_ehsize: read_u16_from_u8_vec(bytes, 0x34, endianness),
        e_phentsize: read_u16_from_u8_vec(bytes, 0x36, endianness),
        e_phnum: read_u16_from_u8_vec(bytes, 0x38, endianness),
        e_shentsize: read_u16_from_u8_vec(bytes, 0x3a, endianness),
        e_shnum: read_u16_from_u8_vec(bytes, 0x3c, endianness),
        e_shstrndx: read_u16_from_u8_vec(bytes, 0x3e, endianness),
    }
}

fn read_program_header_32(bytes: &Vec<u8>, phnum: u16, phsize: u16, start: u64, endianness: u8) -> Vec<ProgramHeaderEntry> {
    let mut out = Vec::<ProgramHeaderEntry>::with_capacity(phnum as usize);
    let mut s = start as usize;
    for _ in 0..phnum {
        out.push(ProgramHeaderEntry{
            p_type: read_u32_from_u8_vec(bytes, s + 0x0, endianness),
            p_flags: read_u32_from_u8_vec(bytes, s + 0x18, endianness),
            p_offset: read_u32_to_u64_from_u8_vec(bytes, s + 0x4, endianness),
            p_vaddr: read_u32_to_u64_from_u8_vec(bytes, s + 0x8, endianness),
            p_paddr: read_u32_to_u64_from_u8_vec(bytes, s + 0xc, endianness),
            p_filesz: read_u32_to_u64_from_u8_vec(bytes, s + 0x10, endianness),
            p_memsz: read_u32_to_u64_from_u8_vec(bytes, s + 0x14, endianness),
            p_align: read_u32_to_u64_from_u8_vec(bytes, s + 0x1c, endianness),
        });
        s += phsize as usize;
    }
    out
}

fn read_program_header_64(bytes: &Vec<u8>, phnum: u16, phsize: u16, start: u64, endianness: u8) -> Vec<ProgramHeaderEntry> {
    let mut out = Vec::<ProgramHeaderEntry>::with_capacity(phnum as usize);
    let mut s = start as usize;
    for _ in 0..phnum {
        out.push(ProgramHeaderEntry {
            p_type: read_u32_from_u8_vec(bytes, s + 0x0, endianness),
            p_flags: read_u32_from_u8_vec(bytes, s + 0x4, endianness),
            p_offset: read_u64_from_u8_vec(bytes, s + 0x8, endianness),
            p_vaddr: read_u64_from_u8_vec(bytes, s + 0x10, endianness),
            p_paddr: read_u64_from_u8_vec(bytes, s + 0x18, endianness),
            p_filesz: read_u64_from_u8_vec(bytes, s + 0x20, endianness),
            p_memsz: read_u64_from_u8_vec(bytes, s + 0x28, endianness),
            p_align: read_u64_from_u8_vec(bytes, s + 0x30, endianness),
        });
        s += phsize as usize;
    }
    out
}

fn read_section_header_32(bytes: &Vec<u8>, shnum: u16, shsize: u16, start: u64, endianness: u8) -> Vec<SectionHeaderEntry> {
    let mut out = Vec::<SectionHeaderEntry>::with_capacity(shnum as usize);
    let mut s = start as usize;
    for _ in 0..shnum {
        out.push(SectionHeaderEntry{
            sh_name: read_u32_from_u8_vec(bytes, s + 0x0, endianness),
            sh_type: read_u32_from_u8_vec(bytes, s + 0x4, endianness),
            sh_flags: read_u32_to_u64_from_u8_vec(bytes, s + 0x8, endianness),
            sh_addr: read_u32_to_u64_from_u8_vec(bytes, s + 0xc, endianness),
            sh_offset: read_u32_to_u64_from_u8_vec(bytes, s + 0x10, endianness),
            sh_size: read_u32_to_u64_from_u8_vec(bytes, s + 0x14, endianness),
            sh_link: read_u32_from_u8_vec(bytes, s + 0x18, endianness),
            sh_info: read_u32_from_u8_vec(bytes, s + 0x1c, endianness),
            sh_addralign: read_u32_to_u64_from_u8_vec(bytes, s + 0x20, endianness),
            sh_entsize: read_u32_to_u64_from_u8_vec(bytes, s + 0x24, endianness),
        });
        s += shsize as usize;
    }
    out
}

fn read_section_header_64(bytes: &Vec<u8>, shnum: u16, shsize: u16, start: u64, endianness: u8) -> Vec<SectionHeaderEntry> {
    let mut out = Vec::<SectionHeaderEntry>::with_capacity(shnum as usize);
    let mut s = start as usize;
    for _ in 0..shnum {
        out.push(SectionHeaderEntry{
            sh_name: read_u32_from_u8_vec(bytes, s + 0x0, endianness),
            sh_type: read_u32_from_u8_vec(bytes, s + 0x4, endianness),
            sh_flags: read_u64_from_u8_vec(bytes, s + 0x8, endianness),
            sh_addr: read_u64_from_u8_vec(bytes, s + 0x10, endianness),
            sh_offset: read_u64_from_u8_vec(bytes, s + 0x18, endianness),
            sh_size: read_u64_from_u8_vec(bytes, s + 0x20, endianness),
            sh_link: read_u32_from_u8_vec(bytes, s + 0x28, endianness),
            sh_info: read_u32_from_u8_vec(bytes, s + 0x2c, endianness),
            sh_addralign: read_u64_from_u8_vec(bytes, s + 0x30, endianness),
            sh_entsize: read_u64_from_u8_vec(bytes, s + 0x38, endianness),
        });
        s += shsize as usize;
    }
    out
}

fn abi_string(abi: u8) -> String {
    match abi {
        0x0 => format!("none"),
        0x3 => format!("Linux"),
        _ => format!("unknown (0x{abi:02x})")
    }
}

fn shstring(bytes: &Vec<u8>, idx: u32) -> String {
    let i = idx as usize;
    let mut j = i;
    while j < bytes.len() {
        if bytes[j] == 0x0 {
            break;
        }
        j += 1;
    }
    let s = &bytes[i..j];
    // println!("0x{:08x}..{}, 0x{:02x} 0x{:02x}", i, s.len(), s[0], s[1]);
    let s = match std::str::from_utf8(s) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    String::from(s)
}

fn build_section_table(bytes: &Vec<u8>, common_header: &HeaderCommon, section_headers: &Vec<SectionHeaderEntry>) -> HashMap<String, Section> {
    let mut hashmap = HashMap::<String, Section>::new();
    for entry in section_headers {
        let key = shstring(bytes, section_headers[common_header.e_shstrndx as usize].sh_offset as u32 + entry.sh_name);
        hashmap.insert(key, Section {
            addr: entry.sh_addr,
            bytes: bytes[entry.sh_offset as usize..(entry.sh_offset as usize + entry.sh_size as usize)].to_vec()
        });
    }
    hashmap
}

fn build_program_table(bytes: &Vec<u8>, common_header: &HeaderCommon, program_headers: &Vec<ProgramHeaderEntry>) -> Vec<Segment> {
    let mut v = Vec::<Segment>::new();
    for entry in program_headers {
        v.push(Segment {
            perm: entry.p_flags as u8,
            offset: entry.p_offset,
            paddr: entry.p_paddr,
            vaddr: entry.p_vaddr,
            size: entry.p_filesz as usize,
        });
    }
    v
}

fn build_program(bytes: &Vec<u8>, header: &Header, common_header: &HeaderCommon, program_headers: &Vec<ProgramHeaderEntry>, section_headers: &Vec<SectionHeaderEntry>) -> Program {
    Program{
        bits: if header.class == 0x1 { 32 } else if header.class == 0x2 { 64 } else { 0 },
        endianess: if header.data == 0x1 { LITTLE_ENDIAN } else { BIG_ENDIAN },
        machine_type: machine_type_string(common_header.e_machine).to_string(),
        program_table: build_program_table(bytes, common_header, program_headers),
        section_table: build_section_table(bytes, common_header, section_headers)
    }
}

pub fn load_program_from_bytes(bytes: &Vec<u8>) -> Program {
    let header = read_header(bytes);
    // println!("ELF version {}, {}-bit, {}, ABI {} version {}",
    //     header.version, 
    //     match header.class {
    //         0x1 => "32",
    //         0x2 => "64",
    //         _ => "?"
    //     }, 
    //     match header.data {
    //         0x1 => "little-endian",
    //         0x2 => "big-endian",
    //         _ => "unknown-endian"
    //     }, 
    //     abi_string(header.abi), 
    //     header.abi_version);
    let common_header = if header.class == 0x1 {
        read_common_header_32(bytes, header.data)
    } else {
        read_common_header_64(bytes, header.data)
    };
    // println!("{} file, {}, version {}",
    //     elf_file_type_string(common_header.e_type),
    //     machine_type_string(common_header.e_machine),
    //     common_header.e_version);
    // println!("entry point = 0x{:08x}", common_header.e_entry);
    // println!("program header = 0x{:08x}", common_header.e_phoff);
    // println!("section header = 0x{:08x}", common_header.e_shoff);
    // println!("header size = 0x{:08x}", common_header.e_ehsize);
    let program_headers = if header.class == 0x1 {
        read_program_header_32(bytes, common_header.e_phnum, common_header.e_phentsize, common_header.e_phoff, header.data)
    } else {
        read_program_header_64(bytes, common_header.e_phnum, common_header.e_phentsize, common_header.e_phoff, header.data)
    };
    // println!("Program headers: count={}", common_header.e_phnum);
    // for entry in &program_headers {
    //     println!("{} offset=0x{:08x}, size=0x{:08x}, align=0x{:04x}", 
    //         rwx_string(entry.p_flags), entry.p_offset, entry.p_filesz, entry.p_align);
    // }
    let section_headers = if header.class == 0x1 {
        read_section_header_32(bytes, common_header.e_shnum, common_header.e_shentsize, common_header.e_shoff, header.data)
    } else {
        read_section_header_64(bytes, common_header.e_shnum, common_header.e_shentsize, common_header.e_shoff, header.data)
    };
    // println!("Section headers: count={}", common_header.e_shnum);
    // for entry in &section_headers {
    //     println!("name={:<16} offset=0x{:08x}, size=0x{:08x}", 
    //         shstring(bytes, section_headers[common_header.e_shstrndx as usize].sh_offset as u32 + entry.sh_name),
    //         entry.sh_offset,
    //         entry.sh_size);
    // }
    build_program(bytes, &header, &common_header, &program_headers, &section_headers)
}
