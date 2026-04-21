//! Control flow graph extraction and visualization.
//!
//! Builds a [`Cfg`] from a [`Tape`] by splitting at branch points, then
//! renders it as a Mermaid diagram for documentation and debugging.

use crate::tape::{Op, Tape};
use std::fmt::{self, Write as _};

/// Unique identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B{}", self.0)
    }
}

/// How a basic block ends.
#[derive(Debug)]
pub enum Terminator {
    /// Return a value from a register.
    Return { reg: u8 },
    /// Unconditional jump.
    Jump(BlockId),
    /// Conditional branch (on Nothing).
    BranchNothing {
        reg: u8,
        then_block: BlockId,
        else_block: BlockId,
    },
    /// Conditional branch (on true).
    BranchTrue {
        reg: u8,
        then_block: BlockId,
        else_block: BlockId,
    },
    /// Program halts with an error.
    Halt { message: String },
    /// Falls off the end of the tape.
    FallOff,
}

/// A basic block: a straight-line sequence of ops with no internal branches.
#[derive(Debug)]
pub struct BasicBlock {
    pub id: BlockId,
    /// Index range `[start, end)` into the original tape.
    pub start: usize,
    pub end: usize,
    /// How this block ends.
    pub terminator: Terminator,
}

impl BasicBlock {
    /// Number of operations (excluding the terminator instruction).
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether the block is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A control flow graph built from a [`Tape`].
#[derive(Debug)]
pub struct Cfg {
    pub entry: BlockId,
    pub blocks: Vec<BasicBlock>,
}

impl Cfg {
    /// Build a CFG from a tape.
    #[allow(clippy::too_many_lines)]
    pub fn from_tape(tape: &Tape) -> Self {
        if tape.is_empty() {
            return Self {
                entry: BlockId(0),
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    start: 0,
                    end: 0,
                    terminator: Terminator::FallOff,
                }],
            };
        }

        // Step 1: find leaders (start of basic blocks).
        // A leader is: the first op, any branch target, any op after a branch/jump/return.
        let mut is_leader = vec![false; tape.ops.len()];
        is_leader[0] = true;

        for (i, op) in tape.ops.iter().enumerate() {
            match op {
                Op::BranchNothing { target, .. }
                | Op::BranchTrue { target, .. }
                | Op::Jump { target } => {
                    if *target < tape.ops.len() {
                        is_leader[*target] = true;
                    }
                    if i + 1 < tape.ops.len() {
                        is_leader[i + 1] = true;
                    }
                }
                Op::Return { .. } | Op::Halt { .. } if i + 1 < tape.ops.len() => {
                    is_leader[i + 1] = true;
                }
                _ => {}
            }
        }

        // Step 2: map each op index to its block ID.
        let mut op_to_block = vec![0usize; tape.ops.len()];
        let mut block_id = 0;
        for (i, &leader) in is_leader.iter().enumerate() {
            if leader && i > 0 {
                block_id += 1;
            }
            op_to_block[i] = block_id;
        }
        let num_blocks = block_id + 1;

        // Step 3: build blocks.
        let mut blocks = Vec::with_capacity(num_blocks);
        let mut block_start = 0;

        for bid in 0..num_blocks {
            // Find the end of this block.
            let mut block_end = block_start;
            while block_end < tape.ops.len() && op_to_block[block_end] == bid {
                block_end += 1;
            }

            // The last op in the block determines the terminator.
            let last_idx = block_end - 1;
            let terminator = match &tape.ops[last_idx] {
                Op::Return { reg } => Terminator::Return { reg: *reg },
                Op::Halt { message } => Terminator::Halt {
                    message: message.clone(),
                },
                Op::Jump { target } => Terminator::Jump(BlockId(op_to_block[*target])),
                Op::BranchNothing { reg, target } => {
                    let then_block = BlockId(op_to_block[*target]);
                    let else_block = if block_end < tape.ops.len() {
                        BlockId(op_to_block[block_end])
                    } else {
                        then_block
                    };
                    Terminator::BranchNothing {
                        reg: *reg,
                        then_block,
                        else_block,
                    }
                }
                Op::BranchTrue { reg, target } => {
                    let then_block = BlockId(op_to_block[*target]);
                    let else_block = if block_end < tape.ops.len() {
                        BlockId(op_to_block[block_end])
                    } else {
                        then_block
                    };
                    Terminator::BranchTrue {
                        reg: *reg,
                        then_block,
                        else_block,
                    }
                }
                _ => {
                    // Fallthrough to next block.
                    if block_end < tape.ops.len() {
                        Terminator::Jump(BlockId(bid + 1))
                    } else {
                        Terminator::FallOff
                    }
                }
            };

            blocks.push(BasicBlock {
                id: BlockId(bid),
                start: block_start,
                end: block_end,
                terminator,
            });

            block_start = block_end;
        }

