use std::collections::HashMap;

use crate::{
    analyze::Analyzer,
    asm::{Arg64, BinArgs, Instruction, MemRef, MovArgs, Reg},
    parser::{BinOp, ElseClause, Expression, Function, Program, Statement, Term},
};

const EXIT_SYSCALL: usize = 60;
const WORD_SIZE: usize = 8;

#[derive(Debug, Clone, Copy)]
enum VariableLocation {
    // Offset from RBP
    Offset(usize),
    Register(Reg),
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
            Instruction::Call("main".to_string()),
        ];

        Self {
            program,
            instructions,
            stack_offset: WORD_SIZE,
            seq_no: 0,
        }
    }

    pub fn compile(mut self) -> Vec<Instruction> {
        let res = Analyzer::new(&self.program).check();
        if res.is_err() {
            panic!("Semantic analysis resulted in an error: {res:?}");
        }

        let functions = std::mem::take(&mut self.program.functions);
        let mut identifiers = vec![HashMap::new()];

        if let Some(main) = std::mem::take(&mut self.program.main) {
            self.compile_function(main, &mut identifiers);
        } else {
            todo!("Support lib files that don't have a main function");
        }

        for f in functions {
            self.compile_function(f, &mut identifiers);
        }

        self.instructions
    }

    fn compile_function(
        &mut self,
        function: Function,
        identifiers: &mut Vec<HashMap<String, VariableLocation>>,
    ) {
        identifiers.push(HashMap::new());
        let scope = identifiers.last_mut().expect("scope must exist");
        let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
        for (arg, reg) in function.args.iter().zip(arg_regs.iter()) {
            scope.insert(arg.name.name.clone(), VariableLocation::Register(*reg));
        }
        self.instructions
            .push(Instruction::Label(function.name.name.clone()));

        self.instructions.push(Instruction::Push(Reg::Rbp));
        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
            Reg::Rbp,
            Arg64::Reg(Reg::Rsp),
        )));

        let local_vars = count_vars(&function.body);
        self.instructions.push(Instruction::Sub(BinArgs::ToReg(
            Reg::Rsp,
            Arg64::Unsigned(local_vars * WORD_SIZE),
        )));

        self.compile_scope(function.body, identifiers, false);

        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
            Reg::Rsp,
            Arg64::Reg(Reg::Rbp),
        )));
        self.instructions.push(Instruction::Pop(Reg::Rbp));
        self.instructions.push(Instruction::Ret);
        identifiers.pop();
        self.stack_offset = WORD_SIZE;
    }

    fn compile_scope(
        &mut self,
        body: Vec<Statement>,
        identifiers: &mut Vec<HashMap<String, VariableLocation>>,
        new_scope: bool,
    ) {
        if new_scope {
            identifiers.push(HashMap::new());
        }
        for stmt in body {
            self.compile_statement(stmt, identifiers);
        }
        if new_scope {
            identifiers.pop();
        }
    }

    fn compile_statement(
        &mut self,
        stmt: Statement,
        identifiers: &mut Vec<HashMap<String, VariableLocation>>,
    ) {
        match stmt {
            Statement::Exit(expr) => {
                match expr {
                    Expression::Term(term) => {
                        self.compile_term(term, identifiers);
                        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                            Reg::Rdi,
                            Arg64::Reg(Reg::Rax),
                        )));
                    }
                    Expression::BinaryExpr(lhs, rhs, op) => {
                        self.compile_expr(*lhs, identifiers, None);
                        self.compile_expr(*rhs, identifiers, None);
                        self.instructions.push(Instruction::Pop(Reg::Rax));
                        self.instructions.push(Instruction::Pop(Reg::Rdi));

                        let instr = self.bin_op_to_instr(op, Reg::Rdi, Reg::Rax);
                        self.instructions.extend(instr);
                    }
                    Expression::FunctionCall { name, args } => {
                        if args.len() > 4 {
                            todo!("function call with more than 4 args. need to use stack")
                        }

                        let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
                        for (arg, reg) in args.into_iter().zip(arg_regs) {
                            self.compile_expr(arg, identifiers, None);
                            self.instructions.push(Instruction::Pop(reg));
                        }

                        self.instructions.push(Instruction::Call(name.name));
                    }
                }

                self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(EXIT_SYSCALL),
                )));
                self.instructions.push(Instruction::Syscall);
            }
            Statement::Let { ident, expr } => {
                let arg = match expr {
                    Expression::Term(term) => {
                        let scope = identifiers.last_mut().expect("scope must exist");

                        match term {
                            Term::Identifier(name) => {
                                let loc = scope
                                    .get(&name)
                                    .unwrap_or_else(|| panic!("Undeclared identifier: {name:?}"));

                                match loc {
                                    VariableLocation::Offset(offset) => {
                                        let mem_ref = MemRef {
                                            reg: Reg::Rbp,
                                            offset: *offset,
                                        };

                                        scope.insert(
                                            ident.name,
                                            VariableLocation::Offset(self.stack_offset),
                                        );
                                        Arg64::Mem(mem_ref)
                                    }
                                    VariableLocation::Register(reg) => {
                                        let r = *reg;
                                        scope.insert(
                                            ident.name,
                                            VariableLocation::Offset(self.stack_offset),
                                        );

                                        Arg64::Reg(r)
                                    }
                                }
                            }
                            Term::IntLit(value) => {
                                let loc = VariableLocation::Offset(self.stack_offset);
                                scope.insert(ident.name, loc);

                                Arg64::Unsigned(value)
                            }
                            Term::Bool(b) => {
                                let loc = VariableLocation::Offset(self.stack_offset);
                                scope.insert(ident.name, loc);

                                Arg64::Unsigned(b as usize)
                            }
                        }
                    }
                    Expression::BinaryExpr(lhs, rhs, op) => {
                        self.compile_expr(*lhs, identifiers, None);
                        self.compile_expr(*rhs, identifiers, None);
                        self.instructions.push(Instruction::Pop(Reg::Rbx));
                        self.instructions.push(Instruction::Pop(Reg::Rax));

                        let instr = self.bin_op_to_instr(op, Reg::Rax, Reg::Rbx);
                        self.instructions.extend(instr);

                        let loc = VariableLocation::Offset(self.stack_offset);
                        let scope = identifiers.last_mut().expect("scope must exist");
                        scope.insert(ident.name, loc);

                        Arg64::Reg(Reg::Rax)
                    }
                    Expression::FunctionCall { name, args } => {
                        if args.len() > 4 {
                            todo!("function call with more than 4 args. need to use stack")
                        }

                        let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
                        for (arg, reg) in args.into_iter().zip(arg_regs) {
                            self.compile_expr(arg, identifiers, None);
                            self.instructions.push(Instruction::Pop(reg));
                        }

                        self.instructions.push(Instruction::Call(name.name));

                        let loc = VariableLocation::Offset(self.stack_offset);
                        let scope = identifiers.last_mut().expect("scope must exist");
                        scope.insert(ident.name, loc);

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
            Statement::If { cond, then, els } => {
                self.compile_if(identifiers, cond, then, els);
            }
            Statement::While { cond, body } => {
                let cond_label = format!("_while_cond{}", self.seq());
                let done_label = format!("_while_done{}", self.seq());
                let op = extract_binary_op(&cond);

                self.instructions
                    .push(Instruction::Label(cond_label.clone()));

                self.compile_expr(cond, identifiers, Some(done_label.clone()));
                if op.is_none_or(|x| !x.is_cmp()) {
                    self.compile_condition_check(op, done_label.clone());
                }

                self.compile_scope(body, identifiers, true);

                self.instructions.push(Instruction::Jmp(cond_label));
                self.instructions.push(Instruction::Label(done_label));
            }
            Statement::Assignment { ident, expr } => {
                self.compile_expr(expr, identifiers, None);
                let loc = find_identifier(identifiers, &ident.name).expect("identifier not found");
                let offset = match loc {
                    VariableLocation::Offset(offset) => *offset,
                    VariableLocation::Register(_) => {
                        todo!("Allow mutating function args");
                    }
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
            Statement::FunctionCall { name, args } => {
                if args.len() > 4 {
                    todo!("function call with more than 4 args. need to use stack")
                }

                let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
                for (arg, reg) in args.into_iter().zip(arg_regs) {
                    self.compile_expr(arg, identifiers, None);
                    self.instructions.push(Instruction::Pop(reg));
                }

                self.instructions.push(Instruction::Call(name.name));
            }
            Statement::Return(expr) => {
                self.compile_expr(expr, identifiers, None);
                self.instructions.push(Instruction::Pop(Reg::Rax));
                self.instructions.push(Instruction::Pop(Reg::Rbp));
                self.instructions.push(Instruction::Ret);
            }
        }
    }

    fn compile_expr(
        &mut self,
        expr: Expression,
        identifiers: &mut Vec<HashMap<String, VariableLocation>>,
        label: Option<String>,
    ) {
        match expr {
            Expression::Term(term) => {
                self.compile_term(term, identifiers);
                self.instructions.push(Instruction::Push(Reg::Rax));
            }
            Expression::BinaryExpr(lhs, rhs, op) => {
                self.compile_expr(*lhs, identifiers, label.clone());
                self.compile_expr(*rhs, identifiers, label.clone());
                self.instructions.push(Instruction::Pop(Reg::Rbx));
                self.instructions.push(Instruction::Pop(Reg::Rax));

                let instr = self.bin_op_to_instr(op, Reg::Rax, Reg::Rbx);
                self.instructions.extend(instr);

                if !op.is_cmp() {
                    self.instructions.push(Instruction::Push(Reg::Rax));
                } else {
                    self.compile_condition_check(
                        Some(op),
                        label.expect("Must have a jump label for cmps"),
                    );
                }
            }
            Expression::FunctionCall { name, args } => {
                if args.len() > 4 {
                    todo!("function call with more than 4 args. need to use stack")
                }

                let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
                for (arg, reg) in args.into_iter().zip(arg_regs.iter()) {
                    self.compile_expr(arg, identifiers, None);
                    self.instructions.push(Instruction::Pop(*reg));
                }

                self.instructions.push(Instruction::Call(name.name));
            }
        }
    }

    fn compile_if(
        &mut self,
        identifiers: &mut Vec<HashMap<String, VariableLocation>>,
        cond: Expression,
        then: Vec<Statement>,
        els: Option<ElseClause>,
    ) {
        let fail_condition = format!("_if{}", self.seq());
        let done = format!("_if_done{}", self.seq());
        let op = extract_binary_op(&cond);

        self.compile_expr(cond, identifiers, Some(fail_condition.clone()));

        if op.is_none_or(|x| !x.is_cmp()) {
            self.compile_condition_check(op, fail_condition.clone());
        }

        self.compile_scope(then, identifiers, true);

        if els.is_some() {
            // Skip the else branch if the condition was true
            self.instructions.push(Instruction::Jmp(done.clone()));
        }
        self.instructions.push(Instruction::Label(fail_condition));

        if let Some(clause) = els {
            if let Some(cond) = clause.cond {
                self.compile_if(identifiers, cond, clause.body, *clause.els);
            } else {
                self.compile_scope(clause.body, identifiers, true);
            }
        }

        self.instructions.push(Instruction::Label(done));
    }

    fn compile_term(&mut self, term: Term, identifiers: &[HashMap<String, VariableLocation>]) {
        match term {
            Term::IntLit(value) => {
                self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(value),
                )));
            }
            Term::Identifier(name) => {
                let var = find_identifier(identifiers, &name).expect("identifier not found");

                match var {
                    VariableLocation::Offset(offset) => {
                        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                            Reg::Rax,
                            Arg64::Mem(MemRef {
                                reg: Reg::Rbp,
                                offset: *offset,
                            }),
                        )));
                    }
                    VariableLocation::Register(reg) => {
                        self.instructions
                            .push(Instruction::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(*reg))));
                    }
                }
            }
            Term::Bool(b) => {
                self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(b as usize),
                )));
            }
        }
    }

    fn compile_condition_check(&mut self, op: Option<BinOp>, fail_condition: String) {
        match op {
            Some(op) if op.is_bool() || op.is_cmp() => {
                if op.is_bool() {
                    self.instructions
                        .push(Instruction::Jne(fail_condition.clone()));
                } else {
                    self.instructions
                        .push(to_jmp_instr(op, fail_condition.clone()));
                }
            }
            _ => {
                self.instructions.push(Instruction::Cmp(BinArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(0),
                )));
                self.instructions
                    .push(Instruction::Je(fail_condition.clone()));
            }
        }
    }

    fn seq(&mut self) -> usize {
        self.seq_no += 1;
        self.seq_no - 1
    }

    fn bin_op_to_instr(&mut self, op: BinOp, reg1: Reg, reg2: Reg) -> Vec<Instruction> {
        match op {
            BinOp::Add => vec![Instruction::Add(BinArgs::ToReg(reg1, Arg64::Reg(reg2)))],
            BinOp::Sub => vec![Instruction::Sub(BinArgs::ToReg(reg1, Arg64::Reg(reg2)))],
            BinOp::Mul => vec![Instruction::Mul(BinArgs::ToReg(reg1, Arg64::Reg(reg2)))],
            BinOp::Eq | BinOp::Gt | BinOp::Geq | BinOp::Lt | BinOp::Leq | BinOp::Neq => {
                vec![Instruction::Cmp(BinArgs::ToReg(reg1, Arg64::Reg(reg2)))]
            }
            BinOp::And => {
                vec![
                    Instruction::And(BinArgs::ToReg(reg1, Arg64::Reg(reg2))),
                    Instruction::Cmp(BinArgs::ToReg(reg1, Arg64::Unsigned(1))),
                ]
            }
            BinOp::Or => {
                vec![
                    Instruction::Or(BinArgs::ToReg(reg1, Arg64::Reg(reg2))),
                    Instruction::Cmp(BinArgs::ToReg(reg1, Arg64::Unsigned(1))),
                ]
            }
        }
    }
}

