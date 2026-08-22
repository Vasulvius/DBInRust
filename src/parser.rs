use std::{iter::Peekable, matches, vec::IntoIter};

use crate::lexer::{Keyword, LexError, Token, tokenize};

pub fn parse(sql: &str) -> Result<Statement, ParseError> {
    let tokens = tokenize(sql)?;
    Parser::new(tokens).parse()
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    CreateTable {
        table: String,
        columns: Vec<ColumnDef>,
    },
    Insert {
        table: String,
        columns: Option<Vec<String>>,
        values: Vec<Value>,
    },
    Select {
        selection: Selection,
        table: String,
        filter: Option<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    Integer,
    Text,
}

#[derive(Debug, PartialEq)]
pub enum Selection {
    All,
    Columns(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
}

#[derive(Debug, PartialEq)]
pub enum CompareOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Column(String),
    Literal(Value),
    Compare {
        left: Box<Expr>,
        op: CompareOp,
        right: Box<Expr>,
    },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnexpectedEnd,
    Lex(LexError),
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> ParseError {
        ParseError::Lex(err)
    }
}

struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn parse(&mut self) -> Result<Statement, ParseError> {
        let statement = match self.next()? {
            Token::Keyword(Keyword::Create) => self.parse_create_table()?,
            Token::Keyword(Keyword::Insert) => self.parse_insert()?,
            Token::Keyword(Keyword::Select) => self.parse_select()?,
            token => return Err(ParseError::UnexpectedToken(token)),
        };
        match self.tokens.next() {
            Some(Token::Semicolon) => match self.tokens.next() {
                Some(token) => Err(ParseError::UnexpectedToken(token)),
                None => Ok(statement),
            },
            Some(token) => Err(ParseError::UnexpectedToken(token)),
            None => Ok(statement),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Keyword(Keyword::Table))?;
        let table = self.ident()?;
        self.expect(Token::LParen)?;
        let columns = self.cook_columns_def()?;
        self.expect(Token::RParen)?;

        Ok(Statement::CreateTable { table, columns })
    }

    fn parse_insert(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Keyword(Keyword::Into))?;
        let table = self.ident()?;

        let mut columns: Option<Vec<String>> = None;

        if self.eat(Token::LParen) {
            columns = Some(self.cook_columns()?);
            self.expect(Token::RParen)?;
        }
        self.expect(Token::Keyword(Keyword::Values))?;
        self.expect(Token::LParen)?;
        let values = self.cook_values()?;
        self.expect(Token::RParen)?;

        Ok(Statement::Insert {
            table,
            columns,
            values,
        })
    }

    fn parse_select(&mut self) -> Result<Statement, ParseError> {
        let selection = if self.eat(Token::Star) {
            Selection::All
        } else {
            Selection::Columns(self.cook_columns()?)
        };
        self.expect(Token::Keyword(Keyword::From))?;

        let table = self.ident()?;

        let mut filter: Option<Expr> = None;

        if self.eat(Token::Keyword(Keyword::Where)) {
            filter = Some(self.parse_expression()?);
        }

        Ok(Statement::Select {
            selection,
            table,
            filter,
        })
    }

    fn next(&mut self) -> Result<Token, ParseError> {
        self.tokens.next().ok_or(ParseError::UnexpectedEnd)
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let candidate = self.next()?;

        if candidate == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(candidate))
        }
    }

    fn expect_compare_op(&mut self) -> Result<CompareOp, ParseError> {
        match self.next()? {
            Token::Eq => Ok(CompareOp::Eq),
            Token::NotEq => Ok(CompareOp::NotEq),
            Token::Lt => Ok(CompareOp::Lt),
            Token::LtEq => Ok(CompareOp::LtEq),
            Token::Gt => Ok(CompareOp::Gt),
            Token::GtEq => Ok(CompareOp::GtEq),
            token => Err(ParseError::UnexpectedToken(token)),
        }
    }

    fn eat(&mut self, expected: Token) -> bool {
        if Some(&expected) == self.tokens.peek() {
            self.tokens.next();
            true
        } else {
            false
        }
    }

    fn ident(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Token::Ident(name) => Ok(name),
            token => Err(ParseError::UnexpectedToken(token)),
        }
    }

    fn value(&mut self) -> Result<Value, ParseError> {
        match self.next()? {
            Token::Int(n) => Ok(Value::Int(n)),
            Token::Str(s) => Ok(Value::Str(s)),
            token => Err(ParseError::UnexpectedToken(token)),
        }
    }

    fn data_type(&mut self) -> Result<DataType, ParseError> {
        match self.next()? {
            Token::Keyword(Keyword::Integer) => Ok(DataType::Integer),
            Token::Keyword(Keyword::Text) => Ok(DataType::Text),
            token => Err(ParseError::UnexpectedToken(token)),
        }
    }

    fn cook_columns(&mut self) -> Result<Vec<String>, ParseError> {
        let mut cooked: Vec<String> = Vec::new();

        loop {
            cooked.push(self.ident()?);

            if self.eat(Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        Ok(cooked)
    }

    fn cook_columns_def(&mut self) -> Result<Vec<ColumnDef>, ParseError> {
        let mut columns: Vec<ColumnDef> = Vec::new();

        loop {
            let column_name = self.ident()?;
            let column_type = self.data_type()?;

            columns.push(ColumnDef {
                name: column_name,
                data_type: column_type,
            });

            if self.eat(Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        Ok(columns)
    }

    fn cook_values(&mut self) -> Result<Vec<Value>, ParseError> {
        let mut cooked: Vec<Value> = Vec::new();

        loop {
            cooked.push(self.value()?);

            if self.eat(Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        Ok(cooked)
    }

    // Parsing expressions
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and_expr()?;

        loop {
            if self.eat(Token::Keyword(Keyword::Or)) {
                let right = self.and_expr()?;
                expr = Expr::Or(Box::new(expr), Box::new(right));
                continue;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.compare_expr()?;

        loop {
            if self.eat(Token::Keyword(Keyword::And)) {
                let right = self.compare_expr()?;
                expr = Expr::And(Box::new(expr), Box::new(right));
                continue;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn compare_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_primary()?;
        if matches!(
            self.tokens.peek(),
            Some(Token::Eq | Token::NotEq | Token::Lt | Token::LtEq | Token::Gt | Token::GtEq)
        ) {
            let op = self.expect_compare_op()?;
            let right = self.parse_primary()?;

            Ok(Expr::Compare {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.next()? {
            Token::Ident(s) => Ok(Expr::Column(s)),
            Token::Str(s) => Ok(Expr::Literal(Value::Str(s))),
            Token::Int(n) => Ok(Expr::Literal(Value::Int(n))),
            Token::LParen => {
                let expr = self.parse_expression();
                self.expect(Token::RParen)?;
                expr
            }
            token => Err(ParseError::UnexpectedToken(token)),
        }
    }
}
