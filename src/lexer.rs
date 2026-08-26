use std::{fmt, iter::Peekable, str::CharIndices};

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

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Keyword::Select => "SELECT",
            Keyword::From => "FROM",
            Keyword::Where => "WHERE",
            Keyword::Insert => "INSERT",
            Keyword::Into => "INTO",
            Keyword::Values => "VALUES",
            Keyword::Create => "CREATE",
            Keyword::Table => "TABLE",
            Keyword::And => "AND",
            Keyword::Or => "OR",
            Keyword::Integer => "INTEGER",
            Keyword::Text => "TEXT",
        };

        f.write_str(text)
    }
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(f, "{keyword}"),
            Token::Ident(string) => write!(f, "{string}"),
            Token::Int(int) => write!(f, "{int}"),
            Token::Str(string) => write!(f, "'{string}'"),
            Token::Comma => f.write_str(","),
            Token::Semicolon => f.write_str(";"),
            Token::LParen => f.write_str("("),
            Token::RParen => f.write_str(")"),
            Token::Star => f.write_str("*"),
            Token::Eq => f.write_str("="),
            Token::NotEq => f.write_str("!="),
            Token::Lt => f.write_str("<"),
            Token::LtEq => f.write_str("<="),
            Token::Gt => f.write_str(">"),
            Token::GtEq => f.write_str(">="),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar { ch: char, at: usize },
    UnterminatedString { at: usize },
    NumberOutOfRange { text: String, at: usize },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar { ch, at } => {
                write!(f, "unexpected character '{}' at position {}", ch, at)
            }
            LexError::UnterminatedString { at } => {
                write!(f, "unterminated string starting at position {}", at)
            }
            LexError::NumberOutOfRange { text, at } => {
                write!(f, "number out of range at position {}: {}", at, text)
            }
        }
    }
}

impl std::error::Error for LexError {}

struct Lexer<'a> {
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.char_indices().peekable(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(&(pos, c)) = self.chars.peek() {
            match c {
                _ if c.is_ascii_whitespace() => {
                    self.chars.next();
                }
                _ if c.is_ascii_digit() => tokens.push(self.tokenize_number(pos)?),
                _ if c.is_ascii_alphabetic() || c == '_' => {
                    tokens.push(self.tokenize_ident_or_keyword())
                }
                '\'' => tokens.push(self.tokenize_string(pos)?),
                _ => tokens.push(self.tokenize_symbol(c, pos)?),
            }
        }
        Ok(tokens)
    }

    fn tokenize_number(&mut self, pos: usize) -> Result<Token, LexError> {
        let nb = self.take_while(|c| c.is_ascii_digit());
        match nb.parse::<i64>() {
            Ok(parsed_nb) => Ok(Token::Int(parsed_nb)),
            Err(_) => Err(LexError::NumberOutOfRange { text: nb, at: pos }),
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
        while let Some(&(_, c)) = self.chars.peek() {
            if pred(c) {
                expression.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        expression
    }

    fn tokenize_string(&mut self, pos: usize) -> Result<Token, LexError> {
        let mut str_data = String::new();
        self.chars.next();
        while let Some((_, c)) = self.chars.next() {
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
        Err(LexError::UnterminatedString { at: pos })
    }

    fn tokenize_symbol(&mut self, c: char, pos: usize) -> Result<Token, LexError> {
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
                    Err(LexError::UnexpectedChar { ch: '!', at: pos })
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
            _ => Err(LexError::UnexpectedChar { ch: c, at: pos }),
        }
    }

    fn eat(&mut self, expected: char) -> bool {
        match self.chars.peek() {
            Some(&(_, c)) if c == expected => {
                self.chars.next();
                true
            }
            _ => false,
        }
    }
}
