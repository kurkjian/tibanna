use anyhow::Result;
use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Eq, Clone)]
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
}

impl Token {
    pub fn is_cmp(&self) -> bool {
        matches!(
            self,
            Token::Lt | Token::Leq | Token::Gt | Token::Geq | Token::EqEq | Token::Neq
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
}

pub struct Lexer<'a> {
    text: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut iter = self.text.chars().peekable();

        while let Some(char) = iter.peek() {
            match char {
                char if char.is_alphabetic() => tokens.push(self.string(&mut iter)?),
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

                char if char.is_whitespace() => {
                    iter.next();
                }

                _ => {
                    return Err(anyhow::anyhow!("Unknown token: {}", char));
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

    fn string(&mut self, iter: &mut Peekable<Chars>) -> Result<Token> {
        let ident = self.take_while(iter, |c| c.is_alphanumeric());
        let token = match ident.as_str() {
            "exit" => Token::Exit,
            "let" => Token::Let,
            "if" => Token::If,
            "else" => Token::Else,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(ident),
        };

        Ok(token)
    }

    fn number(&mut self, iter: &mut Peekable<Chars>) -> Result<Token> {
        let num = self
            .take_while(iter, |c| c.is_ascii_digit())
            .parse::<usize>()?;

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
