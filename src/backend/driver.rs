use crate::{
    backend::{
        liveness::liveness,
        pass::OptimizationPass,
        regalloc::{allocate_registers, create_interference_graph},
    },
    ir::types::TIRFunction,
};

// FIXME: temp
const NUM_REGISTERS: usize = 7;

pub struct Driver {
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl Driver {
    pub fn new(passes: Vec<Box<dyn OptimizationPass>>) -> Self {
        Self { passes }
    }

    pub fn run(&mut self, functions: Vec<TIRFunction>) {
        for mut function in functions {
            loop {
                let mut changed = false;
                for pass in &mut self.passes {
                    changed |= pass.run(&mut function);
                }
                if !changed {
                    break;
                }
            }

            let liveness = liveness(&function);
            let interference_graph = create_interference_graph(&function, liveness); // FIXME: this should probably just be done in allocate_registers()
            let _alloc = allocate_registers(interference_graph, NUM_REGISTERS);
        }
    }
}
