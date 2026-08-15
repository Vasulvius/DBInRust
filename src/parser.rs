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
