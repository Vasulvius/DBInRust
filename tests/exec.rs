//! Étape 5 — tests d'acceptation du moteur d'exécution.
//!
//! Ce fichier est la spécification exécutable de l'étape. Tu ne le modifies
//! pas : tu écris `src/exec.rs` jusqu'à ce que `cargo test` soit vert.

use std::error::Error;

use minidb::exec::{Database, DbError, ExecError, Output};
use minidb::parser::{DataType, Value};

// --- raccourcis -------------------------------------------------------------

fn int(n: i64) -> Value {
    Value::Int(n)
}
fn txt(s: &str) -> Value {
    Value::Str(s.to_string())
}
fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

/// Une base avec `users (id INTEGER, name TEXT)`, vide.
fn base() -> Database {
    let mut db = Database::new();
    db.execute("CREATE TABLE users (id INTEGER, name TEXT);")
        .expect("la création doit réussir");
    db
}

/// La même, avec deux lignes déjà insérées.
fn base_peuplee() -> Database {
    let mut db = base();
    db.execute("INSERT INTO users VALUES (1, 'alice');").unwrap();
    db.execute("INSERT INTO users VALUES (2, 'bob');").unwrap();
    db
}

/// Raccourci pour construire un `Output::Rows` attendu.
fn rows(columns: &[&str], lignes: Vec<Vec<Value>>) -> Output {
    Output::Rows {
        columns: names(columns),
        rows: lignes,
    }
}

// --- CREATE TABLE -----------------------------------------------------------

#[test]
fn creer_une_table() {
    let mut db = Database::new();
    assert_eq!(
        db.execute("CREATE TABLE users (id INTEGER, name TEXT);"),
        Ok(Output::TableCreated)
    );
}

#[test]
fn une_table_fraichement_creee_est_vide() {
    // Zéro ligne, mais les colonnes sont là : c'est ce qui permet d'afficher
    // un en-tête même sans résultat.
    let mut db = base();
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(&["id", "name"], vec![]))
    );
}

#[test]
fn recreer_une_table_existante_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("CREATE TABLE users (id INTEGER);"),
        Err(DbError::Exec(ExecError::TableAlreadyExists(
            "users".to_string()
        )))
    );
}

#[test]
fn deux_tables_differentes_coexistent() {
    let mut db = base();
    assert_eq!(
        db.execute("CREATE TABLE posts (id INTEGER, title TEXT);"),
        Ok(Output::TableCreated)
    );
    db.execute("INSERT INTO posts VALUES (1, 'hello');").unwrap();
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(&["id", "name"], vec![]))
    );
    assert_eq!(
        db.execute("SELECT * FROM posts;"),
        Ok(rows(&["id", "title"], vec![vec![int(1), txt("hello")]]))
    );
}

// --- INSERT -----------------------------------------------------------------

#[test]
fn inserer_une_ligne() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users VALUES (1, 'alice');"),
        Ok(Output::RowsInserted(1))
    );
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(&["id", "name"], vec![vec![int(1), txt("alice")]]))
    );
}

#[test]
fn les_lignes_sortent_dans_leur_ordre_dinsertion() {
    let mut db = base();
    db.execute("INSERT INTO users VALUES (3, 'carol');").unwrap();
    db.execute("INSERT INTO users VALUES (1, 'alice');").unwrap();
    db.execute("INSERT INTO users VALUES (2, 'bob');").unwrap();
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(
            &["id", "name"],
            vec![
                vec![int(3), txt("carol")],
                vec![int(1), txt("alice")],
                vec![int(2), txt("bob")],
            ]
        ))
    );
}

#[test]
fn inserer_avec_une_liste_de_colonnes() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users (id, name) VALUES (1, 'alice');"),
        Ok(Output::RowsInserted(1))
    );
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(&["id", "name"], vec![vec![int(1), txt("alice")]]))
    );
}

#[test]
fn une_liste_de_colonnes_dans_le_desordre_est_reordonnee() {
    // C'est le cœur de l'exercice : la ligne stockée suit toujours l'ordre du
    // schéma, quel que soit l'ordre d'écriture de la requête.
    let mut db = base();
    db.execute("INSERT INTO users (name, id) VALUES ('alice', 1);")
        .unwrap();
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(&["id", "name"], vec![vec![int(1), txt("alice")]]))
    );
}

