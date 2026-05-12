use std::collections::{HashMap, HashSet};

use crate::ir::types::{
    BlockId, Instruction, Operation, TIRBlock, TIRFunction, Terminator, VirtualRegister,
};

#[derive(Debug)]
pub struct UseDef {
    pub use_set: HashSet<VirtualRegister>,
    pub def_set: HashSet<VirtualRegister>,
}

#[derive(Debug)]
pub struct BlockLiveness {
    pub use_def: UseDef,
    pub live_in: HashSet<VirtualRegister>,
    pub live_out: HashSet<VirtualRegister>,
}

#[derive(Debug)]
pub struct Liveness {
    pub blocks: HashMap<BlockId, BlockLiveness>,
}

impl Instruction {
    pub fn uses(&self) -> Vec<VirtualRegister> {
        match &self.op {
            Operation::ConstInt(_) | Operation::ConstBool(_) => {
                vec![]
            }
            Operation::Add(a, b)
            | Operation::Sub(a, b)
            | Operation::Mul(a, b)
            | Operation::Eq(a, b)
            | Operation::Lt(a, b)
            | Operation::Leq(a, b)
            | Operation::Gt(a, b)
            | Operation::Geq(a, b)
            | Operation::Neq(a, b)
            | Operation::And(a, b)
            | Operation::Or(a, b) => {
                vec![*a, *b]
            }
            Operation::Call(_, args) => args.clone(),
        }
    }

    pub fn def(&self) -> VirtualRegister {
        self.dest
    }
}

impl Terminator {
    pub fn uses(&self) -> Vec<VirtualRegister> {
        match self {
            Terminator::Void => vec![],
            Terminator::Exit(v) | Terminator::Return(v) => {
                vec![*v]
            }
            Terminator::Branch { params, .. } => params.clone(),
            Terminator::ConditionalBranch {
                cond,
                then_params,
                else_params,
                ..
            } => {
                let mut regs = vec![*cond];
                regs.extend(then_params.clone());
                regs.extend(else_params.clone());
                regs
            }
        }
    }

    pub fn successors(&self) -> HashSet<BlockId> {
        match self {
            Terminator::Void | Terminator::Exit(_) | Terminator::Return(_) => HashSet::new(),
            Terminator::Branch { target, .. } => HashSet::from([*target]),
            Terminator::ConditionalBranch {
                then_target,
                else_target,
                ..
            } => HashSet::from([*then_target, *else_target]),
        }
    }

    pub fn edge_uses(&self) -> Vec<(BlockId, Vec<VirtualRegister>)> {
        match self {
            Terminator::Void | Terminator::Exit(_) | Terminator::Return(_) => vec![],
            Terminator::Branch { target, params } => {
                vec![(*target, params.clone())]
            }
            Terminator::ConditionalBranch {
                cond: _,
                then_target,
                then_params,
                else_target,
                else_params,
            } => {
                vec![
                    (*then_target, then_params.clone()),
                    (*else_target, else_params.clone()),
                ]
            }
        }
    }
}

fn find_use_def(block: &TIRBlock) -> UseDef {
    let mut use_set = HashSet::new();
    let mut def_set = HashSet::new();

    for param in &block.params {
        def_set.insert(*param);
    }

    for instr in &block.instructions {
        for vreg in instr.uses() {
            if !def_set.contains(&vreg) {
                use_set.insert(vreg);
            }
        }

        def_set.insert(instr.def());
    }

    for used in block.terminator.uses() {
        if !def_set.contains(&used) {
            use_set.insert(used);
        }
    }

    UseDef { use_set, def_set }
}

pub fn liveness(function: &TIRFunction) -> Liveness {
    let mut blocks = HashMap::new();
    for block in &function.blocks {
        let use_def = find_use_def(block);

        blocks.insert(
            block.label,
            BlockLiveness {
                use_def,
                live_in: HashSet::new(),
                live_out: HashSet::new(),
            },
        );
    }

    loop {
        let mut changed = false;
        for block in function.blocks.iter().rev() {
            let old_in = blocks[&block.label].live_in.clone();
            let old_out = blocks[&block.label].live_out.clone();

            // Block params are treated as defs at the start of blocks
            //
            // live_out' = \union edge_live
            // edge_live = edge_args \union (live_in - params)
            let mut new_out = HashSet::new();
            for (succ, edge_args) in block.terminator.edge_uses() {
                new_out.extend(edge_args);
                let succ_block = function
                    .blocks
                    .iter()
                    .find(|b| b.label == succ)
                    .expect("successor block must exist");
                for v in &blocks[&succ].live_in {
                    if !succ_block.params.contains(v) {
                        new_out.insert(*v);
                    }
                }
            }

            // live_in' = use_set \union (live_out - def_set)
            let mut new_in = blocks[&block.label].use_def.use_set.clone();
            for vreg in &new_out {
                if !blocks[&block.label].use_def.def_set.contains(vreg) {
                    new_in.insert(*vreg);
                }
            }

            if old_in != new_in || old_out != new_out {
                changed = true;
            }

            let b = blocks
                .get_mut(&block.label)
                .expect("Block liveness must exist");
            b.live_in = new_in;
            b.live_out = new_out;
        }

        if !changed {
            break;
        }
    }

    Liveness { blocks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::types::{
        BlockId, Instruction, Operation, TIRBlock, TIRFunction, Terminator, VirtualRegister,
    };

    fn block(
        label: usize,
        params: Vec<VirtualRegister>,
        instructions: Vec<Instruction>,
        terminator: Terminator,
    ) -> TIRBlock {
        TIRBlock {
            label: BlockId(label),
            params,
            instructions,
            terminator,
        }
    }

    #[test]
    fn test_liveness_single_block() {
        // block0:
        //   _v1 = const 10
        //   _v2 = add _v1, _v1
        //   return _v2

        let block0 = block(
            0,
            vec![],
            vec![
                instr(1, Operation::ConstInt(10)),
                instr(2, Operation::Add(vr(1), vr(3))),
            ],
            Terminator::Return(vr(2)),
        );
        let func = TIRFunction {
            name: "test".to_string(),
            params: vec![],
            blocks: vec![block0],
        };

        let live = liveness(&func);
        let b0 = &live.blocks[&BlockId(0)];
        assert_eq!(b0.live_in, HashSet::from([vr(3)]));
        assert!(b0.live_out.is_empty());
    }

    #[test]
    fn test_liveness_branch() {
        // block0:
        //   _v1 = const 1
        //   br block1(_v1)
        //
        // block1(_v2):
        //   return _v2

        let block0 = block(
            0,
            vec![],
            vec![instr(1, Operation::ConstInt(1))],
            Terminator::Branch {
                target: BlockId(1),
                params: vec![vr(1)],
            },
        );
        let block1 = block(1, vec![vr(2)], vec![], Terminator::Return(vr(2)));
        let func = TIRFunction {
            name: "test".to_string(),
            params: vec![],
            blocks: vec![block0, block1],
        };

        let live = liveness(&func);
        let b0 = &live.blocks[&BlockId(0)];
        let b1 = &live.blocks[&BlockId(1)];
        assert!(b0.live_in.is_empty());
        assert_eq!(b0.live_out, HashSet::from([vr(1)]));
        assert!(b1.live_in.is_empty());
        assert!(b1.live_out.is_empty());
    }

    fn vr(n: usize) -> VirtualRegister {
        VirtualRegister(n)
    }

    fn instr(dest: usize, op: Operation) -> Instruction {
        Instruction { dest: vr(dest), op }
    }
}
