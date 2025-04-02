use crate::args::*;
use crate::insn::*;
use crate::isa::*;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Disassembler {

}

impl Disassembler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn disassmeble_one(&self, code: u32) -> Option<Insn> { 
        // iterator over all isa specs
        for spec in RV_ISA_SPECS.iter() {
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
                    } else if arg.is_shared() {
                        let (src, dst) = arg.split_shared();
                        let tag_parts: Vec<&str> = tag.split('_').collect();
                        src_args.insert(tag_parts[0].to_string(), src);
                        dst_args.insert(tag_parts[1].to_string(), dst);
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

    pub fn disassemble_from_str(&self, code: &str) -> Option<Insn> { 
        let code = u32::from_str_radix(code, 16).unwrap();
        self.disassmeble_one(code)
     }
    pub fn disassemble_all(&self, code: &[u8], entry_point: u64) -> HashMap<usize, Insn> { 
        let mut insns = HashMap::new();
        for i in (0..code.len()).step_by(4) {
            let code_u32 = u32::from_le_bytes([code[i], code[i+1], code[i+2], code[i+3]]);
            let insn = self.disassmeble_one(code_u32).unwrap();
            insns.insert(i + entry_point as usize, insn);
        }
        insns
    }
}