#[test]
fn inserer_dans_une_table_inconnue_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO ghosts VALUES (1);"),
        Err(DbError::Exec(ExecError::TableNotFound("ghosts".to_string())))
    );
}

#[test]
fn trop_de_valeurs_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users VALUES (1, 'alice', 3);"),
        Err(DbError::Exec(ExecError::WrongNumberOfValues {
            expected: 2,
            found: 3
        }))
    );
}

#[test]
fn pas_assez_de_valeurs_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users VALUES (1);"),
        Err(DbError::Exec(ExecError::WrongNumberOfValues {
            expected: 2,
            found: 1
        }))
    );
}

#[test]
fn une_liste_de_colonnes_incomplete_est_une_erreur() {
    // Sans `NULL`, on ne saurait pas quoi ranger dans `name`.
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users (id) VALUES (1);"),
        Err(DbError::Exec(ExecError::WrongNumberOfValues {
            expected: 2,
            found: 1
        }))
    );
}

#[test]
fn une_colonne_inconnue_dans_un_insert_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users (id, age) VALUES (1, 30);"),
        Err(DbError::Exec(ExecError::ColumnNotFound("age".to_string())))
    );
}

#[test]
fn du_texte_dans_une_colonne_entiere_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users VALUES ('un', 'alice');"),
        Err(DbError::Exec(ExecError::TypeMismatch {
            column: "id".to_string(),
            expected: DataType::Integer,
            found: DataType::Text,
        }))
    );
}

#[test]
fn un_entier_dans_une_colonne_texte_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users VALUES (1, 42);"),
        Err(DbError::Exec(ExecError::TypeMismatch {
            column: "name".to_string(),
            expected: DataType::Text,
            found: DataType::Integer,
        }))
    );
}

#[test]
fn le_type_est_verifie_apres_reordonnancement() {
    // La colonne citée dans l'erreur est celle du schéma, pas la position
    // dans la requête.
    let mut db = base();
    assert_eq!(
        db.execute("INSERT INTO users (name, id) VALUES ('alice', 'x');"),
        Err(DbError::Exec(ExecError::TypeMismatch {
            column: "id".to_string(),
            expected: DataType::Integer,
            found: DataType::Text,
        }))
    );
}

#[test]
fn une_ligne_refusee_nest_pas_stockee() {
    let mut db = base();
    let _ = db.execute("INSERT INTO users VALUES (1, 42);");
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(&["id", "name"], vec![]))
    );
}

// --- SELECT -----------------------------------------------------------------

#[test]
fn select_etoile_rend_les_colonnes_dans_lordre_du_schema() {
    let mut db = base_peuplee();
    assert_eq!(
        db.execute("SELECT * FROM users;"),
        Ok(rows(
            &["id", "name"],
            vec![vec![int(1), txt("alice")], vec![int(2), txt("bob")]]
        ))
    );
}

#[test]
fn select_dune_seule_colonne() {
    let mut db = base_peuplee();
    assert_eq!(
        db.execute("SELECT name FROM users;"),
        Ok(rows(&["name"], vec![vec![txt("alice")], vec![txt("bob")]]))
    );
}

#[test]
fn select_rend_les_colonnes_dans_lordre_demande() {
    // `name` avant `id`, alors que le schéma dit l'inverse.
    let mut db = base_peuplee();
    assert_eq!(
        db.execute("SELECT name, id FROM users;"),
        Ok(rows(
            &["name", "id"],
            vec![vec![txt("alice"), int(1)], vec![txt("bob"), int(2)]]
        ))
    );
}

#[test]
fn select_sur_une_table_inconnue_est_une_erreur() {
    let mut db = base();
    assert_eq!(
        db.execute("SELECT * FROM ghosts;"),
        Err(DbError::Exec(ExecError::TableNotFound("ghosts".to_string())))
    );
}

#[test]
fn select_dune_colonne_inconnue_est_une_erreur() {
    let mut db = base_peuplee();
    assert_eq!(
        db.execute("SELECT age FROM users;"),
        Err(DbError::Exec(ExecError::ColumnNotFound("age".to_string())))
    );
}

#[test]
fn la_casse_des_identifiants_compte() {
    // Décision assumée : `Users` n'est pas `users`. Le vrai SQL est
    // insensible à la casse ; ce sera pour plus tard.
    let mut db = base();
    assert_eq!(
        db.execute("SELECT * FROM Users;"),
        Err(DbError::Exec(ExecError::TableNotFound("Users".to_string())))
    );
}