        Self {
            entry: BlockId(0),
            blocks,
        }
    }

    /// Render the CFG as a Mermaid flowchart.
    pub fn to_mermaid(&self, tape: &Tape) -> String {
        let mut out = String::from("graph TD\n");

        for block in &self.blocks {
            let label = block_label(tape, block);
            let _ = writeln!(out, "    {}[\"{label}\"]", block.id);

            match &block.terminator {
                Terminator::Return { reg } => {
                    let _ = writeln!(out, "    {} -->|return r{reg}| END([End])", block.id);
                }
                Terminator::Jump(target) => {
                    let _ = writeln!(out, "    {} --> {target}", block.id);
                }
                Terminator::BranchNothing {
                    reg,
                    then_block,
                    else_block,
                } => {
                    let _ = writeln!(out, "    {} -->|r{reg} = Nothing| {then_block}", block.id);
                    let _ = writeln!(
                        out,
                        "    {} -->|r{reg} \u{2260} Nothing| {else_block}",
                        block.id
                    );
                }
                Terminator::BranchTrue {
                    reg,
                    then_block,
                    else_block,
                } => {
                    let _ = writeln!(out, "    {} -->|r{reg} = true| {then_block}", block.id);
                    let _ = writeln!(
                        out,
                        "    {} -->|r{reg} \u{2260} true| {else_block}",
                        block.id
                    );
                }
                Terminator::Halt { .. } | Terminator::FallOff => {
                    let _ = writeln!(out, "    {} --> END([End])", block.id);
                }
            }
        }

        out
    }
}

/// Build a human-readable label for a basic block.
fn block_label(tape: &Tape, block: &BasicBlock) -> String {
    let mut parts = Vec::new();
    for i in block.start..block.end {
        let desc = match &tape.ops[i] {
            Op::Effect { instr, result } => format!("r{result} = {}", instr_name(instr)),
            Op::LoadConst { val, result } => format!("r{result} = {val}"),
            Op::Move { src, dst } => format!("r{dst} = r{src}"),
            Op::MakeList {
                start,
                count,
                result,
            } => format!("r{result} = list(r{start}..+{count})"),
            Op::UnwrapJust { src, result } => format!("r{result} = unwrap(r{src})"),
            Op::Return { reg } => format!("return r{reg}"),
            Op::BranchNothing { reg, .. } => format!("if r{reg} = Nothing"),
            Op::BranchTrue { reg, .. } => format!("if r{reg} = true"),
            Op::Jump { .. } => "jump".to_string(),
            Op::Halt { .. } => "halt".to_string(),
        };
        parts.push(desc);
    }
    if parts.is_empty() {
        "(empty)".to_string()
    } else {
        parts.join("\\n")
    }
}

