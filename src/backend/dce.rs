use std::collections::HashSet;

use crate::ir::types::{Instruction, Operation, TIRFunction, Terminator, VirtualRegister};

pub fn dce(function: &mut TIRFunction) {
    loop {
        let used_registers = find_used_registers(function);
        let mut changed = false;

        for block in &mut function.blocks {
            block.instructions.retain(|instr| {
                let keep = used_registers.contains(&instr.dest) || unremovable(instr);
                changed |= !keep;
                keep
            });
        }

        if !changed {
            break;
        }
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
    use crate::ir::types::{BlockId, TIRBlock};

    use super::*;

    #[test]
    fn test_it_works() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                vec![Instruction {
                    dest: VirtualRegister(1),
                    op: Operation::ConstInt(2),
                }],
                Terminator::Void,
            )],
        };

        dce(&mut func);
        assert!(func.blocks[0].instructions.is_empty());
    }

    #[test]
    fn test_multipass() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
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

        dce(&mut func);
        assert!(func.blocks[0].instructions.is_empty());
    }

    #[test]
    fn test_keeps_return() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
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

        dce(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    #[test]
    fn test_keeps_unremovable() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
                vec![Instruction {
                    dest: VirtualRegister(1),
                    op: Operation::Call("foo".into(), vec![]),
                }],
                Terminator::Void,
            )],
        };

        dce(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_keeps_branch_params() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
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

        dce(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_keeps_branch_cond() {
        let mut func = TIRFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![block(
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

        dce(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    fn block(instructions: Vec<Instruction>, terminator: Terminator) -> TIRBlock {
        TIRBlock {
            label: BlockId(0),
            params: vec![],
            instructions,
            terminator,
        }
    }
}
