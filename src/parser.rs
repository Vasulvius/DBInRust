//! Étape 3 — analyse syntaxique : transformer une suite de tokens en arbre.
//!
//! Le tokenizer t'a donné des mots. Le parser leur donne une **structure**. Il
//! sait qu'un `SELECT` attend une liste de colonnes puis `FROM`, qu'une
//! parenthèse ouverte doit se refermer, et que dans `a = 1 AND b = 2 OR c = 3`
//! le `AND` se noue plus serré que le `OR`. Sa sortie est un arbre que l'étape
//! 5 n'aura plus qu'à exécuter.
//!
//! C'est ici que `Box` arrive — et pas comme une notion de chapitre 15 à
//! réciter. Un `Expr` contient des `Expr`. Sans indirection, sa taille serait
//! infinie et le compilateur refusera net de le compiler.
//!
//! # À écrire dans ce fichier
//!
//! ```text
//! pub fn parse(sql: &str) -> Result<Statement, ParseError>
//! ```
//!
//! Elle prend du **SQL brut**, pas des tokens : elle appelle `tokenize`
//! elle-même. Une erreur du lexer doit donc pouvoir devenir une `ParseError`.
//!
//! ## Les types de l'arbre
//!
//! ```text
//! pub enum Statement {
//!     CreateTable { table: String, columns: Vec<ColumnDef> },
//!     Insert      { table: String, columns: Option<Vec<String>>, values: Vec<Value> },
//!     Select      { selection: Selection, table: String, filter: Option<Expr> },
//! }
//!
//! pub struct ColumnDef { pub name: String, pub data_type: DataType }
//!
//! pub enum DataType  { Integer, Text }
//! pub enum Selection { All, Columns(Vec<String>) }
//! pub enum Value     { Int(i64), Str(String) }
//! pub enum CompareOp { Eq, NotEq, Lt, LtEq, Gt, GtEq }
//!
//! pub enum Expr {
//!     Column(String),
//!     Literal(Value),
//!     Compare { left: Box<Expr>, op: CompareOp, right: Box<Expr> },
//!     And(Box<Expr>, Box<Expr>),
//!     Or(Box<Expr>, Box<Expr>),
//! }
//!
//! pub enum ParseError {
//!     UnexpectedToken(Token),  // un token qui n'a rien à faire là
//!     UnexpectedEnd,           // la requête s'arrête trop tôt
//!     Lex(LexError),           // le tokenizer a échoué avant nous
//! }
//! ```
//!
//! Le `columns: Option<Vec<String>>` de `Insert` distingue
//! `INSERT INTO t VALUES (...)` (`None`) de
//! `INSERT INTO t (a, b) VALUES (...)` (`Some`).
//!
//! # La grammaire
//!
//! Notation : `?` = optionnel, `*` = répété zéro fois ou plus, `|` = ou.
//!
//! ```text
//! statement   := ( create | insert | select ) ";"?
//!
//! create      := CREATE TABLE ident "(" column_def ( "," column_def )* ")"
//! column_def  := ident ( INTEGER | TEXT )
//!
//! insert      := INSERT INTO ident ( "(" ident ( "," ident )* ")" )?
//!                VALUES "(" value ( "," value )* ")"
//!
//! select      := SELECT selection FROM ident ( WHERE expr )?
//! selection   := "*" | ident ( "," ident )*
//!
//! expr        := or_expr
//! or_expr     := and_expr ( OR and_expr )*
//! and_expr    := comparison ( AND comparison )*
//! comparison  := primary ( op primary )?
//! primary     := ident | value | "(" expr ")"
//!
//! op          := "=" | "!=" | "<" | "<=" | ">" | ">="
//! value       := int | string
//! ```
//!
//! Lis la partie `expr` de haut en bas : **c'est l'empilement des niveaux qui
//! encode les priorités.** `or_expr` est le plus permissif donc le plus haut
//! dans l'arbre, `primary` le plus serré donc le plus profond. Tu n'as aucune
//! table de priorités à écrire — la forme de la grammaire *est* la priorité.
//!
//! Les `*` se traduisent par des boucles, et une boucle qui replie à chaque
//! tour donne l'associativité à gauche : `a OR b OR c` doit produire
//! `Or(Or(a, b), c)` et non `Or(a, Or(b, c))`.
//!
//! # Les règles
//!
//! - Le `;` final est **facultatif**. En revanche, s'il reste quoi que ce soit
//!   après la fin du statement, c'est une erreur.
//! - Une entrée vide est une `UnexpectedEnd`.
//! - Les mots-clés sont déjà normalisés par le tokenizer : tu ne compares que
//!   des variantes de `Keyword`, jamais des chaînes.
//! - `parse` ne valide **que la syntaxe**. Que la table existe, que le nombre
//!   de valeurs corresponde au nombre de colonnes, que comparer un entier à
//!   une chaîne ait un sens : rien de tout ça ne te regarde ici. C'est le
//!   travail de l'exécution, à l'étape 5.
//!
//! # Hors périmètre
//!
//! Pas de `NOT`, pas de `NULL`, pas d'alias `AS`, pas d'`ORDER BY`, pas de
//! `UPDATE` ni `DELETE`, pas de jointures, pas de sous-requêtes.
//!
//! # Comment savoir si c'est bon
//!
//! `cargo test`. La spécification exécutable est dans `tests/parser.rs`.
//!
//! Un indice pour t'éviter une heure de perplexité : selon la façon dont tu
//! fais circuler les tokens, tu auras peut-être besoin d'ajouter une
//! dérivation à `Token`. Si ça arrive, le compilateur te dira laquelle et
//! pourquoi — lis son message, il y a une décision de conception derrière.

