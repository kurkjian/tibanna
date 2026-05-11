use crate::{backend::regalloc::Allocation, ir::types::TIRFunction};

pub trait Target {
    fn num_gp_registers(&self) -> usize;

    fn asm_header(&mut self);

    fn emit(&mut self, function: TIRFunction, alloc: Allocation);

    fn asm(self) -> String;
}
