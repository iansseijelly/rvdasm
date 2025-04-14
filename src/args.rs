#[derive(Debug, Clone)]
pub enum Arg {
    DstReg(u32),
    SrcReg(u32),
    Imm(i32),
    UImm(u32),
    Flag(u32),
    CSR(u32),
    Nothing,
    Error,
}

impl Arg {
    
    /// Helper: Check if the argument is a source operand
    pub fn is_src(&self) -> bool {
        matches!(self, Arg::SrcReg(_))
    }

    /// Helper: Check if the argument is an immediate operand
    pub fn is_imm(&self) -> bool {
        matches!(self, Arg::Imm(_) | Arg::UImm(_))
    }

    /// Helper: Check if the argument is a destination operand
    pub fn is_dst(&self) -> bool {
        matches!(self, Arg::DstReg(_))
    }

    /// Helper: Check if the argument is a flag operand
    pub fn is_flag(&self) -> bool {
        matches!(self, Arg::Flag(_))
    }

    /// Helper: Check if the argument is a CSR operand
    pub fn is_csr(&self) -> bool {
        matches!(self, Arg::CSR(_))
    }

    /// Helper: Check if the argument is an error
    pub fn is_error(&self) -> bool {
        matches!(self, Arg::Error)
    }

    /// Helper: Format the argument to a string representation
    pub fn to_string(&self) -> String {
        match self {
            Arg::DstReg(val) => format!("{}", val),
            Arg::SrcReg(val) => format!("{}", val),
            Arg::Imm(val) => format!("{}", val),
            Arg::UImm(val) => format!("{}", val),
            Arg::Flag(val) => format!("{}", val),
            Arg::CSR(val) => format!("{}", val),
            _ => "".to_string(),
        }
    }

    /// Helper: Get the actual value of the immediate as a signed integer
    /// must be an immediate
    pub fn get_val_signed_imm(&self) -> i32 {
        match self {
            Arg::Imm(val) => *val,
            Arg::UImm(val) => *val as i32,
            _ => panic!("Invalid argument type for get_val_s"),
        }
    }

    /// Helper: Get the actual value of the argument as an unsigned integer
    /// must NOT be an immediate
    pub fn get_val(&self) -> u32 {
        match self {
            Arg::DstReg(val) => *val,
            Arg::SrcReg(val) => *val,
            Arg::Flag(val) => *val,
            Arg::CSR(val) => *val,
            _ => panic!("Invalid argument type for get_val_u"),
        }
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
pub fn rs3(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 27, 5)), "rs3".to_string()) }

// immediates - signed 
// I-type immediate
pub fn imm12(insn: u32) -> (Arg, String) { (Arg::Imm(xs(insn, 20, 12)), "imm".to_string()) }
// U-type immediate
pub fn imm20(insn: u32) -> (Arg, String) { (Arg::Imm(xs(insn, 12, 20) << 12), "imm".to_string()) }
// UJ-type immediate
pub fn jimm20(insn: u32) -> (Arg, String) { (Arg::Imm((x(insn, 21, 10) << 1) as i32 + (x(insn, 20, 1) << 11) as i32 + 
    (x(insn, 12, 8) << 12) as i32 + (imm_sign(insn) << 20)), "imm".to_string()) }
// S-type immediate
pub fn imm12hi(insn: u32) -> (Arg, String) { (Arg::Imm(x(insn, 7, 5) as i32 + (xs(insn, 25, 7) << 5)), "imm".to_string()) }
pub fn imm12lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
// SB-type immediate
pub fn bimm12hi(insn: u32) -> (Arg, String) { (Arg::Imm((x(insn, 8, 4) << 1) as i32 + (x(insn, 25, 6) << 5) as i32 + 
    (x(insn, 7, 1) << 11) as i32 + (imm_sign(insn) << 12)), "imm".to_string()) }
pub fn bimm12lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }

// shift amounts - unsigned 
pub fn shamtd(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 20, 6)), "imm".to_string()) }
pub fn shamtw(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 20, 5)), "imm".to_string()) }

// csr
pub fn csr(insn: u32) -> (Arg, String) { (Arg::CSR(x(insn, 20, 12)), "csr".to_string()) }
pub fn zimm5(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 15, 5)), "imm".to_string()) }

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

// floating point
pub fn fd(insn: u32) -> (Arg, String) { (Arg::DstReg(x(insn, 7, 5)), "fd".to_string()) }
pub fn fs1(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 15, 5)), "fs1".to_string()) }
pub fn fs2(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 20, 5)), "fs2".to_string()) }
pub fn fs3(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 27, 5)), "fs3".to_string()) }
pub fn rm(insn: u32) -> (Arg, String) { (Arg::Flag(x(insn, 12, 3)), "rm".to_string()) }