/// Short name for an instruction (for labels).
fn instr_name(instr: &crate::instruction::Instruction) -> &'static str {
    use crate::instruction::Instruction;
    match instr {
        Instruction::Obj(o) => match o {
            crate::instruction::ObjInstruction::Create { .. } => "create_obj",
            crate::instruction::ObjInstruction::Destroy(_) => "destroy_obj",
            crate::instruction::ObjInstruction::Call { .. } => "call",
            crate::instruction::ObjInstruction::Receive => "receive",
        },
        Instruction::Introspect(i) => match i {
            crate::instruction::IntrospectInstruction::GetSelf => "get_self",
            crate::instruction::IntrospectInstruction::GetInput => "get_input",
            crate::instruction::IntrospectInstruction::GetCurrentMachine => "get_machine",
            crate::instruction::IntrospectInstruction::GetState => "get_state",
            crate::instruction::IntrospectInstruction::SetState(_) => "set_state",
            crate::instruction::IntrospectInstruction::GetSender => "get_sender",
        },
        Instruction::Tx(t) => match t {
            crate::instruction::TxInstruction::Begin(_) => "begin_tx",
            crate::instruction::TxInstruction::Commit(_) => "commit_tx",
            crate::instruction::TxInstruction::Abort(_) => "abort_tx",
        },
        Instruction::Pure(_) => "call_pure",
        Instruction::Reflect(_) => "reflect",
        Instruction::Reify(_) => "reify",
        Instruction::Machine(_) => "machine",
        Instruction::Controller(_) => "controller",
        Instruction::Fd(_) => "fd",
        Instruction::Nondet(_) => "nondet",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction;
    use crate::tape::TapeBuilder;
    use crate::types::Val;

    #[test]
    fn cfg_linear_tape() {
        let mut b = TapeBuilder::new();
        let _r0 = b.load_const(Val::Nat(1));
        let r1 = b.load_const(Val::Nat(2));
        b.ret(r1);
        let tape = b.build();

        let cfg = Cfg::from_tape(&tape);
        assert_eq!(cfg.blocks.len(), 1);
        assert!(matches!(
            cfg.blocks[0].terminator,
            Terminator::Return { reg: 1 }
        ));
    }

    #[test]
    fn cfg_with_branch() {
        let mut b = TapeBuilder::new();
        let r0 = b.load_const(Val::Nothing);
        let br = b.branch_nothing(r0);
        let r1 = b.load_const(Val::Nat(1));
        b.ret(r1);
        b.patch(br);
        let r2 = b.load_const(Val::Nat(2));
        b.ret(r2);
        let tape = b.build();

        let cfg = Cfg::from_tape(&tape);
        // Should have 3 blocks: [load+branch], [load+ret], [load+ret]
        assert_eq!(cfg.blocks.len(), 3);
        assert!(matches!(
            cfg.blocks[0].terminator,
            Terminator::BranchNothing { .. }
        ));
    }

    #[test]
    fn cfg_mermaid_output() {
        let mut b = TapeBuilder::new();
        let _r0 = b.effect(instruction::begin_tx(None));
        let r1 = b.effect(instruction::create_obj("ping", None));
        b.ret(r1);
        let tape = b.build();

        let cfg = Cfg::from_tape(&tape);
        let mermaid = cfg.to_mermaid(&tape);
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("begin_tx"));
        assert!(mermaid.contains("create_obj"));
        assert!(mermaid.contains("return"));
    }

    #[test]
    fn cfg_with_jump() {
        let mut b = TapeBuilder::new();
        let jmp = b.jump();
        let r0 = b.load_const(Val::Nat(999));
        b.ret(r0);
        b.patch(jmp);
        let r1 = b.load_const(Val::Nat(42));
        b.ret(r1);
        let tape = b.build();

        let cfg = Cfg::from_tape(&tape);
        assert_eq!(cfg.blocks.len(), 3);
        assert!(matches!(cfg.blocks[0].terminator, Terminator::Jump(_)));
    }

    #[test]
    fn cfg_compiled_program() {
        use crate::avm_do;
        use crate::itree::{ret, trigger};
        use crate::tape::compile::compile;

        let tree: crate::itree::ITree<crate::instruction::Instruction, Val> = avm_do! {
            let _tx <- trigger(instruction::begin_tx(None));
            let _obj <- trigger(instruction::create_obj("test", None));
            ret(Val::str("done"))
        };
        let tape = compile(tree);
        let cfg = Cfg::from_tape(&tape);

        // Linear program → 1 block
        assert_eq!(cfg.blocks.len(), 1);

        let mermaid = cfg.to_mermaid(&tape);
        assert!(mermaid.contains("begin_tx"));
        assert!(mermaid.contains("create_obj"));
    }

    #[test]
    fn empty_tape_cfg() {
        let tape = Tape {
            ops: vec![],
            register_count: 0,
        };
        let cfg = Cfg::from_tape(&tape);
        assert_eq!(cfg.blocks.len(), 1);
        assert!(matches!(cfg.blocks[0].terminator, Terminator::FallOff));
    }
}
