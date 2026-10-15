#!/usr/bin/env python3
"""Writes the reference that tests/objdump.rs checks Insn::asm against: GNU objdump's text for a fixed corpus of
encodings, for RV64 and RV32.

The corpus holds, for every spec in src/isa.rs that the XLEN uses, encodings matching the spec with the other bits
random, and as many again with register fields biased to x0, x1, or each other and immediates to 0, 1, or -1, where
objdump's aliases apply; random encodings in every major opcode and in the compressed space, so that it also covers
encodings rvdasm does not decode; and a few edge cases. The script assembles them with `.insn`, for an ISA string that names the extensions rvdasm
decodes, disassembles them with `objdump -d -z`, with and without `-M no-aliases`, and writes one line per encoding:

    offset<TAB>encoding<TAB>no-aliases text<TAB>default text

A text is objdump's, its mnemonic followed by one space and its operands, without comments; a branch or jump target
is written `0x<address>`, without objdump's symbol; an encoding objdump does not decode is `.insn`.

Usage: scripts/objdump_ref.py [--as riscv64-unknown-elf-as] [--objdump riscv64-unknown-elf-objdump]
Run it from the repository root; it rewrites tests/data/objdump_rv64.tsv and tests/data/objdump_rv32.tsv.
"""
import argparse
import random
import re
import subprocess
import tempfile
from pathlib import Path

SPECS_PER_INSTRUCTION = 12
BIASED_PER_INSTRUCTION = 12
RANDOM_PER_OPCODE = 200
RANDOM_COMPRESSED = 2000
TARGETS = {
    64: ("rv64gcv_zvbb", "lp64d"),
    32: ("rv32gcv_zvbb", "ilp32d"),
}
# Encodings worth checking that random ones rarely hit, as (size, encoding): c.unimp, c.nop, c.slli zero,0, c.jr zero
# (reserved), c.ebreak, nop, ecall, ebreak, the unimp alias of csrrw zero,cycle,zero, a fence, fence.tso, and
# fence.i.
EDGE_CASES = [(2, 0x0000), (2, 0x0001), (2, 0x0002), (2, 0x8002), (2, 0x9002), (4, 0x00000013), (4, 0x00000073),
              (4, 0x00100073), (4, 0xc0001073), (4, 0x0ff0000f), (4, 0x8330000f), (4, 0x0000100f)]
# Aliases objdump prints for particular operands: ret, jr, jalr, and mv through jalr and addi; c.addi and c.mv with
# a zero immediate or rd = rs; counters, floating-point CSRs, sfence.vma, and zext.b, andi with 255.
EDGE_CASES += [(4, w) for w in (
    0x00008067, 0x00050067, 0x000500e7, 0x00058567, 0x00050513, 0x00000513, 0x00051513, 0x0005051b,
    0xc0002573, 0xc0102573, 0xc0202573, 0xc8002573, 0xc8102573, 0xc8202573, 0xc0302573, 0xb0002573,
    0x00102573, 0x00202573, 0x00302573, 0x00159573, 0x00259573, 0x00359573, 0x00159073, 0x00259073,
    0x00359073, 0x0011d573, 0x0021d573, 0x0011d073, 0x0021d073, 0x00106573, 0x00206573, 0x00306573,
    0x00107573, 0x00a59073, 0x00a5a073, 0x00a5b073, 0x12a00073, 0x12050073, 0x12000073, 0x00002033,
    0x00003533, 0x00a02533, 0x0005a533, 0x40a00533, 0x40a0053b, 0xfff54513, 0x00153513, 0x0ff57513,
    0x0ff5f593, 0x0fe57513,
)] + [(2, w) for w in (0x0501, 0x852a, 0x8082, 0x8502, 0x9502, 0x2501, 0x4501, 0x6505)]
# Major opcodes of 32-bit instructions, bits 6..0; each ends in 11 and is not a longer encoding.
OPCODES = [0x03, 0x07, 0x0f, 0x13, 0x17, 0x1b, 0x23, 0x27, 0x2f, 0x33, 0x37, 0x3b, 0x43, 0x47, 0x4b, 0x4f, 0x53,
           0x57, 0x63, 0x67, 0x6f, 0x73]


def specs(isa_rs: str, xlen: int):
    """(name, mask, match) of every spec in src/isa.rs that applies to the XLEN: a table's specs run from its
    `pub static RV_ISA_SPECS_...` line to the next table's."""
    result = []
    chunks = re.split(r"^pub static (RV_ISA_SPECS_\w+)", isa_rs, flags=re.M)
    for table, body in zip(chunks[1::2], chunks[2::2]):
        if ("_32_" in table and xlen != 32) or ("_64_" in table and xlen != 64):
            continue
        for s in re.finditer(r'Spec::new\(\s*"([^"]+)",\s*(0x[0-9a-fA-F]+|\d+),\s*(0x[0-9a-fA-F]+|\d+),', body):
            result.append((s.group(1), int(s.group(2), 0), int(s.group(3), 0)))
    return result