// compressed
pub fn rd_p(insn: u32) -> (Arg, String) { (Arg::DstReg(x(insn, 2, 3)), "rd".to_string()) }
pub fn rs1_p(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 7, 3)), "rs1".to_string()) }
pub fn rs2_p(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 2, 3)), "rs2".to_string()) }
pub fn rs1_n0(insn: u32) -> (Arg, String) { match x(insn, 7, 5) { 0 => (Arg::Error, "".to_string()), val => (Arg::SrcReg(val), "rs1".to_string()) } }
pub fn rd_n0(insn: u32) -> (Arg, String) { match x(insn, 7, 5) { 0 => (Arg::Error, "".to_string()), val => (Arg::DstReg(val), "rd".to_string()) } }
pub fn rd_n2(insn: u32) -> (Arg, String) { match x(insn, 7, 5) { 0 | 2 => (Arg::Error, "".to_string()), val => (Arg::DstReg(val), "rd".to_string()) } }
pub fn c_rs1_n0(insn: u32) -> (Arg, String) { match x(insn, 7, 5) { 0 => (Arg::Error, "".to_string()), val => (Arg::SrcReg(val), "rs1".to_string()) } }
pub fn c_rs2_n0(insn: u32) -> (Arg, String) { match x(insn, 2, 5) { 0 => (Arg::Error, "".to_string()), val => (Arg::SrcReg(val), "rs2".to_string()) } }
pub fn c_rs2(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 2, 5)), "rs2".to_string()) }
pub fn c_nzimm6hi(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 2, 5) + (x(insn, 12, 1) << 5)), "imm".to_string()) }
pub fn c_nzimm6lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_imm6hi(insn: u32) -> (Arg, String) { (Arg::Imm(x(insn, 2, 5) as i32 + (xs(insn, 12, 1) << 5)), "imm".to_string()) }
pub fn c_imm6lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_nzimm10hi(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 6, 1) << 2) + (x(insn, 5, 1) << 3) + 
    (x(insn, 11, 2) << 4) + (x(insn, 7, 4) << 6)), "imm".to_string()) }
pub fn c_nzimm10lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_nzuimm10(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 6, 1) << 2) + (x(insn, 5, 1) << 3) + 
    (x(insn, 11, 2) << 4) + (x(insn, 7, 4) << 6)), "imm".to_string()) }
pub fn c_bimm9hi(insn: u32) -> (Arg, String) { (Arg::Imm((x(insn, 3, 2) << 1) as i32 + (x(insn, 10, 2) << 3) as i32 + 
    (x(insn, 2, 1) << 5) as i32 + (x(insn, 5, 2) << 6) as i32 + (xs(insn, 12, 1) << 8)), "imm".to_string()) }
pub fn c_bimm9lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_imm12(insn: u32) -> (Arg, String) { (Arg::Imm((x(insn, 3, 3) << 1) as i32 + (x(insn, 11, 1) << 4) as i32 + 
    (x(insn, 2, 1) << 5) as i32 + (x(insn, 7, 1) << 6) as i32 + (x(insn, 6, 1) << 7) as i32 + (x(insn, 9, 2) << 8) as i32 + 
    (x(insn, 8, 1) << 10) as i32 + (xs(insn, 12, 1) << 11)), "imm".to_string()) }
pub fn c_uimm8hi(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 10, 3) << 3) + (x(insn, 5, 2) << 6)), "imm".to_string()) }
pub fn c_uimm8lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_uimm9sphi(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 5, 2) << 3) + (x(insn, 12, 1) << 5) + (x(insn, 2, 3) << 6)), "imm".to_string()) }
pub fn c_uimm9splo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_nzimm18hi(insn: u32) -> (Arg, String) { (Arg::Imm((x(insn, 2, 5) << 12) as i32 + (xs(insn, 12, 1) << 17)), "imm".to_string()) }
pub fn c_nzimm18lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_uimm7hi(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 6, 1) << 2) + (x(insn, 10, 3) << 3) + (x(insn, 5, 1) << 6)), "imm".to_string()) }
pub fn c_uimm7lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_uimm8sphi(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 4, 3) << 2) + (x(insn, 12, 1) << 5) + (x(insn, 2, 2) << 6)), "imm".to_string()) }
pub fn c_uimm8splo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_uimm9sp_s(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 10, 3) << 3) + (x(insn, 7, 3) << 6)), "imm".to_string()) }
pub fn c_nzuimm6hi(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 2,4)) + (x(insn, 12, 1) << 5)), "imm".to_string()) }
pub fn c_nzuimm6lo(_insn: u32) -> (Arg, String) { (Arg::Nothing, "".to_string()) }
pub fn c_uimm8sp_s(insn: u32) -> (Arg, String) { (Arg::UImm((x(insn, 9, 4) << 2) + (x(insn, 7, 2) << 6)), "imm".to_string()) }

// vector
pub fn vd(insn: u32) -> (Arg, String) { (Arg::DstReg(x(insn, 7, 5)), "vd".to_string()) }
pub fn vs3(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 7, 5)), "vs3".to_string()) }
pub fn vs1(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 15, 5)), "vs1".to_string()) }
pub fn vs2(insn: u32) -> (Arg, String) { (Arg::SrcReg(x(insn, 20, 5)), "vs2".to_string()) }

pub fn vm(insn: u32) -> (Arg, String) { (Arg::Flag(x(insn, 25, 1)), "vm".to_string()) }
pub fn simm5(insn: u32) -> (Arg, String) { (Arg::Imm(xs(insn, 15, 5)), "imm".to_string()) }
pub fn zimm10(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 20, 10)), "imm".to_string()) }
pub fn zimm11(insn: u32) -> (Arg, String) { (Arg::UImm(x(insn, 20, 11)), "imm".to_string()) }
