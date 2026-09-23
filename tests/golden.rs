// Regression: the text form of every instruction in tests/data/test.bin must match
// tests/data/test.golden.txt, captured from rvdasm 0.2.6 before the flat Insn layout.
use rvdasm::disassembler::*;
use std::fs;

#[test]
fn test_golden_text() {
    let d = Disassembler::new(Xlen::XLEN64);
    let bin = fs::read("tests/data/test.bin").unwrap();
    let insns = d.disassemble_all(&bin, 0x80000000);
    let mut keys: Vec<u64> = insns.keys().cloned().collect();
    keys.sort();
    let got: Vec<String> = keys
        .iter()
        .map(|k| format!("0x{:08x}: {:08x}     {}", k, insns[k].get_raw(), insns[k].to_string()))
        .collect();
    let want: Vec<String> = fs::read_to_string("tests/data/test.golden.txt")
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(got, want);
}

#[test]
fn test_disassemble_each_matches_all() {
    let d = Disassembler::new(Xlen::XLEN64);
    let bin = fs::read("tests/data/test.bin").unwrap();
    let all = d.disassemble_all(&bin, 0x80000000);
    let mut n = 0;
    d.disassemble_each(&bin, 0x80000000, |addr, insn| {
        assert_eq!(all[&addr].get_raw(), insn.get_raw());
        n += 1;
    });
    assert_eq!(n, all.len());
}
