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
fn cmd_dump(args: Vec<String>) {
    if let Some(in_file) = args.get(0) {
        let out_file = args.get(1);
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
    else {
        eprintln!("Usage: baretk dump <in_file> [out_file]");
    }
}

fn cmd_disassemble(args: Vec<String>) {
    if let Some(in_file) = args.get(0) {
        let out_file = args.get(1);
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
    else {
        eprintln!("Usage: baretk dis <in_file> [out_file]");
    }
}

fn cmd_help() {
    println!("Available commands:");
    for cmd in COMMANDS {
        println!("    baretk {} - {}", cmd.name, cmd.desc);
    }
    println!("    baretk help - Prints this help.");
}

struct Command {
    name: &'static str,
    desc: &'static str,
    func: fn(Vec<String>),
}

const COMMANDS: &[Command] = &[
    Command { name: "dis", desc: "Disassembles an input binary.", func: cmd_disassemble },
    Command { name: "dump", desc: "Dumps information from an input binary.", func: cmd_dump }
];

fn main() {
    let mut args = env::args();
    args.next().expect("program");

    if let Some(command) = args.next() {
        if let Some(cmd) = COMMANDS.iter().find(|cmd| cmd.name == command.as_str()) {
            (cmd.func)(args.collect());
            return;
        }
        cmd_help();
        return;
    }
    else {
        eprintln!("Usage: baretk <command>");
        cmd_help();
        return;
    }
}
