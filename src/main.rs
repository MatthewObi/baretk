use std::env;
use std::fs::File;
use std::io::{Read, Write};
mod dis;
mod query;
mod prog;
mod dump;
mod util;

mod elf;
mod pe;

mod arm;
mod x86;
mod riscv;

// An objdump-like utility.
fn cmd_dump(in_file: &String, out_file: Option<&String>) {
    let output = dump::dump_program(&prog::load_program_from_file(in_file).unwrap());
    if out_file.is_some() {
        let out = out_file.unwrap();
        let mut file = match File::open(out) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("Error opening file {}: {}", out, error);
                return;
            }
        };
        if let Err(error) = file.write(output.as_bytes()) {
            eprintln!("Error writing file {}: {}", out, error);
            return;
        }
    }
    else {
        println!("{}", output);
    }
}

fn cmd_disassemble(in_file: &String, out_file: Option<&String>) {
    let mut file = match File::open(in_file) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error opening file {}: {}", in_file, error);
            return;
        }
    };

    let mut contents: Vec<u8> = vec![];
    if let Err(error) = file.read_to_end(&mut contents) {
        eprintln!("Error reading file {}: {}", in_file, error);
        return;
    }

    let disassembly = dis::disassemble(&contents);
    if out_file.is_some() {
        let out = out_file.unwrap();
        let mut file = match File::open(out) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("Error opening file {}: {}", out, error);
                return;
            }
        };
        if let Err(error) = file.write(disassembly.as_bytes()) {
            eprintln!("Error writing file {}: {}", out, error);
            return;
        }
    }
    else {
        println!("{}", disassembly);
    }
}

fn cmd_help() {
    println!("Available commands:");
    println!("    baretk dis");
    println!("    baretk dump");
    println!("    baretk help");
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: baretk <subcommand>");
        return;
    }

    match args[1].as_str() {
        "dis" => {
            if args.len() < 3 {
                eprintln!("Usage: baretk dis <input> [output]");
                return;
            }
            else if args.len() < 4 {
                cmd_disassemble(&args[2], None);
            }
            else {
                cmd_disassemble(&args[2], Some(&args[3]));
            }
        }
        "dump" => {
            if args.len() < 3 {
                eprintln!("Usage: baretk dump <input> [output]");
                return;
            }
            else if args.len() < 4 {
                cmd_dump(&args[2], None);
            }
            else {
                cmd_dump(&args[2], Some(&args[3]));
            }
        }
        _ => cmd_help(),
    }
}
