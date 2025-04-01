use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Arg {
    DstReg(u32),
    SrcReg(u32),
    Imm(i32),
    UImm(u32),
    Flag(u32),
    Nothing,
}

impl Arg {
    pub fn is_src(&self) -> bool {
        matches!(self, Arg::SrcReg(_) | Arg::Imm(_) | Arg::UImm(_))
    }

    pub fn is_dst(&self) -> bool {
        matches!(self, Arg::DstReg(_))
    }

    pub fn is_flag(&self) -> bool {
        matches!(self, Arg::Flag(_))
    }
}

#[derive(Debug, Clone)]
pub struct Insn {
    pub raw: u32,
    pub name: String,
    pub src: HashMap<String, Arg>,
    pub dst: HashMap<String, Arg>,
    pub flags: HashMap<String, Arg>,
}

impl Insn {
    pub fn new(raw: u32, name: &str, src: HashMap<String, Arg>, dst: HashMap<String, Arg>, flags: HashMap<String, Arg>) -> Self {
        Self { raw, name: name.to_string(), src, dst, flags }
    }

    pub fn to_string(&self) -> String {
        format!("{:08x}: {}, src: {:?}, dst: {:?}, flags: {:?}", self.raw, self.name, self.src, self.dst, self.flags)
    }
}

// helper functions
fn x(insn: u32, lo: u32, len: u32) -> u32 { (insn >> lo) & ((1 << len) - 1) }
fn xs(insn: u32, lo: u32, len: u32) -> i32 { (insn as i32) << (32 - lo - len) >> (32 - len) }
fn imm_sign(insn: u32) -> i32 { xs(insn, 31, 1)}

// dst operands
pub fn rd(insn: u32) -> (Arg, String) { (Arg::DstReg(x(insn, 7, 5)), "rd".to_string()) }

// src operands
pub fn rs1(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 15, 5)), "rs1".to_string()) }
pub fn rs2(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 20, 5)), "rs2".to_string()) }

// immediates - signed 
// I-type immediate
pub fn imm12(insn: u32) -> (Arg, String) { (Arg::Imm(xs(insn, 20, 12)), "imm".to_string()) }
// U-type immediate
pub fn imm20(insn: u32) -> (Arg, String) { (Arg::Imm(imm_sign(insn)), "imm".to_string()) }
// UJ-type immediate
pub fn jimm20(insn: u32) -> (Arg, String) { (Arg::Imm(x(insn, 21, 10) as i32 + (x(insn, 20, 1) << 11) as i32 + 
    (x(insn, 12, 8) << 12) as i32 + (imm_sign(insn) << 20)), "imm".to_string()) }
// S-type immediate
pub fn imm12hi(insn: u32) -> (Arg, String) { (Arg::Imm(x(insn, 7, 5) as i32 + (xs(insn, 25, 7) << 5)), "imm".to_string()) }
pub fn imm12lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
// SB-type immediate
pub fn bimm12hi(insn: u32) -> (Arg, String) { (Arg::Imm(x(insn, 8, 4) as i32 + (x(insn, 25, 6) << 5) as i32 + 
    (x(insn, 7, 1) << 11) as i32 + (imm_sign(insn) << 12)), "imm".to_string()) }
pub fn bimm12lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }

// shift amounts - unsigned 
pub fn shamtd(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 20, 6)), "imm".to_string()) }
pub fn shamtw(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 20, 5)), "imm".to_string()) }

// fence 
// fence mode - TSO or normal
pub fn fm(insn: u32) -> (Arg, String) { (Arg::Flag(x(insn, 28, 4)), "fm".to_string()) }
// predecessor - I/O/R/W
pub fn pred(insn: u32) -> (Arg, String) { (Arg::Flag(x(insn, 24, 4)), "pred".to_string()) }
// successor - I/O/R/W
pub fn succ(insn: u32) -> (Arg, String) { (Arg::Flag(x(insn, 20, 4)), "succ".to_string()) }

// atomics
// acquire - no later memop can be reordered before this
pub fn aq(insn: u32) -> (Arg, String) { (Arg::Flag(x(insn, 26, 1)), "aq".to_string()) }
// release - no earlier memop can be reordered after this
pub fn rl(insn: u32) -> (Arg, String) { (Arg::Flag(x(insn, 25, 1)), "rl".to_string()) }
