use std::env;
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
        if let Some(out) = out_file {
            util::try_write_file(out, output.as_bytes());
        }
        else {
            println!("{}", output);
        }
    }
    else {
        cmd_dump_help();
    }
}

fn cmd_dump_help() {
    eprintln!("Usage: baretk dump <in_file> [out_file]");
    eprintln!("");
}

fn cmd_disassemble(args: ArgList) {
    if let Some(in_file) = args.pos_args.get(0) {
        let out_file = args.pos_args.get(1);
        let contents = match util::try_read_file_contents(in_file.as_str()) {
            Err(()) => { return; },
            Ok(bytes) => bytes,
        };

        let disassembly = dis::disassemble(&contents);
        let output = disassembly.print(true);
        if let Some(out) = out_file {
            util::try_write_file(out, output.as_bytes());
        }
        else {
            println!("{}", output);
        }
    }
    else {
        cmd_disassemble_help();
    }
}

fn cmd_disassemble_help() {
    eprintln!("Usage: baretk dis <in_file> [out_file]");
    eprintln!("");
}

fn cmd_decompile(args: ArgList) {
    if let Some(in_file) = args.pos_args.get(0) {
        let contents = match util::try_read_file_contents(in_file.as_str()) {
            Err(()) => { return; },
            Ok(bytes) => bytes,
        };

        let dest_lang = match args.named_args.get("lang").unwrap_or(&"pseudocode".to_string()).as_str() {
            "C" => decomp::Language::C,
            "c" => decomp::Language::C,
            "pseudo" => decomp::Language::Pseudocode,
            "pseudocode" => decomp::Language::Pseudocode,
            _ => decomp::Language::Pseudocode,
        };
        let decomp = decomp::decomp_program_from_bytes(&contents, dest_lang);
        println!("{}", decomp.print());
    }
    else {
        cmd_decompile_help();
    }
}


fn cmd_decompile_help() {
    eprintln!("Usage: baretk decomp <in_file> [out_file]");
    eprintln!("Optional params");
    eprintln!("    -lang <dest_lang> - Selects the output language.");
    eprintln!("        Valid options: C, c, pseudo, pseudocode");
    eprintln!("");
}

fn cmd_strings(args: ArgList) {
    if let Some(in_file) = args.pos_args.get(0) {
        let out_file = args.pos_args.get(1);
        let contents = match util::try_read_file_contents(in_file.as_str()) {
            Err(()) => { return; },
            Ok(bytes) => bytes,
        };

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

        let strings = query::get_strings(contents.as_slice(), min_len, printable);
        if let Some(out) = out_file {
            util::try_write_file_lines(out.as_str(), strings);
        }
        else {
            println!("ASCII strings found in {}:", in_file);
            for str in strings {
                println!(" {}", str);
            }
        }
    }
    else {
        cmd_strings_help();
    }
}

fn cmd_strings_help() {
    eprintln!("Usage: baretk strings <in_file> [out_file]");
    eprintln!("Optional params");
    eprintln!("    -n <num> min. string length (default 4)");
    eprintln!("    --printable - Restricts output to ASCII");
    eprintln!("                  strings");
    eprintln!("");
}

fn cmd_help(args: ArgList) {
    if args.named_args.len() == 0 && args.pos_args.len() == 0 {
        println!("Available commands:");
        for cmd in COMMANDS {
            println!("    baretk {} - {}", cmd.name, cmd.desc);
        }
        println!("Use `baretk help <command>` for more info on each command.");
    }
    else if let Some(command_name) = args.pos_args.get(0) {
        if let Some(cmd) = COMMANDS.iter().find(|cmd| cmd.name == command_name.as_str()) {
            (cmd.help)();
            return;
        }
        println!("Help for command `{}` not available yet.", command_name);
        return;
    }
    else {
        println!("Available commands:");
        for cmd in COMMANDS {
            println!("    baretk {} - {}", cmd.name, cmd.desc);
        }
        println!("Use `baretk help <command>` for more info on each command.");
    }
}

fn cmd_help_help() {
    println!("baretk help <command>");
    println!(" Prints help for <command>.");
}

struct Command {
    name: &'static str,
    desc: &'static str,
    func: fn(ArgList),
    help: fn(),
}

const COMMANDS: &[Command] = &[
    Command { name: "dis", desc: "Disassembles an input binary.", func: cmd_disassemble, help: cmd_disassemble_help },
    Command { name: "decomp", desc: "Decompiles an input binary.", func: cmd_decompile, help: cmd_decompile_help },
    Command { name: "dump", desc: "Dumps information from an input binary.", func: cmd_dump, help: cmd_dump_help },
    Command { name: "strings", desc: "Prints strings found in an input binary.", func: cmd_strings, help: cmd_strings_help },
    Command { name: "help", desc: "Prints this help.", func: cmd_help, help: cmd_help_help },
];

fn main() {
    let mut args = env::args();
    args.next().expect("program");

    if let Some(command) = args.next() {
        let args_list = parse_cmd_args(args.collect());
        if let Some(cmd) = COMMANDS.iter().find(|cmd| cmd.name == command.as_str()) {
            (cmd.func)(args_list);
            return;
        }
        eprintln!("Unknown command `{command}`.");
        cmd_help(args_list);
        return;
    }
    else {
        eprintln!("Usage: baretk <command>");
        let named_args = HashMap::<String, String>::new();
        let pos_args = Vec::<String>::new();
        cmd_help(ArgList { named_args, pos_args });
        return;
    }
}
