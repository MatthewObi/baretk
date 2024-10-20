use core::str;
use std::collections::HashMap;

use crate::prog::{Program, Section, Segment};
use crate::util::{read_u16_from_u8_vec, read_u32_from_u8_vec, read_u32_to_u64_from_u8_vec, read_u64_from_u8_vec, LITTLE_ENDIAN, RWX_EXEC, RWX_WRITE, RWX_READ};

const PE_OFFSET_OFFSET: usize = 0x3c;

pub fn check_is_pe_executable(bytes: &Vec<u8>) -> bool {
    // DOS header
    if !bytes.starts_with(&[0x4du8, 0x5au8]) {
        return false;
    };
    if bytes.len() < PE_OFFSET_OFFSET + 4 {
        return false;
    };
    let b: &[u8; 4] = (&bytes[PE_OFFSET_OFFSET..PE_OFFSET_OFFSET + 4]).try_into().unwrap();
    let offset = u32::from_le_bytes(*b) as usize;
    if bytes.len() < offset + 4 {
        return false;
    };
    // PE header
    bytes[offset..offset+4].starts_with(&[0x50u8, 0x45u8, 0x00u8, 0x00u8])
}

#[derive(PartialEq)]
struct MachineType(u16);

impl MachineType {
    const UNKNOWN: MachineType = MachineType(0x0);
    const RISCV32: MachineType = MachineType(0x5032);
    const RISCV64: MachineType = MachineType(0x5064);
    const I386: MachineType = MachineType(0x14c); // i386 (x86 32-bit)
    const AMD64: MachineType = MachineType(0x8664); // (x86-64)
}

fn get_machine_type_string(machine: u16) -> &'static str {
    let m = MachineType(machine);
    match m {
        MachineType::UNKNOWN => "unknown",
        MachineType::RISCV32 => "riscv32",
        MachineType::RISCV64 => "riscv64",
        MachineType::I386 => "x86",
        MachineType::AMD64 => "amd64",
        _ => "?",
    }
}

const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;

fn get_rwx_perm(flags: u32) -> u8 {
    let mut out = 0u8;
    if (flags & IMAGE_SCN_MEM_EXECUTE) != 0 {
        out |= RWX_EXEC;
    }
    if (flags & IMAGE_SCN_MEM_WRITE) != 0 {
        out |= RWX_WRITE;
    }
    if (flags & IMAGE_SCN_MEM_READ) != 0 {
        out |= RWX_READ;
    }
    out
}

