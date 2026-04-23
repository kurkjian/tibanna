use anyhow::Result;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Exit,
    OpenParen,
    CloseParen,
    Int(usize),
    Semi,
    Let,
    Ident(String),
    Equal,
    Plus,
    Minus,
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
        let mut str_buffer = String::new();
        let mut iter = self.text.chars().peekable();

        while let Some(char) = iter.peek() {
            if char.is_alphabetic() {
                str_buffer.push(iter.next().unwrap());

                while let Some(next) = iter.peek()
                    && next.is_alphanumeric()
                {
                    str_buffer.push(iter.next().unwrap());
                }

                match str_buffer.as_str() {
                    "exit" => tokens.push(Token::Exit),
                    "let" => tokens.push(Token::Let),
                    _ => tokens.push(Token::Ident(str_buffer.clone())),
                }
            } else if char.is_ascii_digit() {
                str_buffer.push(iter.next().unwrap());

                while let Some(next) = iter.peek()
                    && next.is_ascii_digit()
                {
                    str_buffer.push(iter.next().unwrap());
                }

                tokens.push(Token::Int(str_buffer.parse::<usize>()?));
            } else if *char == '(' {
                tokens.push(Token::OpenParen);
                iter.next();
            } else if *char == ')' {
                tokens.push(Token::CloseParen);
                iter.next();
            } else if *char == ';' {
                tokens.push(Token::Semi);
                iter.next();
            } else if *char == '=' {
                tokens.push(Token::Equal);
                iter.next();
            } else if *char == '+' {
                tokens.push(Token::Plus);
                iter.next();
            } else if *char == '-' {
                tokens.push(Token::Minus);
                iter.next();
            } else if char.is_whitespace() {
                iter.next();
            } else {
                return Err(anyhow::anyhow!("Unknown token: {}", char));
            }

            str_buffer.clear();
        }

        Ok(tokens)
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
