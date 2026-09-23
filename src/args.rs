use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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


/// Operand role, as the spec table names it. Replaces the per-operand heap `String`
/// tag: a `Copy` byte that formats back to the same text through `as_str()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tag {
    None,
    Rd, Rs1, Rs2, Rs3,
    Fd, Fs1, Fs2, Fs3,
    Imm, Csr,
    Rm, Aq, Rl, Fm, Pred, Succ,
    Vd, Vs1, Vs2, Vs3, Vm,
}

impl Tag {
    /// The tag string the spec table used to carry ("rd", "rs1", "imm", ...).
    pub fn as_str(self) -> &'static str {
        match self {
            Tag::None => "",
            Tag::Rd => "rd", Tag::Rs1 => "rs1", Tag::Rs2 => "rs2", Tag::Rs3 => "rs3",
            Tag::Fd => "fd", Tag::Fs1 => "fs1", Tag::Fs2 => "fs2", Tag::Fs3 => "fs3",
            Tag::Imm => "imm", Tag::Csr => "csr",
            Tag::Rm => "rm", Tag::Aq => "aq", Tag::Rl => "rl", Tag::Fm => "fm",
            Tag::Pred => "pred", Tag::Succ => "succ",
            Tag::Vd => "vd", Tag::Vs1 => "vs1", Tag::Vs2 => "vs2", Tag::Vs3 => "vs3",
            Tag::Vm => "vm",
        }
    }

    /// Register-file prefix used when printing an operand: "x" for integer, "f" for
    /// float, nothing for an immediate, the tag itself otherwise.
    pub fn prefix(self) -> &'static str {
        match self {
            Tag::Rd | Tag::Rs1 | Tag::Rs2 | Tag::Rs3 => "x",
            Tag::Fd | Tag::Fs1 | Tag::Fs2 | Tag::Fs3 => "f",
            Tag::Imm => "",
            other => other.as_str(),
        }
    }
}

// helper functions
fn x(insn: u32, lo: u32, len: u32) -> u32 {
    (insn >> lo) & ((1 << len) - 1)
}
fn xs(insn: u32, lo: u32, len: u32) -> i32 {
    (insn as i32) << (32 - lo - len) >> (32 - len)
}
fn imm_sign(insn: u32) -> i32 {
    xs(insn, 31, 1)
}

// dst operands
pub fn rd(insn: u32) -> (Arg, Tag) {
    (Arg::DstReg(x(insn, 7, 5)), Tag::Rd)
}

// src operands
pub fn rs1(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 15, 5)), Tag::Rs1)
}
pub fn rs2(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 20, 5)), Tag::Rs2)
}
pub fn rs3(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 27, 5)), Tag::Rs3)
}

// immediates - signed
// I-type immediate
pub fn imm12(insn: u32) -> (Arg, Tag) {
    (Arg::Imm(xs(insn, 20, 12)), Tag::Imm)
}
// U-type immediate
pub fn imm20(insn: u32) -> (Arg, Tag) {
    (Arg::Imm(xs(insn, 12, 20) << 12), Tag::Imm)
}
// UJ-type immediate
pub fn jimm20(insn: u32) -> (Arg, Tag) {
    (
        Arg::Imm(
            (x(insn, 21, 10) << 1) as i32
                + (x(insn, 20, 1) << 11) as i32
                + (x(insn, 12, 8) << 12) as i32
                + (imm_sign(insn) << 20),
        ),
        Tag::Imm,
    )
}
// S-type immediate
pub fn imm12hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::Imm(x(insn, 7, 5) as i32 + (xs(insn, 25, 7) << 5)),
        Tag::Imm,
    )
}
pub fn imm12lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
// SB-type immediate
pub fn bimm12hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::Imm(
            (x(insn, 8, 4) << 1) as i32
                + (x(insn, 25, 6) << 5) as i32
                + (x(insn, 7, 1) << 11) as i32
                + (imm_sign(insn) << 12),
        ),
        Tag::Imm,
    )
}
pub fn bimm12lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}

