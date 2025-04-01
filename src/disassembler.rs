use crate::insn::*;
use crate::isa::*;
use std::collections::HashMap;

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
                for (arg, tag) in args {
                    if arg.is_src() {
                        src_args.insert(tag, arg);
                    } else if arg.is_dst() {
                        dst_args.insert(tag, arg);
                    } else if arg.is_flag() {
                        flags.insert(tag, arg);
                    }
                }
                let insn = Insn::new(code, &spec.name, src_args, dst_args, flags);
                return Some(insn);
            }
        }
        None
    }
    
    pub fn disassemble_from_str(&self, code: &str) -> Option<Insn> { 
        let code = u32::from_str_radix(code, 16).unwrap();
        self.disassmeble_one(code)
     }
    // fn disassemble_all(&self, code: &[u8], entry_point: u64) -> Vec<Insn> { return vec![]; }
}
