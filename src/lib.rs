use core::slice;
use std::{ffi::{c_int, CStr}, ptr::{null, null_mut}};

use dis::Disassembly;
use prog::{Program, Segment};
use decomp::Decomp;
use util::LITTLE_ENDIAN;

mod query;
mod dis;
mod decomp;
mod prog;
mod util;

mod arm;
mod riscv;
mod pe;
mod elf;
mod x86;

#[repr(C)]
pub struct SegmentArray {
    ptr: *const Segment,
    size: usize
}

#[repr(C)]
pub struct U8Array {
    ptr: *const u8,
    size: usize
}

#[repr(C)]
pub struct SectionC {
    addr: u64,
    bytes: U8Array,
}

fn cstr_to_string(s: *const i8) -> Option<String> {
    if s.is_null() {
        None
    }
    else {
        unsafe {
            match CStr::from_ptr(s).to_str() {
                Ok(s) => Some(String::from(s)),
                Err(error) => {
                    eprintln!("Error parsing string: {}", error);
                    None
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn baretk_print_strings(path: *const i8, min_len: i32, printable: bool, out_path: *const i8) -> i32 {
    let Some(in_file) = cstr_to_string(path) else {
        return 0;
    };

    let Ok(contents) = util::try_read_file_contents(in_file.as_str()) else {
        return 0;
    };

    let strings = query::get_strings(contents.as_slice(), min_len as usize, printable);
    if let Some(out) = cstr_to_string(out_path) {
        if !util::try_write_file_lines(out.as_str(), strings) {
            return 0;
        }
        return 1;
    }
    else {
        println!("ASCII strings found in {}:", in_file);
        for str in strings {
            println!(" {}", str);
        }
        return 1;
    }
}

#[no_mangle]
pub extern "C" fn baretk_print_strings_from_bytes(bytes: *const u8, size: usize, min_len: i32, out_path: *const i8) -> i32 {
    if bytes.is_null() {
        return 0
    }
    let slice = unsafe {
        slice::from_raw_parts(bytes, size)
    };
    let strings = query::get_strings(slice, min_len as usize, true);
    let out_file = unsafe { 
        if out_path.is_null() {
            None
        } else {
            match CStr::from_ptr(out_path).to_str() {
                Ok(s) => Some(s),
                Err(_error) => None,
            }
        }
    };

    if let Some(out) = out_file {
        if !util::try_write_file_lines(out, strings) {
            return 0;
        }
        return 1;
    }
    else {
        println!("ASCII strings found:");
        for str in strings {
            println!(" {}", str);
        }
        return 1;
    }
}

#[no_mangle]
pub extern "C" fn baretk_disassemble_file(path: *const i8, out_path: *const i8) -> i32 {
    let Some(in_file) = cstr_to_string(path) else {
        return 0;
    };

    let Ok(contents) = util::try_read_file_contents(in_file.as_str()) else {
        return 0;
    };

    let dis = dis::disassemble(&contents);

    let output = dis.print(true);

    if let Some(out) = cstr_to_string(out_path) {
        if !util::try_write_file(out.as_str(), output.as_bytes()) {
            return 0;
        }
        return 1;
    }

    return 1;
}

#[no_mangle]
pub extern "C" fn baretk_load_program(path: *const i8) -> *mut prog::Program {
    let Some(in_file) = cstr_to_string(path) else {
        return null_mut();
    };

    let Ok(prog) = prog::load_program_from_file(&in_file) else {
        return null_mut();
    };

    Box::into_raw(Box::new(prog))
}

#[no_mangle]
pub extern "C" fn baretk_clone_program(program: *const Program) -> *mut Program {
    if program.is_null() {
        return null_mut();
    }
    
    unsafe {
        let prog = Box::new((*program).clone());
        Box::into_raw(prog)
    }
}

#[no_mangle]
pub extern "C" fn baretk_free_program(program: *mut Program) {
    if program.is_null() {
        return;
    }
    
    unsafe {
        drop(Box::from_raw(program))
    }
}

#[no_mangle]
pub extern "C" fn baretk_get_endianess(program: *const Program) -> c_int {
    if program.is_null() {
        return LITTLE_ENDIAN as c_int;
    }

    unsafe { (*program).endianess as c_int }
}

#[no_mangle]
pub extern "C" fn baretk_get_machine_type(program: *const Program) -> *const i8 {
    if program.is_null() {
        return "???".as_ptr().cast();
    }

    unsafe { (*program).machine_type.as_str().as_ptr().cast() }
}

#[no_mangle]
pub extern "C" fn baretk_get_segments(program: *const Program) -> SegmentArray {
    if program.is_null() {
        return SegmentArray { ptr: null(), size: 0usize }
    }

    unsafe { 
        SegmentArray { ptr: (*program).program_table.as_ptr(), size: (*program).program_table.len() }
    }
}

#[no_mangle]
pub extern "C" fn baretk_get_section(program: *const Program, k: *const i8) -> SectionC {
    if program.is_null() {
        return SectionC { addr: 0, bytes: U8Array { ptr: null(), size: 0usize } };
    }

    let key = match cstr_to_string(k) {
        Some(s) => s,
        None => { return SectionC { addr: 0, bytes: U8Array { ptr: null(), size: 0usize } }; }
    };

    unsafe { 
        let section = (*program).section_table.get(&key);
        if let Some(sect) = section {
            SectionC { addr: sect.addr, bytes: U8Array { ptr: sect.bytes.as_ptr(), size: sect.bytes.len() } }
        } else {
            SectionC { addr: 0, bytes: U8Array { ptr: null(), size: 0usize } }
        }
    }
}

#[no_mangle]
pub extern "C" fn baretk_disassemble_from_program(program: *mut Program) -> *mut Disassembly {
    if program.is_null() {
        return null_mut();
    }

    let dis = unsafe {
        let prog = Box::from_raw(program.cast());
        dis::disassemble_program(*prog)
    };
    Box::into_raw(Box::new(dis))
}

#[no_mangle]
pub extern "C" fn baretk_disassemble_from_file(path: *const i8) -> *mut Disassembly {
    let Some(in_file) = cstr_to_string(path) else {
        return null_mut();
    };

    let Ok(prog) = prog::load_program_from_file(&in_file) else {
        return null_mut();
    };

    let dis = dis::disassemble_program(prog);
    Box::into_raw(Box::new(dis))
}

#[no_mangle]
pub extern "C" fn baretk_get_program_from_disassembly(disasm: *const Disassembly) -> *const Program {
    if disasm.is_null() {
        return null_mut();
    }

    unsafe { (*disasm).program() }
}

#[no_mangle]
pub extern "C" fn baretk_free_disassembly(disasm: *mut Disassembly) {
    if disasm.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(disasm))
    }
}

#[no_mangle]
pub extern "C" fn baretk_decomp_disassembly(disasm: *mut Disassembly, lang: i32) -> *mut Decomp {
    if disasm.is_null() {
        return null_mut();
    }

    unsafe {
        let dis = Box::from_raw(disasm);
        let lang = match lang {
            0 => decomp::Language::Pseudocode,
            1 => decomp::Language::C,
            _ => decomp::Language::Pseudocode,
        };
        let dec = decomp::decomp_program(*dis, lang);
        Box::into_raw(Box::new(dec))
    }
}

#[no_mangle]
pub extern "C" fn baretk_decomp_from_file(path: *const i8) -> *mut Decomp {
    let Some(in_file) = cstr_to_string(path) else {
        return null_mut();
    };

    let Ok(bytes) = util::try_read_file_contents(in_file.as_str()) else {
        return null_mut();
    };

    let decomp = decomp::decomp_program_from_bytes(bytes.as_slice(), decomp::Language::Pseudocode);
    Box::into_raw(Box::new(decomp))
}

#[no_mangle]
pub extern "C" fn baretk_get_disassembly_from_decomp(decomp: *const Decomp) -> *const Disassembly {
    if decomp.is_null() {
        return null_mut();
    }

    unsafe { &(*decomp).disassembly }
}

#[no_mangle]
pub extern "C" fn baretk_free_decomp(decomp: *mut Decomp) {
    if decomp.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(decomp))
    }
}
