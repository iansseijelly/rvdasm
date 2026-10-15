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

## Supported Extensions

I, M, A, C, F, D, V (with its segment loads and stores), Zicsr, Zifencei, and Zvbb.

Distinguishes XLEN of 32 or 64.

## Development Notes

The `isa.rs` file is generated from [iansseijelly:riscv-opcodes](https://github.com/iansseijelly/riscv-opcodes).
This repo is also registered as a submodule in `${ROOT}/riscv-opcodes`.
Run `gen.sh` in that repo, with a Python that has PyYAML, to generate `isa.rs` and `isa_consts.rs`, then format
them with `rustfmt --edition 2021 src/isa.rs src/isa_consts.rs`.