// shift amounts - unsigned
pub fn shamtd(insn: u32) -> (Arg, Tag) {
    (Arg::UImm(x(insn, 20, 6)), Tag::Imm)
}
pub fn shamtw(insn: u32) -> (Arg, Tag) {
    (Arg::UImm(x(insn, 20, 5)), Tag::Imm)
}

// csr
pub fn csr(insn: u32) -> (Arg, Tag) {
    (Arg::CSR(x(insn, 20, 12)), Tag::Csr)
}
pub fn zimm5(insn: u32) -> (Arg, Tag) {
    (Arg::UImm(x(insn, 15, 5)), Tag::Imm)
}

// fence
// fence mode - TSO or normal
pub fn fm(insn: u32) -> (Arg, Tag) {
    (Arg::Flag(x(insn, 28, 4)), Tag::Fm)
}
// predecessor - I/O/R/W
pub fn pred(insn: u32) -> (Arg, Tag) {
    (Arg::Flag(x(insn, 24, 4)), Tag::Pred)
}
// successor - I/O/R/W
pub fn succ(insn: u32) -> (Arg, Tag) {
    (Arg::Flag(x(insn, 20, 4)), Tag::Succ)
}

// atomics
// acquire - no later memop can be reordered before this
pub fn aq(insn: u32) -> (Arg, Tag) {
    (Arg::Flag(x(insn, 26, 1)), Tag::Aq)
}
// release - no earlier memop can be reordered after this
pub fn rl(insn: u32) -> (Arg, Tag) {
    (Arg::Flag(x(insn, 25, 1)), Tag::Rl)
}

// floating point
pub fn fd(insn: u32) -> (Arg, Tag) {
    (Arg::DstReg(x(insn, 7, 5)), Tag::Fd)
}
pub fn fs1(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 15, 5)), Tag::Fs1)
}
pub fn fs2(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 20, 5)), Tag::Fs2)
}
pub fn fs3(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 27, 5)), Tag::Fs3)
}
pub fn rm(insn: u32) -> (Arg, Tag) {
    (Arg::Flag(x(insn, 12, 3)), Tag::Rm)
}