fn count_vars(statements: &[Statement]) -> usize {
    let mut count = 0;
    for stmt in statements {
        match &stmt {
            Statement::Let { .. } => count += 1,
            Statement::Exit(_) => {}
            Statement::If { then, els, .. } => {
                count += count_vars(then);
                if let Some(els) = els {
                    count += count_vars(&els.body);

                    let mut next = els.els.as_ref();
                    while let Some(e) = next {
                        count += count_vars(&e.body);
                        next = e.els.as_ref();
                    }
                }
            }
            Statement::While { body, .. } => {
                count += count_vars(body);
            }
            Statement::Assignment { .. } => {}
            Statement::FunctionCall { .. } => {}
            Statement::Return(_) => {}
        }
    }
    count
}

fn to_jmp_instr(op: BinOp, label: String) -> Instruction {
    match op {
        BinOp::Eq => Instruction::Jne(label),
        BinOp::Neq => Instruction::Je(label),
        BinOp::Gt => Instruction::Jle(label),
        BinOp::Geq => Instruction::Jl(label),
        BinOp::Lt => Instruction::Jge(label),
        BinOp::Leq => Instruction::Jg(label),
        _ => unreachable!(),
    }
}

fn extract_binary_op(expr: &Expression) -> Option<BinOp> {
    if let Expression::BinaryExpr(_, _, op) = expr {
        Some(*op)
    } else {
        None
    }
}

fn find_identifier<'a>(
    identifiers: &'a [HashMap<String, VariableLocation>],
    ident: &str,
) -> Option<&'a VariableLocation> {
    identifiers.iter().rev().find_map(|scope| scope.get(ident))
}
