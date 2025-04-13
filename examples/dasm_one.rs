use rvdasm::disassembler::*;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    input: String,
		#[clap(short, long, default_value = "false")]
		canonical: bool,
		#[clap(short, long, default_value = "64")]
		xlen: String,
}

fn main() {
	let args = Args::parse();
	// parse input as hex, which is NOT a file
	let hex = u32::from_str_radix(&args.input, 16).unwrap();
	// disassemble the line
	let xlen = match args.xlen.as_str() {
		"32" => Xlen::XLEN32,
		"64" => Xlen::XLEN64,
		_ => panic!("Invalid xlen: {}", args.xlen),
	};
	let disassembler = Disassembler::new(xlen);
	let insn = disassembler.disassmeble_one(hex).unwrap();
	if args.canonical {
		println!("{}", insn.to_canonical());
	} else {
		println!("{}", insn.to_string());
	}
}
