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
        els: Option<ElseClause>,
    },
    Assignment {
        ident: Identifier,
        expr: Expression,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Expression {
    pub variant: ExpressionVariant,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionVariant {
    BinaryExpr(Box<Expression>, Box<Expression>, BinOp),
    Term(Term),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ElseClause {
    pub cond: Option<Expression>,
    pub body: Vec<Statement>,
    pub els: Box<Option<ElseClause>>,
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
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

                    let els = self.parse_else();
                    Statement {
                        variant: StatementVariant::If {
                            cond,
                            then: body,
                            els,
                        },
                    }
                }
                Token::Ident(name) => {
                    let name = name.to_owned();
                    self.inc();

                    self.parse_eq();
                    let expr = self.parse_expr();
                    Statement {
                        variant: StatementVariant::Assignment {
                            ident: Identifier { name },
                            expr,
                        },
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

            if self.peek().is_some_and(|x| x.is_binary_op()) {
                let expr = self.climb_precedence(expr, 0);
                if self.peek().is_some_and(|x| x.is_bool()) {
                    let op = self.peek().unwrap().to_owned();
                    self.inc();

                    let rhs = self.parse_expr();
                    return Expression {
                        variant: ExpressionVariant::BinaryExpr(
                            Box::new(expr),
                            Box::new(rhs),
                            BinOp::from(op),
                        ),
                    };
                }

                return expr;
            } else if self.peek().is_some_and(|x| x.is_bool()) {
                let op = self.peek().unwrap().to_owned();
                self.inc();
                let rhs = self.parse_expr();
                return Expression {
                    variant: ExpressionVariant::BinaryExpr(
                        Box::new(expr),
                        Box::new(rhs),
                        BinOp::from(op),
                    ),
                };
            }

            return expr;
        }

        todo!("Error handling: Expected expression");
    }

    fn climb_precedence(&mut self, mut lhs: Expression, min_precedence: usize) -> Expression {
        let mut lookahead = self.peek().unwrap().to_owned();
        while lookahead.is_binary_op() && lookahead.precedence() >= min_precedence {
            let op = lookahead;
            self.inc();
            let mut rhs = Expression {
                variant: ExpressionVariant::Term(self.parse_term()),
            };

            lookahead = self.peek().unwrap().to_owned();
            while lookahead.is_binary_op() && lookahead.precedence() > op.precedence() {
                // FIXME: this doesn't handle right-associative operators correctly
                rhs = self.climb_precedence(rhs, op.precedence() + 1);
                lookahead = self.peek().unwrap().to_owned();
            }

            lhs = Expression {
                variant: ExpressionVariant::BinaryExpr(Box::new(lhs), Box::new(rhs), op.into()),
            }
        }

        lhs
    }

    fn parse_else(&mut self) -> Option<ElseClause> {
        if let Some(Token::Else) = self.peek() {
            self.inc();

            let mut cond = None;
            if self.peek().is_some_and(|x| *x == Token::If) {
                self.inc();
                cond = Some(self.parse_expr());
            }
            self.parse_brace(true);

            let mut body = Vec::new();
            while !matches!(self.peek(), Some(Token::CloseBrace)) {
                let statement = self.parse_statement();
                body.push(statement);
            }

            self.parse_brace(false);

            Some(ElseClause {
                cond,
                body,
                els: Box::new(self.parse_else()),
            })
        } else {
            None
        }
    }

    fn parse_term(&mut self) -> Term {
        if let Some(token) = self.peek() {
            match token {
                Token::Int(value) => {
                    let value = *value;
                    self.inc();

                    return Term::IntLit(value);
                }
                Token::Ident(name) => {
                    let name = name.to_string();
                    self.inc();

                    return Term::Identifier(name);
                }
                Token::True => {
                    self.inc();
                    return Term::Bool(true);
                }
                Token::False => {
                    self.inc();
                    return Term::Bool(false);
                }
                _ => {
                    todo!("Error handling: Expected term, found: {:?}", token);
                }
            }
        }

        todo!("Error handling: Expected token, found none");
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

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn test_exit() {
        let tokens = Lexer::new("exit(1);").tokenize().unwrap();

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        assert_eq!(
            p,
            Program {
                statements: vec![exit(int(1))],
            }
        );
    }

    #[test]
    fn test_let_assignment() {
        let tokens = Lexer::new("let x = 420;").tokenize().unwrap();

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        assert_eq!(
            p,
            Program {
                statements: vec![let_("x", int(420))],
            }
        );
    }

    #[test]
    fn test_bin_op() {
        let tokens = Lexer::new("let x = y + 2;").tokenize().unwrap();

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        assert_eq!(
            p,
            Program {
                statements: vec![let_("x", bin(ident("y"), BinOp::Add, int(2)))],
            }
        );
    }

    #[test]
    fn test_cond_simple_inequality() {
        let tokens = Lexer::new("if x < y { let z = 1; }").tokenize().unwrap();

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        let cond = bin(ident("x"), BinOp::Lt, ident("y"));
        assert_eq!(
            p,
            Program {
                statements: vec![if_(cond, vec![let_("z", int(1))])],
            }
        );
    }

    #[test]
    fn test_cond_ineq_with_ops() {
        let tokens = Lexer::new("if x + 1 < y * z { let w = 1; }")
            .tokenize()
            .unwrap();

        let mut parser = Parser::new(tokens);
        let p = parser.parse();

        let cond = bin(
            bin(ident("x"), BinOp::Add, int(1)),
            BinOp::Lt,
            bin(ident("y"), BinOp::Mul, ident("z")),
        );
        assert_eq!(
            p,
            Program {
                statements: vec![if_(cond, vec![let_("w", int(1))])],
            }
        );
    }

    fn ident(name: &str) -> Expression {
        Expression {
            variant: ExpressionVariant::Term(Term::Identifier(name.to_string())),
        }
    }

    fn int(n: usize) -> Expression {
        Expression {
            variant: ExpressionVariant::Term(Term::IntLit(n)),
        }
    }

    fn exit(expr: Expression) -> Statement {
        Statement {
            variant: StatementVariant::Exit(expr),
        }
    }

    fn bin(lhs: Expression, op: BinOp, rhs: Expression) -> Expression {
        Expression {
            variant: ExpressionVariant::BinaryExpr(Box::new(lhs), Box::new(rhs), op),
        }
    }

    fn let_(name: &str, expr: Expression) -> Statement {
        Statement {
            variant: StatementVariant::Let {
                ident: Identifier {
                    name: name.to_string(),
                },
                expr,
            },
        }
    }

    fn if_(cond: Expression, then: Vec<Statement>) -> Statement {
        Statement {
            variant: StatementVariant::If {
                cond,
                then,
                els: None,
            },
        }
    }
}
