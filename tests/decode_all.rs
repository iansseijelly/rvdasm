// tests for the disassembler
use rvdasm::disassembler::Disassembler;
use std::fs::File;
use std::io::Read;

fn test_decode_all(to_canonical: bool) {
    let disassembler = Disassembler::new();
    let mut file = File::open("tests/data/test.bin").unwrap();
    let mut bin = Vec::new();
    file.read_to_end(&mut bin).unwrap();
    let insns = disassembler.disassemble_all(&bin, 0x80000000);
    // sort keys by address 
    let mut keys: Vec<usize> = insns.keys().cloned().collect();
    keys.sort();
    for key in keys {
        if to_canonical {
            println!("START INST {} TIMESTAMP 0 END", insns[&key].to_canonical());
        } else {
            println!("0x{:08x}: {:08x}     {}", key, insns[&key].raw, insns[&key].to_string());
        }
    }
}

#[test]
fn test_decode_all_canonical() {
    test_decode_all(true);
}

#[test]
fn test_decode_all_string() {
    test_decode_all(false);
}
