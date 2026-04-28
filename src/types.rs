use anyhow::{Result, bail};
use std::collections::HashMap;

use crate::parser::{
    ElseClause, Expression, ExpressionVariant, Program, Statement, StatementVariant, Term,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Type {
    Int,
    Bool,
}

pub struct TypeChecker<'a> {
    ast: &'a Program,
    symbols: HashMap<String, Type>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(ast: &'a Program) -> Self {
        Self {
            ast,
            symbols: HashMap::new(),
        }
    }

    pub fn check(&mut self) -> Result<()> {
        for statement in self.ast.statements.iter() {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Statement) -> Result<()> {
        match &stmt.variant {
            StatementVariant::Exit(expression) => {
                let expr_type = self.resolve_expr(expression)?;
                if expr_type != Type::Int {
                    return Err(anyhow::anyhow!(
                        "Exit expression must be an integer: {:?}",
                        expr_type
                    ));
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
                    self.resolve_statement(stmt)?;
                }

                if let Some(els) = els {
                    self.resolve_else(els)?;
                }

                Ok(())
            }
            StatementVariant::Assignment { ident, expr } => {
                let expr_type = self.resolve_expr(expr)?;

                match self.symbols.get(&ident.name) {
                    Some(expected_type) if expected_type == &expr_type => Ok(()),
                    Some(expected_type) => {
                        bail!(
                            "Type mismatch: expected {:?}, found {:?}",
                            expected_type,
                            expr_type
                        );
                    }
                    None => {
                        bail!("Identifier {:?} does not exist", ident);
                    }
                }
            }
        }
    }

    fn resolve_else(&mut self, els: &ElseClause) -> Result<()> {
        if let Some(c) = &els.cond {
            let _ = self.resolve_expr(c)?;
        }
        for s in &els.body {
            self.resolve_statement(s)?;
        }

        if let Some(e) = &*els.els {
            self.resolve_else(e)?;
        }

        Ok(())
    }

    fn resolve_expr(&self, expr: &Expression) -> Result<Type> {
        match &expr.variant {
            ExpressionVariant::BinaryExpr(lhs, rhs, _bin_op) => {
                let lhs_type = self.resolve_expr(lhs)?;
                let rhs_type = self.resolve_expr(rhs)?;

                if lhs_type != rhs_type {
                    bail!("Type mismatch: lhs {:?}, rhs {:?}", lhs_type, rhs_type);
                }

                // TODO: verify <bin_op> can be applied to <lhs_type> and <rhs_type>

                Ok(lhs_type)
            }
            ExpressionVariant::Term(term) => self.resolve_term(term),
        }
    }

    fn resolve_term(&self, term: &Term) -> Result<Type> {
        match term {
            Term::IntLit(_) => Ok(Type::Int),
            Term::Bool(_) => Ok(Type::Bool),
            Term::Identifier(ident) => self
                .symbols
                .get(ident)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing identifier: {}", ident)),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn test_binary_expr() {
        let ast = Parser::new(Lexer::new("let x = true + 2;").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(Lexer::new("let x = 2 + 2;").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());
    }

    #[test]
    fn test_exit() {
        let ast = Parser::new(Lexer::new("exit(false);").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(Lexer::new("exit(2 + false);").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(Lexer::new("exit(2);").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(Lexer::new("exit(2 + 2);").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());
    }

    #[test]
    fn test_assignment() {
        let ast = Parser::new(Lexer::new("let x = 2; x = 3;").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(Lexer::new("let x = 2; x = false;").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = TypeChecker::new(&ast);
        assert!(checker.check().is_err());
    }
}
