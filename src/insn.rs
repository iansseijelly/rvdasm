use crate::args::Arg;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Insn {
    pub raw: u32,
    pub name: String,
    pub len: u32, 
    pub src: HashMap<String, Arg>,
    pub imm: Option<Arg>,
    pub dst: HashMap<String, Arg>,
    pub flags: HashMap<String, Arg>,
    pub csr: Option<Arg>,
}

fn get_insn_size(raw: u32) -> u32 { if ((raw) & 0x03) < 0x03 { 2 } else { 4 }}

fn tag_to_string(tag: &str) -> String {
    match tag {
        "rd" | "rs1" | "rs2" => "x".to_string(),
        "imm" => "".to_string(),
        _ => tag.to_string(),
    }
}

impl Insn {
    pub fn new(raw: u32, name: &str, src: HashMap<String, Arg>, imm: Option<Arg>, dst: HashMap<String, Arg>, flags: HashMap<String, Arg>, csr: Option<Arg>) -> Self {
        Self { raw, name: name.to_string(), len: get_insn_size(raw), src, imm, dst, flags, csr }
    }

    pub fn get_len(&self) -> u32 {
        self.len
    }

    pub fn to_string(&self) -> String {
        // Format the instruction name
        let mut parts = vec![self.name.clone()];
        
        // TODO: add flags
        // Collect all operand parts
        let mut operands = Vec::new();
        
        // Add dst args
        for (k, v) in &self.dst {
            operands.push(format!("{}{}", tag_to_string(k), v.to_string()));
        }
        
        // Add src args - sort by tag
        let mut src_operands = self.src.iter().map(|(k, v)| format!("{}{}", tag_to_string(k), v.to_string())).collect::<Vec<String>>();
        src_operands.sort();
        for operand in src_operands {
            operands.push(operand);
        }
        
        // Add imm arg
        if let Some(imm) = &self.imm {
            operands.push(imm.to_string());
        }
        
        // Add csr arg
        if let Some(csr) = &self.csr {
            operands.push(format!("CSR#{}", csr.to_string()));
        }
        
        // Join all operands with commas
        if !operands.is_empty() {
            parts.push(operands.join(", "));
        }
        
        // Join instruction name and operands with space
        parts.join(" ")
    }

    pub fn to_canonical(&self) -> String {
        // Format the instruction name
        let mut parts = vec![self.name.clone()];

        // Collect all operand parts
        let mut operands = Vec::new();
        
        // Add dst args
        for (k, v) in &self.dst {
            operands.push(format!("{} {}{}", k.to_uppercase(), tag_to_string(k), v.to_string()));
        }
        
        // Add src args - sort by tag
        let mut src_operands = self.src.iter().map(|(k, v)| format!("{} {}{}", k.to_uppercase(), tag_to_string(k), v.to_string())).collect::<Vec<String>>();
        src_operands.sort();
        for operand in src_operands {
            operands.push(operand);
        }
        
        // Add imm arg
        if let Some(imm) = &self.imm {
            operands.push(format!("{} {}", "IMM", imm.to_string()));
        }
        
        // Add csr arg
        if let Some(csr) = &self.csr {
            operands.push(format!("{} {}", "CSR", csr.to_string()));
        }

        // TODO: add flags
        
        // Join all operands with commas
        if !operands.is_empty() {
            parts.push(operands.join(" "));
        }
        
        // Join instruction name and operands with space
        parts.join(" ")
    }
}