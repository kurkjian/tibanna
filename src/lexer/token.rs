#[derive(Debug, PartialEq, Eq, Clone, strum_macros::Display)]
pub enum Token {
    Exit,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Int(usize),
    Semi,
    Let,
    Ident(String),
    Equal,
    Plus,
    Minus,
    Star,
    If,
    Else,
    Lt,
    Leq,
    Gt,
    Geq,
    EqEq,
    Neq,
    Bang,
    True,
    False,
    Ampersand,
    Pipe,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, PartialEq, Eq, Clone, strum_macros::Display)]
pub enum TokenKind {
    Term, // ?
    Exit,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Int,
    Semi,
    Let,
    Identifier,
    Equal,
    Plus,
    Minus,
    Star,
    If,
    Else,
    Lt,
    Leq,
    Gt,
    Geq,
    EqEq,
    Neq,
    Bang,
    True,
    False,
    Ampersand,
    Pipe,
    LogicalAnd,
    LogicalOr,
}

impl Token {
    pub fn is_bool(&self) -> bool {
        matches!(
            self,
            Token::Lt
                | Token::Leq
                | Token::Gt
                | Token::Geq
                | Token::EqEq
                | Token::Neq
                | Token::LogicalAnd
                | Token::LogicalOr
        )
    }

    pub fn is_binary_op(&self) -> bool {
        matches!(self, Token::Plus | Token::Minus | Token::Star)
    }

    pub fn precedence(&self) -> usize {
        match self {
            Token::Plus | Token::Minus => 0,
            Token::Star => 1,
            _ => 0,
        }
    }

    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Int(_) => TokenKind::Int,
            Token::Ident(_) => TokenKind::Identifier,
            Token::OpenBrace => TokenKind::OpenBrace,
            Token::CloseBrace => TokenKind::CloseBrace,
            Token::OpenParen => TokenKind::OpenParen,
            Token::CloseParen => TokenKind::CloseParen,
            Token::Semi => TokenKind::Semi,
            Token::Let => TokenKind::Let,
            Token::Equal => TokenKind::Equal,
            Token::Plus => TokenKind::Plus,
            Token::Minus => TokenKind::Minus,
            Token::Star => TokenKind::Star,
            Token::If => TokenKind::If,
            Token::Else => TokenKind::Else,
            Token::Lt => TokenKind::Lt,
            Token::Leq => TokenKind::Leq,
            Token::Gt => TokenKind::Gt,
            Token::Geq => TokenKind::Geq,
            Token::EqEq => TokenKind::EqEq,
            Token::Neq => TokenKind::Neq,
            Token::Bang => TokenKind::Bang,
            Token::True => TokenKind::True,
            Token::False => TokenKind::False,
            Token::Ampersand => TokenKind::Ampersand,
            Token::Pipe => TokenKind::Pipe,
            Token::LogicalAnd => TokenKind::LogicalAnd,
            Token::LogicalOr => TokenKind::LogicalOr,
            Token::Exit => TokenKind::Exit,
        }
    }
}
