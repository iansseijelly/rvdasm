use clap::Parser;
use object::{Object, ObjectSection};
use rvdasm::disassembler::*;
use std::fs::File;
use std::io::Read;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    file: String,
    #[clap(short, long, default_value = "false")]
    canonical: bool,
}

fn main() {
    let args = Args::parse();
    let mut elf_file = File::open(args.file.clone()).unwrap();
    let mut elf_buffer = Vec::new();
    elf_file.read_to_end(&mut elf_buffer).unwrap();
    let elf = object::File::parse(&*elf_buffer).unwrap();

    let elf_arch = elf.architecture();
    // elf.architecture() == object::Architecture::Riscv64
    let xlen = if elf_arch == object::Architecture::Riscv64 {
        Xlen::XLEN64
    } else if elf_arch == object::Architecture::Riscv32 {
        Xlen::XLEN32
    } else {
        panic!("Unsupported architecture: {:?}", elf_arch);
    };
    let disassembler = Disassembler::new(xlen);

    let text_section = elf.section_by_name(".text").unwrap();
    let text_data = text_section.data().unwrap();
    let entry_point = text_section.address();

    println!("entry point: 0x{:08x}", entry_point);

    let decoded_insns = disassembler.disassemble_all(&text_data, entry_point);

    // sort keys by address
    let mut keys: Vec<u64> = decoded_insns.keys().cloned().collect();
    keys.sort();

    // write to file with extension .dump
    // let mut dump_file = File::create(format!("{}.dump", args.file)).unwrap();
    for key in keys {
        if args.canonical {
            println!("{}", decoded_insns[&key].to_canonical());
        } else {
            println!(
                "0x{:08x}: {:08x}     {}",
                key,
                decoded_insns[&key].get_raw(),
                decoded_insns[&key].to_string()
            );
        }
    }
}
