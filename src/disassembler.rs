use crate::args::*;
use crate::insn::*;
use crate::isa::*;
use std::collections::HashMap;

/// Helper: Check if the instruction is RVC
pub fn is_compressed_byte(byte: u8) -> bool {
    byte & 0x03 < 0x03
}

pub fn is_compressed(code: u32) -> bool {
    code & 0x03 < 0x03
}
pub fn get_opcode(code: u32) -> u8 {
    (code & 0x7f) as u8
}

pub enum Xlen {
    XLEN32,
    XLEN64,
}

pub struct Disassembler {
    xlen: Xlen,
}

impl Disassembler {
    pub fn new(xlen: Xlen) -> Self {
        Self { xlen }
    }

    pub fn extract_from_mask_match(&self, spec: &Spec, code: u32) -> Option<Insn> {
        // Evaluate the spec's operand extractors into a fixed buffer. Any extractor
        // reporting Arg::Error means the encoding is not this instruction after all.
        let mut ops = [(Arg::Nothing, Tag::None); MAX_ARGS];
        let mut n = 0;
        for extract in spec.args.iter() {
            let (arg, tag) = extract(code);
            if arg.is_error() {
                return None;
            }
            if matches!(arg, Arg::Nothing) {
                continue;
            }
            assert!(n < MAX_ARGS, "spec {} lists more than {} operands", spec.name, MAX_ARGS);
            ops[n] = (arg, tag);
            n += 1;
        }
        Some(Insn::new(code, spec.name, &ops[..n]))
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
                    if result.is_some() {
                        return result;
                    } else {
                        continue;
                    }
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
                    if result.is_some() {
                        return result;
                    } else {
                        continue;
                    }
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
                    if result.is_some() {
                        return result;
                    } else {
                        continue;
                    }
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
                    if result.is_some() {
                        return result;
                    } else {
                        continue;
                    }
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

    /// Disassemble every instruction in a code blob, handing each (address, insn)
    /// to `sink` in address order. Undecodable encodings become an "unknown" insn
    /// of the length their low bits imply, so the walk never desynchronises.
    ///
    /// This is the allocation-free primitive: a caller indexing a large binary
    /// inserts straight into its own map instead of receiving a temporary one.
    pub fn disassemble_each(&self, code: &[u8], entry_point: u64, mut sink: impl FnMut(u64, Insn)) {
        let mut i = 0;
        while i < code.len() {
            let is_compressed = is_compressed_byte(code[i]);
            let code_u32 = if is_compressed {
                u32::from_le_bytes([code[i], code[i + 1], 0, 0])
            } else {
                u32::from_le_bytes([code[i], code[i + 1], code[i + 2], code[i + 3]])
            };
            let addr = i as u64 + entry_point;
            match self.disassmeble_one(code_u32) {
                Some(insn) => {
                    i += insn.get_len() as usize;
                    sink(addr, insn);
                }
                None => {
                    i += if is_compressed { 2 } else { 4 };
                    sink(addr, Insn::new(code_u32, "unknown", &[]));
                }
            }
        }
    }

    /// Disassemble all instructions in a chunk of binary into a map keyed by address.
    pub fn disassemble_all(&self, code: &[u8], entry_point: u64) -> HashMap<u64, Insn> {
        let mut insns = HashMap::with_capacity(code.len() / 2);
        self.disassemble_each(code, entry_point, |addr, insn| {
            insns.insert(addr, insn);
        });
        insns
    }
}
