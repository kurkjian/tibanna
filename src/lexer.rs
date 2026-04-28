use anyhow::Result;
use std::{iter::Peekable, str::Chars};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("unknown token: {0}")]
    UnknownToken(String),
    #[error("invalid number: {0}")]
    InvalidNumber(String),
}

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

pub struct Lexer<'a> {
    text: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        let mut iter = self.text.chars().peekable();

        while let Some(char) = iter.peek() {
            match char {
                char if char.is_alphabetic() => tokens.push(self.string(&mut iter)),
                char if char.is_ascii_digit() => tokens.push(self.number(&mut iter)?),
                '(' => {
                    iter.next();
                    tokens.push(Token::OpenParen);
                }
                ')' => {
                    iter.next();
                    tokens.push(Token::CloseParen);
                }
                '{' => {
                    iter.next();
                    tokens.push(Token::OpenBrace);
                }
                '}' => {
                    iter.next();
                    tokens.push(Token::CloseBrace);
                }
                ';' => {
                    iter.next();
                    tokens.push(Token::Semi);
                }
                '=' => {
                    iter.next();
                    if iter.peek() == Some(&'=') {
                        iter.next();
                        tokens.push(Token::EqEq);
                    } else {
                        tokens.push(Token::Equal);
                    }
                }
                '+' => {
                    iter.next();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    iter.next();
                    tokens.push(Token::Minus);
                }
                '*' => {
                    iter.next();
                    tokens.push(Token::Star);
                }
                '/' => {
                    iter.next();
                    if iter.peek() == Some(&'/') {
                        while iter.peek().is_some_and(|x| *x != '\n') {
                            iter.next();
                        }
                    }
                }
                '<' => {
                    iter.next();
                    if iter.peek() == Some(&'=') {
                        iter.next();
                        tokens.push(Token::Leq);
                    } else {
                        tokens.push(Token::Lt);
                    }
                }
                '>' => {
                    iter.next();
                    if iter.peek() == Some(&'=') {
                        iter.next();
                        tokens.push(Token::Geq);
                    } else {
                        tokens.push(Token::Gt);
                    }
                }
                '!' => {
                    iter.next();
                    if iter.peek() == Some(&'=') {
                        iter.next();
                        tokens.push(Token::Neq);
                    } else {
                        tokens.push(Token::Bang);
                    }
                }
                '&' => {
                    iter.next();
                    if iter.peek() == Some(&'&') {
                        iter.next();
                        tokens.push(Token::LogicalAnd);
                    } else {
                        tokens.push(Token::Ampersand);
                    }
                }
                '|' => {
                    iter.next();
                    if iter.peek() == Some(&'|') {
                        iter.next();
                        tokens.push(Token::LogicalOr);
                    } else {
                        tokens.push(Token::Pipe);
                    }
                }

                char if char.is_whitespace() => {
                    iter.next();
                }

                _ => {
                    return Err(LexerError::UnknownToken(char.to_string()));
                }
            }
        }

        Ok(tokens)
    }

    fn take_while<F>(&mut self, iter: &mut Peekable<Chars>, cond: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut buf = String::new();

        while let Some(&c) = iter.peek() {
            if cond(c) {
                buf.push(c);
                iter.next();
            } else {
                break;
            }
        }

        buf
    }

    fn string(&mut self, iter: &mut Peekable<Chars>) -> Token {
        let ident = self.take_while(iter, |c| c.is_alphanumeric());
        match ident.as_str() {
            "exit" => Token::Exit,
            "let" => Token::Let,
            "if" => Token::If,
            "else" => Token::Else,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(ident),
        }
    }

    fn number(&mut self, iter: &mut Peekable<Chars>) -> Result<Token, LexerError> {
        let num = self
            .take_while(iter, |c| c.is_ascii_digit())
            .parse::<usize>()
            .map_err(|e| LexerError::InvalidNumber(e.to_string()))?;

        Ok(Token::Int(num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit() {
        let input = "exit";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens, vec![Token::Exit]);
    }

    #[test]
    fn test_int() {
        let input = "123";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens, vec![Token::Int(123)]);
    }

    #[test]
    fn test_exit_statement() {
        let input = "exit(69);";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Exit,
                Token::OpenParen,
                Token::Int(69),
                Token::CloseParen,
                Token::Semi
            ]
        );
    }

    #[test]
    fn test_let_assignment() {
        let input = "let x = 420;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident("x".to_string()),
                Token::Equal,
                Token::Int(420),
                Token::Semi
            ]
        );
    }
}
