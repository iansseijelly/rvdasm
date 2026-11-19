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

## Instructions

A decoded `Insn`'s `op` is its instruction, an `Op`, generated from riscv-opcodes with one variant per mnemonic
(`Op::Add`, `Op::CAddi`, `Op::FenceI`), so a consumer can `match` on it. `Op::name()` gives the mnemonic, and
`Op::ext()` the riscv-opcodes extension that defines it, an `Ext` such as `Ext::RvI` or `Ext::Rv64I`. `Op::ALL` and
`Ext::ALL` list them. A disassembly walk gives an encoding that decodes to no instruction as `Op::Unknown`, which is
not in `Op::ALL` and has no extension.

Its operands are `Insn::rd()`, `rs1()`, `rs2()`, `imm()`, and `csr()`, each `None` if the instruction has no such
operand. `imm()` is the immediate as the instruction uses it: a branch's offset assembled from its fields, `lui`'s
shifted left by 12, sign-extended if it is signed and zero-extended if not.

## Assembly text

`Insn::asm(pc)` gives an instruction's text as GNU objdump prints it, as at `pc`: ABI register names, `imm(rs1)`
memory operands, CSR names, rounding modes, vector types such as `e64,m8,ta,ma`, `v0.t` on masked vector
instructions, a compressed instruction as its base instruction, and aliases such as `li`, `mv`, `ret`, `beqz`, and
`csrr`. `Insn::asm_no_aliases(pc)` gives it as objdump prints it with `-M no-aliases`, canonical mnemonics only. A
branch or jump target is its absolute address, `0x80000050`, where objdump adds a symbol.

`tests/objdump.rs` checks both, and that rvdasm decodes exactly the encodings objdump decodes, against objdump's text
for every encoding in `tests/data/objdump_rv64.tsv` and `objdump_rv32.tsv`: random encodings of every instruction,
as many with operands biased to the registers and immediates aliases look for, random encodings in every opcode, and
edge cases. `scripts/objdump_ref.py` writes them with GNU binutils 2.46.

## Supported Extensions

I, M, A, C, F, D, V (with its segment loads and stores), Zicsr, Zifencei, and Zvbb.

Distinguishes XLEN of 32 or 64.

## Development Notes

The `isa.rs` file is generated from [iansseijelly:riscv-opcodes](https://github.com/iansseijelly/riscv-opcodes).
This repo is also registered as a submodule in `${ROOT}/riscv-opcodes`.
Run `gen.sh` in that repo, with a Python that has PyYAML, to generate `isa.rs` and `isa_consts.rs`, then format
them with `rustfmt --edition 2021 src/isa.rs src/isa_consts.rs`.
