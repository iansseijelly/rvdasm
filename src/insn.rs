use crate::args::Arg;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Insn {
    pub raw: u32,
    pub name: String,
    pub len: u32, 
    pub src: HashMap<String, Arg>,
    pub dst: HashMap<String, Arg>,
    pub flags: HashMap<String, Arg>,
    pub csr: Option<Arg>,
}

fn get_insn_size(raw: u32) -> u32 { if ((raw) & 0x03) < 0x03 { 2 } else { 4 }}

impl Insn {
    pub fn new(raw: u32, name: &str, src: HashMap<String, Arg>, dst: HashMap<String, Arg>, flags: HashMap<String, Arg>, csr: Option<Arg>) -> Self {
        Self { raw, name: name.to_string(), len: get_insn_size(raw), src, dst, flags, csr }
    }

    pub fn to_string(&self) -> String {
        format!("{}, src: {:?}, dst: {:?}, flags: {:?}, csr: {:?}", self.name, self.src, self.dst, self.flags, self.csr)
    }
}