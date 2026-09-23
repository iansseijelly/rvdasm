use crate::args::{Arg, Tag};
use serde::Serialize;

const BRANCH_OPCODES: &[&str] = &[
    "beq", "bge", "bgeu", "blt", "bltu", "bne", "beqz", "bnez", "bgez", "blez", "bltz", "bgtz",
    "bgt", "ble", "bgtu", "bleu", "c.beqz", "c.bnez", "c.bltz", "c.bgez",
];
const IJ_OPCODES: &[&str] = &["jal", "j", "call", "tail", "c.j", "c.jal"];
const UJ_OPCODES: &[&str] = &["jalr", "jr", "c.jr", "c.jalr", "ret"];

const BRANCH_MASK: u8 = 0x01;
const IJ_MASK: u8 = 0x02;
const IJ_OFFSET: u8 = 1;
const UJ_MASK: u8 = 0x04;
const UJ_OFFSET: u8 = 2;
const CFC_MASK: u8 = 0x07; //0b111

/// Most operands any spec in the table lists (fmadd-class: rd, rs1, rs2, rs3, rm).
pub const MAX_ARGS: usize = 6;

/// A decoded instruction.
///
/// Flat and allocation-free: the mnemonic is a `&'static str` out of the spec table
/// and the operands live in a fixed array. A consumer that indexes a whole binary
/// (tens of millions of these) pays ~90 bytes each and no heap traffic, where the
/// previous `String` + boxed `HashMap<String, Arg>` layout cost ~1 KB and seven
/// allocations per instruction. Everything the old accessors returned is derivable:
/// `get_imm`, `get_src`, `get_dst`, `to_string` and `to_canonical` keep their output.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Insn {
    pub len: u8,
    pub kind_mask: u8,
    nargs: u8,
    pub offset: i32,
    pub raw: u32,
    pub name: &'static str,
    tags: [Tag; MAX_ARGS],
    args: [Arg; MAX_ARGS],
}

/// Helper: Get the size of the instruction in bytes
fn get_insn_size(raw: u32) -> u8 {
    if ((raw) & 0x03) < 0x03 { 2 } else { 4 }
}

impl Insn {
    /// Build from the operands a spec produced, in table order. `Arg::Nothing`
    /// placeholders (the `*lo` halves of split immediates) are dropped. A repeated
    /// tag replaces the earlier operand, which is what the previous map-based
    /// layout did on insert; `offset` is the signed value of the immediate, if any.
    pub fn new(raw: u32, name: &'static str, operands: &[(Arg, Tag)]) -> Self {
        let mut tags = [Tag::None; MAX_ARGS];
        let mut args = [Arg::Nothing; MAX_ARGS];
        let mut nargs = 0usize;
        let mut offset = 0i32;
        for &(arg, tag) in operands {
            if matches!(arg, Arg::Nothing | Arg::Error) {
                continue;
            }
            if arg.is_imm() {
                offset = arg.get_val_signed_imm();
            }
            if let Some(i) = tags[..nargs].iter().position(|&t| t == tag && tag != Tag::None) {
                args[i] = arg;
                continue;
            }
            assert!(nargs < MAX_ARGS, "instruction {} has more than {} operands", name, MAX_ARGS);
            tags[nargs] = tag;
            args[nargs] = arg;
            nargs += 1;
        }

        let is_branch = BRANCH_OPCODES.contains(&name);
        let is_direct_jump = IJ_OPCODES.contains(&name);
        let is_indirect_jump = UJ_OPCODES.contains(&name);
        let kind_mask = (is_branch as u8)
            | ((is_direct_jump as u8) << IJ_OFFSET)
            | ((is_indirect_jump as u8) << UJ_OFFSET);

        Self {
            len: get_insn_size(raw),
            kind_mask,
            nargs: nargs as u8,
            offset,
            raw,
            name,
            tags,
            args,
        }
    }

    pub fn get_len(&self) -> u8 {
        self.len
    }

    pub fn get_raw(&self) -> u32 {
        self.raw
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    /// Every operand, in spec-table order, with its role tag.
    pub fn operands(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        (0..self.nargs as usize).map(move |i| (self.tags[i], self.args[i]))
    }

    pub fn get_src(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        self.operands().filter(|(_, a)| a.is_src())
    }

    pub fn get_dst(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        self.operands().filter(|(_, a)| a.is_dst())
    }

    pub fn get_flags(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        self.operands().filter(|(_, a)| a.is_flag())
    }

    pub fn get_imm(&self) -> Option<Arg> {
        self.operands().map(|(_, a)| a).find(|a| a.is_imm())
    }

    pub fn get_csr(&self) -> Option<Arg> {
        self.operands().map(|(_, a)| a).find(|a| a.is_csr())
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

    pub fn is_cfc_insn(&self) -> bool {
        self.kind_mask & CFC_MASK != 0
    }

    /// Sources ordered by tag name, the order the text forms print them in.
    fn sorted_src(&self) -> Vec<(Tag, Arg)> {
        let mut v: Vec<(Tag, Arg)> = self.get_src().collect();
        v.sort_by_key(|(t, _)| t.as_str());
        v
    }

    /// Helper: Format the instruction to a string representation
    pub fn to_string(&self) -> String {
        let mut operands = Vec::new();
        for (t, a) in self.get_dst() {
            operands.push(format!("{}{}", t.prefix(), a.to_string()));
        }
        for (t, a) in self.sorted_src() {
            operands.push(format!("{}{}", t.prefix(), a.to_string()));
        }
        if let Some(imm) = self.get_imm() {
            operands.push(imm.to_string());
        }
        if let Some(csr) = self.get_csr() {
            operands.push(format!("CSR#{}", csr.to_string()));
        }
        // flags are not printed, as before
        if operands.is_empty() {
            self.name.to_string()
        } else {
            format!("{} {}", self.name, operands.join(", "))
        }
    }

    /// Helper: Format the instruction to a canonicalized string representation
    pub fn to_canonical(&self) -> String {
        let mut operands = Vec::new();
        for (t, a) in self.get_dst() {
            operands.push(format!("{} {}{}", t.as_str().to_uppercase(), t.prefix(), a.to_string()));
        }
        for (t, a) in self.sorted_src() {
            operands.push(format!("{} {}{}", t.as_str().to_uppercase(), t.prefix(), a.to_string()));
        }
        if let Some(imm) = self.get_imm() {
            operands.push(format!("IMM {}", imm.to_string()));
        }
        if let Some(csr) = self.get_csr() {
            operands.push(format!("CSR {}", csr.to_string()));
        }
        if operands.is_empty() {
            self.name.to_string()
        } else {
            format!("{} {}", self.name, operands.join(" "))
        }
    }
}
