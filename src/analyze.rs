use anyhow::{Result, bail};
use std::collections::HashMap;
use thiserror::Error;

use crate::parser::{
    Argument, ElseClause, Expression, ExpressionVariant, Function, Program, Statement,
    StatementVariant, Term, Type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlFlow {
    Continues,
    Returns,
}

#[derive(Debug, Error)]
pub enum SemanticAnalysisError {
    #[error("type mismatch: expected {0}, got {1}")]
    TypeMismatch(Type, Type),
    #[error("not all code paths return in function {0}")]
    MissingReturn(String),
    #[error("use of undeclared identifier {0}")]
    UndeclaredIdentifier(String),
    #[error("use of undefined function {0}")]
    UndefinedFunction(String),
}

pub struct Analyzer<'a> {
    ast: &'a Program,
    symbols: Vec<HashMap<String, Type>>,
    functions: HashMap<String, (&'a Vec<Argument>, &'a Type)>,
}

impl<'a> Analyzer<'a> {
    pub fn new(ast: &'a Program) -> Self {
        Self {
            ast,
            symbols: Vec::new(),
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
        self.symbols.push(HashMap::new());
        let scoped = self
            .symbols
            .last_mut()
            .expect("symbols should not be empty");
        for arg in function.args.iter() {
            scoped.insert(arg.name.name.clone(), arg.ty.clone());
        }

        let flow = self.resolve_block(&function.body, function, false)?;

        if function.ret_sig != Type::Void && flow != ControlFlow::Returns {
            bail!(SemanticAnalysisError::MissingReturn(
                function.name.name.clone()
            ));
        }

        self.symbols.pop();
        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Statement, parent: &Function) -> Result<ControlFlow> {
        match &stmt.variant {
            StatementVariant::Exit(expression) => {
                let expr_type = self.resolve_expr(expression)?;
                if expr_type != Type::Int {
                    bail!(SemanticAnalysisError::TypeMismatch(Type::Int, expr_type));
                }

                Ok(ControlFlow::Returns)
            }
            StatementVariant::Let { ident, expr } => {
                let expr_type = self.resolve_expr(expr)?;
                let scoped = self
                    .symbols
                    .last_mut()
                    .expect("symbols should not be empty");
                scoped.insert(ident.name.clone(), expr_type);

                Ok(ControlFlow::Continues)
            }
            StatementVariant::If { cond, then, els } => {
                let _ = self.resolve_expr(cond)?;

                let then_flow = self.resolve_block(then, parent, true)?;

                let else_flow = if let Some(els) = els {
                    self.resolve_else(els, parent)?
                } else {
                    ControlFlow::Continues
                };

                if then_flow == ControlFlow::Returns && else_flow == ControlFlow::Returns {
                    Ok(ControlFlow::Returns)
                } else {
                    Ok(ControlFlow::Continues)
                }
            }
            StatementVariant::While { cond, body } => {
                let _ = self.resolve_expr(cond)?;
                let _ = self.resolve_block(body, parent, true)?;

                Ok(ControlFlow::Continues)
            }
            StatementVariant::Assignment { ident, expr } => {
                let expr_type = self.resolve_expr(expr)?;

                if let Some(expected_type) = self.find_identifier(&ident.name) {
                    if expected_type == &expr_type {
                        Ok(ControlFlow::Continues)
                    } else {
                        bail!(SemanticAnalysisError::TypeMismatch(
                            expected_type.to_owned(),
                            expr_type
                        ))
                    }
                } else {
                    bail!(SemanticAnalysisError::UndeclaredIdentifier(
                        ident.name.clone()
                    ))
                }
            }
            StatementVariant::FunctionCall { name, args } => {
                let func = self
                    .functions
                    .get(&name.name)
                    .ok_or_else(|| SemanticAnalysisError::UndefinedFunction(name.name.clone()))?;

                self.validate_function_args(args, func.0)?;
                Ok(ControlFlow::Continues)
            }
            StatementVariant::Return(expr) => {
                let return_type = self.resolve_expr(expr)?;

                if return_type != parent.ret_sig {
                    bail!(SemanticAnalysisError::TypeMismatch(
                        parent.ret_sig.to_owned(),
                        return_type
                    ));
                }
                Ok(ControlFlow::Returns)
            }
        }
    }

    fn resolve_else(&mut self, els: &ElseClause, parent: &Function) -> Result<ControlFlow> {
        if let Some(c) = &els.cond {
            let _ = self.resolve_expr(c)?;
        }

        let flow = self.resolve_block(&els.body, parent, true)?;

        let next_flow = match &*els.els {
            Some(next) => self.resolve_else(next, parent)?,
            None => ControlFlow::Returns,
        };

        if flow == ControlFlow::Returns && next_flow == ControlFlow::Returns {
            Ok(ControlFlow::Returns)
        } else {
            Ok(ControlFlow::Continues)
        }
    }

    fn resolve_block(
        &mut self,
        block: &[Statement],
        parent: &Function,
        new_scope: bool,
    ) -> Result<ControlFlow> {
        if new_scope {
            self.symbols.push(HashMap::new());
        }

        for stmt in block {
            let flow = self.resolve_statement(stmt, parent)?;

            if flow == ControlFlow::Returns {
                // TODO: Warn on unreachable code if there's still statements?
                return Ok(ControlFlow::Returns);
            }
        }

        if new_scope {
            self.symbols.pop();
        }
        Ok(ControlFlow::Continues)
    }

    fn resolve_expr(&self, expr: &Expression) -> Result<Type> {
        match &expr.variant {
            ExpressionVariant::BinaryExpr(lhs, rhs, _bin_op) => {
                let lhs_type = self.resolve_expr(lhs)?;
                let rhs_type = self.resolve_expr(rhs)?;

                if lhs_type != rhs_type {
                    bail!(SemanticAnalysisError::TypeMismatch(
                        lhs_type.to_owned(),
                        rhs_type
                    ));
                }

                // TODO: verify <bin_op> can be applied to <lhs_type> and <rhs_type>

                Ok(lhs_type)
            }
            ExpressionVariant::Term(term) => self.resolve_term(term),
            ExpressionVariant::FunctionCall { name, args } => {
                let ctx = self
                    .functions
                    .get(&name.name)
                    .ok_or_else(|| SemanticAnalysisError::UndefinedFunction(name.name.clone()))?;

                self.validate_function_args(args, ctx.0)?;

                Ok(ctx.1.to_owned())
            }
        }
    }

    fn resolve_term(&self, term: &Term) -> Result<Type> {
        match term {
            Term::IntLit(_) => Ok(Type::Int),
            Term::Bool(_) => Ok(Type::Bool),
            Term::Identifier(ident) => self.find_identifier(ident).cloned().ok_or_else(|| {
                anyhow::anyhow!(SemanticAnalysisError::UndeclaredIdentifier(ident.clone()))
            }),
        }
    }

    fn validate_function_args(&self, args: &[Expression], expected: &[Argument]) -> Result<()> {
        for (arg, expected) in args.iter().zip(expected.iter()) {
            let arg_type = self.resolve_expr(arg)?;
            if arg_type != expected.ty {
                bail!(SemanticAnalysisError::TypeMismatch(
                    expected.ty.to_owned(),
                    arg_type
                ));
            }
        }
        Ok(())
    }

    fn find_identifier(&self, ident: &str) -> Option<&Type> {
        self.symbols.iter().rev().find_map(|scope| scope.get(ident))
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
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(
            Lexer::new("fn main() { let x = 2 + 2; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_ok());
    }

    #[test]
    fn test_exit() {
        let ast = Parser::new(Lexer::new("fn main() { exit(false); }").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(
            Lexer::new("fn main() { exit(2 + false); }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(Lexer::new("fn main() { exit(2); }").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(Lexer::new("fn main() { exit(2 + 2); }").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = Analyzer::new(&ast);
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
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(
            Lexer::new("fn main() { let x = 2; x = false; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = Analyzer::new(&ast);
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
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_ok());

        let ast = Parser::new(
            Lexer::new("fn main() = int { return false; }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = Analyzer::new(&ast);
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
        let mut checker = Analyzer::new(&ast);
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
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_err());
    }

    #[test]
    fn test_return_path_validation() {
        let ast = Parser::new(
            Lexer::new(
                "fn main() { let x = 1; let y = inc(x); } fn inc(x: int) = int { if x == 1 { return 2; } else { let y = 3; } }",
            )
            .tokenize()
            .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(
            Lexer::new(
                "fn main() { let x = 1; let y = inc(x); } fn inc(x: int) = int { if x == 1 { return 2; } else { let y = 3; } return x + 1; }",
            )
            .tokenize()
            .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_ok());
    }

    #[test]
    fn test_variable_scope() {
        let ast = Parser::new(Lexer::new("fn main() { x = 1; }").tokenize().unwrap())
            .parse()
            .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_err());

        let ast = Parser::new(
            Lexer::new("fn main() { if true { let x = 1; } exit(x); }")
                .tokenize()
                .unwrap(),
        )
        .parse()
        .unwrap();
        let mut checker = Analyzer::new(&ast);
        assert!(checker.check().is_err());
    }
}
