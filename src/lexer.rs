//! Étape 2 — analyse lexicale : découper du SQL brut en tokens.
//!
//! Le tokenizer ne comprend rien à la grammaire SQL. Il ne sait pas qu'un
//! `SELECT` doit être suivi de colonnes, ni qu'une parenthèse ouverte doit se
//! refermer : c'est le travail du parser, à l'étape 3. Ici on ne fait qu'une
//! chose — transformer une suite de caractères en une suite de **mots**
//! classés. `select*from t` et `SELECT * FROM t` doivent produire exactement
//! la même sortie.
//!
//! # À écrire dans ce fichier
//!
//! ```text
//! pub enum Keyword    // les mots réservés du langage
//! pub enum Token      // un mot classé
//! pub enum LexError   // ce qui peut mal se passer
//! pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError>
//! ```
//!
//! ## `Keyword`
//!
//! Douze variantes sans donnée associée :
//! `Select`, `From`, `Where`, `Insert`, `Into`, `Values`, `Create`, `Table`,
//! `And`, `Or`, `Integer`, `Text`.
//!
//! ## `Token`
//!
//! | Variante          | Donnée   | Ce que ça représente          |
//! |-------------------|----------|-------------------------------|
//! | `Keyword`         | `Keyword`| un mot réservé                |
//! | `Ident`           | `String` | un nom de table ou de colonne |
//! | `Int`             | `i64`    | un littéral entier            |
//! | `Str`             | `String` | un littéral chaîne            |
//! | `Comma`           | —        | `,`                           |
//! | `Semicolon`       | —        | `;`                           |
//! | `LParen`          | —        | `(`                           |
//! | `RParen`          | —        | `)`                           |
//! | `Star`            | —        | `*`                           |
//! | `Eq`              | —        | `=`                           |
//! | `NotEq`           | —        | `!=` et `<>`                  |
//! | `Lt` / `LtEq`     | —        | `<` et `<=`                   |
//! | `Gt` / `GtEq`     | —        | `>` et `>=`                   |
//!
//! ## `LexError`
//!
//! - `UnexpectedChar(char)` — un caractère qui n'a rien à faire là
//! - `UnterminatedString` — une apostrophe ouverte jamais refermée
//! - `NumberOutOfRange(String)` — des chiffres qui ne tiennent pas dans un `i64`
//!
//! # Les règles
//!
//! - **Espaces** (espace, tabulation, retour à la ligne) : ignorés, ils ne
//!   servent qu'à séparer. Ils ne sont pas obligatoires : `id=1` donne trois
//!   tokens.
//! - **Mots** : commencent par une lettre ou `_`, continuent avec des lettres,
//!   des chiffres ou des `_`. Si le mot complet correspond à un mot réservé
//!   (sans tenir compte de la casse), c'est un `Keyword` ; sinon un `Ident`,
//!   dont on **conserve la casse d'origine**. Attention : `selection` n'est pas
//!   `SELECT` suivi de `ion`.
//! - **Entiers** : une suite de chiffres. Convertis en `i64` — et cette
//!   conversion peut échouer.
//! - **Chaînes** : entre apostrophes simples. Une apostrophe à l'intérieur
//!   s'écrit en la doublant, comme en SQL : `'it''s'` contient `it's`. Le
//!   contenu n'est jamais interprété — `'select'` est une chaîne, pas un
//!   mot-clé.
//! - **Opérateurs à deux caractères** : `<=`, `>=`, `!=`, `<>`. C'est le cœur
//!   de l'exercice — en voyant `<`, tu dois regarder le caractère suivant
//!   **sans le consommer** pour savoir si tu tiens `Lt` ou `LtEq`. Un `!` qui
//!   n'est pas suivi de `=` est une erreur.
//! - Tout le reste est un `UnexpectedChar`.
//!
//! # Hors périmètre (volontairement)
//!
//! Pas de nombres négatifs (`-` n'est pas reconnu), pas de flottants, pas de
//! commentaires `--`, pas d'identifiants entre guillemets. On ajoutera au fur
//! et à mesure des besoins du parser.
//!
//! Les positions dans le texte source n'y sont pas non plus : `UnexpectedChar`
//! dit *quoi*, pas *où*. On les ajoutera à l'étape 4, quand on unifiera les
//! erreurs du moteur — tu verras alors ce que coûte un type d'erreur qu'on
//! enrichit après coup.
//!
//! # Comment savoir si c'est bon
//!
//! `cargo test`. La spécification exécutable est dans `tests/lexer.rs`.
