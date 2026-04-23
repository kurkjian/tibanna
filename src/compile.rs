use std::collections::HashMap;

use crate::{
    asm::{Arg64, Instruction, MemRef, Mov, Reg},
    parser::{ExpressionVariant, Program, Statement, StatementVariant},
};

const EXIT_SYSCALL: usize = 60;

#[derive(Clone, Copy)]
enum VariableLocation {
    // Offset from RSP
    Offset(usize),
}

pub struct Compiler {
    stack_offset: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self { stack_offset: 0 }
    }

    pub fn compile(&mut self, program: Program) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let mut identifiers = HashMap::new();

        for stmt in program.statements {
            self.compile_statement(stmt, &mut instructions, &mut identifiers);
        }

        instructions
    }

    fn compile_statement(
        &mut self,
        stmt: Statement,
        instructions: &mut Vec<Instruction>,
        identifiers: &mut HashMap<String, VariableLocation>,
    ) {
        match stmt.variant {
            StatementVariant::Exit(expr) => {
                match expr.variant {
                    ExpressionVariant::Identifier(name) => {
                        let loc = identifiers
                            .get(&name)
                            .expect(&format!("Undeclared identifier: {name:?}"));
                        let mem_ref = match loc {
                            VariableLocation::Offset(offset) => MemRef {
                                reg: Reg::Rsp,
                                offset: *offset,
                            },
                        };

                        instructions
                            .push(Instruction::Mov(Mov::ToReg(Reg::Rdi, Arg64::Mem(mem_ref))));
                    }
                    ExpressionVariant::IntLit(value) => {
                        instructions.push(Instruction::Mov(Mov::ToReg(
                            Reg::Rdi,
                            Arg64::Unsigned(value),
                        )));
                    }
                }

                instructions.push(Instruction::Mov(Mov::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(EXIT_SYSCALL),
                )));
                instructions.push(Instruction::Syscall);
            }
            StatementVariant::Let { ident, expr } => {
                let arg = match expr.variant {
                    ExpressionVariant::Identifier(name) => {
                        let loc = identifiers
                            .get(&name)
                            .expect(&format!("Undeclared identifier: {name:?}"));
                        let offset = match loc {
                            VariableLocation::Offset(offset) => *offset,
                            _ => unreachable!(), // FIXME: eventually var loc should be either a reg or a stack offset
                        };
                        let mem_ref = MemRef {
                            reg: Reg::Rsp,
                            offset,
                        };
                        identifiers.insert(ident.name, VariableLocation::Offset(self.stack_offset));
                        self.stack_offset += 8;

                        Arg64::Mem(mem_ref)
                    }
                    ExpressionVariant::IntLit(value) => {
                        let loc = VariableLocation::Offset(self.stack_offset);
                        self.stack_offset += 8;
                        identifiers.insert(ident.name, loc);

                        Arg64::Unsigned(value)
                    }
                };

                // Every binding (for now) is put into rax and then immediately pushed onto the stack.
                // This can be optimized later to avoid the stack probably (register allocation, code-gen ??)
                instructions.push(Instruction::Mov(Mov::ToReg(Reg::Rax, arg)));
                instructions.push(Instruction::Push(Reg::Rax));
            }
        }
    }
}

// let x = 5;
// exit(x);
// <------->
// mov rax, 5
// push rax,
// mov rax, 60
// mov rdi, [rsp]
// syscall

// let x = 5;
// let y = x;
// exit(y);
// <------->
// mov rax, 5
// push rax,
// mov rax, [rsp]
// push rax
// mov rax, 60
// mov rdi, [rsp]
// syscall
