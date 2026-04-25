use std::collections::HashMap;

use crate::{
    asm::{Arg64, BinArgs, Instruction, MemRef, MovArgs, Reg},
    parser::{BinOp, Expression, ExpressionVariant, Program, Statement, StatementVariant, Term},
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
    seq_no: usize,
}

impl Compiler {
    pub fn new(program: Program) -> Self {
        // Preamble to define entry point and initialize stack frame
        let instructions = vec![
            Instruction::Directive("global".to_string(), "_start".to_string()),
            Instruction::Label("_start".to_string()),
            Instruction::Push(Reg::Rbp),
            Instruction::Mov(MovArgs::ToReg(Reg::Rbp, Arg64::Reg(Reg::Rsp))),
        ];

        Self {
            program,
            instructions,
            stack_offset: WORD_SIZE,
            seq_no: 0,
        }
    }

    pub fn compile(mut self) -> Vec<Instruction> {
        let statements = std::mem::take(&mut self.program.statements);
        let mut identifiers = HashMap::new();

        let local_vars = count_vars(&statements);
        self.instructions.push(Instruction::Sub(BinArgs::ToReg(
            Reg::Rsp,
            Arg64::Unsigned(local_vars * WORD_SIZE),
        )));

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
                    ExpressionVariant::Term(term) => {
                        self.compile_term(term, identifiers);
                        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                            Reg::Rdi,
                            Arg64::Reg(Reg::Rax),
                        )));
                    }
                    ExpressionVariant::BinaryExpr(lhs, rhs, op) => {
                        self.compile_expr(*lhs, identifiers);
                        self.compile_expr(*rhs, identifiers);
                        self.instructions.push(Instruction::Pop(Reg::Rax));
                        self.instructions.push(Instruction::Pop(Reg::Rdi));

                        self.instructions
                            .push(bin_op_to_instr(op, Reg::Rdi, Reg::Rax));
                    }
                }

                self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(EXIT_SYSCALL),
                )));
                self.instructions.push(Instruction::Syscall);
            }
            StatementVariant::Let { ident, expr } => {
                let arg = match expr.variant {
                    ExpressionVariant::Term(term) => match term {
                        Term::Identifier(name) => {
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
                            identifiers
                                .insert(ident.name, VariableLocation::Offset(self.stack_offset));

                            Arg64::Mem(mem_ref)
                        }
                        Term::IntLit(value) => {
                            let loc = VariableLocation::Offset(self.stack_offset);
                            identifiers.insert(ident.name, loc);

                            Arg64::Unsigned(value)
                        }
                    },
                    ExpressionVariant::BinaryExpr(lhs, rhs, op) => {
                        self.compile_expr(*lhs, identifiers);
                        self.compile_expr(*rhs, identifiers);
                        self.instructions.push(Instruction::Pop(Reg::Rbx));
                        self.instructions.push(Instruction::Pop(Reg::Rax));

                        self.instructions
                            .push(bin_op_to_instr(op, Reg::Rax, Reg::Rbx));

                        let loc = VariableLocation::Offset(self.stack_offset);
                        identifiers.insert(ident.name, loc);

                        Arg64::Reg(Reg::Rax)
                    }
                };

                // Every binding (for now) is put into rax and then put onto the
                // stack at the reserved stack slot. Later, we can do some register
                // allocation
                self.instructions
                    .push(Instruction::Mov(MovArgs::ToReg(Reg::Rax, arg)));
                self.instructions.push(Instruction::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rbp,
                        offset: self.stack_offset,
                    },
                    Reg::Rax,
                )));

                // Point stack offset to next slot reserved for local vars
                self.stack_offset += WORD_SIZE;
            }
            StatementVariant::If { cond, then } => {
                self.compile_expr(cond, identifiers);
                self.instructions.push(Instruction::Cmp(BinArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(0),
                )));

                let label = format!("_if{}", self.seq());
                self.instructions.push(Instruction::Je(label.clone()));
                for stmt in then {
                    self.compile_statement(stmt, identifiers);
                }

                self.instructions.push(Instruction::Label(label));
            }
            StatementVariant::Assignment { ident, expr } => {
                self.compile_expr(expr, identifiers);
                let loc = identifiers.get(&ident.name).expect("identifier not found");
                let offset = match loc {
                    VariableLocation::Offset(offset) => *offset,
                };

                self.instructions.push(Instruction::Pop(Reg::Rax));
                self.instructions.push(Instruction::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rbp,
                        offset,
                    },
                    Reg::Rax,
                )))
            }
        }
    }

    fn compile_expr(
        &mut self,
        expr: Expression,
        identifiers: &mut HashMap<String, VariableLocation>,
    ) {
        match expr.variant {
            ExpressionVariant::Term(term) => {
                self.compile_term(term, identifiers);
                self.instructions.push(Instruction::Push(Reg::Rax));
            }
            ExpressionVariant::BinaryExpr(lhs, rhs, op) => {
                self.compile_expr(*lhs, identifiers);
                self.compile_expr(*rhs, identifiers);
                self.instructions.push(Instruction::Pop(Reg::Rbx));
                self.instructions.push(Instruction::Pop(Reg::Rax));

                self.instructions
                    .push(bin_op_to_instr(op, Reg::Rax, Reg::Rbx));
                self.instructions.push(Instruction::Push(Reg::Rax));
            }
        }
    }

    fn compile_term(&mut self, term: Term, identifiers: &HashMap<String, VariableLocation>) {
        match term {
            Term::IntLit(value) => {
                self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(value),
                )));
            }
            Term::Identifier(name) => {
                let var = identifiers
                    .get(&name)
                    .unwrap_or_else(|| panic!("Undeclared identifier: {name:?}"));

                let offset = match var {
                    VariableLocation::Offset(offset) => *offset,
                };
                self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::Rbp,
                        offset,
                    }),
                )));
            }
        }
    }

    fn seq(&mut self) -> usize {
        self.seq_no += 1;
        self.seq_no - 1
    }
}

fn count_vars(statements: &[Statement]) -> usize {
    let mut count = 0;
    for stmt in statements {
        if matches!(
            stmt,
            Statement {
                variant: StatementVariant::Let { .. }
            }
        ) {
            count += 1;
        }
    }
    count
}

fn bin_op_to_instr(op: BinOp, reg1: Reg, reg2: Reg) -> Instruction {
    match op {
        BinOp::Add => Instruction::Add(BinArgs::ToReg(reg1, Arg64::Reg(reg2))),
        BinOp::Sub => Instruction::Sub(BinArgs::ToReg(reg1, Arg64::Reg(reg2))),
        BinOp::Mul => Instruction::Mul(BinArgs::ToReg(reg1, Arg64::Reg(reg2))),
    }
}
