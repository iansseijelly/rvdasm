//! Assembly text for a decoded instruction, as GNU objdump prints it with `-M no-aliases`: ABI register names, a
//! memory operand as `imm(rs1)`, operands in assembly order, CSR names, rounding modes, a vector type as
//! `e64,m8,ta,ma`, and `v0.t` on a masked vector instruction. A branch or jump target is its absolute address, as
//! `0x80000050`, where objdump follows the address with a symbol. tests/objdump.rs checks the text against objdump's
//! for a corpus of encodings that scripts/objdump_ref.py writes.

use crate::args::{Arg, Tag};
use crate::insn::Insn;
use crate::isa_consts::csr_name;

const X: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];
const F: [&str; 32] = [
    "ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7", "fs0", "fs1", "fa0", "fa1", "fa2",
    "fa3", "fa4", "fa5", "fa6", "fa7", "fs2", "fs3", "fs4", "fs5", "fs6", "fs7", "fs8", "fs9",
    "fs10", "fs11", "ft8", "ft9", "ft10", "ft11",
];
/// Rounding modes by `rm`; 7, dynamic, is not printed.
const ROUNDING: [&str; 8] = [
    "rne", "rtz", "rdn", "rup", "rmm", "unknown", "unknown", "dyn",
];
/// Vector multiply-adds, whose multiplier, vs1 or a scalar, comes before vs2.
const MULTIPLY_ADD: &[&str] = &[
    "vmacc.vv",
    "vmacc.vx",
    "vnmsac.vv",
    "vnmsac.vx",
    "vmadd.vv",
    "vmadd.vx",
    "vnmsub.vv",
    "vnmsub.vx",
    "vwmacc.vv",
    "vwmacc.vx",
    "vwmaccu.vv",
    "vwmaccu.vx",
    "vwmaccsu.vv",
    "vwmaccsu.vx",
    "vwmaccus.vx",
    "vfmacc.vv",
    "vfmacc.vf",
    "vfnmacc.vv",
    "vfnmacc.vf",
    "vfmsac.vv",
    "vfmsac.vf",
    "vfnmsac.vv",
    "vfnmsac.vf",
    "vfmadd.vv",
    "vfmadd.vf",
    "vfnmadd.vv",
    "vfnmadd.vf",
    "vfmsub.vv",
    "vfmsub.vf",
    "vfnmsub.vv",
    "vfnmsub.vf",
    "vfwmacc.vv",
    "vfwmacc.vf",
    "vfwnmacc.vv",
    "vfwnmacc.vf",
    "vfwmsac.vv",
    "vfwmsac.vf",
    "vfwnmsac.vv",
    "vfwnmsac.vf",
];

fn reg_name(tag: Tag, n: u32) -> String {
    match tag {
        Tag::Rd | Tag::Rs1 | Tag::Rs2 | Tag::Rs3 => X[n as usize].to_string(),
        Tag::Fd | Tag::Fs1 | Tag::Fs2 | Tag::Fs3 => F[n as usize].to_string(),
        _ => format!("v{n}"),
    }
}

/// A vector type, or its value when it is not one, as a reserved field or encoding makes it.
fn vtype(v: u32) -> String {
    const SEW: [Option<&str>; 8] = [
        Some("e8"),
        Some("e16"),
        Some("e32"),
        Some("e64"),
        None,
        None,
        None,
        None,
    ];
    const LMUL: [Option<&str>; 8] = [
        Some("m1"),
        Some("m2"),
        Some("m4"),
        Some("m8"),
        None,
        Some("mf8"),
        Some("mf4"),
        Some("mf2"),
    ];
    match (SEW[(v >> 3 & 7) as usize], LMUL[(v & 7) as usize], v >> 8) {
        (Some(sew), Some(lmul), 0) => {
            let ta = if v >> 6 & 1 == 1 { "ta" } else { "tu" };
            let ma = if v >> 7 & 1 == 1 { "ma" } else { "mu" };
            format!("{sew},{lmul},{ta},{ma}")
        }
        _ => v.to_string(),
    }
}

/// A fence's predecessor or successor set, as letters from `iorw`, or `unknown` when empty.
fn fence_set(v: u32) -> String {
    let set: String = "iorw"
        .chars()
        .enumerate()
        .filter(|(i, _)| v >> (3 - i) & 1 == 1)
        .map(|(_, c)| c)
        .collect();
    if set.is_empty() {
        "unknown".to_string()
    } else {
        set
    }
}

