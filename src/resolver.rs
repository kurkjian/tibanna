use std::collections::HashMap;

use crate::parser::{BinOp, ElseClause, Expression, Function, Program, Statement, Term, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolId {
    pub name: String,
    pub id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId {
    pub name: String,
    pub id: usize,
}

#[derive(Debug, Clone)]
pub struct ResolvedProgram {
    pub main: Option<ResolvedFunction>,
    pub functions: Vec<ResolvedFunction>,
}

#[derive(Debug, Clone)]
pub struct ResolvedFunction {
    pub id: FunctionId,
    pub name: String,
    pub args: Vec<ResolvedArgument>,
    pub ret_sig: Type,
    pub body: Vec<ResolvedStatement>,
}

#[derive(Debug, Clone)]
pub struct ResolvedArgument {
    pub symbol: SymbolId,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum ResolvedStatement {
    Exit(ResolvedExpression),
    Let {
        symbol: SymbolId,
        expr: ResolvedExpression,
    },
    If {
        cond: ResolvedExpression,
        then: Vec<ResolvedStatement>,
        els: Option<ResolvedElseClause>,
    },
    While {
        cond: ResolvedExpression,
        body: Vec<ResolvedStatement>,
    },
    Assignment {
        symbol: SymbolId,
        expr: ResolvedExpression,
    },
    FunctionCall {
        function: FunctionId,
        args: Vec<ResolvedExpression>,
    },
    Return(ResolvedExpression),
}

#[derive(Debug, Clone)]
pub enum ResolvedExpression {
    BinaryExpr(Box<ResolvedExpression>, Box<ResolvedExpression>, BinOp),
    Term(ResolvedTerm),
    FunctionCall {
        function: FunctionId,
        args: Vec<ResolvedExpression>,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedElseClause {
    pub cond: Option<ResolvedExpression>,
    pub body: Vec<ResolvedStatement>,
    pub els: Box<Option<ResolvedElseClause>>,
}

#[derive(Debug, Clone)]
pub enum ResolvedTerm {
    Identifier(SymbolId),
    IntLit(usize),
    Bool(bool),
}

#[derive(Default)]
pub struct Resolver {
    scopes: Vec<HashMap<String, SymbolId>>,
    functions: HashMap<String, FunctionId>,

    next_symbol: usize,
    next_function: usize,
}

impl Resolver {
    pub fn resolve_program(mut self, program: Program) -> Result<ResolvedProgram, String> {
        if let Some(main) = &program.main {
            self.register_function(&main.name.name)?;
        }
        for func in &program.functions {
            self.register_function(&func.name.name)?;
        }

        let main = match program.main {
            Some(f) => Some(self.resolve_function(f)?),
            None => None,
        };

        let mut functions = Vec::with_capacity(program.functions.len());
        for func in program.functions {
            functions.push(self.resolve_function(func)?);
        }

        Ok(ResolvedProgram { main, functions })
    }

    fn register_function(&mut self, name: &str) -> Result<FunctionId, String> {
        if self.functions.contains_key(name) {
            return Err(format!("duplicate function '{name}'"));
        }

        let id = FunctionId {
            name: name.to_string(),
            id: self.next_function,
        };
        self.next_function += 1;
        self.functions.insert(name.to_string(), id.clone());

        Ok(id)
    }

    fn resolve_function(&mut self, func: Function) -> Result<ResolvedFunction, String> {
        let id = self.functions[&func.name.name].clone();
        self.push_scope();
        let mut args = Vec::with_capacity(func.args.len());

        for arg in func.args {
            let sym = self.declare(&arg.name.name)?;

            args.push(ResolvedArgument {
                symbol: sym,
                ty: arg.ty,
            });
        }

        let body = self.resolve_statements(func.body)?;
        self.pop_scope();

        Ok(ResolvedFunction {
            id,
            name: func.name.name,
            args,
            ret_sig: func.ret_sig,
            body,
        })
    }

    fn resolve_statements(
        &mut self,
        stmts: Vec<Statement>,
    ) -> Result<Vec<ResolvedStatement>, String> {
        let mut out = vec![];
        for stmt in stmts {
            out.push(self.resolve_statement(stmt)?);
        }

        Ok(out)
    }

    fn resolve_statement(&mut self, stmt: Statement) -> Result<ResolvedStatement, String> {
        match stmt {
            Statement::Exit(expr) => Ok(ResolvedStatement::Exit(self.resolve_expr(expr)?)),
            Statement::Return(expr) => Ok(ResolvedStatement::Return(self.resolve_expr(expr)?)),
            Statement::Let { ident, expr } => {
                let expr = self.resolve_expr(expr)?;
                let sym = self.declare(&ident.name)?;

                Ok(ResolvedStatement::Let { symbol: sym, expr })
            }
            Statement::Assignment { ident, expr } => {
                let sym = self.lookup(&ident.name)?;

                Ok(ResolvedStatement::Assignment {
                    symbol: sym,
                    expr: self.resolve_expr(expr)?,
                })
            }
            Statement::FunctionCall { name, args } => {
                let function = self.lookup_function(&name.name)?;
                let mut resolved_args = Vec::with_capacity(args.len());
                for arg in args {
                    resolved_args.push(self.resolve_expr(arg)?);
                }

                Ok(ResolvedStatement::FunctionCall {
                    function,
                    args: resolved_args,
                })
            }
            Statement::While { cond, body } => {
                let cond = self.resolve_expr(cond)?;
                self.push_scope();
                let body = self.resolve_statements(body)?;
                self.pop_scope();

                Ok(ResolvedStatement::While { cond, body })
            }
            Statement::If { cond, then, els } => {
                let cond = self.resolve_expr(cond)?;
                self.push_scope();
                let then = self.resolve_statements(then)?;
                self.pop_scope();
                let els = self.resolve_else_clause(els)?;

                Ok(ResolvedStatement::If { cond, then, els })
            }
        }
    }

    fn resolve_else_clause(
        &mut self,
        els: Option<ElseClause>,
    ) -> Result<Option<ResolvedElseClause>, String> {
        match els {
            None => Ok(None),
            Some(els) => {
                let cond = match els.cond {
                    Some(c) => Some(self.resolve_expr(c)?),
                    None => None,
                };
                self.push_scope();
                let body = self.resolve_statements(els.body)?;
                self.pop_scope();
                let nested = self.resolve_else_clause(*els.els)?;

                Ok(Some(ResolvedElseClause {
                    cond,
                    body,
                    els: Box::new(nested),
                }))
            }
        }
    }

    fn resolve_expr(&mut self, expr: Expression) -> Result<ResolvedExpression, String> {
        match expr {
            Expression::BinaryExpr(lhs, rhs, op) => Ok(ResolvedExpression::BinaryExpr(
                Box::new(self.resolve_expr(*lhs)?),
                Box::new(self.resolve_expr(*rhs)?),
                op,
            )),
            Expression::Term(term) => Ok(ResolvedExpression::Term(self.resolve_term(term)?)),
            Expression::FunctionCall { name, args } => {
                let function = self.lookup_function(&name.name)?;
                let mut resolved_args = Vec::with_capacity(args.len());
                for arg in args {
                    resolved_args.push(self.resolve_expr(arg)?);
                }

                Ok(ResolvedExpression::FunctionCall {
                    function,
                    args: resolved_args,
                })
            }
        }
    }

    fn resolve_term(&mut self, term: Term) -> Result<ResolvedTerm, String> {
        match term {
            Term::Identifier(name) => {
                let sym = self.lookup(&name)?;

                Ok(ResolvedTerm::Identifier(sym))
            }
            Term::IntLit(v) => Ok(ResolvedTerm::IntLit(v)),
            Term::Bool(v) => Ok(ResolvedTerm::Bool(v)),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) -> Result<SymbolId, String> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(name) {
            return Err(format!("duplicate variable '{name}' in scope"));
        }

        let sym = SymbolId {
            name: name.to_string(),
            id: self.next_symbol,
        };
        self.next_symbol += 1;
        scope.insert(name.to_string(), sym.clone());

        Ok(sym)
    }

    fn lookup(&self, name: &str) -> Result<SymbolId, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Ok(sym.clone());
            }
        }

        Err(format!("undefined variable '{name}'"))
    }

    fn lookup_function(&self, name: &str) -> Result<FunctionId, String> {
        self.functions
            .get(name)
            .cloned()
            .ok_or_else(|| format!("undefined function '{name}'"))
    }
}
