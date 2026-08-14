//! Étape 2 — tests d'acceptation du tokenizer.
//!
//! Ce fichier est la spécification exécutable de l'étape. Tu ne le modifies
//! pas : tu écris `src/lexer.rs` jusqu'à ce que `cargo test` soit vert.

use minidb::lexer::Token::*;
use minidb::lexer::{Keyword as Kw, LexError, Token, tokenize};

// Deux raccourcis, pour que les tableaux attendus restent lisibles.
fn ident(name: &str) -> Token {
    Ident(name.to_string())
}
fn text(value: &str) -> Token {
    Str(value.to_string())
}

// --- entrée vide ------------------------------------------------------------

#[test]
fn une_entree_vide_ne_produit_aucun_token() {
    assert_eq!(tokenize(""), Ok(vec![]));
    assert_eq!(tokenize("   \t\n  "), Ok(vec![]));
}

// --- mots-clés --------------------------------------------------------------

#[test]
fn les_mots_cles_sont_reconnus() {
    assert_eq!(
        tokenize("SELECT INSERT INTO VALUES CREATE TABLE"),
        Ok(vec![
            Keyword(Kw::Select),
            Keyword(Kw::Insert),
            Keyword(Kw::Into),
            Keyword(Kw::Values),
            Keyword(Kw::Create),
            Keyword(Kw::Table),
        ])
    );
    assert_eq!(
        tokenize("FROM WHERE AND OR INTEGER TEXT"),
        Ok(vec![
            Keyword(Kw::From),
            Keyword(Kw::Where),
            Keyword(Kw::And),
            Keyword(Kw::Or),
            Keyword(Kw::Integer),
            Keyword(Kw::Text),
        ])
    );
}

#[test]
fn les_mots_cles_ignorent_la_casse() {
    assert_eq!(
        tokenize("select SELECT SeLeCt"),
        Ok(vec![
            Keyword(Kw::Select),
            Keyword(Kw::Select),
            Keyword(Kw::Select),
        ])
    );
}

// --- identifiants -----------------------------------------------------------

#[test]
fn un_identifiant_conserve_sa_casse() {
    assert_eq!(tokenize("Users"), Ok(vec![ident("Users")]));
}

#[test]
fn un_identifiant_accepte_chiffres_et_underscores() {
    assert_eq!(
        tokenize("user_id2 _tmp x9"),
        Ok(vec![ident("user_id2"), ident("_tmp"), ident("x9")])
    );
}

#[test]
fn un_mot_qui_commence_comme_un_mot_cle_reste_un_identifiant() {
    // Le classement se fait sur le mot entier, pas sur son début.
    assert_eq!(
        tokenize("selection fromage tables"),
        Ok(vec![ident("selection"), ident("fromage"), ident("tables")])
    );
}

// --- entiers ----------------------------------------------------------------

#[test]
fn les_entiers_sont_convertis() {
    assert_eq!(
        tokenize("0 42 1234567890"),
        Ok(vec![Int(0), Int(42), Int(1234567890)])
    );
}

#[test]
fn un_entier_trop_grand_est_une_erreur() {
    assert_eq!(
        tokenize("99999999999999999999"),
        Err(LexError::NumberOutOfRange("99999999999999999999".to_string()))
    );
}

// --- chaînes ----------------------------------------------------------------

#[test]
fn une_chaine_est_delimitee_par_des_apostrophes() {
    assert_eq!(tokenize("'alice'"), Ok(vec![text("alice")]));
}

#[test]
fn une_chaine_peut_etre_vide() {
    assert_eq!(tokenize("''"), Ok(vec![text("")]));
}

#[test]
fn une_apostrophe_se_double_dans_une_chaine() {
    assert_eq!(tokenize("'it''s'"), Ok(vec![text("it's")]));
    assert_eq!(tokenize("''''"), Ok(vec![text("'")]));
}

#[test]
fn le_contenu_dune_chaine_nest_jamais_interprete() {
    assert_eq!(tokenize("'select * from'"), Ok(vec![text("select * from")]));
}

