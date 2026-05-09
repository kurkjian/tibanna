use std::collections::HashMap;

use crate::{
    backend::pass::OptimizationPass,
    ir::types::{Operation, TIRFunction, VirtualRegister},
};

#[derive(Debug, Clone, Copy)]
pub enum Const {
    Int(usize),
    Bool(bool),
}

impl From<Const> for Operation {
    fn from(c: Const) -> Self {
        match c {
            Const::Int(n) => Operation::ConstInt(n),
            Const::Bool(b) => Operation::ConstBool(b),
        }
    }
}

#[derive(Default)]
pub struct ConstantFolding;

impl OptimizationPass for ConstantFolding {
    fn run(&mut self, function: &mut TIRFunction) -> bool {
        let mut env: HashMap<VirtualRegister, Const> = HashMap::new();
        let mut changed = false;

        for block in &mut function.blocks {
            for instr in &mut block.instructions {
                if let Some(result) = fold_operation(&instr.op, &env) {
                    env.insert(instr.dest, result);
                    instr.op = result.into();
                    changed = true;
                } else if let Some(c) = extract_const(&instr.op) {
                    env.insert(instr.dest, c);
                }
            }
        }

        changed
    }
}

fn extract_const(op: &Operation) -> Option<Const> {
    match op {
        Operation::ConstInt(n) => Some(Const::Int(*n)),
        Operation::ConstBool(b) => Some(Const::Bool(*b)),
        _ => None,
    }
}

fn fold_operation(op: &Operation, env: &HashMap<VirtualRegister, Const>) -> Option<Const> {
    let (lhs, rhs) = match op {
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
        | Operation::Or(a, b) => (a, b),
        _ => return None,
    };

    let l = env.get(lhs)?;
    let r = env.get(rhs)?;

    eval_binop(op, l, r)
}

fn eval_binop(op: &Operation, l: &Const, r: &Const) -> Option<Const> {
    match (op, l, r) {
        (Operation::Add(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Int(a + b)),
        (Operation::Sub(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Int(a - b)),
        (Operation::Mul(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Int(a * b)),

        (Operation::Eq(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Bool(a == b)),
        (Operation::Lt(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Bool(a < b)),
        (Operation::Leq(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Bool(a <= b)),
        (Operation::Gt(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Bool(a > b)),
        (Operation::Geq(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Bool(a >= b)),
        (Operation::Neq(_, _), Const::Int(a), Const::Int(b)) => Some(Const::Bool(a != b)),

        (Operation::And(_, _), Const::Bool(a), Const::Bool(b)) => Some(Const::Bool(*a && *b)),
        (Operation::Or(_, _), Const::Bool(a), Const::Bool(b)) => Some(Const::Bool(*a || *b)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::types::{BlockId, Instruction, TIRBlock, Terminator};

    use super::*;

    #[test]
    fn test_folds_add() {
        let mut f = func(vec![block(vec![
            instr(0, Operation::ConstInt(2)),
            instr(1, Operation::ConstInt(3)),
            instr(2, Operation::Add(VirtualRegister(0), VirtualRegister(1))),
        ])]);

        let mut pass = ConstantFolding;
        pass.run(&mut f);

        match &f.blocks[0].instructions[2].op {
            Operation::ConstInt(v) => assert_eq!(*v, 5),
            _ => panic!("Expected ConstInt(5)"),
        }
    }

    #[test]
    fn test_folds_chained_operations() {
        let mut f = func(vec![block(vec![
            instr(0, Operation::ConstInt(2)),
            instr(1, Operation::ConstInt(3)),
            instr(2, Operation::Add(VirtualRegister(0), VirtualRegister(1))),
            instr(3, Operation::Mul(VirtualRegister(2), VirtualRegister(1))),
        ])]);

        let mut pass = ConstantFolding;
        pass.run(&mut f);

        assert_eq!(f.blocks[0].instructions[3].op, Operation::ConstInt(15))
    }

    #[test]
    fn test_folds_comparisons() {
        let mut f = func(vec![block(vec![
            instr(0, Operation::ConstInt(4)),
            instr(1, Operation::ConstInt(4)),
            instr(2, Operation::Eq(VirtualRegister(0), VirtualRegister(1))),
        ])]);

        let mut pass = ConstantFolding;
        pass.run(&mut f);

        assert_eq!(f.blocks[0].instructions[2].op, Operation::ConstBool(true))
    }

    #[test]
    fn test_folds_boolean_ops() {
        let mut f = func(vec![block(vec![
            instr(0, Operation::ConstBool(true)),
            instr(1, Operation::ConstBool(false)),
            instr(2, Operation::And(VirtualRegister(0), VirtualRegister(1))),
            instr(3, Operation::Or(VirtualRegister(0), VirtualRegister(1))),
        ])]);

        let mut pass = ConstantFolding;
        pass.run(&mut f);

        assert_eq!(f.blocks[0].instructions[2].op, Operation::ConstBool(false));
        assert_eq!(f.blocks[0].instructions[3].op, Operation::ConstBool(true));
    }

    #[test]
    fn test_mixed_types_dont_fold() {
        let mut f = func(vec![block(vec![
            instr(0, Operation::ConstInt(1)),
            instr(1, Operation::ConstBool(true)),
            instr(2, Operation::Add(VirtualRegister(0), VirtualRegister(1))), // invalid
        ])]);

        let mut pass = ConstantFolding;
        pass.run(&mut f);

        assert_eq!(
            f.blocks[0].instructions[2].op,
            Operation::Add(VirtualRegister(0), VirtualRegister(1))
        );
    }

    fn instr(dest: usize, op: Operation) -> Instruction {
        Instruction {
            dest: VirtualRegister(dest),
            op,
        }
    }

    fn block(instructions: Vec<Instruction>) -> TIRBlock {
        TIRBlock {
            label: BlockId(0),
            params: vec![],
            instructions,
            terminator: Terminator::Void,
        }
    }

    fn func(blocks: Vec<TIRBlock>) -> TIRFunction {
        TIRFunction {
            name: "test".to_string(),
            params: vec![],
            blocks,
        }
    }
}
