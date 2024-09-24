use crate::prog::Program;

pub fn dump_program(program: &Program) -> String {
    let mut s = String::new();
    s += format!("{}, {}, {} executable\n", 
        if (program.flags & 1) != 0 { "32-bit" } else { "64-bit" },
        if (program.flags & 2) != 0 { "little-endian" } else { "big-endian" },
        program.machine_type
    ).as_str();
    s += format!("Sections:\n  {:<16} {:<8} {:<8}\n", " Name", "Offset", "Size").as_str();
    for item in program.section_table.iter() {
        s += format!("  {:<16} {:08x} {:08x}\n", item.0, item.1.addr, item.1.bytes.len()).as_str();
    }
    s
}
