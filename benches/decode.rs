//! Decode throughput, in nanoseconds per instruction: `cargo bench --bench decode`.
//!
//! - all specs: every encoding in the objdump corpus once, which is mostly vector instructions.
//! - scalar specs: the same, without the vector ones.
//! - the ELF named by `RVDASM_BENCH_ELF`, if set: the 32-bit instructions in its code, a real instruction mix.
//!
//! Each figure is the best of several runs, which is steadier than the mean on a shared machine.

use object::{Object, ObjectSection, SectionKind};
use rvdasm::disassembler::{Disassembler, Xlen};
use rvdasm::isa::Ext;
use std::hint::black_box;
use std::time::Instant;

const RUNS: usize = 10;
/// About how many decodes one run makes, so each stream takes about as long whatever its size.
const DECODES_PER_RUN: usize = 2_000_000;

fn main() {
    let dasm = Disassembler::new(Xlen::XLEN64);

    let corpus = corpus();
    let scalar: Vec<u32> = corpus
        .iter()
        .copied()
        .filter(|&w| {
            let ext = dasm.disassmeble_one(w).and_then(|i| i.op.ext());
            !matches!(ext, Some(Ext::RvV | Ext::RvZvbb))
        })
        .collect();
    bench(&dasm, "all specs", &corpus);
    bench(&dasm, "scalar specs", &scalar);

    if let Ok(path) = std::env::var("RVDASM_BENCH_ELF") {
        bench(&dasm, &path, &elf_code(&path));
    }
}

/// Decodes `words` over and over and prints the best run's time per decode.
fn bench(dasm: &Disassembler, name: &str, words: &[u32]) {
    let reps = DECODES_PER_RUN.div_ceil(words.len());
    let mut best = f64::INFINITY;
    for _ in 0..RUNS {
        let start = Instant::now();
        for _ in 0..reps {
            for &w in words {
                black_box(dasm.disassmeble_one(black_box(w)));
            }
        }
        let ns = start.elapsed().as_nanos() as f64 / (reps * words.len()) as f64;
        best = best.min(ns);
    }
    println!("{name}: {best:.1} ns/insn ({} words)", words.len());
}

/// The 32-bit encodings in the RV64 objdump corpus.
fn corpus() -> Vec<u32> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/objdump_rv64.tsv");
    let tsv = std::fs::read_to_string(path).unwrap();
    tsv.lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split('\t').nth(1))
        .map(|h| u32::from_str_radix(h, 16).unwrap())
        .filter(|w| w & 3 == 3)
        .collect()
}

/// The 32-bit instructions in the ELF at `path`'s code sections, stepping over its compressed ones.
fn elf_code(path: &str) -> Vec<u32> {
    let bytes = std::fs::read(path).unwrap();
    let elf = object::File::parse(&*bytes).unwrap();
    let mut words = Vec::new();
    for section in elf.sections().filter(|s| s.kind() == SectionKind::Text) {
        let code = section.data().unwrap();
        let mut i = 0;
        while i + 2 <= code.len() {
            if code[i] & 3 != 3 {
                i += 2;
            } else if i + 4 <= code.len() {
                words.push(u32::from_le_bytes(code[i..i + 4].try_into().unwrap()));
                i += 4;
            } else {
                break;
            }
        }
    }
    words
}
