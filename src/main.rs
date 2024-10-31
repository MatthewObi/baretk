use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashMap;
mod dis;
mod decomp;
mod query;
mod prog;
mod dump;
mod util;

mod elf;
mod pe;

mod arm;
mod x86;
mod riscv;

struct ArgList {
    named_args: HashMap<String, String>,
    pos_args: Vec<String>
}

fn parse_cmd_args(args: Vec<String>) -> ArgList {
    let mut named_args = HashMap::<String, String>::new();
    let mut pos_args = Vec::<String>::new();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg.starts_with("--") {
            named_args.insert(arg.strip_prefix("--").unwrap().to_string(), "".to_string());
        }
        else if arg.starts_with("-") {
            if let Some(v) = it.next() {
                named_args.insert(arg.strip_prefix("-").unwrap().to_string(), v.clone());
            }
        }
        else {
            pos_args.push(arg.clone())
        }
    }
    ArgList { named_args, pos_args }
}

// An objdump-like utility.
fn cmd_dump(args: ArgList) {
    if let Some(in_file) = args.pos_args.get(0) {
        let out_file = args.pos_args.get(1);
        let output = dump::dump_program(&prog::load_program_from_file(in_file).unwrap());
        if out_file.is_some() {
            let out = out_file.unwrap();
            let mut file = match File::create(out) {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Error creating file {}: {}", out, error);
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

fn cmd_disassemble(args: ArgList) {
    if let Some(in_file) = args.pos_args.get(0) {
        let out_file = args.pos_args.get(1);
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
        let output = disassembly.print(true);
        if out_file.is_some() {
            let out = out_file.unwrap();
            let mut file = match File::create(out) {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Error creating file {}: {}", out, error);
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
        eprintln!("Usage: baretk dis <in_file> [out_file]");
    }
}

fn cmd_decompile(args: ArgList) {
    if let Some(in_file) = args.pos_args.get(0) {
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

        let _decomp = decomp::decomp_program_from_bytes(&contents);
        return;
    }
    else {
        eprintln!("Usage: baretk dis <in_file> [out_file]");
    }
}

fn cmd_strings(args: ArgList) {
    if let Some(in_file) = args.pos_args.get(0) {
        let out_file = args.pos_args.get(1);
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

        let min_len = if let Some(opt) = args.named_args.get("n") {
            let res = opt.parse::<usize>();
            if let Err(err) = res {
                eprintln!("Can't convert \"{}\" to number: {}", opt, err);
                return;
            }
            else { 
                res.ok() 
            }
        } else {
            None
        }.unwrap_or(4);

        let printable = args.named_args.contains_key("printable");

        let strings = query::get_strings(&contents, min_len, printable);
        if out_file.is_some() {
            let out = out_file.unwrap();
            let mut file = match File::create(out) {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Error creating file {}: {}", out, error);
                    return;
                }
            };
            for str in strings {
                if let Err(error) = file.write((str + "\n").as_bytes()) {
                    eprintln!("Error writing file {}: {}", out, error);
                    return;
                }
            }
        }
        else {
            println!("ASCII strings found in {}:", in_file);
            for str in strings {
                println!(" {}", str);
            }
        }
    }
    else {
        eprintln!("Usage: baretk strings <in_file> [out_file]");
        eprintln!("    -n <num> min. string length (default 4)");
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
    func: fn(ArgList),
}

const COMMANDS: &[Command] = &[
    Command { name: "dis", desc: "Disassembles an input binary.", func: cmd_disassemble },
    Command { name: "decomp", desc: "Decompiles an input binary.", func: cmd_decompile },
    Command { name: "dump", desc: "Dumps information from an input binary.", func: cmd_dump },
    Command { name: "strings", desc: "Prints strings found in an input binary.", func: cmd_strings },
];

fn main() {
    let mut args = env::args();
    args.next().expect("program");

    if let Some(command) = args.next() {
        if let Some(cmd) = COMMANDS.iter().find(|cmd| cmd.name == command.as_str()) {
            (cmd.func)(parse_cmd_args(args.collect()));
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