impl Insn {
    /// The instruction as GNU objdump prints it with `-M no-aliases`, as at `pc`, to which a branch or jump target is
    /// relative.
    pub fn asm_no_aliases(&self, pc: u64) -> String {
        let operands = self.asm_operands(pc);
        let name = self.asm_mnemonic();
        if operands.is_empty() {
            name
        } else {
            format!("{name} {}", operands.join(","))
        }
    }

    fn arg(&self, tag: Tag) -> Option<Arg> {
        self.operands().find(|(t, _)| *t == tag).map(|(_, a)| a)
    }

    fn reg(&self, tag: Tag) -> Option<String> {
        self.arg(tag).map(|a| reg_name(tag, a.get_val()))
    }

    /// The first register present among `tags`.
    fn reg_of(&self, tags: &[Tag]) -> Option<String> {
        tags.iter().find_map(|&t| self.reg(t))
    }

    fn imm(&self) -> i64 {
        match self.arg(Tag::Imm) {
            Some(Arg::Imm(v)) => v as i64,
            Some(Arg::UImm(v)) => v as i64,
            _ => 0,
        }
    }

    fn target(&self, pc: u64) -> String {
        let target = pc.wrapping_add(self.imm() as u64);
        let target = if self.get_xlen() == 32 {
            target & 0xffff_ffff
        } else {
            target
        };
        format!("0x{target:x}")
    }

    /// The mnemonic, with an atomic's ordering bits as `.aq`, `.rl`, or `.aqrl`.
    fn asm_mnemonic(&self) -> String {
        let set = |tag| matches!(self.arg(tag), Some(Arg::Flag(1)));
        match (set(Tag::Aq), set(Tag::Rl)) {
            (true, true) => format!("{}.aqrl", self.name),
            (true, false) => format!("{}.aq", self.name),
            (false, true) => format!("{}.rl", self.name),
            (false, false) => self.name.to_string(),
        }
    }

