use core::slice;
use std::{alloc::Layout, ffi::{c_int, CStr}};

use prog::Program;
use util::LITTLE_ENDIAN;

mod query;
mod dis;
mod prog;
mod util;

mod arm;
mod riscv;
mod pe;
mod elf;
mod x86;

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
pub extern "C" fn baretk_print_strings(path: *const i8, min_len: i32, out_path: *const i8) -> i32 {
    let in_file = match cstr_to_string(path) {
        Some(s) => s,
        None => { return 0; }
    };

    let contents = match util::try_read_file_contents(in_file.as_str()) {
        Err(()) => return 0,
        Ok(vec) => vec,
    };

    let printable = false;

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
pub extern "C" fn baretk_disassemble_from_file(path: *const i8, out_path: *const i8) -> i32 {
    let in_file = match cstr_to_string(path) {
        Some(s) => s,
        None => { return 0; }
    };

    let contents = match util::try_read_file_contents(in_file.as_str()) {
        Err(()) => return 0,
        Ok(vec) => vec,
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
    let in_file = match cstr_to_string(path) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let layout = Layout::new::<Program>();

    unsafe {
        let prog = match prog::load_program_from_file(&in_file) {
            Ok(prog) => prog,
            Err(()) => {
                return std::ptr::null_mut()
            },
        };

        let dst: *mut Program = std::alloc::alloc(layout).cast();
        dst.copy_from(&prog, 1);

        dst
    }
}

#[no_mangle]
pub extern "C" fn baretk_free_program(program: *mut Program) {
    let layout = Layout::new::<Program>();

    if program.is_null() {
        return;
    }
    
    unsafe {
        std::alloc::dealloc(program.cast(), layout);
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
