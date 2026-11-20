use crate::args::{Arg, Tag};
use crate::isa::Op;
use serde::{Serialize, Serializer};

const BRANCH_MASK: u8 = 0x01;
const IJ_MASK: u8 = 0x02;
const UJ_MASK: u8 = 0x04;
const CFC_MASK: u8 = 0x07; //0b111

/// Most operands any spec in the table lists (fmadd-class: rd, rs1, rs2, rs3, rm).
pub const MAX_ARGS: usize = 6;

/// A decoded instruction.
///
/// Flat and allocation-free: the instruction is its [`Op`], out of the spec table,
/// and the operands live in a fixed array. A consumer that indexes a whole binary
/// (tens of millions of these) pays ~70 bytes each and no heap traffic, where the
/// previous `String` + boxed `HashMap<String, Arg>` layout cost ~1 KB and seven
/// allocations per instruction. Everything the old accessors returned is derivable:
/// `get_imm`, `get_src`, `get_dst`, `to_string` and `to_canonical` keep their output.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Insn {
    pub len: u8,
    pub kind_mask: u8,
    nargs: u8,
    /// The XLEN it was decoded for, 32 or 64, to which a branch or jump target wraps.
    xlen: u8,
    pub offset: i32,
    pub raw: u32,
    /// The instruction. It serializes as its mnemonic, under `name`.
    #[serde(rename = "name")]
    pub op: Op,
    tags: [Tag; MAX_ARGS],
    args: [Arg; MAX_ARGS],
}

impl Serialize for Op {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

/// Whether `op` is a branch, a direct jump, or an indirect jump, as `Insn::kind_mask` holds it.
const fn kind_mask(op: Op) -> u8 {
    match op {
        Op::Beq | Op::Bne | Op::Blt | Op::Bge | Op::Bltu | Op::Bgeu | Op::CBeqz | Op::CBnez => {
            BRANCH_MASK
        }
        Op::Jal | Op::CJ | Op::CJal => IJ_MASK,
        Op::Jalr | Op::CJr | Op::CJalr => UJ_MASK,
        _ => 0,
    }
}

/// Helper: Get the size of the instruction in bytes
fn get_insn_size(raw: u32) -> u8 {
    if ((raw) & 0x03) < 0x03 {
        2
    } else {
        4
    }
}

impl Insn {
    /// Build from the operands a spec produced, in table order. `Arg::Nothing`
    /// placeholders (the `*lo` halves of split immediates) are dropped. A repeated
    /// tag replaces the earlier operand, which is what the previous map-based
    /// layout did on insert; `offset` is the signed value of the immediate, if any.
    pub fn new(raw: u32, op: Op, operands: &[(Arg, Tag)]) -> Self {
        let name = op.name();
        let mut tags = [Tag::None; MAX_ARGS];
        let mut args = [Arg::Nothing; MAX_ARGS];
        let mut nargs = 0usize;
        let mut offset = 0i32;
        for &(arg, tag) in operands {
            if matches!(arg, Arg::Nothing | Arg::Error) {
                continue;
            }
            if arg.is_imm() {
                offset = arg.get_val_signed_imm();
            }
            if let Some(i) = tags[..nargs]
                .iter()
                .position(|&t| t == tag && tag != Tag::None)
            {
                args[i] = arg;
                continue;
            }
            assert!(
                nargs < MAX_ARGS,
                "instruction {} has more than {} operands",
                name,
                MAX_ARGS
            );
            tags[nargs] = tag;
            args[nargs] = arg;
            nargs += 1;
        }

        Self::from_packed(raw, op, 64, nargs, offset, tags, args)
    }

    /// Build from all of a spec's operands at once, as a generated packer calls it, for an XLEN of `xlen`: `None` if
    /// any is `Arg::Error`, and otherwise what `new` builds from them. Inlined into each packer, where `N` and the tags are constants,
    /// it compiles to straight-line code, with no loop left. Unlike `new`, it does not merge a repeated tag: no
    /// spec has one, which `tests/operands.rs` checks.
    #[inline(always)]
    pub(crate) fn from_parts<const N: usize>(
        raw: u32,
        op: Op,
        xlen: u8,
        operands: [(Arg, Tag); N],
    ) -> Option<Self> {
        const {
            assert!(
                N <= MAX_ARGS,
                "a spec lists more operands than an Insn holds"
            )
        };
        let mut tags = [Tag::None; MAX_ARGS];
        let mut args = [Arg::Nothing; MAX_ARGS];
        let mut nargs = 0;
        let mut offset = 0i32;
        for (arg, tag) in operands {
            match arg {
                Arg::Error => return None,
                Arg::Nothing => continue,
                _ => {}
            }
            if arg.is_imm() {
                offset = arg.get_val_signed_imm();
            }
            tags[nargs] = tag;
            args[nargs] = arg;
            nargs += 1;
        }
        Some(Self::from_packed(raw, op, xlen, nargs, offset, tags, args))
    }

    /// Build from operands already in their arrays.
    #[inline(always)]
    fn from_packed(
        raw: u32,
        op: Op,
        xlen: u8,
        nargs: usize,
        offset: i32,
        tags: [Tag; MAX_ARGS],
        args: [Arg; MAX_ARGS],
    ) -> Self {
        Self {
            len: get_insn_size(raw),
            kind_mask: kind_mask(op),
            nargs: nargs as u8,
            xlen,
            offset,
            raw,
            op,
            tags,
            args,
        }
    }

