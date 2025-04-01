#[derive(Debug, Clone)]
pub enum Arg {
    DstReg(u32),
    SrcReg(u32),
    Imm(i32),
    UImm(u32),
    Nothing,
}

impl Arg {
    pub fn is_src(&self) -> bool {
        matches!(self, Arg::SrcReg(_) | Arg::Imm(_) | Arg::UImm(_))
    }

    pub fn is_dst(&self) -> bool {
        matches!(self, Arg::DstReg(_))
    }
}

#[derive(Debug, Clone)]
pub struct Insn {
    pub raw: u32,
    pub name: String,
    pub src: Vec<Arg>,
    pub dst: Vec<Arg>,
}

impl Insn {
    pub fn new(raw: u32, name: &str, src: Vec<Arg>, dst: Vec<Arg>) -> Self {
        Self { raw, name: name.to_string(), src, dst }
    }

    pub fn to_string(&self) -> String {
        format!("{:08x}: {}, src: {:?}, dst: {:?}", self.raw, self.name, self.src, self.dst)
    }
}

// helper functions
fn x(insn: u32, lo: u32, len: u32) -> u32 { (insn >> lo) & ((1 << len) - 1) }
fn xs(insn: u32, lo: u32, len: u32) -> i32 { (insn as i32) << (32 - lo - len) >> (32 - len) }
fn imm_sign(insn: u32) -> i32 { xs(insn, 31, 1)}

// dst operands
pub fn rd(insn: u32) -> Arg { Arg::DstReg(x(insn, 7, 5)) }

// src operands
pub fn rs1(insn: u32) -> Arg { Arg::SrcReg(x(insn, 15, 5)) }
pub fn rs2(insn: u32) -> Arg { Arg::SrcReg(x(insn, 20, 5)) }
pub fn rs3(insn: u32) -> Arg { Arg::SrcReg(x(insn, 27, 5)) }

// immediates - signed 
pub fn imm12(insn: u32) -> Arg { Arg::Imm(xs(insn, 20, 12)) }
pub fn imm20(insn: u32) -> Arg { Arg::Imm(imm_sign(insn)) }
pub fn jimm20(insn: u32) -> Arg { Arg::Imm(x(insn, 21, 10) as i32 + (x(insn, 20, 1) << 11) as i32 + (x(insn, 12, 8) << 12) as i32 + (imm_sign(insn) << 20)) }
pub fn imm12hi(insn: u32) -> Arg { Arg::Imm(x(insn, 7, 5) as i32 + (xs(insn, 25, 7) << 5)) }
pub fn imm12lo(_insn: u32) -> Arg { Arg::Nothing }
pub fn bimm12hi(insn: u32) -> Arg { Arg::Imm(x(insn, 8, 4) as i32 + (x(insn, 25, 6) << 5) as i32 + (x(insn, 7, 1) << 11) as i32 + (imm_sign(insn) << 12)) }
pub fn bimm12lo(_insn: u32) -> Arg { Arg::Nothing }

// shift amounts - unsigned 
pub fn shamtd(insn: u32) -> Arg { Arg::UImm(x(insn, 20, 5)) }
pub fn shamtw(insn: u32) -> Arg { Arg::UImm(x(insn, 20, 5)) }

// fence operands
pub fn fm(insn: u32) -> Arg { Arg::UImm(x(insn, 28, 4)) }
pub fn pred(insn: u32) -> Arg { Arg::UImm(x(insn, 24, 4)) }
pub fn succ(insn: u32) -> Arg { Arg::UImm(x(insn, 20, 4)) }
