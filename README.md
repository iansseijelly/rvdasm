# RVDASM

A RISC-V disassmbler written in rust.

## Key features

* **Correct**: code-gen from riscv-opcodes
* **Programmer-friedly**: outputs to struct, not string
* **Simple**: easy to read and modify

## Usage

Example:

```bash
RUST_LOG=debug cargo run  --example dasm -- --file [ELF] --print
```

## Assembly text

`Insn::asm_no_aliases(pc)` gives an instruction's text as GNU objdump prints it with `-M no-aliases`, as at `pc`:
ABI register names, `imm(rs1)` memory operands, CSR names, rounding modes, vector types such as `e64,m8,ta,ma`, and
`v0.t` on masked vector instructions. A branch or jump target is its absolute address, `0x80000050`, where objdump
adds a symbol.

`tests/objdump.rs` checks it, and that rvdasm decodes exactly the encodings objdump decodes, against objdump's text
for every encoding in `tests/data/objdump_rv64.tsv` and `objdump_rv32.tsv`: random encodings of every instruction,
random encodings in every opcode, and a few edge cases. `scripts/objdump_ref.py` writes them with GNU binutils 2.46.

## Supported Extensions

I, M, A, C, F, D, V (with its segment loads and stores), Zicsr, Zifencei, and Zvbb.

Distinguishes XLEN of 32 or 64.

## Development Notes

The `isa.rs` file is generated from [iansseijelly:riscv-opcodes](https://github.com/iansseijelly/riscv-opcodes).
This repo is also registered as a submodule in `${ROOT}/riscv-opcodes`.
Run `gen.sh` in that repo, with a Python that has PyYAML, to generate `isa.rs` and `isa_consts.rs`, then format
them with `rustfmt --edition 2021 src/isa.rs src/isa_consts.rs`.