fn get_name_from_section_header(hdr: &SectionHeader) -> String {
    let mut s = String::new();
    for c in hdr.name {
        if c.is_ascii() && c != 0 {
            s.push(c as char);
        }
        else {
            return s;
        }
    }
    s
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
struct MachChar(u16);

impl MachChar {
    const RELOCS_STRIPPED: MachChar = MachChar(0x1);
    const EXECUTABLE_IMAGE: MachChar = MachChar(0x2);
    const LARGE_ADDRESS_AWARE: MachChar = MachChar(0x20);

    fn is_large_address_aware(self) -> bool { (self.0 & MachChar::LARGE_ADDRESS_AWARE.0) != 0 }
    fn is_executable(self) -> bool { (self.0 & MachChar::EXECUTABLE_IMAGE.0) != 0 }
    fn is_relocs_stripped(self) -> bool { (self.0 & MachChar::RELOCS_STRIPPED.0) != 0 }
}

fn characteristics_string(c: u16) -> String {
    let mut s = String::new();
    let cs = MachChar(c);
    if cs.is_executable() {
        s += "executable, "
    }
    if cs.is_relocs_stripped() {
        s += "stripped, "
    }
    if cs.is_large_address_aware() {
        s += "large address aware, "
    }
    s.strip_suffix(", ").unwrap_or(s.as_str()).to_string()
}

#[derive(Debug)]
struct CoffHeader {
    machine: u16,
    num_sections: u16,
    timestamp: u32,
    // depracated_symbol_table_ptr: u32,  // We don't need this.
    // depracated_number_of_symbols: u32, // or this.
    optional_header_size: u16,
    characteristics: u16,
}

struct OptionalHeader {
    magic: u16,
    major_link_ver: u8,
    minor_link_ver: u8,
    code_size: u32,
    data_size: u32,
    bss_size: u32,
    entry_point: u32,
    base_addr: u32,
}

struct WinHeader {
    section_alignment: u32,
    file_alignment: u32,
}

struct SectionHeader {
    name: [u8; 8],
    virtual_size: u32,
    virtual_addr: u32,
    data_size: u32,
    data_ptr: u32,
    reloc_ptr: u32,
    _line_num_ptr: u32,
    _reloc_count: u16,
    _line_num_count: u16,
    characteristics: u32,
}

fn read_coff_header(bytes: &Vec<u8>, offset: usize) -> CoffHeader {
    CoffHeader {
        machine: read_u16_from_u8_vec(bytes, offset+0x4, LITTLE_ENDIAN),
        num_sections: read_u16_from_u8_vec(bytes, offset+0x6, LITTLE_ENDIAN),
        timestamp: read_u32_from_u8_vec(bytes, offset+0x8, LITTLE_ENDIAN),
        optional_header_size: read_u16_from_u8_vec(bytes, offset+0x14, LITTLE_ENDIAN),
        characteristics: read_u16_from_u8_vec(bytes, offset+0x16, LITTLE_ENDIAN),
    }
}

fn read_optional_header(bytes: &Vec<u8>, offset: usize) -> OptionalHeader {
    OptionalHeader {
        magic: read_u16_from_u8_vec(bytes, offset, LITTLE_ENDIAN),
        major_link_ver: bytes[offset+0x2],
        minor_link_ver: bytes[offset+0x3],
        code_size: read_u32_from_u8_vec(bytes, offset+0x4, LITTLE_ENDIAN),
        data_size: read_u32_from_u8_vec(bytes, offset+0x8, LITTLE_ENDIAN),
        bss_size: read_u32_from_u8_vec(bytes, offset+0xc, LITTLE_ENDIAN),
        entry_point: read_u32_from_u8_vec(bytes, offset+0x10, LITTLE_ENDIAN),
        base_addr: read_u32_from_u8_vec(bytes, offset+0x14, LITTLE_ENDIAN),
    }
}

fn read_windows_header_32p(bytes: &Vec<u8>, offset: usize) -> WinHeader {
    WinHeader {
        section_alignment: read_u32_from_u8_vec(bytes, offset+0x4, LITTLE_ENDIAN),
        file_alignment: read_u32_from_u8_vec(bytes, offset+0x8, LITTLE_ENDIAN),
    }
}

fn read_section_header_32(bytes: &Vec<u8>, offset: usize) -> SectionHeader {
    SectionHeader {
        name: bytes[offset..offset+8].try_into().expect("Bad array slice"),
        virtual_size: read_u32_from_u8_vec(bytes, offset+0x8, LITTLE_ENDIAN),
        virtual_addr: read_u32_from_u8_vec(bytes, offset+0xc, LITTLE_ENDIAN),
        data_size: read_u32_from_u8_vec(bytes, offset+0x10, LITTLE_ENDIAN),
        data_ptr: read_u32_from_u8_vec(bytes, offset+0x14, LITTLE_ENDIAN),
        reloc_ptr: read_u32_from_u8_vec(bytes, offset+0x18, LITTLE_ENDIAN),
        _line_num_ptr: read_u32_from_u8_vec(bytes, offset+0x1c, LITTLE_ENDIAN),
        _reloc_count: read_u16_from_u8_vec(bytes, offset+0x20, LITTLE_ENDIAN),
        _line_num_count: read_u16_from_u8_vec(bytes, offset+0x22, LITTLE_ENDIAN),
        characteristics: read_u32_from_u8_vec(bytes, offset+0x24, LITTLE_ENDIAN),
    }
}

fn build_section_table(bytes: &Vec<u8>, _coff_header: &CoffHeader, section_headers: &HashMap<String, SectionHeader>) -> HashMap<String, Section> {
    let mut hashmap = HashMap::<String, Section>::new();
    for (k, v) in section_headers {
        hashmap.insert(k.to_string(), Section {
            addr: v.data_ptr as u64,
            bytes: bytes[v.data_ptr as usize..(v.data_ptr as usize + v.data_size as usize)].to_vec()
        });
    }
    hashmap
}

fn build_program_table(_bytes: &Vec<u8>, _coff_header: &CoffHeader, section_headers: &HashMap<String, SectionHeader>) -> Vec<Segment> {
    let mut v = Vec::<Segment>::new();
    for (_, entry) in section_headers {
        v.push(Segment {
            perm: get_rwx_perm(entry.characteristics),
            offset: entry.data_ptr as u64,
            paddr: entry.data_ptr as u64,
            vaddr: entry.virtual_addr as u64,
            size: entry.data_size as usize,
        });
    }
    v
}

fn build_program(bytes: &Vec<u8>, coff_header: &CoffHeader, opt_header: Option<OptionalHeader>, section_headers: &HashMap<String, SectionHeader>) -> Program {
    Program {
        bits: if let Some(opt) = opt_header { match opt.magic { 0x10b => 32, 0x20b => 64, _ => 32} } else { 32 },
        endianess: LITTLE_ENDIAN,
        machine_type: get_machine_type_string(coff_header.machine).to_string(),
        program_table: build_program_table(bytes, coff_header, section_headers),
        section_table: build_section_table(bytes, coff_header, section_headers)
    }
}

pub fn load_program_from_bytes(bytes: &Vec<u8>) -> Program {
    let b: &[u8; 4] = (&bytes[PE_OFFSET_OFFSET..PE_OFFSET_OFFSET + 4]).try_into().unwrap();
    let offset = u32::from_le_bytes(*b) as usize;
    let coff_header = read_coff_header(bytes, offset);
    println!("{} machine ({}), {} section(s)", get_machine_type_string(coff_header.machine), characteristics_string(coff_header.characteristics),
        coff_header.num_sections);
    let optional_header = if coff_header.optional_header_size > 0 {
        Some(read_optional_header(bytes, offset+0x18))
    } else {
        None
    };
    if let Some(ref opt) = optional_header {
        println!("{} v{}.{}, base_addr=0x{:08x} code_size=0x{:08x} entry_point=0x{:08x}", 
            match opt.magic { 0x10b => "PE32", 0x20b => "PE32+", _ => ""},
            opt.major_link_ver,
            opt.minor_link_ver,
            opt.base_addr,
            opt.code_size,
            opt.entry_point);
    }
    let toffset = coff_header.optional_header_size as usize + offset + 0x18;
    println!("Section table: 0x{:08x}", toffset);
    let mut section_table = HashMap::<String, SectionHeader>::new();
    for i in 0..coff_header.num_sections {
        let section_header = read_section_header_32(bytes, toffset+(i as usize * 40));
        let section_name = get_name_from_section_header(&section_header);
        println!("{:<8} 0x{:<08x}, 0x{:<08x}", section_name, section_header.virtual_addr, section_header.virtual_size);
        section_table.insert(section_name.to_string(), section_header);
    }
    println!("TODO: finish parsing PE executable files.\n");
    // prog::build_program_from_binary(bytes, Some(bits), Some(LITTLE_ENDIAN), Some(get_machine_type_string(coff_header.machine).to_string()))
    build_program(bytes, &coff_header, optional_header, &section_table)
    // Program {

    // }
}