// --- WHERE, pas encore ------------------------------------------------------

#[test]
fn un_where_est_refuse_explicitement() {
    // Refuser vaut mieux qu'ignorer le filtre et rendre de mauvais résultats.
    let mut db = base_peuplee();
    assert_eq!(
        db.execute("SELECT * FROM users WHERE id = 1;"),
        Err(DbError::Exec(ExecError::Unsupported("WHERE")))
    );
}

// --- les erreurs de syntaxe traversent execute ------------------------------

#[test]
fn une_erreur_de_syntaxe_remonte_en_dberror_parse() {
    let mut db = Database::new();
    let err = db.execute("SELECT FROM users").unwrap_err();
    assert!(
        matches!(err, DbError::Parse(_)),
        "attendu DbError::Parse, obtenu {err:?}"
    );
    assert_eq!(err.to_string(), "unexpected token: FROM");
}

#[test]
fn une_erreur_du_lexer_remonte_aussi() {
    let mut db = Database::new();
    let err = db.execute("SELECT # FROM t").unwrap_err();
    assert_eq!(err.to_string(), "unexpected character '#' at position 7");
}

// --- Display ----------------------------------------------------------------

#[test]
fn les_messages_des_erreurs_dexecution() {
    assert_eq!(
        ExecError::TableNotFound("users".to_string()).to_string(),
        "no such table: users"
    );
    assert_eq!(
        ExecError::TableAlreadyExists("users".to_string()).to_string(),
        "table users already exists"
    );
    assert_eq!(
        ExecError::ColumnNotFound("age".to_string()).to_string(),
        "no such column: age"
    );
    assert_eq!(
        ExecError::WrongNumberOfValues {
            expected: 2,
            found: 3
        }
        .to_string(),
        "table has 2 columns but 3 values were supplied"
    );
    assert_eq!(
        ExecError::TypeMismatch {
            column: "id".to_string(),
            expected: DataType::Integer,
            found: DataType::Text,
        }
        .to_string(),
        "type mismatch for column id: expected INTEGER, found TEXT"
    );
    assert_eq!(
        ExecError::Unsupported("WHERE").to_string(),
        "unsupported feature: WHERE"
    );
}

#[test]
fn un_type_de_colonne_saffiche_en_majuscules() {
    assert_eq!(DataType::Integer.to_string(), "INTEGER");
    assert_eq!(DataType::Text.to_string(), "TEXT");
}

#[test]
fn dberror_delegue_son_message() {
    // Comme `ParseError::Lex` à l'étape 4 : pas de couche de texte en plus.
    let inner = ExecError::TableNotFound("users".to_string());
    assert_eq!(DbError::Exec(inner).to_string(), "no such table: users");
}

#[test]
fn dberror_expose_sa_cause() {
    let err = DbError::Exec(ExecError::ColumnNotFound("age".to_string()));
    let cause = err.source().expect("DbError doit exposer sa cause");
    assert_eq!(cause.to_string(), "no such column: age");
}

#[test]
fn une_erreur_dexecution_devient_un_objet_trait() {
    fn run(db: &mut Database, sql: &str) -> Result<Output, Box<dyn Error>> {
        Ok(db.execute(sql)?)
    }

    let mut db = base();
    assert!(run(&mut db, "SELECT * FROM users").is_ok());
    assert_eq!(
        run(&mut db, "SELECT * FROM ghosts").unwrap_err().to_string(),
        "no such table: ghosts"
    );
}

// --- de bout en bout --------------------------------------------------------

#[test]
fn un_scenario_complet() {
    let mut db = Database::new();

    assert_eq!(
        db.execute("CREATE TABLE products (name TEXT, price INTEGER);"),
        Ok(Output::TableCreated)
    );
    assert_eq!(
        db.execute("INSERT INTO products VALUES ('coffee', 3);"),
        Ok(Output::RowsInserted(1))
    );
    assert_eq!(
        db.execute("INSERT INTO products (price, name) VALUES (2, 'tea');"),
        Ok(Output::RowsInserted(1))
    );
    assert_eq!(
        db.execute("SELECT price, name FROM products;"),
        Ok(rows(
            &["price", "name"],
            vec![
                vec![int(3), txt("coffee")],
                vec![int(2), txt("tea")],
            ]
        ))
    );
}
