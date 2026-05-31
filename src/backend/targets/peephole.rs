// TODO: this should really be in an x86 mod or something

use crate::backend::targets::x86_64::{Instruction, Operand};

pub fn peephole(instructions: &mut Vec<Instruction>) {
    loop {
        let mut changed = false;

        changed |= remove_redundant_movs(instructions);
        changed |= remove_push_pop(instructions);

        if !changed {
            break;
        }
    }
}

fn remove_redundant_movs(instructions: &mut Vec<Instruction>) -> bool {
    let before = instructions.len();

    instructions.retain(|instr| {
        !matches!(
            instr,
            Instruction::Mov(
                Operand::Reg(r1),
                Operand::Reg(r2)
            ) if r1 == r2
        )
    });

    before != instructions.len()
}

fn remove_push_pop(instructions: &mut [Instruction]) -> bool {
    let before = instructions.len();
    let mut out = Vec::with_capacity(instructions.len());
    let mut iter = instructions.iter().peekable();

    while let Some(instr) = iter.next() {
        match (&instr, iter.peek()) {
            (Instruction::Push(r1), Some(Instruction::Pop(r2))) if r1 == r2 => {
                iter.next();
            }
            _ => out.push(instr),
        }
    }

    before != out.len()
}
