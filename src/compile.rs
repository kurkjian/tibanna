use std::collections::HashMap;

use crate::{
    asm::{Arg64, Instruction, MemRef, Mov, Reg},
    parser::{Expression, ExpressionVariant, Program, Statement, StatementVariant},
};

const EXIT_SYSCALL: usize = 60;
const WORD_SIZE: usize = 8;

#[derive(Debug, Clone, Copy)]
enum VariableLocation {
    // Offset from RBP
    Offset(usize),
}

pub struct Compiler {
    program: Program,
    instructions: Vec<Instruction>,
    stack_offset: usize,
}

impl Compiler {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            instructions: Vec::new(),
            stack_offset: WORD_SIZE,
        }
    }

    pub fn compile(mut self) -> Vec<Instruction> {
        let statements = std::mem::take(&mut self.program.statements);
        let mut identifiers = HashMap::new();

        for stmt in statements {
            self.compile_statement(stmt, &mut identifiers);
        }

        self.instructions
    }

    fn compile_statement(
        &mut self,
        stmt: Statement,
        identifiers: &mut HashMap<String, VariableLocation>,
    ) {
        match stmt.variant {
            StatementVariant::Exit(expr) => {
                match expr.variant {
                    ExpressionVariant::Identifier(name) => {
                        let loc = identifiers
                            .get(&name)
                            .unwrap_or_else(|| panic!("Undeclared identifier: {name:?}"));
                        let mem_ref = match loc {
                            VariableLocation::Offset(offset) => MemRef {
                                reg: Reg::Rbp,
                                offset: *offset,
                            },
                        };

                        self.instructions
                            .push(Instruction::Mov(Mov::ToReg(Reg::Rdi, Arg64::Mem(mem_ref))));
                    }
                    ExpressionVariant::IntLit(value) => {
                        self.instructions.push(Instruction::Mov(Mov::ToReg(
                            Reg::Rdi,
                            Arg64::Unsigned(value),
                        )));
                    }
                    ExpressionVariant::BinaryAdd(lhs, rhs) => {
                        self.compile_expr(*lhs, identifiers);
                        self.compile_expr(*rhs, identifiers);
                        self.instructions.push(Instruction::Pop(Reg::Rdi));
                        self.instructions.push(Instruction::Pop(Reg::Rax));
                        self.stack_offset -= 2 * WORD_SIZE;

                        self.instructions.push(Instruction::Add(Reg::Rdi, Reg::Rax));
                    }
                    ExpressionVariant::BinarySub(lhs, rhs) => {
                        self.compile_expr(*lhs, identifiers);
                        self.compile_expr(*rhs, identifiers);
                        self.instructions.push(Instruction::Pop(Reg::Rdi));
                        self.instructions.push(Instruction::Pop(Reg::Rax));
                        self.stack_offset -= 2 * WORD_SIZE;

                        self.instructions.push(Instruction::Sub(Reg::Rdi, Reg::Rax));
                    }
                }

                self.instructions.push(Instruction::Mov(Mov::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(EXIT_SYSCALL),
                )));
                self.instructions.push(Instruction::Syscall);
            }
            StatementVariant::Let { ident, expr } => {
                let arg = match expr.variant {
                    ExpressionVariant::Identifier(name) => {
                        let loc = identifiers
                            .get(&name)
                            .unwrap_or_else(|| panic!("Undeclared identifier: {name:?}"));
                        let offset = match loc {
                            VariableLocation::Offset(offset) => *offset,
                        };
                        let mem_ref = MemRef {
                            reg: Reg::Rbp,
                            offset,
                        };
                        identifiers.insert(ident.name, VariableLocation::Offset(self.stack_offset));
                        Arg64::Mem(mem_ref)
                    }
                    ExpressionVariant::IntLit(value) => {
                        let loc = VariableLocation::Offset(self.stack_offset);
                        identifiers.insert(ident.name, loc);

                        Arg64::Unsigned(value)
                    }
                    ExpressionVariant::BinaryAdd(lhs, rhs) => {
                        self.compile_expr(*lhs, identifiers);
                        self.compile_expr(*rhs, identifiers);
                        self.instructions.push(Instruction::Pop(Reg::Rax));
                        self.instructions.push(Instruction::Pop(Reg::Rbx));
                        self.stack_offset -= 2 * WORD_SIZE;

                        self.instructions.push(Instruction::Add(Reg::Rax, Reg::Rbx));

                        let loc = VariableLocation::Offset(self.stack_offset);
                        identifiers.insert(ident.name, loc);

                        Arg64::Reg(Reg::Rax)
                    }
                    ExpressionVariant::BinarySub(lhs, rhs) => {
                        self.compile_expr(*lhs, identifiers);
                        self.compile_expr(*rhs, identifiers);
                        self.instructions.push(Instruction::Pop(Reg::Rbx));
                        self.instructions.push(Instruction::Pop(Reg::Rax));
                        self.stack_offset -= 2 * WORD_SIZE;

                        self.instructions.push(Instruction::Sub(Reg::Rax, Reg::Rbx));

                        let loc = VariableLocation::Offset(self.stack_offset);
                        identifiers.insert(ident.name, loc);

                        Arg64::Reg(Reg::Rax)
                    }
                };

                // Every binding (for now) is put into rax and then immediately pushed onto the stack.
                // This can be optimized later to avoid the stack probably (register allocation, code-gen ??)
                self.instructions
                    .push(Instruction::Mov(Mov::ToReg(Reg::Rax, arg)));
                self.instructions.push(Instruction::Push(Reg::Rax));
                self.stack_offset += WORD_SIZE;
            }
        }
    }

    fn compile_expr(
        &mut self,
        expr: Expression,
        identifiers: &mut HashMap<String, VariableLocation>,
    ) {
        match expr.variant {
            ExpressionVariant::IntLit(value) => {
                self.instructions.push(Instruction::Mov(Mov::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(value),
                )));
                self.instructions.push(Instruction::Push(Reg::Rax));
                self.stack_offset += WORD_SIZE;
            }
            ExpressionVariant::BinaryAdd(lhs, rhs) => {
                self.compile_expr(*lhs, identifiers);
                self.compile_expr(*rhs, identifiers);
                self.instructions.push(Instruction::Pop(Reg::Rax));
                self.instructions.push(Instruction::Pop(Reg::Rbx));
                self.stack_offset -= 2 * WORD_SIZE;

                self.instructions.push(Instruction::Add(Reg::Rax, Reg::Rbx));
                self.instructions.push(Instruction::Push(Reg::Rax));
                self.stack_offset += WORD_SIZE;
            }
            ExpressionVariant::BinarySub(lhs, rhs) => {
                self.compile_expr(*lhs, identifiers);
                self.compile_expr(*rhs, identifiers);
                self.instructions.push(Instruction::Pop(Reg::Rax));
                self.instructions.push(Instruction::Pop(Reg::Rbx));
                self.stack_offset -= 2 * WORD_SIZE;

                self.instructions.push(Instruction::Sub(Reg::Rax, Reg::Rbx));
                self.instructions.push(Instruction::Push(Reg::Rax));
                self.stack_offset += WORD_SIZE;
            }
            ExpressionVariant::Identifier(name) => {
                let var = identifiers
                    .get(&name)
                    .unwrap_or_else(|| panic!("Undeclared identifier: {name:?}"));

                let offset = match var {
                    VariableLocation::Offset(offset) => *offset,
                };
                self.instructions.push(Instruction::Mov(Mov::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::Rbp,
                        offset,
                    }),
                )));
                self.instructions.push(Instruction::Push(Reg::Rax));
                self.stack_offset += WORD_SIZE;
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