use std::{iter::Peekable, println, vec::IntoIter};

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
        let statement: Statement;
        match self.next()? {
            Token::Keyword(Keyword::Create) => statement = self.parse_create_table()?,
            Token::Keyword(Keyword::Insert) => statement = self.parse_insert()?,
            Token::Keyword(Keyword::Select) => statement = self.parse_select()?,
            token => return Err(ParseError::UnexpectedToken(token)),
        }
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

        Ok(Statement::CreateTable { table, columns })
    }

    fn parse_insert(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Keyword(Keyword::Into))?;
        let table = self.ident()?;

        let mut columns: Option<Vec<String>> = None;

        if self.eat(Token::LParen) {
            columns = Some(self.cook_columns_name()?);
        }
        self.expect(Token::Keyword(Keyword::Values))?;
        self.expect(Token::LParen)?;

        let values = self.cook_values()?;

        Ok(Statement::Insert {
            table,
            columns,
            values,
        })
    }

    fn parse_select(&mut self) -> Result<Statement, ParseError> {
        let selection: Selection;
        if self.eat(Token::Star) {
            selection = Selection::All;
            self.expect(Token::Keyword(Keyword::From))?;
        } else {
            selection = Selection::Columns(self.cook_select_columns_name()?);
        }

        let table = self.ident()?;

        let mut filter: Option<Expr> = None;

        if self.eat(Token::Keyword(Keyword::Where)) {
            filter = Some(*self.parse_expression()?);
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

    fn eat_compare_op(&mut self) -> bool {
        match self.tokens.peek() {
            Some(Token::Eq) | Some(Token::NotEq) | Some(Token::Lt) | Some(Token::LtEq)
            | Some(Token::Gt) | Some(Token::GtEq) => true,
            _ => false,
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

    // TODO: factorize with cook_values and cook_columns_def
    fn cook_columns_name(&mut self) -> Result<Vec<String>, ParseError> {
        let mut cooked: Vec<String> = Vec::new();

        loop {
            cooked.push(self.ident()?);

            match self.next()? {
                Token::Comma => continue,
                Token::RParen => break,
                token => return Err(ParseError::UnexpectedToken(token)),
            }
        }

        Ok(cooked)
    }

    fn cook_select_columns_name(&mut self) -> Result<Vec<String>, ParseError> {
        let mut cooked: Vec<String> = Vec::new();

        loop {
            cooked.push(self.ident()?);

            match self.next()? {
                Token::Comma => continue,
                Token::Keyword(Keyword::From) => break,
                token => return Err(ParseError::UnexpectedToken(token)),
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

            match self.next()? {
                Token::Comma => continue,
                Token::RParen => break,
                token => return Err(ParseError::UnexpectedToken(token)),
            }
        }

        Ok(columns)
    }

    fn cook_values(&mut self) -> Result<Vec<Value>, ParseError> {
        let mut cooked: Vec<Value> = Vec::new();

        loop {
            cooked.push(self.value()?);

            match self.next()? {
                Token::Comma => continue,
                Token::RParen => break,
                token => return Err(ParseError::UnexpectedToken(token)),
            }
        }

        Ok(cooked)
    }

    // Parsing expressions
    fn parse_expression(&mut self) -> Result<Box<Expr>, ParseError> {
        self.or_expr(None)
    }

    fn or_expr(&mut self, left: Option<Box<Expr>>) -> Result<Box<Expr>, ParseError> {
        // Todo: factorise match case with and_expr ?
        let mut expr: Box<Expr>;
        match left {
            Some(left) => expr = left,
            None => expr = self.and_expr(None)?,
        }

        // Stop condition => cannot eat an OR token
        if self.eat(Token::Keyword(Keyword::Or)) {
            let right = self.and_expr(None)?;
            expr = self.or_expr(Some(Box::new(Expr::Or(expr, right))))?;
        }

        Ok(expr)
    }

    fn and_expr(&mut self, left: Option<Box<Expr>>) -> Result<Box<Expr>, ParseError> {
        let mut expr: Box<Expr>;
        match left {
            Some(left) => expr = left,
            None => expr = self.compare_expr()?,
        }

        // Stop condition => cannot eat an AND token
        if self.eat(Token::Keyword(Keyword::And)) {
            let right = self.compare_expr()?;
            expr = self.and_expr(Some(Box::new(Expr::And(expr, right))))?;
        }

        Ok(expr)
    }

    fn compare_expr(&mut self) -> Result<Box<Expr>, ParseError> {
        if self.eat(Token::LParen) {
            let mut expr = self.compare_expr();
            println!("{:?}", expr);
            if Some(&Token::Keyword(Keyword::Or)) == self.tokens.peek() {
                expr = self.or_expr(Some(expr?));
                println!("{:?}", expr);
            }
            println!("Coucou");
            self.expect(Token::RParen)?;
            return expr;
        }

        let col = Expr::Column(self.ident()?);

        if self.eat_compare_op() {
            let left = Box::new(col);
            let op = self.expect_compare_op()?;
            let right: Box<Expr>;

            match self.next()? {
                Token::Str(s) => right = Box::new(Expr::Literal(Value::Str(s))),
                Token::Int(n) => right = Box::new(Expr::Literal(Value::Int(n))),
                token => return Err(ParseError::UnexpectedToken(token)),
            }

            Ok(Box::new(Expr::Compare { left, op, right }))
        } else {
            Ok(Box::new(col))
        }
    }
}
