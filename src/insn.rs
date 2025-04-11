use crate::args::Arg;
use std::collections::HashMap;

const BRANCH_OPCODES: &[&str] = &["beq", "bge", "bgeu", "blt", "bltu", "bne", "beqz", "bnez",
                                "bgez", "blez", "bltz", "bgtz", "bgt", "ble", "bgtu", "bleu",
                                "c.beqz", "c.bnez", "c.bltz", "c.bgez"];
const IJ_OPCODES: &[&str] = &["jal", "j", "call", "tail", "c.j", "c.jal"];
const UJ_OPCODES: &[&str] = &["jalr", "jr", "c.jr", "c.jalr", "ret"];

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

/// Helper: Get the size of the instruction in bytes
fn get_insn_size(raw: u32) -> u32 { if ((raw) & 0x03) < 0x03 { 2 } else { 4 }}

/// Helper: Convert a tag to a string
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

    pub fn get_raw(&self) -> u32 {
        self.raw
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_src(&self) -> HashMap<String, Arg> {
        self.src.clone()
    }

    pub fn get_imm(&self) -> Option<Arg> {
        self.imm.clone()
    }

    pub fn get_dst(&self) -> HashMap<String, Arg> {
        self.dst.clone()
    }

    pub fn is_branch(&self) -> bool {
        BRANCH_OPCODES.contains(&self.name.as_str())
    }

    pub fn is_direct_jump(&self) -> bool {
        IJ_OPCODES.contains(&self.name.as_str())
    }

    pub fn is_indirect_jump(&self) -> bool {
        UJ_OPCODES.contains(&self.name.as_str())
    }

    /// Helper: Format the instruction to a string representation
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
        
        // Add src args - sort by tag, not by value
        let mut src_tags = self.src.keys().collect::<Vec<&String>>();   
        src_tags.sort();
        for tag in src_tags {
            operands.push(format!("{}{}", tag_to_string(tag), self.src[tag].to_string()));
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

    /// Helper: Format the instruction to a canonicalized string representation
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
        let mut src_tags = self.src.keys().collect::<Vec<&String>>();   
        src_tags.sort();
        for tag in src_tags {
            operands.push(format!("{} {}{}", tag.to_uppercase(), tag_to_string(tag), self.src[tag].to_string()));
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