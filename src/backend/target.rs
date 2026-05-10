use std::collections::HashMap;

use crate::ir::types::{TIRFunction, VirtualRegister};

pub trait Target {
    fn num_gp_registers(&self) -> usize;

    fn asm_header(&mut self);

    fn emit(
        &mut self,
        function: TIRFunction,
        alloc: (HashMap<VirtualRegister, usize>, Vec<VirtualRegister>),
    );

    fn asm(self) -> String;
}
