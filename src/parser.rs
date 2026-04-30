use crate::lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unexpected token: found {0}, expected {1}")]
    UnexpectedToken(Token, TokenKind),
    #[error("missing token: expected {0}")]
    MissingToken(TokenKind),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub args: Vec<Argument>,
    pub ret_sig: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Argument {
    pub name: Identifier,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Void,
    Int,
    Bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Statement {
    pub variant: StatementVariant,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StatementVariant {
    Exit(Expression),
    Let {
        ident: Identifier,
        expr: Expression,
    },
    If {
        cond: Expression,
        then: Vec<Statement>,
        els: Option<ElseClause>,
    },
    While {
        cond: Expression,
        body: Vec<Statement>,
    },
    Assignment {
        ident: Identifier,
        expr: Expression,
    },
    FunctionCall {
        name: Identifier,
        args: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Expression {
    pub variant: ExpressionVariant,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpressionVariant {
    BinaryExpr(Box<Expression>, Box<Expression>, BinOp),
    Term(Term),
    FunctionCall {
        name: Identifier,
        args: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct ElseClause {
    pub cond: Option<Expression>,
    pub body: Vec<Statement>,
    pub els: Box<Option<ElseClause>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Term {
    Identifier(String),
    IntLit(usize),
    Bool(bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Lt,
    Leq,
    Gt,
    Geq,
    Eq,
    Neq,
    And,
    Or,
}

impl BinOp {
    pub fn is_cmp(&self) -> bool {
        matches!(
            self,
            BinOp::Lt
                | BinOp::Leq
                | BinOp::Gt
                | BinOp::Geq
                | BinOp::Eq
                | BinOp::Neq
                | BinOp::And
                | BinOp::Or
        )
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, BinOp::And | BinOp::Or)
    }
}

impl From<Token> for BinOp {
    fn from(token: Token) -> Self {
        match token {
            Token::Plus => BinOp::Add,
            Token::Minus => BinOp::Sub,
            Token::Star => BinOp::Mul,
            Token::Lt => BinOp::Lt,
            Token::Leq => BinOp::Leq,
            Token::Gt => BinOp::Gt,
            Token::Geq => BinOp::Geq,
            Token::EqEq => BinOp::Eq,
            Token::Neq => BinOp::Neq,
            Token::LogicalAnd => BinOp::And,
            Token::LogicalOr => BinOp::Or,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Identifier {
    pub name: String,
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn inc(&mut self) {
        self.index += 1;
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut functions = Vec::new();
        while self.peek().is_some() {
            functions.push(self.parse_function()?);
        }

        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        self.parse_token(Token::Fn)?;
        let fn_name = self.parse_ident()?;
        self.parse_token(Token::OpenParen)?;
        let args = self.parse_fn_args()?;
        self.parse_token(Token::CloseParen)?;

        let ret_sig = self.parse_return_signature()?;

        // TODO: pull this into a fn + combine with Token::While
        self.parse_token(Token::OpenBrace)?;
        let mut body = Vec::new();
        while !matches!(self.peek(), Some(Token::CloseBrace)) {
            let statement = self.parse_statement()?;
            body.push(statement);
        }
        self.parse_token(Token::CloseBrace)?;

        Ok(Function {
            name: fn_name,
            args,
            ret_sig,
            body,
        })
    }

    fn parse_fn_args(&mut self) -> Result<Vec<Argument>, ParseError> {
        let mut args = Vec::new();
        while self.peek() != Some(&Token::CloseParen) {
            let arg_name = self.parse_ident()?;
            self.parse_token(Token::Colon)?;
            let arg_type = self.parse_type()?;

            args.push(Argument {
                name: arg_name,
                ty: arg_type,
            });

            if self.peek() == Some(&Token::Comma) {
                self.inc();
            }
        }

        Ok(args)
    }

    fn parse_return_signature(&mut self) -> Result<Type, ParseError> {
        if let Some(Token::Equal) = self.peek() {
            self.inc();
            let ty = self.parse_type()?;
            return Ok(ty);
        }

        Ok(Type::Void)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.peek() {
            Some(Token::Int) => {
                self.inc();
                Ok(Type::Int)
            }
            Some(Token::Bool) => {
                self.inc();
                Ok(Type::Bool)
            }
            _ => Err(ParseError::UnexpectedToken(
                self.peek().unwrap().clone(),
                TokenKind::Type,
            )),
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let mut end_of_scope = false;
        if let Some(token) = self.peek() {
            let statement = match token {
                Token::Exit => {
                    self.inc();

                    self.parse_token(Token::OpenParen)?;
                    let expr = self.parse_expr()?;
                    self.parse_token(Token::CloseParen)?;

                    Statement {
                        variant: StatementVariant::Exit(expr),
                    }
                }
                Token::Let => {
                    self.inc();

                    let ident = self.parse_ident()?;
                    self.parse_token(Token::Equal)?;
                    let expr = self.parse_expr()?;

                    Statement {
                        variant: StatementVariant::Let { ident, expr },
                    }
                }
                Token::If => {
                    self.inc();

                    let cond = self.parse_expr()?;
                    self.parse_token(Token::OpenBrace)?;
                    let mut body = Vec::new();
                    while !matches!(self.peek(), Some(Token::CloseBrace)) {
                        let statement = self.parse_statement()?;
                        body.push(statement);
                    }
                    self.parse_token(Token::CloseBrace)?;

                    end_of_scope = true;

                    let els = self.parse_else()?;
                    Statement {
                        variant: StatementVariant::If {
                            cond,
                            then: body,
                            els,
                        },
                    }
                }
                Token::While => {
                    self.inc();

                    let cond = self.parse_expr()?;
                    self.parse_token(Token::OpenBrace)?;
                    let mut body = Vec::new();
                    while !matches!(self.peek(), Some(Token::CloseBrace)) {
                        let statement = self.parse_statement()?;
                        body.push(statement);
                    }
                    self.parse_token(Token::CloseBrace)?;

                    end_of_scope = true;
                    Statement {
                        variant: StatementVariant::While { cond, body },
                    }
                }
                Token::Ident(name) => {
                    let name = name.to_owned();
                    self.inc();

                    if matches!(self.peek(), Some(Token::OpenParen)) {
                        self.inc();
                        let mut args = Vec::new();
                        while self.peek() != Some(&Token::CloseParen) {
                            let expr = self.parse_expr()?;
                            args.push(expr);
                            if self.peek() == Some(&Token::Comma) {
                                self.inc();
                            }
                        }
                        self.inc();

                        Statement {
                            variant: StatementVariant::FunctionCall {
                                name: Identifier { name },
                                args,
                            },
                        }
                    } else {
                        self.parse_token(Token::Equal)?;
                        let expr = self.parse_expr()?;
                        Statement {
                            variant: StatementVariant::Assignment {
                                ident: Identifier { name },
                                expr,
                            },
                        }
                    }
                }
                _ => {
                    todo!("Error handling: Unexpected token: {:?}", token);
                }
            };

            let next = self.peek();
            if matches!(next, Some(Token::Semi)) {
                self.inc();
            } else if !end_of_scope {
                todo!(
                    "Error handling: Expected semicolon after statement: {:?}",
                    next
                );
            }

            return Ok(statement);
        }

        todo!("Error handling: Expected statement");
    }

    fn parse_expr(&mut self) -> Result<Expression, ParseError> {
        if let Some(token) = self.peek() {
            let expr = Expression::try_from(token.to_owned())
                .expect("Could not convert token to expression");
            self.inc();

            if self.peek().is_some_and(|x| x.is_binary_op()) {
                let expr = self.climb_precedence(expr, 0)?;
                if self.peek().is_some_and(|x| x.is_bool()) {
                    let op = self.peek().unwrap().to_owned();
                    self.inc();

                    let rhs = self.parse_expr()?;
                    return Ok(Expression {
                        variant: ExpressionVariant::BinaryExpr(
                            Box::new(expr),
                            Box::new(rhs),
                            BinOp::from(op),
                        ),
                    });
                }

                return Ok(expr);
            } else if self.peek().is_some_and(|x| x.is_bool()) {
                let op = self.peek().unwrap().to_owned();
                self.inc();
                let rhs = self.parse_expr()?;
                return Ok(Expression {
                    variant: ExpressionVariant::BinaryExpr(
                        Box::new(expr),
                        Box::new(rhs),
                        BinOp::from(op),
                    ),
                });
            }

            return Ok(expr);
        }

        todo!("Error handling: Expected expression");
    }

    fn climb_precedence(
        &mut self,
        mut lhs: Expression,
        min_precedence: usize,
    ) -> Result<Expression, ParseError> {
        let mut lookahead = self.peek().unwrap().to_owned();
        while lookahead.is_binary_op() && lookahead.precedence() >= min_precedence {
            let op = lookahead;
            self.inc();
            let mut rhs = Expression {
                variant: ExpressionVariant::Term(self.parse_term()?),
            };

            lookahead = self.peek().unwrap().to_owned();
            while lookahead.is_binary_op() && lookahead.precedence() > op.precedence() {
                // FIXME: this doesn't handle right-associative operators correctly
                rhs = self.climb_precedence(rhs, op.precedence() + 1)?;
                lookahead = self.peek().unwrap().to_owned();
            }

            lhs = Expression {
                variant: ExpressionVariant::BinaryExpr(Box::new(lhs), Box::new(rhs), op.into()),
            }
        }

        Ok(lhs)
    }

    fn parse_else(&mut self) -> Result<Option<ElseClause>, ParseError> {
        if let Some(Token::Else) = self.peek() {
            self.inc();

            let mut cond = None;
            if self.peek().is_some_and(|x| *x == Token::If) {
                self.inc();
                cond = Some(self.parse_expr()?);
            }
            self.parse_token(Token::OpenBrace)?;

            let mut body = Vec::new();
            while !matches!(self.peek(), Some(Token::CloseBrace)) {
                let statement = self.parse_statement()?;
                body.push(statement);
            }

            self.parse_token(Token::CloseBrace)?;

            Ok(Some(ElseClause {
                cond,
                body,
                els: Box::new(self.parse_else()?),
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_term(&mut self) -> Result<Term, ParseError> {
        match self.peek() {
            Some(Token::IntLit(value)) => {
                let v = *value;
                self.inc();
                Ok(Term::IntLit(v))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.inc();
                Ok(Term::Identifier(name))
            }
            Some(Token::True) | Some(Token::False) => {
                let val = matches!(self.peek(), Some(Token::True));
                self.inc();
                Ok(Term::Bool(val))
            }
            Some(token) => Err(ParseError::UnexpectedToken(
                token.clone(),
                TokenKind::IntLit,
            )),
            None => Err(ParseError::MissingToken(TokenKind::Term)),
        }
    }

    fn parse_ident(&mut self) -> Result<Identifier, ParseError> {
        match self.peek() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.inc();
                Ok(Identifier { name })
            }
            Some(token) => Err(ParseError::UnexpectedToken(
                token.clone(),
                TokenKind::Identifier,
            )),
            None => Err(ParseError::MissingToken(TokenKind::Identifier)),
        }
    }

    fn parse_token(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.peek() {
            Some(token) if *token == expected => {
                self.inc();
                Ok(())
            }
            Some(token) => Err(ParseError::UnexpectedToken(token.clone(), expected.kind())),
            None => Err(ParseError::MissingToken(expected.kind())),
        }
    }
}

impl TryFrom<Token> for Expression {
    type Error = ();

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::IntLit(n) => {
                let variant = ExpressionVariant::Term(Term::IntLit(n));
                Ok(Expression { variant })
            }
            Token::Ident(ident) => {
                let variant = ExpressionVariant::Term(Term::Identifier(ident.to_string()));
                Ok(Expression { variant })
            }
            Token::True => {
                let variant = ExpressionVariant::Term(Term::Bool(true));
                Ok(Expression { variant })
            }
            Token::False => {
                let variant = ExpressionVariant::Term(Term::Bool(false));
                Ok(Expression { variant })
            }
            _ => Err(()),
        }
    }
}

// FIXME: These tests won't play well with adding functions
#[cfg(test)]
mod tests {
    // use crate::lexer::Lexer;

    // use super::*;

    // #[test]
    // fn test_exit() {
    //     let tokens = Lexer::new("exit(1);").tokenize().unwrap();

    //     let mut parser = Parser::new(tokens);
    //     let p = parser.parse().unwrap();

    //     assert_eq!(
    //         p,
    //         Program {
    //             statements: vec![exit(int(1))],
    //         }
    //     );
    // }

    // #[test]
    // fn test_let_assignment() {
    //     let tokens = Lexer::new("let x = 420;").tokenize().unwrap();

    //     let mut parser = Parser::new(tokens);
    //     let p = parser.parse().unwrap();

    //     assert_eq!(
    //         p,
    //         Program {
    //             statements: vec![let_("x", int(420))],
    //         }
    //     );
    // }

    // #[test]
    // fn test_bin_op() {
    //     let tokens = Lexer::new("let x = y + 2;").tokenize().unwrap();

    //     let mut parser = Parser::new(tokens);
    //     let p = parser.parse().unwrap();

    //     assert_eq!(
    //         p,
    //         Program {
    //             statements: vec![let_("x", bin(ident("y"), BinOp::Add, int(2)))],
    //         }
    //     );
    // }

    // #[test]
    // fn test_cond_simple_inequality() {
    //     let tokens = Lexer::new("if x < y { let z = 1; }").tokenize().unwrap();

    //     let mut parser = Parser::new(tokens);
    //     let p = parser.parse().unwrap();

    //     let cond = bin(ident("x"), BinOp::Lt, ident("y"));
    //     assert_eq!(
    //         p,
    //         Program {
    //             statements: vec![if_(cond, vec![let_("z", int(1))])],
    //         }
    //     );
    // }

    // #[test]
    // fn test_cond_ineq_with_ops() {
    //     let tokens = Lexer::new("if x + 1 < y * z { let w = 1; }")
    //         .tokenize()
    //         .unwrap();

    //     let mut parser = Parser::new(tokens);
    //     let p = parser.parse().unwrap();

    //     let cond = bin(
    //         bin(ident("x"), BinOp::Add, int(1)),
    //         BinOp::Lt,
    //         bin(ident("y"), BinOp::Mul, ident("z")),
    //     );
    //     assert_eq!(
    //         p,
    //         Program {
    //             statements: vec![if_(cond, vec![let_("w", int(1))])],
    //         }
    //     );
    // }

    // fn ident(name: &str) -> Expression {
    //     Expression {
    //         variant: ExpressionVariant::Term(Term::Identifier(name.to_string())),
    //     }
    // }

    // fn int(n: usize) -> Expression {
    //     Expression {
    //         variant: ExpressionVariant::Term(Term::IntLit(n)),
    //     }
    // }

    // fn exit(expr: Expression) -> Statement {
    //     Statement {
    //         variant: StatementVariant::Exit(expr),
    //     }
    // }

    // fn bin(lhs: Expression, op: BinOp, rhs: Expression) -> Expression {
    //     Expression {
    //         variant: ExpressionVariant::BinaryExpr(Box::new(lhs), Box::new(rhs), op),
    //     }
    // }

    // fn let_(name: &str, expr: Expression) -> Statement {
    //     Statement {
    //         variant: StatementVariant::Let {
    //             ident: Identifier {
    //                 name: name.to_string(),
    //             },
    //             expr,
    //         },
    //     }
    // }

    // fn if_(cond: Expression, then: Vec<Statement>) -> Statement {
    //     Statement {
    //         variant: StatementVariant::If {
    //             cond,
    //             then,
    //             els: None,
    //         },
    //     }
    // }
}
