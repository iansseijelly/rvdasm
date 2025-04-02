// tests for the disassembler
use riscv_disasm::disassembler::Disassembler;
use std::fs::File;
use std::io::Read;

#[test]
fn test_decode_all() {
    let disassembler = Disassembler::new();
    let mut file = File::open("tests/data/test.bin").unwrap();
    let mut bin = Vec::new();
    file.read_to_end(&mut bin).unwrap();
    let mut insns = disassembler.disassemble_all(&bin, 0);
    // sort keys by address 
    let mut keys: Vec<usize> = insns.keys().cloned().collect();
    keys.sort();
    for key in keys {
        println!("{}: {}", key, insns[&key].to_string());
    }
}