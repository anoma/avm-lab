//! Tape IR — a flat, allocation-free program representation.
//!
//! A [`Tape`] is an alternative to [`ITree`](crate::itree::ITree) that stores
//! instructions as a contiguous `Vec<Op>` with a register file. This gives:
//!
//! - **Zero heap allocation** during execution (no boxed closures)
//! - **Sequential memory access** (cache-friendly)
//! - **Control flow analysis** via [`CFG`](crate::tape::cfg::Cfg) extraction
//!
//! Programs can be authored as `ITrees` (using `avm_do!`) and compiled to `Tapes`,
//! or built directly using the [`TapeBuilder`].

pub mod cfg;
pub mod compile;
pub mod interpret;

use crate::instruction::Instruction;
use crate::types::Val;

/// Register index (0–255).
pub type Reg = u8;

/// Maximum number of registers in the register file.
pub const MAX_REGS: usize = 256;

/// A single operation in the tape.
#[derive(Debug)]
pub enum Op {
    /// Emit an AVM instruction; store the response in `result`.
    Effect { instr: Instruction, result: Reg },

    /// Store a constant value into a register.
    LoadConst { val: Val, result: Reg },

    /// Copy one register to another.
    Move { src: Reg, dst: Reg },

    /// Build a list from a range of consecutive registers.
    MakeList { start: Reg, count: u8, result: Reg },

    /// Unwrap `Just(v)` → `v`, or leave as-is.
    UnwrapJust { src: Reg, result: Reg },

    /// Branch to `target` if the register holds `Nothing`.
    BranchNothing { reg: Reg, target: usize },

    /// Branch to `target` if the register holds `Bool(true)`.
    BranchTrue { reg: Reg, target: usize },

    /// Unconditional jump.
    Jump { target: usize },

    /// Return the value in the register.
    Return { reg: Reg },

    /// Halt with an error message.
    Halt { message: String },
}

/// A flat, contiguous program.
#[derive(Debug)]
pub struct Tape {
    pub ops: Vec<Op>,
    /// Number of registers actually used (for diagnostics).
    pub register_count: u8,
}

impl Tape {
    /// Number of operations in the tape.
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Whether the tape is empty.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// Builder for constructing tapes incrementally.
pub struct TapeBuilder {
    ops: Vec<Op>,
    next_reg: u8,
}

impl TapeBuilder {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            next_reg: 0,
        }
    }

    /// Allocate a fresh register.
    pub fn alloc_reg(&mut self) -> Reg {
        let r = self.next_reg;
        self.next_reg = self.next_reg.checked_add(1).expect("register overflow");
        r
    }

    /// Emit an instruction effect, returning the result register.
    pub fn effect(&mut self, instr: Instruction) -> Reg {
        let r = self.alloc_reg();
        self.ops.push(Op::Effect { instr, result: r });
        r
    }

    /// Load a constant into a fresh register.
    pub fn load_const(&mut self, val: Val) -> Reg {
        let r = self.alloc_reg();
        self.ops.push(Op::LoadConst { val, result: r });
        r
    }

    /// Copy a register.
    pub fn mov(&mut self, src: Reg) -> Reg {
        let r = self.alloc_reg();
        self.ops.push(Op::Move { src, dst: r });
        r
    }

    /// Build a list from consecutive registers `[start, start+count)`.
    pub fn make_list(&mut self, start: Reg, count: u8) -> Reg {
        let r = self.alloc_reg();
        self.ops.push(Op::MakeList {
            start,
            count,
            result: r,
        });
        r
    }

    /// Unwrap a `Just` value.
    pub fn unwrap_just(&mut self, src: Reg) -> Reg {
        let r = self.alloc_reg();
        self.ops.push(Op::UnwrapJust { src, result: r });
        r
    }

    /// Emit a conditional branch; returns the index to patch later.
    pub fn branch_nothing(&mut self, reg: Reg) -> usize {
        let idx = self.ops.len();
        self.ops.push(Op::BranchNothing { reg, target: 0 });
        idx
    }

    /// Emit a conditional branch on true; returns the index to patch.
    pub fn branch_true(&mut self, reg: Reg) -> usize {
        let idx = self.ops.len();
        self.ops.push(Op::BranchTrue { reg, target: 0 });
        idx
    }

    /// Emit an unconditional jump; returns the index to patch.
    pub fn jump(&mut self) -> usize {
        let idx = self.ops.len();
        self.ops.push(Op::Jump { target: 0 });
        idx
    }

    /// Patch a branch/jump target to point at the current position.
    pub fn patch(&mut self, branch_idx: usize) {
        let target = self.ops.len();
        match &mut self.ops[branch_idx] {
            Op::BranchNothing { target: t, .. }
            | Op::BranchTrue { target: t, .. }
            | Op::Jump { target: t } => *t = target,
            _ => panic!("patch called on non-branch op"),
        }
    }

    /// Emit a return.
    pub fn ret(&mut self, reg: Reg) {
        self.ops.push(Op::Return { reg });
    }

    /// Emit a halt.
    pub fn halt(&mut self, message: impl Into<String>) {
        self.ops.push(Op::Halt {
            message: message.into(),
        });
    }

    /// Current position (for jump targets).
    pub fn pos(&self) -> usize {
        self.ops.len()
    }

    /// Finalize into a Tape.
    pub fn build(self) -> Tape {
        Tape {
            ops: self.ops,
            register_count: self.next_reg,
        }
    }
}

impl Default for TapeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction;

    #[test]
    fn builder_basic() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::Nat(42));
        b.ret(r0);
        let tape = b.build();
        assert_eq!(tape.len(), 2);
        assert_eq!(tape.register_count, 1);
    }

    #[test]
    fn builder_branch_and_patch() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::Nothing);
        let br = b.branch_nothing(r0);
        let r1 = b.load_const(Val::Nat(1));
        b.ret(r1);
        b.patch(br); // branch target is here
        let r2 = b.load_const(Val::Nat(2));
        b.ret(r2);
        let tape = b.build();
        assert_eq!(tape.len(), 6);
        // The branch target should point to op index 4
        assert!(matches!(tape.ops[1], Op::BranchNothing { target: 4, .. }));
    }

    #[test]
    fn builder_effect() {
        let mut b = TapeBuilder::new();
        let r = b.effect(instruction::get_self());
        b.ret(r);
        let tape = b.build();
        assert_eq!(tape.len(), 2);
        assert!(matches!(tape.ops[0], Op::Effect { result: 0, .. }));
    }
}
