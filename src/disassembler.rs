use crate::insn::*;
use crate::isa::*;

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
                let args: Vec<Arg> = spec.args.iter().map(|arg| arg(code)).collect();
                let src_args: Vec<Arg> = args.iter().filter(|arg| arg.is_src()).cloned().collect();
                let dst_args: Vec<Arg> = args.iter().filter(|arg| arg.is_dst()).cloned().collect();
                let insn = Insn::new(code, &spec.name, src_args, dst_args);
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
