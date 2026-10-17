//! Checks `Insn::asm_no_aliases` and `Insn::asm` against GNU objdump's text, with `-M no-aliases` and by default, over
//! the encodings in tests/data/objdump_rv64.tsv and objdump_rv32.tsv, which scripts/objdump_ref.py writes: rvdasm
//! decodes exactly the encodings objdump decodes, and writes each as objdump does.
use rvdasm::disassembler::{Disassembler, Xlen};
use rvdasm::insn::Insn;
use std::fs;

/// Each line is offset, encoding, objdump's text with `-M no-aliases`, and its text by default.
const NO_ALIASES: usize = 2;
const ALIASES: usize = 3;

fn check(path: &str, xlen: Xlen, column: usize, text: fn(&Insn, u64) -> String) {
    let d = Disassembler::new(xlen);
    let mut failures = Vec::new();
    let mut checked = 0;
    for line in fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let f: Vec<&str> = line.split('\t').collect();
        let offset = u64::from_str_radix(f[0], 16).unwrap();
        let word = u32::from_str_radix(f[1], 16).unwrap();
        let want = f[column];
        let got = d.disassmeble_one(word).map(|insn| text(&insn, offset));
        checked += 1;
        let same = match &got {
            None => want == ".insn",
            Some(text) => text == want,
        };
        if !same {
            let got = got.unwrap_or_else(|| "(not decoded)".to_string());
            failures.push(format!(
                "{:>8}  objdump: {:<44} rvdasm: {}",
                f[1], want, got
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{path}: {} of {checked} encodings differ from objdump:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn rv64_no_aliases() {
    check(
        "tests/data/objdump_rv64.tsv",
        Xlen::XLEN64,
        NO_ALIASES,
        Insn::asm_no_aliases,
    );
}

#[test]
fn rv32_no_aliases() {
    check(
        "tests/data/objdump_rv32.tsv",
        Xlen::XLEN32,
        NO_ALIASES,
        Insn::asm_no_aliases,
    );
}

#[test]
fn rv64() {
    check(
        "tests/data/objdump_rv64.tsv",
        Xlen::XLEN64,
        ALIASES,
        Insn::asm,
    );
}

#[test]
fn rv32() {
    check(
        "tests/data/objdump_rv32.tsv",
        Xlen::XLEN32,
        ALIASES,
        Insn::asm,
    );
}
