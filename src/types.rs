use anyhow::{Result, bail};
use std::collections::HashMap;
use thiserror::Error;

use crate::parser::{
    Argument, ElseClause, Expression, ExpressionVariant, Function, Program, Statement,
    StatementVariant, Term, Type,
};

#[derive(Debug, Error)]
pub enum TypeCheckError {
    #[error("type mismatch: expected {0}, got {1}")]
    TypeMismatch(Type, Type),
}

pub struct TypeChecker<'a> {
    ast: &'a Program,
    symbols: HashMap<String, Type>,
    functions: HashMap<String, (&'a Vec<Argument>, &'a Type)>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(ast: &'a Program) -> Self {
        Self {
            ast,
            symbols: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn check(&mut self) -> Result<()> {
        for function in self.ast.functions.iter() {
            self.functions.insert(
                function.name.name.clone(),
                (&function.args, &function.ret_sig),
            );
        }

        if let Some(main) = &self.ast.main {
            self.functions
                .insert(main.name.name.clone(), (&main.args, &main.ret_sig));
            self.resolve_function(main)?;
        }

        for function in self.ast.functions.iter() {
            self.resolve_function(function)?;
        }

        Ok(())
    }

    fn resolve_function(&mut self, function: &Function) -> Result<()> {
        for arg in function.args.iter() {
            self.symbols.insert(arg.name.name.clone(), arg.ty.clone());
        }

        for stmt in &function.body {
            self.resolve_statement(stmt, function)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Statement, parent: &Function) -> Result<()> {
        match &stmt.variant {
            StatementVariant::Exit(expression) => {
                let expr_type = self.resolve_expr(expression)?;
                if expr_type != Type::Int {
                    bail!(TypeCheckError::TypeMismatch(Type::Int, expr_type));
                }

                Ok(())
            }
            StatementVariant::Let { ident, expr } => {
                let expr_type = self.resolve_expr(expr)?;
                self.symbols.insert(ident.name.clone(), expr_type);

                Ok(())
            }
            StatementVariant::If { cond, then, els } => {
                let _ = self.resolve_expr(cond)?;

                for stmt in then {
                    self.resolve_statement(stmt, parent)?;
                }

                if let Some(els) = els {
                    self.resolve_else(els, parent)?;
                }

                Ok(())
            }
            StatementVariant::While { cond, body } => {
                let _ = self.resolve_expr(cond)?;

                for stmt in body {
                    self.resolve_statement(stmt, parent)?;
                }

                Ok(())
            }
            StatementVariant::Assignment { ident, expr } => {
                let expr_type = self.resolve_expr(expr)?;

                match self.symbols.get(&ident.name) {
                    Some(expected_type) if expected_type == &expr_type => Ok(()),
                    Some(expected_type) => {
                        bail!(TypeCheckError::TypeMismatch(
                            expected_type.to_owned(),
                            expr_type
                        ));
                    }
                    None => {
                        panic!(
                            "Identifier {:?} does not exist. Parser should have caught this",
                            ident
                        );
                    }
                }
            }
            StatementVariant::FunctionCall { name, args } => {
                let func = self.functions.get(&name.name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Function {:?} does not exist. Parser should have caught this",
                        name
                    )
                })?;

                self.validate_function_args(args, func.0)?;
                Ok(())
            }
            StatementVariant::Return(expr) => {
                let return_type = self.resolve_expr(expr)?;

                if return_type != parent.ret_sig {
                    bail!(TypeCheckError::TypeMismatch(
                        parent.ret_sig.to_owned(),
                        return_type
                    ));
                }
                Ok(())
            }
        }
    }

    fn resolve_else(&mut self, els: &ElseClause, parent: &Function) -> Result<()> {
        if let Some(c) = &els.cond {
            let _ = self.resolve_expr(c)?;
        }
        for s in &els.body {
            self.resolve_statement(s, parent)?;
        }

        if let Some(e) = &*els.els {
            self.resolve_else(e, parent)?;
        }

        Ok(())
    }

    fn resolve_expr(&self, expr: &Expression) -> Result<Type> {
        match &expr.variant {
            ExpressionVariant::BinaryExpr(lhs, rhs, _bin_op) => {
                let lhs_type = self.resolve_expr(lhs)?;
                let rhs_type = self.resolve_expr(rhs)?;

                if lhs_type != rhs_type {
                    bail!(TypeCheckError::TypeMismatch(lhs_type.to_owned(), rhs_type));
                }

                // TODO: verify <bin_op> can be applied to <lhs_type> and <rhs_type>

                Ok(lhs_type)
            }
            ExpressionVariant::Term(term) => self.resolve_term(term),
            ExpressionVariant::FunctionCall { name, args } => {
                let ctx = self.functions.get(&name.name).ok_or_else(|| {
                    anyhow::anyhow!("Call to undeclared function. Parser should have caught this")
                })?;

                self.validate_function_args(args, ctx.0)?;

                Ok(ctx.1.to_owned())
            }
        }
    }

    fn resolve_term(&self, term: &Term) -> Result<Type> {
        match term {
            Term::IntLit(_) => Ok(Type::Int),
            Term::Bool(_) => Ok(Type::Bool),
            Term::Identifier(ident) => self.symbols.get(ident).cloned().ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing identifier: {}. Parser should have caught this",
                    ident
                )
            }),
        }
    }

    fn validate_function_args(&self, args: &[Expression], expected: &[Argument]) -> Result<()> {
        for (arg, expected) in args.iter().zip(expected.iter()) {
            let arg_type = self.resolve_expr(arg)?;
            if arg_type != expected.ty {
                bail!(TypeCheckError::TypeMismatch(
                    expected.ty.to_owned(),
                    arg_type
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn test_binary_expr() {
        let ast = Parser::new(
            Lexer::new("fn main() { let x = true + 2; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(
            Lexer::new("fn main() { let x = 2 + 2; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());
    }

    #[test]
    fn test_exit() {
        let ast = Parser::new(Lexer::new("fn main() { exit(false); }").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(
            Lexer::new("fn main() { exit(2 + false); }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(Lexer::new("fn main() { exit(2); }").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(Lexer::new("fn main() { exit(2 + 2); }").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());
    }

    #[test]
    fn test_assignment() {
        let ast = Parser::new(
            Lexer::new("fn main() { let x = 2; x = 3; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(
            Lexer::new("fn main() { let x = 2; x = false; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());
    }

    #[test]
    fn test_return_type() {
        let ast = Parser::new(
            Lexer::new("fn main() = int { return 2; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(
            Lexer::new("fn main() = int { return false; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());
    }

    #[test]
    fn test_function_arguments() {
        let ast = Parser::new(
            Lexer::new(
                "fn main() { let x = 1; let y = inc(x); } fn inc(x: int) = int { return x + 1; }",
            )
            .tokenize()
            .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(
            Lexer::new(
                "fn main() { let x = false; let y = inc(x); } fn inc(x: int) = int { return x + 1; }",
            )
            .tokenize()
            .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());
    }
}
