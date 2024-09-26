use crate::{prog::Program, util::{BIG_ENDIAN, LITTLE_ENDIAN}};

pub fn rwx_string(flags: u32) -> String {
    format!("{}{}{}", 
        if (flags & 0x4) != 0x0 { "R" } else { " " },
        if (flags & 0x2) != 0x0 { "W" } else { " " },
        if (flags & 0x1) != 0x0 { "X" } else { " " })
}

pub fn dump_program(program: &Program) -> String {
    let mut s = String::new();
    s += format!("{}-bit, {}, {} executable\n", 
        program.bits,
        match program.endianess { LITTLE_ENDIAN => "little-endian", BIG_ENDIAN => "big-endian", _ => "?-endian" },
        program.machine_type
    ).as_str();
    s += format!("Segments:\n  {:<6} {:<8} {:<8} {:<8} {:<8}\n", " Perm", "Offset", "PAddr", "VAddr", "Size").as_str();
    for item in program.program_table.iter() {
        s += format!("  {:<6} {:08x} {:08x} {:08x} {:08x}\n", rwx_string(item.perm as u32), item.offset, item.paddr, item.vaddr, item.size).as_str();
    }
    s += format!("Sections:\n  {:<16} {:<8} {:<8}\n", " Name", "Offset", "Size").as_str();
    for item in program.section_table.iter() {
        s += format!("  {:<16} {:08x} {:08x}\n", item.0, item.1.addr, item.1.bytes.len()).as_str();
    }
    s
}