#[test]
fn une_chaine_non_fermee_est_une_erreur() {
    assert_eq!(tokenize("'abc"), Err(LexError::UnterminatedString));
    // Ici le `''` est un échappement : la chaîne continue, puis le texte finit.
    assert_eq!(tokenize("'a''"), Err(LexError::UnterminatedString));
}

// --- ponctuation ------------------------------------------------------------

#[test]
fn la_ponctuation_est_reconnue() {
    assert_eq!(
        tokenize("(),;*"),
        Ok(vec![LParen, RParen, Comma, Semicolon, Star])
    );
}

// --- opérateurs -------------------------------------------------------------

#[test]
fn les_operateurs_a_un_caractere() {
    assert_eq!(tokenize("= < >"), Ok(vec![Eq, Lt, Gt]));
}

#[test]
fn les_operateurs_a_deux_caracteres() {
    assert_eq!(
        tokenize("<= >= != <>"),
        Ok(vec![LtEq, GtEq, NotEq, NotEq])
    );
}

#[test]
fn un_operateur_ne_mange_pas_le_token_suivant() {
    assert_eq!(tokenize("<5"), Ok(vec![Lt, Int(5)]));
    assert_eq!(tokenize(">=x"), Ok(vec![GtEq, ident("x")]));
    assert_eq!(tokenize("<>="), Ok(vec![NotEq, Eq]));
}

// --- erreurs ----------------------------------------------------------------

#[test]
fn un_caractere_inconnu_est_une_erreur() {
    assert_eq!(
        tokenize("select # from t"),
        Err(LexError::UnexpectedChar('#'))
    );
}

#[test]
fn un_point_dexclamation_seul_est_une_erreur() {
    assert_eq!(tokenize("a ! b"), Err(LexError::UnexpectedChar('!')));
}

// --- requêtes complètes -----------------------------------------------------

#[test]
fn select_etoile() {
    assert_eq!(
        tokenize("SELECT * FROM users;"),
        Ok(vec![
            Keyword(Kw::Select),
            Star,
            Keyword(Kw::From),
            ident("users"),
            Semicolon,
        ])
    );
}

#[test]
fn select_avec_where() {
    assert_eq!(
        tokenize("SELECT id, name FROM users WHERE id = 42;"),
        Ok(vec![
            Keyword(Kw::Select),
            ident("id"),
            Comma,
            ident("name"),
            Keyword(Kw::From),
            ident("users"),
            Keyword(Kw::Where),
            ident("id"),
            Eq,
            Int(42),
            Semicolon,
        ])
    );
}

#[test]
fn insert_avec_une_chaine() {
    assert_eq!(
        tokenize("INSERT INTO users VALUES (1, 'alice');"),
        Ok(vec![
            Keyword(Kw::Insert),
            Keyword(Kw::Into),
            ident("users"),
            Keyword(Kw::Values),
            LParen,
            Int(1),
            Comma,
            text("alice"),
            RParen,
            Semicolon,
        ])
    );
}

#[test]
fn create_table() {
    assert_eq!(
        tokenize("CREATE TABLE users (id INTEGER, name TEXT);"),
        Ok(vec![
            Keyword(Kw::Create),
            Keyword(Kw::Table),
            ident("users"),
            LParen,
            ident("id"),
            Keyword(Kw::Integer),
            Comma,
            ident("name"),
            Keyword(Kw::Text),
            RParen,
            Semicolon,
        ])
    );
}

#[test]
fn les_espaces_ne_sont_pas_obligatoires() {
    assert_eq!(
        tokenize("select*from t where id=1;"),
        Ok(vec![
            Keyword(Kw::Select),
            Star,
            Keyword(Kw::From),
            ident("t"),
            Keyword(Kw::Where),
            ident("id"),
            Eq,
            Int(1),
            Semicolon,
        ])
    );
}

#[test]
fn une_requete_peut_tenir_sur_plusieurs_lignes() {
    let sql = "SELECT *\n\tFROM users\n\tWHERE id >= 10;";
    assert_eq!(
        tokenize(sql),
        Ok(vec![
            Keyword(Kw::Select),
            Star,
            Keyword(Kw::From),
            ident("users"),
            Keyword(Kw::Where),
            ident("id"),
            GtEq,
            Int(10),
            Semicolon,
        ])
    );
}
