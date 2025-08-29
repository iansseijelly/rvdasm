use rvdasm::disassembler::*;
use clap::Parser;
use std::fs;
use regex::Regex;

// Just like spike-dasm.cc,
// This little program finds occurrences of strings like
//  DASM(ffabc013)
// in its input, then replaces them with the disassembly
// enclosed hexadecimal number, interpreted as a RISC-V
// instruction.

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    input_file_path: String,
		#[clap(short, long, default_value = "false")]
		canonical: bool,
		#[clap(short, long, default_value = "64")]
		xlen: String,
		#[clap(short, long)]
		output_file_path: String,
}

fn main() {
	let args = Args::parse();
	// read the file into lines
	let file_content = fs::read_to_string(args.input_file_path).unwrap();
	let lines: Vec<&str> = file_content.lines().collect();
	let xlen = match args.xlen.as_str() {
		"32" => Xlen::XLEN32,
		"64" => Xlen::XLEN64,
		_ => panic!("Invalid xlen: {}", args.xlen),
	};
	let disassembler = Disassembler::new(xlen);

	// Create a vector to hold modified lines
	let mut modified_lines = Vec::new();
	
	// Process each line, replacing DASM patterns with disassembly
	for line in lines {
		let pattern = Regex::new(r"DASM\((0x[0-9a-fA-F]+)\)").unwrap();
		let modified_line = pattern.replace_all(line, |caps: &regex::Captures| {
			let hex_str = &caps[1];
			let hex = u32::from_str_radix(hex_str.trim_start_matches("0x"), 16).unwrap();
			match disassembler.disassmeble_one(hex) {
				Some(insn) => {
					if args.canonical {
						insn.to_canonical()
					} else {
						insn.to_string()
					}
				},
				None => {
					format!("<unknown instruction: {:#010x}>", hex)
				}
			}
		});
		modified_lines.push(modified_line.to_string());
	}
	
	if args.output_file_path.is_empty() {
		// Print all modified content
		println!("{}", modified_lines.join("\n"));
	} else {
		// Print all modified content
		fs::write(args.output_file_path, modified_lines.join("\n")).unwrap();
	}
}
