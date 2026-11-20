// The generated decoders against the spec tables they are built from: for any encoding, a decoder must pick what a
// scan of the tables in order picks, the first spec that matches and whose packer accepts, generic specs before the
// XLEN's own. Checked on every 16-bit encoding, on random encodings of every spec, and on random words.
use rvdasm::disassembler::{Disassembler, Xlen};
use rvdasm::insn::Insn;
use rvdasm::isa::*;

/// What a scan of the tables decodes `code` as.
fn scan(code: u32, xlen: u32) -> Option<Insn> {
    let tables: [Option<&[Spec]>; 2] = if code & 3 != 3 {
        let own = if xlen == 32 {
            RV_ISA_SPECS_32_COMPRESSED
        } else {
            RV_ISA_SPECS_64_COMPRESSED
        };
        [Some(RV_ISA_SPECS_GENERIC_COMPRESSED), Some(own)]
    } else {
        let opcode = (code & 0x7f) as u8;
        let own = if xlen == 32 {
            get_32_full_specs_by_opcode(opcode)
        } else {
            get_64_full_specs_by_opcode(opcode)
        };
        [get_generic_full_specs_by_opcode(opcode), own]
    };
    tables
        .into_iter()
        .flatten()
        .flatten()
        .filter(|spec| spec.compare(code))
        .find_map(|spec| (spec.pack)(code, spec.op, xlen as u8))
}

fn check(dasm: &Disassembler, xlen: u32, code: u32) {
    let got = dasm.disassmeble_one(code).map(|i| (i.op, i.to_string()));
    let want = scan(code, xlen).map(|i| (i.op, i.to_string()));
    assert_eq!(got, want, "{code:#010x} on RV{xlen}");
}

/// xorshift64: deterministic, and enough for spreading bits.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
}

#[test]
fn decoders_match_a_scan_of_the_tables() {
    for (xlen, x) in [(32, Xlen::XLEN32), (64, Xlen::XLEN64)] {
        let dasm = Disassembler::new(x);
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        for code in 0..0x1_0000u32 {
            if code & 3 != 3 {
                check(&dasm, xlen, code);
            }
        }
        let mut all: Vec<&Spec> = vec![];
        all.extend(RV_ISA_SPECS_GENERIC_COMPRESSED);
        all.extend(RV_ISA_SPECS_32_COMPRESSED);
        all.extend(RV_ISA_SPECS_64_COMPRESSED);
        for opcode in 0..0x80u8 {
            for table in [
                get_generic_full_specs_by_opcode(opcode),
                get_32_full_specs_by_opcode(opcode),
                get_64_full_specs_by_opcode(opcode),
            ] {
                all.extend(table.into_iter().flatten());
            }
        }
        for spec in all {
            for _ in 0..256 {
                check(
                    &dasm,
                    xlen,
                    spec.match_bits | (rng.next() & !spec.mask_bits),
                );
            }
        }
        for _ in 0..1_000_000 {
            check(&dasm, xlen, rng.next() | 3);
        }
    }
}
