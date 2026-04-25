use crate::lexer::Token;

#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
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
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Expression {
    pub variant: ExpressionVariant,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionVariant {
    BinaryExpr(Box<Expression>, Box<Expression>, BinOp),
    Identifier(String),
    IntLit(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, PartialEq, Eq)]
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

    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while self.peek().is_some() {
            statements.push(self.parse_statement());
        }

        Program { statements }
    }

    fn parse_statement(&mut self) -> Statement {
        let mut end_of_scope = false;
        if let Some(token) = self.peek() {
            let statement = match token {
                Token::Exit => {
                    self.inc();

                    self.parse_paren(true);
                    let expr = self.parse_expr();
                    self.parse_paren(false);

                    Statement {
                        variant: StatementVariant::Exit(expr),
                    }
                }
                Token::Let => {
                    self.inc();

                    let ident = self.parse_ident();
                    self.parse_eq();
                    let expr = self.parse_expr();

                    Statement {
                        variant: StatementVariant::Let { ident, expr },
                    }
                }
                Token::If => {
                    self.inc();

                    let cond = self.parse_expr();
                    self.parse_brace(true);
                    let mut body = Vec::new();
                    while !matches!(self.peek(), Some(Token::CloseBrace)) {
                        let statement = self.parse_statement();
                        body.push(statement);
                    }
                    self.parse_brace(false);

                    end_of_scope = true;
                    Statement {
                        variant: StatementVariant::If { cond, then: body },
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

            return statement;
        }

        todo!("Error handling: Expected statement");
    }

    fn parse_expr(&mut self) -> Expression {
        if let Some(token) = self.peek() {
            let expr = Expression::try_from(token.to_owned())
                .expect("Could not convert token to expression");
            self.inc();

            match self.peek() {
                Some(Token::Plus) => {
                    self.inc();
                    let right = self.parse_expr();
                    return Expression {
                        variant: ExpressionVariant::BinaryExpr(
                            Box::new(expr),
                            Box::new(right),
                            BinOp::Add,
                        ),
                    };
                }
                Some(Token::Minus) => {
                    self.inc();
                    let right = self.parse_expr();
                    return Expression {
                        variant: ExpressionVariant::BinaryExpr(
                            Box::new(expr),
                            Box::new(right),
                            BinOp::Sub,
                        ),
                    };
                }
                Some(Token::Star) => {
                    self.inc();
                    let right = self.parse_expr();
                    return Expression {
                        variant: ExpressionVariant::BinaryExpr(
                            Box::new(expr),
                            Box::new(right),
                            BinOp::Mul,
                        ),
                    };
                }

                _ => {
                    return expr;
                }
            }
        }

        todo!("Error handling: Expected expression");
    }

    fn parse_ident(&mut self) -> Identifier {
        if let Some(token) = self.peek() {
            match token {
                Token::Ident(name) => {
                    let name = name.to_string();
                    self.inc();

                    return Identifier { name };
                }
                _ => {
                    todo!("Error handling: Expected identifier, found: {:?}", token);
                }
            }
        }

        todo!("Error handling: Expected token, found none");
    }

    fn parse_paren(&mut self, open: bool) {
        if let Some(token) = self.peek() {
            match (token, open) {
                (Token::OpenParen, true) | (Token::CloseParen, false) => {
                    self.inc();
                    return;
                }
                _ => {
                    todo!("Error handling: Expected paren, found: {:?}", token);
                }
            }
        }

        todo!("Error handling: Expected paren, found none");
    }

    fn parse_brace(&mut self, open: bool) {
        if let Some(token) = self.peek() {
            match (token, open) {
                (Token::OpenBrace, true) | (Token::CloseBrace, false) => {
                    self.inc();
                    return;
                }
                _ => {
                    todo!("Error handling: Expected brace, found: {:?}", token);
                }
            }
        }

        todo!("Error handling: Expected brace, found none");
    }

    fn parse_eq(&mut self) {
        if let Some(token) = self.peek() {
            match token {
                Token::Equal => {
                    self.inc();
                    return;
                }

                _ => {
                    todo!("Error handling: Expected eq, found: {:?}", token);
                }
            }
        }

        todo!("Error handling: Expected eq, found none");
    }
}

impl TryFrom<Token> for Expression {
    type Error = ();

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Int(n) => {
                let variant = ExpressionVariant::IntLit(n);
                Ok(Expression { variant })
            }
            Token::Ident(ident) => {
                let variant = ExpressionVariant::Identifier(ident.to_string());
                Ok(Expression { variant })
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit() {
        // exit(1);
        let tokens = vec![
            Token::Exit,
            Token::OpenParen,
            Token::Int(1),
            Token::CloseParen,
            Token::Semi,
        ];

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        assert_eq!(
            p,
            Program {
                statements: vec![Statement {
                    variant: StatementVariant::Exit(Expression {
                        variant: ExpressionVariant::IntLit(1),
                    }),
                }],
            }
        )
    }

    #[test]
    fn test_let_assignment() {
        // let x = 420;
        let tokens = vec![
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Int(420),
            Token::Semi,
        ];

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        assert_eq!(
            p,
            Program {
                statements: vec![Statement {
                    variant: StatementVariant::Let {
                        ident: Identifier {
                            name: "x".to_string()
                        },
                        expr: Expression {
                            variant: ExpressionVariant::IntLit(420),
                        },
                    },
                }],
            }
        )
    }

    #[test]
    fn test_bin_op() {
        // let x = y + 2
        let tokens = vec![
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Ident("y".to_string()),
            Token::Plus,
            Token::Int(2),
            Token::Semi,
        ];

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        assert_eq!(
            p,
            Program {
                statements: vec![Statement {
                    variant: StatementVariant::Let {
                        ident: Identifier {
                            name: "x".to_string()
                        },
                        expr: Expression {
                            variant: ExpressionVariant::BinaryExpr(
                                Box::new(Expression {
                                    variant: ExpressionVariant::Identifier("y".to_string()),
                                }),
                                Box::new(Expression {
                                    variant: ExpressionVariant::IntLit(2),
                                }),
                                BinOp::Add
                            ),
                        },
                    },
                }],
            }
        )
    }
}
