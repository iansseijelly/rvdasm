// The generated Op and Ext enums, and the Op a decode gives.
use rvdasm::disassembler::*;
use rvdasm::{Ext, Op};
use std::collections::HashSet;

#[test]
fn every_op_has_a_distinct_name_and_an_extension() {
    let names: HashSet<_> = Op::ALL.iter().map(|op| op.name()).collect();
    assert_eq!(names.len(), Op::ALL.len());
    assert!(Op::ALL.iter().all(|op| op.ext().is_some()));
    assert!(Op::ALL.windows(2).all(|w| w[0].name() < w[1].name()), "ALL is in mnemonic order");
}

#[test]
fn unknown_is_not_an_instruction() {
    assert!(!Op::ALL.contains(&Op::Unknown));
    assert_eq!(Op::Unknown.name(), "unknown");
    assert_eq!(Op::Unknown.ext(), None);
}

#[test]
fn every_ext_defines_an_instruction() {
    let used: HashSet<_> = Op::ALL.iter().filter_map(|op| op.ext()).collect();
    assert_eq!(used, Ext::ALL.iter().copied().collect());
    assert_eq!(Ext::RvI.name(), "rv_i");
    assert_eq!(Ext::Rv64I.name(), "rv64_i");
}

#[test]
fn decode_gives_the_op() {
    for xlen in [Xlen::XLEN32, Xlen::XLEN64] {
        let dasm = Disassembler::new(xlen);
        let op = |code| dasm.disassmeble_one(code).map(|insn| insn.op);
        assert_eq!(op(0x00b50533), Some(Op::Add)); // add a0,a0,a1
        assert_eq!(op(0x02b50533), Some(Op::Mul)); // mul a0,a0,a1
        assert_eq!(op(0x0000100f), Some(Op::FenceI)); // fence.i
        assert_eq!(op(0x4505), Some(Op::CLi)); // c.li a0,1
        // slli has an RV32 encoding and an RV64 one, and is one Op in RV64I's extension.
        assert_eq!(op(0x00151513), Some(Op::Slli)); // slli a0,a0,1
    }
    assert_eq!(Op::Slli.ext(), Some(Ext::Rv64I));
    assert_eq!(Op::Add.ext(), Some(Ext::RvI));
    assert_eq!(Op::Mul.ext(), Some(Ext::RvM));
}

#[test]
fn a_walk_passes_over_an_undecodable_encoding_as_unknown() {
    let dasm = Disassembler::new(Xlen::XLEN64);
    let code = [0xffffffffu32.to_le_bytes(), 0x00b50533u32.to_le_bytes()].concat();
    let mut ops = Vec::new();
    dasm.disassemble_each(&code, 0x1000, |addr, insn| ops.push((addr, insn.op, insn.get_name())));
    assert_eq!(ops, [(0x1000, Op::Unknown, "unknown"), (0x1004, Op::Add, "add")]);
}
