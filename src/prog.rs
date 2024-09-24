use std::collections::HashMap;

pub struct Section {
    pub name: String,
    pub addr: u64,
    pub bytes: Vec<u8>,
}

pub struct Program {
    pub flags: u8,
    pub machine_type: String,
    pub section_table: HashMap<String, Section>
}

impl Program {
    pub fn has_section(&self, name: &'static str) -> bool {
        self.section_table.contains_key(&String::from(name))
    }
}

pub fn build_program_from_binary(bytes: &Vec<u8>, flags: Option<u8>, machine_type: Option<String>) -> Program {
    let mut section_table = HashMap::<String, Section>::new();
    section_table.insert(String::from("file"), Section {
        name: String::from("file"),
        addr: 0x0,
        bytes: bytes.clone()
    });
    Program {
        flags: flags.unwrap_or_default(),
        machine_type: machine_type.unwrap_or("unknown".to_string()),
        section_table: section_table,
    }
}