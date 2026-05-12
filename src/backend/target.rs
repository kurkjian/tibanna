use crate::{
    backend::allocator::Allocation,
    common::uf::UnionFind,
    ir::types::{TIRFunction, VirtualRegister},
};

pub trait Target {
    fn num_gp_registers(&self) -> usize;

    fn asm_header(&mut self);

    fn emit(
        &mut self,
        function: TIRFunction,
        alloc: Allocation,
        uf: &mut UnionFind<VirtualRegister>,
    );

    fn asm(self) -> String;
}
