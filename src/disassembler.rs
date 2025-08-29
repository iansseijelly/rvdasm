use crate::args::*;
use crate::insn::*;
use crate::isa::*;
use std::collections::HashMap;
use std::io::Write;

/// Helper: Check if the instruction is RVC
pub fn is_compressed_byte(byte: u8) -> bool { byte & 0x03 < 0x03 }

pub fn is_compressed(code: u32) -> bool { code & 0x03 < 0x03 }
pub fn get_opcode(code: u32) -> u8 { (code & 0x7f) as u8 }

pub enum Xlen {
    XLEN32,
    XLEN64,
}

pub struct Disassembler {
    xlen: Xlen,
}

impl Disassembler {
    pub fn new(xlen: Xlen) -> Self {
        Self{ xlen }
    }

    pub fn extract_from_mask_match(&self, spec: &Spec, code: u32) -> Option<Insn> {
        // call the args function to get the arguments
        let args: Vec<(Arg, String)> = spec.args.iter().map(|arg| arg(code)).collect();
        // iterate over the args and check for Args::Error
        let valid = args.iter().all(|(arg, _)| !arg.is_error());
        if !valid {
            return None;
        } else {
            // iterate over the args and check for Args::Error
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

    /// Disassemble a single instruction
    pub fn disassmeble_one(&self, code: u32) -> Option<Insn> { 
        // iterator over all isa specs
        // first, check if the instruction is compressed
        if is_compressed(code) {
            // iterate over generic compressed specs
            for spec in RV_ISA_SPECS_GENERIC_COMPRESSED.iter() {
                if spec.compare(code) {
                    let result = self.extract_from_mask_match(spec, code);
                    if result.is_some() { return result; } else { continue; }
                }
            }
            // iterate over xlen compressed specs
            let xlen_specs = match self.xlen {
                Xlen::XLEN32 => &*RV_ISA_SPECS_32_COMPRESSED,
                Xlen::XLEN64 => &*RV_ISA_SPECS_64_COMPRESSED,
            };
            for spec in xlen_specs.iter() {
                if spec.compare(code) {
                    let result = self.extract_from_mask_match(spec, code);
                    if result.is_some() { return result; } else { continue; }
                }
            }
            return None;
        }
        // then, check if the instruction is a regular instruction
        let spec = get_generic_full_specs_by_opcode(get_opcode(code));
        if spec.is_some() {
            // check if the masked result creates a match
            for spec in spec.unwrap().iter() {
                if spec.compare(code) {
                    let result = self.extract_from_mask_match(spec, code);
                    if result.is_some() { return result; } else { continue; }
                }
            }
        }
        let xlen_specs = match self.xlen {
            Xlen::XLEN32 => get_32_full_specs_by_opcode(get_opcode(code)),
            Xlen::XLEN64 => get_64_full_specs_by_opcode(get_opcode(code)),
        };
        if xlen_specs.is_some() {
            for spec in xlen_specs.unwrap().iter() {
                if spec.compare(code) {
                    let result = self.extract_from_mask_match(spec, code);
                    if result.is_some() { return result; } else { continue; }
                }
            }
        }
        None
    }

    /// Disassemble a single instruction from a string
    pub fn disassemble_from_str(&self, code: &str) -> Option<Insn> { 
        let code = u32::from_str_radix(code, 16).unwrap();
        self.disassmeble_one(code)
    }

    /// Disassemble all instructions in a chunk of binary
    pub fn disassemble_all(&self, code: &[u8], entry_point: u64) -> HashMap<u64, Insn> { 
        let mut insns = HashMap::new();
        let mut i = 0;
        while i < code.len() {
            let code_u32;
            let is_compressed = is_compressed_byte(code[i]);
            if is_compressed {
                code_u32 = u32::from_le_bytes([code[i], code[i+1], 0 as u8, 0 as u8]);
            } else {
                code_u32 = u32::from_le_bytes([code[i], code[i+1], code[i+2], code[i+3]]);
            }
            let insn_opt = self.disassmeble_one(code_u32);
            if insn_opt.is_none() { 
                insns.insert(i as u64 + entry_point, Insn::new(code_u32, "unknown", HashMap::new(), None, HashMap::new(), HashMap::new(), None));
                i += if is_compressed { 2 } else { 4 };
                continue;
            }
            let insn = insn_opt.unwrap();
            let insn_len = insn.get_len() as usize;
            insns.insert(i as u64 + entry_point, insn);
            i += insn_len;
        }
        insns
    }
}
