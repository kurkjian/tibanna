use crate::{backend::allocator::Allocator, ir::types::TIRFunction};

pub trait Target {
    fn num_gp_registers(&self) -> usize;

    fn asm_header(&mut self);

    fn emit(&mut self, function: TIRFunction, allocator: Allocator);

    fn asm(self) -> String;
}
