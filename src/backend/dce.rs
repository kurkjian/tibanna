use std::collections::{HashMap, HashSet};

use crate::{
    backend::pass::OptimizationPass,
    ir::types::{
        BlockId, Instruction, Operation, TIRBlock, TIRFunction, Terminator, VirtualRegister,
    },
};

#[derive(Default)]
pub struct DeadCodeElimination;

impl OptimizationPass for DeadCodeElimination {
    fn run(&mut self, function: &mut TIRFunction) -> bool {
        let mut changed = false;
        let passthrough_blocks = find_passthrough_blocks(function);
        for p in &passthrough_blocks {
            for block in &mut function.blocks {
                replace_target(block, p, &mut changed);
            }
        }

        function
            .blocks
            .retain_mut(|block| !passthrough_blocks.contains_key(&block.label));

        let used_registers = find_used_registers(function);
        for block in &mut function.blocks {
            block.instructions.retain(|instr| {
                let keep = used_registers.contains(&instr.dest) || unremovable(instr);
                changed |= !keep;
                keep
            });
        }

        changed
    }
}

// FIXME: This is only returning one node per iteration right now. This
// needs to be fixed to support transitive passthroughs in `run`
fn find_passthrough_blocks(function: &TIRFunction) -> HashMap<BlockId, BlockId> {
    let mut passthrough = HashMap::new();
    for block in &function.blocks {
        if block.instructions.is_empty()
            && let Some(target) = br_target(&block.terminator)
        {
            passthrough.insert(block.label, target);
            return passthrough;
        }
    }

    passthrough
}

fn br_target(terminator: &Terminator) -> Option<BlockId> {
    match terminator {
        Terminator::Branch { target, .. } => Some(*target),
        _ => None,
    }
}

fn replace_target(block: &mut TIRBlock, passthrough: (&BlockId, &BlockId), changed: &mut bool) {
    let (source, dest) = passthrough;
    match &mut block.terminator {
        Terminator::Branch { target, .. } if target == source => {
            *target = *dest;
            *changed = true;
        }
        Terminator::ConditionalBranch {
            then_target,
            else_target,
            ..
        } => {
            if then_target == source {
                *then_target = *dest;
                *changed = true;
            }
            if else_target == source {
                *else_target = *dest;
                *changed = true;
            }
        }
        _ => {}
    }
}

fn find_used_registers(function: &TIRFunction) -> HashSet<VirtualRegister> {
    let mut used = HashSet::new();
    for block in &function.blocks {
        for instr in &block.instructions {
            match &instr.op {
                Operation::ConstInt(_) | Operation::ConstBool(_) => {}
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
                    used.insert(*a);
                    used.insert(*b);
                }
                Operation::Call(_, args) => {
                    for arg in args {
                        used.insert(*arg);
                    }
                }
            }
        }

        match &block.terminator {
            Terminator::Void => {}
            Terminator::Exit(r) | Terminator::Return(r) => {
                used.insert(*r);
            }
            Terminator::Branch { params, .. } => {
                for r in params {
                    used.insert(*r);
                }
            }
            Terminator::ConditionalBranch {
                cond,
                then_params,
                else_params,
                ..
            } => {
                used.insert(*cond);
                for r in then_params {
                    used.insert(*r);
                }
                for r in else_params {
                    used.insert(*r);
                }
            }
        }
    }

    used
}

fn unremovable(instr: &Instruction) -> bool {
    matches!(instr.op, Operation::Call(_, _))
}

#[cfg(test)]
mod tests {
    use crate::{
        ir::types::{BlockId, TIRBlock},
        resolver::FunctionId,
    };

    use super::*;

