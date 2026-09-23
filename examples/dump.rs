// Golden dump: every instruction of a raw code blob, both text forms, sorted by address.
use rvdasm::disassembler::*;
use std::io::{BufWriter, Write};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let code = std::fs::read(&a[1]).unwrap();
    let base = u64::from_str_radix(a[2].trim_start_matches("0x"), 16).unwrap();
    let d = Disassembler::new(Xlen::XLEN64);
    let insns = d.disassemble_all(&code, base);
    let mut keys: Vec<u64> = insns.keys().cloned().collect();
    keys.sort();
    let mut w = BufWriter::new(std::io::stdout());
    for k in keys {
        let i = &insns[&k];
        writeln!(w, "{:#x} {:08x} len={} kind={} off={} | {} | {}", k, i.get_raw(), i.get_len(),
                 i.is_branch() as u8 | (i.is_direct_jump() as u8) << 1 | (i.is_indirect_jump() as u8) << 2,
                 i.offset, i.to_string(), i.to_canonical()).unwrap();
    }
}
