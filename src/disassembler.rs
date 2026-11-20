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

    /// Disassemble a single instruction
    pub fn disassmeble_one(&self, code: u32) -> Option<Insn> {
        match (is_compressed(code), &self.xlen) {
            (true, Xlen::XLEN32) => decode_compressed_32(code),
            (true, Xlen::XLEN64) => decode_compressed_64(code),
            (false, Xlen::XLEN32) => decode_full_32(code),
            (false, Xlen::XLEN64) => decode_full_64(code),
        }
    }

    /// Disassemble a single instruction from a string
    pub fn disassemble_from_str(&self, code: &str) -> Option<Insn> {
        let code = u32::from_str_radix(code, 16).unwrap();
        self.disassmeble_one(code)
    }

    /// Disassemble every instruction in a code blob, handing each (address, insn)
    /// to `sink` in address order. Undecodable encodings become an `Op::Unknown` insn
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
                    sink(addr, Insn::new(code_u32, Op::Unknown, &[]));
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