    #[test]
    fn test_it_works() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                0,
                vec![Instruction {
                    dest: VirtualRegister(1),
                    op: Operation::ConstInt(2),
                }],
                Terminator::Void,
            )],
        };

        let mut pass = DeadCodeElimination;
        pass.run(&mut func);
        assert!(func.blocks[0].instructions.is_empty());
    }

    #[test]
    fn test_multipass() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                0,
                vec![
                    Instruction {
                        dest: VirtualRegister(1),
                        op: Operation::ConstInt(2),
                    },
                    Instruction {
                        dest: VirtualRegister(2),
                        op: Operation::ConstInt(3),
                    },
                    Instruction {
                        dest: VirtualRegister(3),
                        op: Operation::Add(VirtualRegister(1), VirtualRegister(2)),
                    },
                    Instruction {
                        dest: VirtualRegister(4),
                        op: Operation::Mul(VirtualRegister(3), VirtualRegister(2)),
                    },
                ],
                Terminator::Void,
            )],
        };

        let mut pass = DeadCodeElimination;

        loop {
            let changed = pass.run(&mut func);
            if !changed {
                break;
            }
        }
        assert!(func.blocks[0].instructions.is_empty());
    }

    #[test]
    fn test_keeps_return() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                0,
                vec![
                    Instruction {
                        dest: VirtualRegister(1),
                        op: Operation::ConstInt(2),
                    },
                    Instruction {
                        dest: VirtualRegister(2),
                        op: Operation::ConstInt(3),
                    },
                    Instruction {
                        dest: VirtualRegister(3),
                        op: Operation::Add(VirtualRegister(1), VirtualRegister(2)),
                    },
                ],
                Terminator::Return(VirtualRegister(3)),
            )],
        };

        let mut pass = DeadCodeElimination;
        pass.run(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    #[test]
    fn test_keeps_unremovable() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                0,
                vec![Instruction {
                    dest: VirtualRegister(1),
                    op: Operation::Call(
                        FunctionId {
                            name: "foo".into(),
                            id: 0,
                        },
                        vec![],
                    ),
                }],
                Terminator::Void,
            )],
        };

        let mut pass = DeadCodeElimination;
        pass.run(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_keeps_branch_params() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                0,
                vec![Instruction {
                    dest: VirtualRegister(1),
                    op: Operation::ConstInt(42),
                }],
                Terminator::Branch {
                    target: BlockId(1),
                    params: vec![VirtualRegister(1)],
                },
            )],
        };

        let mut pass = DeadCodeElimination;
        pass.run(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_keeps_branch_cond() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                0,
                vec![Instruction {
                    dest: VirtualRegister(1),
                    op: Operation::ConstBool(true),
                }],
                Terminator::ConditionalBranch {
                    cond: VirtualRegister(1),
                    then_target: BlockId(1),
                    then_params: vec![],
                    else_target: BlockId(2),
                    else_params: vec![],
                },
            )],
        };

        let mut pass = DeadCodeElimination;
        pass.run(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_removes_dead_blocks() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![
                block(
                    0,
                    vec![Instruction {
                        dest: VirtualRegister(0),
                        op: Operation::ConstInt(1),
                    }],
                    Terminator::ConditionalBranch {
                        cond: VirtualRegister(0),
                        then_target: BlockId(1),
                        then_params: vec![],
                        else_target: BlockId(2),
                        else_params: vec![],
                    },
                ),
                block(
                    1,
                    vec![Instruction {
                        dest: VirtualRegister(3),
                        op: Operation::ConstInt(1),
                    }],
                    Terminator::Branch {
                        target: BlockId(3),
                        params: vec![VirtualRegister(3)],
                    },
                ),
                block(
                    2,
                    vec![],
                    Terminator::Branch {
                        target: BlockId(3),
                        params: vec![VirtualRegister(3)],
                    },
                ),
                block(3, vec![], Terminator::Return(VirtualRegister(4))),
            ],
        };

        let mut pass = DeadCodeElimination;
        pass.run(&mut func);
        assert_eq!(func.blocks.len(), 3);
    }

    // TODO: test for passthrough chain in one iteration
    // PASSTHROUGH: {BlockId(3): BlockId(5), BlockId(5): BlockId(6)}

    fn block(id: usize, instructions: Vec<Instruction>, terminator: Terminator) -> TIRBlock {
        TIRBlock {
            label: BlockId(id),
            params: vec![],
            instructions,
            terminator,
        }
    }
}
