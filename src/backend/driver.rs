use std::{fs::File, io::Write, path::PathBuf};

use crate::{
    backend::{allocator::Allocator, pass::OptimizationPass, target::Target},
    ir::types::TIRFunction,
};

pub struct Driver<Target> {
    passes: Vec<Box<dyn OptimizationPass>>,
    target: Target,
    path: PathBuf,
}

impl<T: Target> Driver<T> {
    pub fn new(target: T, path: PathBuf) -> Self {
        Self {
            passes: Vec::new(),
            target,
            path,
        }
    }

    pub fn with_pass(mut self, pass: impl OptimizationPass + 'static) -> Self {
        self.passes.push(Box::new(pass));
        self
    }

    pub fn run(mut self, functions: Vec<TIRFunction>) {
        if functions[0].name != "main" {
            todo!("Support lib files that don't have a main function");
        }

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

            let allocator = Allocator::new(&function, self.target.num_gp_registers());
            self.target.emit(function, allocator);
        }

        let asm = self.target.asm();
        asm_file
            .write_all(asm.as_bytes())
            .expect("Could not write to file");
        asm_file.flush().expect("Could not flush file");
    }
}
