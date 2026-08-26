//! Étape 5 — exécution en mémoire.
//!
//! Jusqu'ici le moteur *comprenait* le SQL sans rien en faire. À partir de
//! maintenant il **exécute** : il garde des tables, y range des lignes, et
//! répond aux requêtes. C'est l'étape où minidb devient une base de données.
//!
//! Le stockage reste en RAM et disparaît à la fermeture — le disque, c'est
//! pour les étapes 7 à 10.
//!
//! # À écrire dans ce fichier
//!
//! ```text
//! pub struct Database
//!
//! impl Database {
//!     pub fn new() -> Self
//!     pub fn execute(&mut self, sql: &str) -> Result<Output, DbError>
//! }
//! ```
//!
//! `execute` prend du **SQL brut** : elle appelle `parse` elle-même, puis
//! exécute l'arbre obtenu. C'est l'API que le REPL utilisera.
//!
//! ## Les types
//!
//! ```text
//! pub enum Output {
//!     TableCreated,
//!     RowsInserted(usize),
//!     Rows { columns: Vec<String>, rows: Vec<Vec<Value>> },
//! }
//!
//! pub enum ExecError {
//!     TableNotFound(String),
//!     TableAlreadyExists(String),
//!     ColumnNotFound(String),
//!     WrongNumberOfValues { expected: usize, found: usize },
//!     TypeMismatch { column: String, expected: DataType, found: DataType },
//!     Unsupported(&'static str),
//! }
//!
//! pub enum DbError {
//!     Parse(ParseError),
//!     Exec(ExecError),
//! }
//! ```
//!
//! `Value`, `DataType` et `ColumnDef` viennent de `parser` — on ne les
//! redéfinit pas.
//!
//! `DbError` unifie les deux couches, exactement comme `ParseError::Lex`
//! unifiait lexer et parser à l'étape 4. Tu sais déjà quoi en faire :
//! `From` pour les deux variantes, `Display` qui **délègue** sans ajouter de
//! couche, `Error` avec un `source()` qui expose la cause.
//!
//! # Les règles
//!
//! ## `CREATE TABLE`
//!
//! Enregistre le schéma. Si la table existe déjà → `TableAlreadyExists`.
//! Rend `Output::TableCreated`.
//!
//! ## `INSERT`
//!
//! - Table inconnue → `TableNotFound`.
//! - **Sans liste de colonnes** : les valeurs suivent l'ordre du schéma.
//! - **Avec liste de colonnes** : les valeurs suivent l'ordre de la liste, et
//!   la ligne stockée est réordonnée selon le schéma. `(b, a)` puis `(2, 'x')`
//!   range bien `'x'` dans `a`.
//! - Le nombre de valeurs doit égaler le nombre de colonnes attendues, sinon
//!   `WrongNumberOfValues`.
//! - La liste de colonnes doit couvrir **toutes** les colonnes du schéma :
//!   faute de `NULL`, on ne sait pas quoi mettre dans celles qu'on omet. Une
//!   liste incomplète est donc un `WrongNumberOfValues` où `expected` est la
//!   taille du schéma.
//! - Une colonne inconnue dans la liste → `ColumnNotFound`.
//! - Chaque valeur doit correspondre au type déclaré, sinon `TypeMismatch`.
//! - Rend `Output::RowsInserted(1)` — une seule ligne par `INSERT` pour
//!   l'instant.
//!
//! ## `SELECT`
//!
//! - Table inconnue → `TableNotFound`, colonne inconnue → `ColumnNotFound`.
//! - `SELECT *` rend toutes les colonnes **dans l'ordre du schéma**.
//! - `SELECT a, b` rend les colonnes **dans l'ordre demandé**, qui n'est pas
//!   forcément celui du schéma.
//! - Les lignes sortent dans leur **ordre d'insertion**.
//! - Une table vide rend zéro ligne, mais la liste des colonnes est quand même
//!   remplie — c'est ce qui permet d'afficher un en-tête.
//! - Un `WHERE` rend `Unsupported("WHERE")`. Refuser explicitement vaut mieux
//!   qu'ignorer le filtre en silence et rendre de mauvais résultats. Ce sera
//!   l'étape 6.
//!
//! # Hors périmètre
//!
//! - Les noms de tables et de colonnes sont **sensibles à la casse**. Le vrai
//!   SQL ne l'est pas ; on s'en occupera plus tard.
//! - Pas de `NULL`, pas de valeur par défaut, pas de clé primaire, pas de
//!   contrainte d'unicité.
//! - Un `INSERT` n'insère qu'une ligne à la fois.
//! - Une colonne répétée dans la liste d'un `INSERT` n'est pas détectée.
//!
//! # Le câblage du REPL
//!
//! Aucun test ne le couvre. Dans `src/main.rs`, remplace l'appel à `parse` par
//! une `Database` créée avant la boucle, et affiche `Output` proprement — un
//! tableau pour les lignes, un message court pour le reste. C'est là que tu
//! verras enfin ta base fonctionner :
//!
//! ```text
//! minidb> CREATE TABLE users (id INTEGER, name TEXT);
//! table created
//! minidb> INSERT INTO users VALUES (1, 'alice');
//! 1 row inserted
//! minidb> SELECT * FROM users;
//! id | name
//! 1  | alice
//! ```
//!
//! # Comment savoir si c'est bon
//!
//! `cargo test`. La spécification exécutable est dans `tests/exec.rs`.
