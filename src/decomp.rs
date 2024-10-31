use crate::dis::{self, Disassembly};

pub struct Decomp {
    disassembly: Disassembly,
}

pub fn decomp_program_from_bytes(bytes: &Vec<u8>) -> Decomp {
    let dis = dis::disassemble(bytes);
    decomp_program(&dis)
}

pub fn decomp_program(dis: &Disassembly) -> Decomp {
    todo!("Put code for C code decompilation.")
}
