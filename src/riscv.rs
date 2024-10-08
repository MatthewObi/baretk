use crate::prog::{Section, Program};

#[derive(PartialEq)]
#[derive(Copy, Clone)]
struct Register(u8);

impl Register {
    const ZERO: Register = Register(0x0);
    const RA: Register = Register(0x1);
    const COUNT: usize = Self::RA.0 as usize + 1;

    const REG_NAMES: [&'static str; Self::COUNT] = [
        "Zero",
        "ra"
    ];

    fn name(self) -> &'static str {
        if (self.0 as usize) < Self::REG_NAMES.len() {
            return Self::REG_NAMES[self.0 as usize]
        }
        "?"
    }
}

fn disassemble_instruction(bytes: &[u8], offset: usize) -> String {
    let ins = u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap());
    if (ins & 3) == 3 {
        // disassemble_32(ins)
    }
    String::new()
}

pub fn disassemble_riscv(section: &Section, program: &Program) -> String {
    disassemble_instruction(section.bytes.as_slice(), 0x0);
    println!("{}", Register::RA.name());
    format!("TODO: RISC-V stuff")
}
