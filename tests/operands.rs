// The operand accessors: rd, rs1, rs2, and imm.
use rvdasm::disassembler::*;
use rvdasm::insn::Insn;

fn decode(code: u32) -> Insn {
    Disassembler::new(Xlen::XLEN64)
        .disassmeble_one(code)
        .unwrap()
}

/// `(rd, rs1, rs2, imm)`.
fn operands(code: u32) -> (Option<u32>, Option<u32>, Option<u32>, Option<i64>) {
    let insn = decode(code);
    (insn.rd(), insn.rs1(), insn.rs2(), insn.imm())
}

#[test]
fn register_operands() {
    assert_eq!(operands(0x00b50533), (Some(10), Some(10), Some(11), None)); // add a0,a0,a1
    assert_eq!(operands(0x02b50533), (Some(10), Some(10), Some(11), None)); // mul a0,a0,a1
}

#[test]
fn a_signed_immediate_is_sign_extended() {
    assert_eq!(operands(0xfff28293), (Some(5), Some(5), None, Some(-1))); // addi t0,t0,-1
    assert_eq!(operands(0xfc3f2223), (None, Some(30), Some(3), Some(-60))); // sw gp,-60(t5)
}

#[test]
fn a_branch_or_jump_offset_is_assembled() {
    assert_eq!(operands(0x0500006f), (Some(0), None, None, Some(0x50))); // jal zero, +0x50
    assert_eq!(operands(0x4e771063), (None, Some(14), Some(7), Some(0x4e0))); // bne a4,t2,+0x4e0
    assert_eq!(operands(0xfe000ee3), (None, Some(0), Some(0), Some(-4))); // beq zero,zero,-4
}

#[test]
fn an_upper_immediate_is_shifted() {
    assert_eq!(operands(0x0000b2b7), (Some(5), None, None, Some(0xb000))); // lui t0,0xb
    assert_eq!(operands(0xfffff517), (Some(10), None, None, Some(-0x1000))); // auipc a0,0xfffff
}

#[test]
fn an_unsigned_immediate_is_zero_extended() {
    assert_eq!(operands(0x03f51513), (Some(10), Some(10), None, Some(63))); // slli a0,a0,63
    assert_eq!(operands(0x30445073), (Some(0), None, None, Some(8))); // csrwi mie,8
}

#[test]
fn a_compressed_destination_that_is_also_a_source_is_both() {
    assert_eq!(operands(0x0505), (Some(10), Some(10), None, Some(1))); // c.addi a0,1
    assert_eq!(operands(0x952e), (Some(10), Some(10), Some(11), None)); // c.add a0,a1
    assert_eq!(operands(0x8d0d), (Some(10), Some(10), Some(11), None)); // c.sub a0,a1
}

#[test]
fn a_csr_instruction_has_its_csr() {
    let csrr = decode(0xf1402573); // csrr a0,mhartid: csrrs a0,mhartid,zero
    assert_eq!((csrr.rd(), csrr.rs1(), csrr.csr()), (Some(10), Some(0), Some(0xf14)));
    let csrwi = decode(0x30445073); // csrwi mie,8
    assert_eq!((csrwi.csr(), csrwi.imm()), (Some(0x304), Some(8)));
    assert_eq!(decode(0x00b50533).csr(), None); // add a0,a0,a1
}
