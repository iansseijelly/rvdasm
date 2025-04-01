/* Automatically generated by parse_opcodes */
use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::insn::*;

pub struct Spec {
  pub name: String,
  pub mask_bits: u32,
  pub match_bits: u32,
  pub args: Vec<(fn(u32)->(Arg, String))>,
}

impl Spec {    
    pub fn new(name: &str, mask_bits: u32, match_bits: u32, args: Vec<(fn(u32)->(Arg, String))>) -> Self {
        Self { name: name.to_string(), mask_bits, match_bits, args }
    }

    pub fn compare(&self, code: u32) -> bool {
        (code & self.mask_bits) == self.match_bits
    }
}

pub static RV_ISA_SPECS: Lazy<Vec<Spec>> = Lazy::new(|| vec![
    Spec::new("add", 0xfe00707f, 0x33, vec![rd, rs1, rs2]),
    Spec::new("addi", 0x707f, 0x13, vec![rd, rs1, imm12]),
    Spec::new("addiw", 0x707f, 0x1b, vec![rd, rs1, imm12]),
    Spec::new("addw", 0xfe00707f, 0x3b, vec![rd, rs1, rs2]),
    Spec::new("amoadd_d", 0xf800707f, 0x302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoadd_w", 0xf800707f, 0x202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoand_d", 0xf800707f, 0x6000302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoand_w", 0xf800707f, 0x6000202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amomax_d", 0xf800707f, 0xa000302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amomax_w", 0xf800707f, 0xa000202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amomaxu_d", 0xf800707f, 0xe000302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amomaxu_w", 0xf800707f, 0xe000202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amomin_d", 0xf800707f, 0x8000302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amomin_w", 0xf800707f, 0x8000202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amominu_d", 0xf800707f, 0xc000302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amominu_w", 0xf800707f, 0xc000202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoor_d", 0xf800707f, 0x4000302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoor_w", 0xf800707f, 0x4000202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoswap_d", 0xf800707f, 0x800302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoswap_w", 0xf800707f, 0x800202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoxor_d", 0xf800707f, 0x2000302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("amoxor_w", 0xf800707f, 0x2000202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("and", 0xfe00707f, 0x7033, vec![rd, rs1, rs2]),
    Spec::new("andi", 0x707f, 0x7013, vec![rd, rs1, imm12]),
    Spec::new("auipc", 0x7f, 0x17, vec![rd, imm20]),
    Spec::new("beq", 0x707f, 0x63, vec![bimm12hi, rs1, rs2, bimm12lo]),
    Spec::new("bge", 0x707f, 0x5063, vec![bimm12hi, rs1, rs2, bimm12lo]),
    Spec::new("bgeu", 0x707f, 0x7063, vec![bimm12hi, rs1, rs2, bimm12lo]),
    Spec::new("blt", 0x707f, 0x4063, vec![bimm12hi, rs1, rs2, bimm12lo]),
    Spec::new("bltu", 0x707f, 0x6063, vec![bimm12hi, rs1, rs2, bimm12lo]),
    Spec::new("bne", 0x707f, 0x1063, vec![bimm12hi, rs1, rs2, bimm12lo]),
    Spec::new("div", 0xfe00707f, 0x2004033, vec![rd, rs1, rs2]),
    Spec::new("divu", 0xfe00707f, 0x2005033, vec![rd, rs1, rs2]),
    Spec::new("divuw", 0xfe00707f, 0x200503b, vec![rd, rs1, rs2]),
    Spec::new("divw", 0xfe00707f, 0x200403b, vec![rd, rs1, rs2]),
    Spec::new("ebreak", 0xffffffff, 0x100073, vec![]),
    Spec::new("ecall", 0xffffffff, 0x73, vec![]),
    Spec::new("fence", 0x707f, 0xf, vec![fm, pred, succ, rs1, rd]),
    Spec::new("jal", 0x7f, 0x6f, vec![rd, jimm20]),
    Spec::new("jalr", 0x707f, 0x67, vec![rd, rs1, imm12]),
    Spec::new("lb", 0x707f, 0x3, vec![rd, rs1, imm12]),
    Spec::new("lbu", 0x707f, 0x4003, vec![rd, rs1, imm12]),
    Spec::new("ld", 0x707f, 0x3003, vec![rd, rs1, imm12]),
    Spec::new("lh", 0x707f, 0x1003, vec![rd, rs1, imm12]),
    Spec::new("lhu", 0x707f, 0x5003, vec![rd, rs1, imm12]),
    Spec::new("lr_d", 0xf9f0707f, 0x1000302f, vec![rd, rs1, aq, rl]),
    Spec::new("lr_w", 0xf9f0707f, 0x1000202f, vec![rd, rs1, aq, rl]),
    Spec::new("lui", 0x7f, 0x37, vec![rd, imm20]),
    Spec::new("lw", 0x707f, 0x2003, vec![rd, rs1, imm12]),
    Spec::new("lwu", 0x707f, 0x6003, vec![rd, rs1, imm12]),
    Spec::new("mul", 0xfe00707f, 0x2000033, vec![rd, rs1, rs2]),
    Spec::new("mulh", 0xfe00707f, 0x2001033, vec![rd, rs1, rs2]),
    Spec::new("mulhsu", 0xfe00707f, 0x2002033, vec![rd, rs1, rs2]),
    Spec::new("mulhu", 0xfe00707f, 0x2003033, vec![rd, rs1, rs2]),
    Spec::new("mulw", 0xfe00707f, 0x200003b, vec![rd, rs1, rs2]),
    Spec::new("or", 0xfe00707f, 0x6033, vec![rd, rs1, rs2]),
    Spec::new("ori", 0x707f, 0x6013, vec![rd, rs1, imm12]),
    Spec::new("rem", 0xfe00707f, 0x2006033, vec![rd, rs1, rs2]),
    Spec::new("remu", 0xfe00707f, 0x2007033, vec![rd, rs1, rs2]),
    Spec::new("remuw", 0xfe00707f, 0x200703b, vec![rd, rs1, rs2]),
    Spec::new("remw", 0xfe00707f, 0x200603b, vec![rd, rs1, rs2]),
    Spec::new("sb", 0x707f, 0x23, vec![imm12hi, rs1, rs2, imm12lo]),
    Spec::new("sc_d", 0xf800707f, 0x1800302f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("sc_w", 0xf800707f, 0x1800202f, vec![rd, rs1, rs2, aq, rl]),
    Spec::new("sd", 0x707f, 0x3023, vec![imm12hi, rs1, rs2, imm12lo]),
    Spec::new("sh", 0x707f, 0x1023, vec![imm12hi, rs1, rs2, imm12lo]),
    Spec::new("sll", 0xfe00707f, 0x1033, vec![rd, rs1, rs2]),
    Spec::new("slli", 0xfc00707f, 0x1013, vec![rd, rs1, shamtd]),
    Spec::new("slliw", 0xfe00707f, 0x101b, vec![rd, rs1, shamtw]),
    Spec::new("sllw", 0xfe00707f, 0x103b, vec![rd, rs1, rs2]),
    Spec::new("slt", 0xfe00707f, 0x2033, vec![rd, rs1, rs2]),
    Spec::new("slti", 0x707f, 0x2013, vec![rd, rs1, imm12]),
    Spec::new("sltiu", 0x707f, 0x3013, vec![rd, rs1, imm12]),
    Spec::new("sltu", 0xfe00707f, 0x3033, vec![rd, rs1, rs2]),
    Spec::new("sra", 0xfe00707f, 0x40005033, vec![rd, rs1, rs2]),
    Spec::new("srai", 0xfc00707f, 0x40005013, vec![rd, rs1, shamtd]),
    Spec::new("sraiw", 0xfe00707f, 0x4000501b, vec![rd, rs1, shamtw]),
    Spec::new("sraw", 0xfe00707f, 0x4000503b, vec![rd, rs1, rs2]),
    Spec::new("srl", 0xfe00707f, 0x5033, vec![rd, rs1, rs2]),
    Spec::new("srli", 0xfc00707f, 0x5013, vec![rd, rs1, shamtd]),
    Spec::new("srliw", 0xfe00707f, 0x501b, vec![rd, rs1, shamtw]),
    Spec::new("srlw", 0xfe00707f, 0x503b, vec![rd, rs1, rs2]),
    Spec::new("sub", 0xfe00707f, 0x40000033, vec![rd, rs1, rs2]),
    Spec::new("subw", 0xfe00707f, 0x4000003b, vec![rd, rs1, rs2]),
    Spec::new("sw", 0x707f, 0x2023, vec![imm12hi, rs1, rs2, imm12lo]),
    Spec::new("xor", 0xfe00707f, 0x4033, vec![rd, rs1, rs2]),
    Spec::new("xori", 0x707f, 0x4013, vec![rd, rs1, imm12]),
]);