    fn asm_operands(&self, pc: u64) -> Vec<String> {
        let name = self.name;
        let rd = || self.reg_of(&[Tag::Rd, Tag::Fd, Tag::Vd]);
        let rs1 = || self.reg_of(&[Tag::Rs1, Tag::Fs1]).unwrap_or_default();
        let rs2 = || self.reg_of(&[Tag::Rs2, Tag::Fs2]);
        let imm = self.imm();
        let mem = |base: String| format!("{imm}({base})");
        let hex = |v: i64| format!("0x{v:x}");
        let all = |v: Vec<Option<String>>| v.into_iter().flatten().collect::<Vec<_>>();

        if name.starts_with("c.") {
            return match name {
                "c.j" | "c.jal" => vec![self.target(pc)],
                "c.jr" | "c.jalr" => vec![rs1()],
                "c.beqz" | "c.bnez" => vec![rs1(), self.target(pc)],
                "c.lw" | "c.ld" | "c.flw" | "c.fld" => all(vec![rd(), Some(mem(rs1()))]),
                "c.lwsp" | "c.ldsp" | "c.flwsp" | "c.fldsp" => {
                    all(vec![rd(), Some(mem("sp".into()))])
                }
                "c.sw" | "c.sd" | "c.fsw" | "c.fsd" => all(vec![rs2(), Some(mem(rs1()))]),
                "c.swsp" | "c.sdsp" | "c.fswsp" | "c.fsdsp" => {
                    all(vec![rs2(), Some(mem("sp".into()))])
                }
                "c.lui" => all(vec![rd(), Some(hex(imm >> 12 & 0xfffff))]),
                "c.addi16sp" => vec!["sp".into(), imm.to_string()],
                "c.addi4spn" => all(vec![rd(), Some("sp".into()), Some(imm.to_string())]),
                "c.slli" | "c.srli" | "c.srai" => all(vec![rd(), Some(hex(imm))]),
                "c.nop" | "c.ebreak" | "c.unimp" => vec![],
                "c.mv" | "c.add" | "c.sub" | "c.xor" | "c.or" | "c.and" | "c.subw" | "c.addw" => {
                    all(vec![rd(), rs2()])
                }
                _ => all(vec![rd(), Some(imm.to_string())]),
            };
        }

        if name.starts_with("vset") {
            let vt = self
                .arg(Tag::Vtype)
                .map(|a| vtype(a.get_val_signed_imm() as u32));
            return match name {
                "vsetvli" => all(vec![rd(), Some(rs1()), vt]),
                "vsetivli" => all(vec![rd(), Some(imm.to_string()), vt]),
                _ => all(vec![rd(), Some(rs1()), rs2()]),
            };
        }

        let vector = self
            .operands()
            .any(|(t, _)| matches!(t, Tag::Vd | Tag::Vs1 | Tag::Vs2 | Tag::Vs3));
        let masked = matches!(self.arg(Tag::Vm), Some(Arg::Flag(0)));
        let opcode = self.raw & 0x7f;
        if vector && (opcode == 0x07 || opcode == 0x27) {
            // A load or store: its data, its base, then a stride or an index.
            let mut v = all(vec![
                self.reg_of(&[Tag::Vd, Tag::Vs3]),
                Some(format!("({})", rs1())),
            ]);
            v.extend(self.reg_of(&[Tag::Rs2, Tag::Vs2]));
            if masked {
                v.push("v0.t".into());
            }
            return v;
        }
        if vector {
            let scalar = self
                .reg_of(&[Tag::Vs1, Tag::Rs1, Tag::Fs1])
                .or(self.arg(Tag::Imm).map(|_| imm.to_string()));
            let vs2 = self.reg(Tag::Vs2);
            let mut v = all(vec![self.reg_of(&[Tag::Vd, Tag::Rd, Tag::Fd])]);
            if MULTIPLY_ADD.contains(&name) {
                v.extend(scalar.into_iter().chain(vs2));
            } else {
                v.extend(vs2.into_iter().chain(scalar));
            }
            // An add or subtract with carry or a merge reads v0 itself; any other instruction is masked by it.
            if [".vvm", ".vxm", ".vim", ".vfm"]
                .iter()
                .any(|s| name.ends_with(s))
            {
                v.push("v0".into());
            } else if masked {
                v.push("v0.t".into());
            }
            return v;
        }

        match opcode {
            0x03 | 0x07 => all(vec![rd(), Some(mem(rs1()))]),
            0x23 | 0x27 => all(vec![rs2(), Some(mem(rs1()))]),
            0x0f if name == "fence" => {
                let set = |tag| fence_set(self.arg(tag).map_or(0, |a| a.get_val()));
                vec![set(Tag::Pred), set(Tag::Succ)]
            }
            0x0f => vec![],
            0x13 | 0x1b if ["slli", "srli", "srai", "slliw", "srliw", "sraiw"].contains(&name) => {
                all(vec![rd(), Some(rs1()), Some(hex(imm))])
            }
            0x13 | 0x1b => all(vec![rd(), Some(rs1()), Some(imm.to_string())]),
            0x17 | 0x37 => all(vec![rd(), Some(hex(imm >> 12 & 0xfffff))]),
            0x2f if name.starts_with("lr.") => all(vec![rd(), Some(format!("({})", rs1()))]),
            0x2f => all(vec![rd(), rs2(), Some(format!("({})", rs1()))]),
            0x63 => all(vec![Some(rs1()), rs2(), Some(self.target(pc))]),
            0x67 => all(vec![rd(), Some(mem(rs1()))]),
            0x6f => all(vec![rd(), Some(self.target(pc))]),
            0x73 if name.starts_with("csrr") => {
                let csr = self.arg(Tag::Csr).map_or(0, |a| a.get_val());
                let csr = csr_name(csr).map_or_else(|| hex(csr as i64), str::to_string);
                let source = if name.ends_with('i') {
                    imm.to_string()
                } else {
                    rs1()
                };
                all(vec![rd(), Some(csr), Some(source)])
            }
            0x73 => all(vec![self.reg(Tag::Rs1), rs2()]),
            _ => {
                // Register-register and floating-point instructions: rd, then rs1, rs2, rs3, then a rounding mode.
                let mut v = all(vec![
                    rd(),
                    self.reg_of(&[Tag::Rs1, Tag::Fs1]),
                    rs2(),
                    self.reg_of(&[Tag::Rs3, Tag::Fs3]),
                ]);
                if let Some(Arg::Flag(rm)) = self.arg(Tag::Rm) {
                    if rm != 7 {
                        v.push(ROUNDING[rm as usize].into());
                    }
                }
                v
            }
        }
    }
}
