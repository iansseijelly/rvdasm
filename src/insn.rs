use crate::args::Arg;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const BRANCH_OPCODES: &[&str] = &["beq", "bge", "bgeu", "blt", "bltu", "bne", "beqz", "bnez",
                                "bgez", "blez", "bltz", "bgtz", "bgt", "ble", "bgtu", "bleu",
                                "c.beqz", "c.bnez", "c.bltz", "c.bgez"];
const IJ_OPCODES: &[&str] = &["jal", "j", "call", "tail", "c.j", "c.jal"];
const UJ_OPCODES: &[&str] = &["jalr", "jr", "c.jr", "c.jalr", "ret"];

const BRANCH_MASK: u8 = 0x01;
const BRANCH_OFFSET: u8 = 0;
const IJ_MASK: u8 = 0x02;
const IJ_OFFSET: u8 = 1;
const UJ_MASK: u8 = 0x04;
const UJ_OFFSET: u8 = 2;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insn {
    pub len: u32, 
    pub imm: Option<Arg>,
    pub kind_mask: u8,
    pub raw: u32,
    pub name: String,
    pub src: HashMap<String, Arg>,
    pub dst: HashMap<String, Arg>,
    pub flags: HashMap<String, Arg>,
    pub csr: Option<Arg>,
}

/// Helper: Get the size of the instruction in bytes
fn get_insn_size(raw: u32) -> u32 { if ((raw) & 0x03) < 0x03 { 2 } else { 4 }}

/// Helper: Convert a tag to a string
fn tag_to_string(tag: &str) -> String {
    match tag {
        "rd" | "rs1" | "rs2" | "rs3" => "x".to_string(),
        "fd" | "fs1" | "fs2" | "fs3" => "f".to_string(),
        "imm" => "".to_string(),
        _ => tag.to_string(),
    }
}

impl Insn {
    pub fn new(raw: u32, name: &str, src: HashMap<String, Arg>, imm: Option<Arg>, dst: HashMap<String, Arg>, flags: HashMap<String, Arg>, csr: Option<Arg>) -> Self {
        let is_branch = BRANCH_OPCODES.contains(&name);
        let is_direct_jump = IJ_OPCODES.contains(&name);
        let is_indirect_jump = UJ_OPCODES.contains(&name);
        let kind_mask = (is_branch as u8) | ((is_direct_jump as u8) << IJ_OFFSET) | ((is_indirect_jump as u8) << UJ_OFFSET);
        Self { len: get_insn_size(raw), imm, kind_mask, raw, name: name.to_string(), src, dst, flags, csr }
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
        self.kind_mask & BRANCH_MASK != 0
    }

    pub fn is_direct_jump(&self) -> bool {
        self.kind_mask & IJ_MASK != 0
    }

    pub fn is_indirect_jump(&self) -> bool {
        self.kind_mask & UJ_MASK != 0
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