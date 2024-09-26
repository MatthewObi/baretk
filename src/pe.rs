use crate::prog::{self, Program};
use crate::util::{LITTLE_ENDIAN, read_u16_from_u8_vec, read_u32_from_u8_vec, read_u64_from_u8_vec, read_u32_to_u64_from_u8_vec};

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

fn get_machine_type_string(machine: u16) -> String {
    let m = MachineType(machine);
    String::from(match m {
        MachineType::UNKNOWN => "unknown",
        MachineType::RISCV32 => "riscv32",
        MachineType::RISCV64 => "riscv64",
        MachineType::I386 => "x86",
        MachineType::AMD64 => "amd64",
        _ => "?",
    })
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
    OptionalHeader {}
}

pub fn load_program_from_bytes(bytes: &Vec<u8>) -> Program {
    let b: &[u8; 4] = (&bytes[PE_OFFSET_OFFSET..PE_OFFSET_OFFSET + 4]).try_into().unwrap();
    let offset = u32::from_le_bytes(*b) as usize;
    let coff_header = read_coff_header(bytes, offset);
    println!("{:?}", coff_header);
    println!("{} machine ({}), {} section(s)", get_machine_type_string(coff_header.machine), characteristics_string(coff_header.characteristics),
        coff_header.num_sections);
    let optional_header = if coff_header.optional_header_size > 0 {
        Some(read_optional_header(bytes, offset+0x18))
    } else {
        None
    };
    let toffset = coff_header.optional_header_size as usize + offset;
    println!("TODO: finish parsing PE executable files.\n");
    prog::build_program_from_binary(bytes, Some(32), Some(LITTLE_ENDIAN), Some(get_machine_type_string(coff_header.machine)))
    // Program {

    // }
}