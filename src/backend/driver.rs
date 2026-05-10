use std::{fs::File, io::Write, path::PathBuf};

use crate::{
    backend::{
        liveness::liveness,
        pass::OptimizationPass,
        regalloc::{allocate_registers, create_interference_graph},
        target::Target,
    },
    ir::types::TIRFunction,
};

pub struct Driver<Target> {
    passes: Vec<Box<dyn OptimizationPass>>,
    target: Target,
    path: PathBuf,
}

impl<T: Target> Driver<T> {
    pub fn new(passes: Vec<Box<dyn OptimizationPass>>, target: T, path: PathBuf) -> Self {
        Self {
            passes,
            target,
            path,
        }
    }

    pub fn run(mut self, functions: Vec<TIRFunction>) {
        let mut asm_file = File::create(&self.path).expect("Could not create file");
        self.target.asm_header();

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
            let alloc = allocate_registers(interference_graph, self.target.num_gp_registers());

            self.target.emit(function, alloc);
        }

        let asm = self.target.asm();
        asm_file
            .write_all(asm.as_bytes())
            .expect("Could not write to file");
        asm_file.flush().expect("Could not flush file");
    }
}
