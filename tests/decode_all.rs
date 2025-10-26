// tests for the disassembler
use rvdasm::disassembler::*;
use std::fs::File;
use std::io::Read;

fn test_decode_all(to_canonical: bool) {
    let disassembler = Disassembler::new(Xlen::XLEN64);
    let mut file = File::open("tests/data/test.bin").unwrap();
    let mut bin = Vec::new();
    file.read_to_end(&mut bin).unwrap();
    let insns = disassembler.disassemble_all(&bin, 0x80000000);
    // sort keys by address
    let mut keys: Vec<u64> = insns.keys().cloned().collect();
    keys.sort();
    for key in keys {
        if to_canonical {
            println!("START INST {} TIMESTAMP 0 END", insns[&key].to_canonical());
        } else {
            println!(
                "0x{:08x}: {:08x}     {}",
                key,
                insns[&key].get_raw(),
                insns[&key].to_string()
            );
        }
    }
}

#[test]
fn test_decode_all_canonical() {
    println!("Testing canonical disassembly");
    test_decode_all(true);
}

#[test]
fn test_decode_all_string() {
    println!("Testing string disassembly");
    test_decode_all(false);
}
