use crate::args::*;
use crate::insn::*;
use crate::isa::*;
use std::collections::HashMap;

/// Helper: Check if the instruction is RVC
pub fn is_compressed(byte: u8) -> bool { byte & 0x03 < 0x03 }

pub enum Xlen {
    XLEN32,
    XLEN64,
}

pub struct Disassembler {
    xlen: Xlen,
    specs: Vec<Spec>,
}

impl Disassembler {
    pub fn new(xlen: Xlen) -> Self {
        let xlen_specs = match xlen {
            Xlen::XLEN32 => &*RV_ISA_SPECS_32,
            Xlen::XLEN64 => &*RV_ISA_SPECS_64,
        };
        // concat regular and xlen spec
        let mut specs = Vec::new();
        specs.extend(RV_ISA_SPECS_REGULAR.iter().cloned());
        specs.extend(xlen_specs.iter().cloned());
        Self{ xlen, specs }
    }

    /// Disassemble a single instruction
    pub fn disassmeble_one(&self, code: u32) -> Option<Insn> { 
        // iterator over all isa specs
        for spec in self.specs.iter() {
            // check if the masked result creates a match
            if spec.compare(code) {
                // call the args function to get the arguments
                let args: Vec<(Arg, String)> = spec.args.iter().map(|arg| arg(code)).collect();
                let mut src_args = HashMap::new();
                let mut dst_args = HashMap::new();
                let mut flags = HashMap::new();
                let mut imm = None;
                let mut csr = None;
                for (arg, tag) in args {
                    if arg.is_src() {
                        src_args.insert(tag, arg);
                    } else if arg.is_imm() {
                        imm = Some(arg);
                    } else if arg.is_dst() {
                        dst_args.insert(tag, arg);
                    } else if arg.is_flag() {
                        flags.insert(tag, arg);
                    } else if arg.is_csr() {
                        csr = Some(arg);
                    }
                }
                let insn = Insn::new(code, &spec.name, src_args, imm, dst_args, flags, csr);
                return Some(insn);
            }
        }
        None
    }

    /// Disassemble a single instruction from a string
    pub fn disassemble_from_str(&self, code: &str) -> Option<Insn> { 
        let code = u32::from_str_radix(code, 16).unwrap();
        self.disassmeble_one(code)
    }

    /// Disassemble all instructions in a binary
    pub fn disassemble_all(&self, code: &[u8], entry_point: u64) -> HashMap<usize, Insn> { 
        let mut insns = HashMap::new();
        let mut i = 0;
        while i < code.len() {
            let code_u32;
            if is_compressed(code[i]) {
                code_u32 = u32::from_le_bytes([code[i], code[i+1], 0 as u8, 0 as u8]);
            } else {
                code_u32 = u32::from_le_bytes([code[i], code[i+1], code[i+2], code[i+3]]);
            }
            let insn = self.disassmeble_one(code_u32).unwrap();
            let insn_len = insn.get_len() as usize;
            insns.insert(i + entry_point as usize, insn);
            i += insn_len;
        }
        insns
    }
}