def biased(rng: random.Random, size: int, word: int) -> int:
    """The encoding with each register field likely x0, x1, or another field's register, and its I-type immediate
    likely 0, 1, or -1: the operands objdump's aliases look for."""
    fields = [(7, 5), (15, 5), (20, 5)] if size == 4 else [(7, 5), (2, 5)]
    values = [(word >> lo) & 31 for lo, _ in fields]
    for i, (lo, width) in enumerate(fields):
        r = rng.random()
        v = 0 if r < 0.35 else 1 if r < 0.5 else rng.choice(values) if r < 0.75 else values[i]
        word = (word & ~(31 << lo)) | (v << lo)
    if size == 4:
        r = rng.random()
        if r < 0.5:
            imm = rng.choice([0, 1, 0xfff])
            word = (word & ~(0xfff << 20)) | (imm << 20)
    return word


def valid_length(word: int, size: int) -> bool:
    if size == 2:
        return word & 3 != 3
    return word & 3 == 3 and (word >> 2) & 7 != 7


def corpus(xlen: int, isa_rs: str):
    rng = random.Random(xlen)
    words = []
    for name, mask, match in specs(isa_rs, xlen):
        size = 2 if match & 3 != 3 else 4
        full = (1 << (8 * size)) - 1
        for _ in range(SPECS_PER_INSTRUCTION):
            words.append((size, (rng.getrandbits(8 * size) & ~mask & full) | match))
        for _ in range(BIASED_PER_INSTRUCTION):
            word = biased(rng, size, rng.getrandbits(8 * size) & full)
            words.append((size, (word & ~mask & full) | match))
    for opcode in OPCODES:
        for _ in range(RANDOM_PER_OPCODE):
            words.append((4, (rng.getrandbits(32) & ~0x7f) | opcode))
    words.extend(EDGE_CASES)
    for _ in range(RANDOM_COMPRESSED):
        w = rng.getrandbits(16)
        while w & 3 == 3:
            w = rng.getrandbits(16)
        words.append((2, w))
    assert all(valid_length(w, s) for s, w in words)
    return words


LINE = re.compile(r"^\s*([0-9a-f]+):\t([0-9a-f]+)\s+\t(.*)$")
TARGET = re.compile(r"\b([0-9a-f]+) <[^>]*>")


def disassemble(obj: Path, objdump: str, options: list[str]):
    out = subprocess.run([objdump, "-d", "-z", *options, str(obj)], check=True, capture_output=True, text=True).stdout
    texts = {}
    for line in out.splitlines():
        m = LINE.match(line)
        if not m:
            continue
        offset, text = int(m.group(1), 16), m.group(3)
        text = text.split("#")[0].rstrip()
        text = TARGET.sub(lambda t: f"0x{t.group(1)}", text)
        mnemonic, _, operands = text.partition("\t")
        text = f"{mnemonic} {operands}".rstrip()
        if mnemonic == ".insn":
            text = ".insn"
        texts[offset] = text
    return texts


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--as", dest="assembler", default="riscv64-unknown-elf-as")
    ap.add_argument("--objdump", default="riscv64-unknown-elf-objdump")
    args = ap.parse_args()
    isa_rs = Path("src/isa.rs").read_text()
    for xlen, (march, mabi) in TARGETS.items():
        words = corpus(xlen, isa_rs)
        with tempfile.TemporaryDirectory() as tmp:
            src, obj = Path(tmp) / "corpus.s", Path(tmp) / "corpus.o"
            src.write_text(".text\n" + "".join(f".insn {s}, 0x{w:0{2 * s}x}\n" for s, w in words))
            subprocess.run([args.assembler, f"-march={march}", f"-mabi={mabi}", "-o", str(obj), str(src)], check=True)
            canonical = disassemble(obj, args.objdump, ["-M", "no-aliases"])
            aliased = disassemble(obj, args.objdump, [])
        lines, offset = [], 0
        for size, word in words:
            lines.append(f"{offset:x}\t{word:0{2 * size}x}\t{canonical[offset]}\t{aliased[offset]}\n")
            offset += size
        path = Path(f"tests/data/objdump_rv{xlen}.tsv")
        path.write_text(f"# objdump {march}, from scripts/objdump_ref.py\n" + "".join(lines))
        print(f"{path}: {len(lines)} encodings")


if __name__ == "__main__":
    main()