// compressed
pub fn rd_p(insn: u32) -> (Arg, Tag) {
    (Arg::DstReg(x(insn, 2, 3)), Tag::Rd)
}
pub fn rs1_p(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 7, 3)), Tag::Rs1)
}
pub fn rs2_p(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 2, 3)), Tag::Rs2)
}
pub fn rs1_n0(insn: u32) -> (Arg, Tag) {
    match x(insn, 7, 5) {
        0 => (Arg::Error, Tag::None),
        val => (Arg::SrcReg(val), Tag::Rs1),
    }
}
pub fn rd_n0(insn: u32) -> (Arg, Tag) {
    match x(insn, 7, 5) {
        0 => (Arg::Error, Tag::None),
        val => (Arg::DstReg(val), Tag::Rd),
    }
}
pub fn rd_n2(insn: u32) -> (Arg, Tag) {
    match x(insn, 7, 5) {
        0 | 2 => (Arg::Error, Tag::None),
        val => (Arg::DstReg(val), Tag::Rd),
    }
}
pub fn c_rs1_n0(insn: u32) -> (Arg, Tag) {
    match x(insn, 7, 5) {
        0 => (Arg::Error, Tag::None),
        val => (Arg::SrcReg(val), Tag::Rs1),
    }
}
pub fn c_rs2_n0(insn: u32) -> (Arg, Tag) {
    match x(insn, 2, 5) {
        0 => (Arg::Error, Tag::None),
        val => (Arg::SrcReg(val), Tag::Rs2),
    }
}
pub fn c_rs2(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 2, 5)), Tag::Rs2)
}
pub fn c_nzimm6hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm(x(insn, 2, 5) + (x(insn, 12, 1) << 5)),
        Tag::Imm,
    )
}
pub fn c_nzimm6lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_imm6hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::Imm(x(insn, 2, 5) as i32 + (xs(insn, 12, 1) << 5)),
        Tag::Imm,
    )
}
pub fn c_imm6lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_nzimm10hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm(
            (x(insn, 6, 1) << 2)
                + (x(insn, 5, 1) << 3)
                + (x(insn, 11, 2) << 4)
                + (x(insn, 7, 4) << 6),
        ),
        Tag::Imm,
    )
}
pub fn c_nzimm10lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_nzuimm10(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm(
            (x(insn, 6, 1) << 2)
                + (x(insn, 5, 1) << 3)
                + (x(insn, 11, 2) << 4)
                + (x(insn, 7, 4) << 6),
        ),
        Tag::Imm,
    )
}
pub fn c_bimm9hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::Imm(
            (x(insn, 3, 2) << 1) as i32
                + (x(insn, 10, 2) << 3) as i32
                + (x(insn, 2, 1) << 5) as i32
                + (x(insn, 5, 2) << 6) as i32
                + (xs(insn, 12, 1) << 8),
        ),
        Tag::Imm,
    )
}
pub fn c_bimm9lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_imm12(insn: u32) -> (Arg, Tag) {
    (
        Arg::Imm(
            (x(insn, 3, 3) << 1) as i32
                + (x(insn, 11, 1) << 4) as i32
                + (x(insn, 2, 1) << 5) as i32
                + (x(insn, 7, 1) << 6) as i32
                + (x(insn, 6, 1) << 7) as i32
                + (x(insn, 9, 2) << 8) as i32
                + (x(insn, 8, 1) << 10) as i32
                + (xs(insn, 12, 1) << 11),
        ),
        Tag::Imm,
    )
}
pub fn c_uimm8hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm((x(insn, 10, 3) << 3) + (x(insn, 5, 2) << 6)),
        Tag::Imm,
    )
}
pub fn c_uimm8lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_uimm9sphi(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm((x(insn, 5, 2) << 3) + (x(insn, 12, 1) << 5) + (x(insn, 2, 3) << 6)),
        Tag::Imm,
    )
}
pub fn c_uimm9splo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_nzimm18hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::Imm((x(insn, 2, 5) << 12) as i32 + (xs(insn, 12, 1) << 17)),
        Tag::Imm,
    )
}
pub fn c_nzimm18lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_uimm7hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm((x(insn, 6, 1) << 2) + (x(insn, 10, 3) << 3) + (x(insn, 5, 1) << 6)),
        Tag::Imm,
    )
}
pub fn c_uimm7lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_uimm8sphi(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm((x(insn, 4, 3) << 2) + (x(insn, 12, 1) << 5) + (x(insn, 2, 2) << 6)),
        Tag::Imm,
    )
}
pub fn c_uimm8splo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_uimm9sp_s(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm((x(insn, 10, 3) << 3) + (x(insn, 7, 3) << 6)),
        Tag::Imm,
    )
}
pub fn c_nzuimm6hi(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm((x(insn, 2, 4)) + (x(insn, 12, 1) << 5)),
        Tag::Imm,
    )
}
pub fn c_nzuimm6lo(_insn: u32) -> (Arg, Tag) {
    (Arg::Nothing, Tag::None)
}
pub fn c_uimm8sp_s(insn: u32) -> (Arg, Tag) {
    (
        Arg::UImm((x(insn, 9, 4) << 2) + (x(insn, 7, 2) << 6)),
        Tag::Imm,
    )
}

// vector
pub fn vd(insn: u32) -> (Arg, Tag) {
    (Arg::DstReg(x(insn, 7, 5)), Tag::Vd)
}
pub fn vs3(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 7, 5)), Tag::Vs3)
}
pub fn vs1(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 15, 5)), Tag::Vs1)
}
pub fn vs2(insn: u32) -> (Arg, Tag) {
    (Arg::SrcReg(x(insn, 20, 5)), Tag::Vs2)
}

pub fn vm(insn: u32) -> (Arg, Tag) {
    (Arg::Flag(x(insn, 25, 1)), Tag::Vm)
}
pub fn simm5(insn: u32) -> (Arg, Tag) {
    (Arg::Imm(xs(insn, 15, 5)), Tag::Imm)
}
pub fn zimm10(insn: u32) -> (Arg, Tag) {
    (Arg::UImm(x(insn, 20, 10)), Tag::Imm)
}
pub fn zimm11(insn: u32) -> (Arg, Tag) {
    (Arg::UImm(x(insn, 20, 11)), Tag::Imm)
}
