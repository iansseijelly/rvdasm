use crate::args::Arg;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Insn {
    pub raw: u32,
    pub name: String,
    // TODO: pub len: u32, 
    pub src: HashMap<String, Arg>,
    pub dst: HashMap<String, Arg>,
    pub flags: HashMap<String, Arg>,
}

impl Insn {
    pub fn new(raw: u32, name: &str, src: HashMap<String, Arg>, dst: HashMap<String, Arg>, flags: HashMap<String, Arg>) -> Self {
        Self { raw, name: name.to_string(), src, dst, flags }
    }

    pub fn to_string(&self) -> String {
        format!("{:08x}: {}, src: {:?}, dst: {:?}, flags: {:?}", self.raw, self.name, self.src, self.dst, self.flags)
    }
}