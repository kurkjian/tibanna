use crate::ir::types::TIRFunction;

pub trait OptimizationPass {
    /// Runs a single iteration of the optimization pass on the given function.
    /// Returns `true` if the function was modified, `false` otherwise.
    fn run(&mut self, function: &mut TIRFunction) -> bool;
}