    pub fn get_len(&self) -> u8 {
        self.len
    }

    pub fn get_xlen(&self) -> u8 {
        self.xlen
    }

    pub fn get_raw(&self) -> u32 {
        self.raw
    }

    pub fn get_name(&self) -> &'static str {
        self.op.name()
    }

    /// Every operand, in spec-table order, with its role tag.
    pub fn operands(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        (0..self.nargs as usize).map(move |i| (self.tags[i], self.args[i]))
    }

    pub fn get_src(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        self.operands().filter(|(_, a)| a.is_src())
    }

    pub fn get_dst(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        self.operands().filter(|(_, a)| a.is_dst())
    }

    pub fn get_flags(&self) -> impl Iterator<Item = (Tag, Arg)> + '_ {
        self.operands().filter(|(_, a)| a.is_flag())
    }

    pub fn get_imm(&self) -> Option<Arg> {
        self.operands().map(|(_, a)| a).find(|a| a.is_imm())
    }

    pub fn get_csr(&self) -> Option<Arg> {
        self.operands().map(|(_, a)| a).find(|a| a.is_csr())
    }

    /// The destination register, if the instruction has one.
    pub fn rd(&self) -> Option<u32> {
        self.reg_num(Tag::Rd)
    }

    /// The first source register, if the instruction has one.
    pub fn rs1(&self) -> Option<u32> {
        self.reg_num(Tag::Rs1)
    }

    /// The second source register, if the instruction has one.
    pub fn rs2(&self) -> Option<u32> {
        self.reg_num(Tag::Rs2)
    }

    /// The immediate, if the instruction has one, as the instruction uses it: assembled from its fields,
    /// sign-extended if it is signed (a branch's or jump's offset, `lui`'s and `auipc`'s already shifted left by
    /// 12), and zero-extended if not (a shift amount, a CSR instruction's `uimm`).
    pub fn imm(&self) -> Option<i64> {
        match self.arg(Tag::Imm)? {
            Arg::Imm(v) => Some(v.into()),
            Arg::UImm(v) => Some(v.into()),
            _ => None,
        }
    }

    /// The CSR number, if the instruction has one.
    pub fn csr(&self) -> Option<u32> {
        match self.arg(Tag::Csr)? {
            Arg::CSR(c) => Some(c),
            _ => None,
        }
    }

    /// The register the operand with role `tag` names, if the instruction has one.
    fn reg_num(&self, tag: Tag) -> Option<u32> {
        match self.arg(tag)? {
            Arg::DstReg(r) | Arg::SrcReg(r) => Some(r),
            _ => None,
        }
    }

    /// The operand with role `tag`, if the instruction has one.
    pub(crate) fn arg(&self, tag: Tag) -> Option<Arg> {
        self.operands().find(|&(t, _)| t == tag).map(|(_, a)| a)
    }

    pub fn is_branch(&self) -> bool {
        self.kind_mask & BRANCH_MASK != 0
    }

    pub fn is_direct_jump(&self) -> bool {
        self.kind_mask & IJ_MASK != 0
    }

    pub fn is_indirect_jump(&self) -> bool {
        self.kind_mask & UJ_MASK != 0
    }

    pub fn is_cfc_insn(&self) -> bool {
        self.kind_mask & CFC_MASK != 0
    }

    /// Sources ordered by tag name, the order the text forms print them in.
    fn sorted_src(&self) -> Vec<(Tag, Arg)> {
        let mut v: Vec<(Tag, Arg)> = self.get_src().collect();
        v.sort_by_key(|(t, _)| t.as_str());
        v
    }

    /// Helper: Format the instruction to a string representation
    pub fn to_string(&self) -> String {
        let mut operands = Vec::new();
        for (t, a) in self.get_dst() {
            operands.push(format!("{}{}", t.prefix(), a.to_string()));
        }
        for (t, a) in self.sorted_src() {
            operands.push(format!("{}{}", t.prefix(), a.to_string()));
        }
        if let Some(imm) = self.get_imm() {
            operands.push(imm.to_string());
        }
        if let Some(csr) = self.get_csr() {
            operands.push(format!("CSR#{}", csr.to_string()));
        }
        // flags are not printed, as before
        if operands.is_empty() {
            self.op.name().to_string()
        } else {
            format!("{} {}", self.op.name(), operands.join(", "))
        }
    }

    /// Helper: Format the instruction to a canonicalized string representation
    pub fn to_canonical(&self) -> String {
        let mut operands = Vec::new();
        for (t, a) in self.get_dst() {
            operands.push(format!(
                "{} {}{}",
                t.as_str().to_uppercase(),
                t.prefix(),
                a.to_string()
            ));
        }
        for (t, a) in self.sorted_src() {
            operands.push(format!(
                "{} {}{}",
                t.as_str().to_uppercase(),
                t.prefix(),
                a.to_string()
            ));
        }
        if let Some(imm) = self.get_imm() {
            operands.push(format!("IMM {}", imm.to_string()));
        }
        if let Some(csr) = self.get_csr() {
            operands.push(format!("CSR {}", csr.to_string()));
        }
        if operands.is_empty() {
            self.op.name().to_string()
        } else {
            format!("{} {}", self.op.name(), operands.join(" "))
        }
    }
}
