use std::{iter::Peekable, str::Chars};

pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(src).tokenize()
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Create,
    Table,
    And,
    Or,
    Integer,
    Text,
}

impl Keyword {
    fn create(src: &str) -> Option<Keyword> {
        match src.to_ascii_lowercase().as_str() {
            "select" => Some(Keyword::Select),
            "from" => Some(Keyword::From),
            "where" => Some(Keyword::Where),
            "insert" => Some(Keyword::Insert),
            "into" => Some(Keyword::Into),
            "values" => Some(Keyword::Values),
            "create" => Some(Keyword::Create),
            "table" => Some(Keyword::Table),
            "and" => Some(Keyword::And),
            "or" => Some(Keyword::Or),
            "integer" => Some(Keyword::Integer),
            "text" => Some(Keyword::Text),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Ident(String),
    Int(i64),
    Str(String),
    Comma,
    Semicolon,
    LParen,
    RParen,
    Star,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar(char),
    UnterminatedString,
    NumberOutOfRange(String),
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.chars().peekable(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(&c) = self.chars.peek() {
            match c {
                _ if c.is_ascii_whitespace() => {
                    self.chars.next();
                }
                _ if c.is_ascii_digit() => tokens.push(self.tokenize_number()?),
                _ if c.is_ascii_alphabetic() || c == '_' => {
                    tokens.push(self.tokenize_ident_or_keyword())
                }
                '\'' => tokens.push(self.tokenize_string()?),
                _ => tokens.push(self.tokenize_symbol(c)?),
            }
        }
        Ok(tokens)
    }

    fn tokenize_number(&mut self) -> Result<Token, LexError> {
        let nb = self.take_while(|c| c.is_ascii_digit());
        match nb.parse::<i64>() {
            Ok(parsed_nb) => Ok(Token::Int(parsed_nb)),
            Err(_) => Err(LexError::NumberOutOfRange(nb)),
        }
    }

    fn tokenize_ident_or_keyword(&mut self) -> Token {
        let expression = self.take_while(|c| c.is_ascii_alphanumeric() || c == '_');
        match Keyword::create(&expression) {
            Some(keyword) => Token::Keyword(keyword),
            None => Token::Ident(expression),
        }
    }

    fn take_while(&mut self, pred: impl Fn(char) -> bool) -> String {
        let mut expression = String::new();
        while let Some(&c) = self.chars.peek() {
            if pred(c) {
                expression.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        expression
    }

    fn tokenize_string(&mut self) -> Result<Token, LexError> {
        let mut str_data = String::new();
        self.chars.next();
        while let Some(c) = self.chars.next() {
            if c == '\'' {
                if self.eat('\'') {
                    str_data.push('\'');
                } else {
                    return Ok(Token::Str(str_data));
                }
            } else {
                str_data.push(c);
            }
        }
        Err(LexError::UnterminatedString)
    }

    fn tokenize_symbol(&mut self, c: char) -> Result<Token, LexError> {
        self.chars.next();
        match c {
            '=' => Ok(Token::Eq),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '*' => Ok(Token::Star),
            '!' => {
                if self.eat('=') {
                    Ok(Token::NotEq)
                } else {
                    Err(LexError::UnexpectedChar('!'))
                }
            }
            '<' => {
                if self.eat('>') {
                    Ok(Token::NotEq)
                } else if self.eat('=') {
                    Ok(Token::LtEq)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                if self.eat('=') {
                    Ok(Token::GtEq)
                } else {
                    Ok(Token::Gt)
                }
            }
            _ => Err(LexError::UnexpectedChar(c)),
        }
    }

    fn eat(&mut self, expected: char) -> bool {
        if Some(&expected) == self.chars.peek() {
            self.chars.next();
            true
        } else {
            false
        }
    }
}
