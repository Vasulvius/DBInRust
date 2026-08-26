use minidb::lexer::{Keyword, LexError, Token};
use minidb::parser::{
    ColumnDef, CompareOp, DataType, Expr, ParseError, Selection, Statement, Value, parse,
};

// Des constructeurs courts, pour que les arbres attendus restent lisibles.
// Remarque le `Box::new` : c'est lui qui rend `Expr` récursif possible.

fn col(name: &str) -> Expr {
    Expr::Column(name.to_string())
}
fn int(n: i64) -> Expr {
    Expr::Literal(Value::Int(n))
}
fn txt(s: &str) -> Expr {
    Expr::Literal(Value::Str(s.to_string()))
}
fn cmp(left: Expr, op: CompareOp, right: Expr) -> Expr {
    Expr::Compare {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}
fn and(left: Expr, right: Expr) -> Expr {
    Expr::And(Box::new(left), Box::new(right))
}
fn or(left: Expr, right: Expr) -> Expr {
    Expr::Or(Box::new(left), Box::new(right))
}
fn coldef(name: &str, data_type: DataType) -> ColumnDef {
    ColumnDef {
        name: name.to_string(),
        data_type,
    }
}
fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

// Raccourci pour les `SELECT` sans `WHERE`, qui reviennent souvent.
fn select(selection: Selection, table: &str, filter: Option<Expr>) -> Statement {
    Statement::Select {
        selection,
        table: table.to_string(),
        filter,
    }
}

// --- CREATE TABLE -----------------------------------------------------------

#[test]
fn create_table_deux_colonnes() {
    assert_eq!(
        parse("CREATE TABLE users (id INTEGER, name TEXT);"),
        Ok(Statement::CreateTable {
            table: "users".to_string(),
            columns: vec![
                coldef("id", DataType::Integer),
                coldef("name", DataType::Text),
            ],
        })
    );
}

#[test]
fn create_table_une_seule_colonne() {
    assert_eq!(
        parse("CREATE TABLE t (a INTEGER);"),
        Ok(Statement::CreateTable {
            table: "t".to_string(),
            columns: vec![coldef("a", DataType::Integer)],
        })
    );
}

// --- INSERT -----------------------------------------------------------------

#[test]
fn insert_sans_liste_de_colonnes() {
    assert_eq!(
        parse("INSERT INTO users VALUES (1, 'alice');"),
        Ok(Statement::Insert {
            table: "users".to_string(),
            columns: None,
            values: vec![Value::Int(1), Value::Str("alice".to_string())],
        })
    );
}

#[test]
fn insert_avec_liste_de_colonnes() {
    assert_eq!(
        parse("INSERT INTO users (id, name) VALUES (1, 'alice');"),
        Ok(Statement::Insert {
            table: "users".to_string(),
            columns: Some(names(&["id", "name"])),
            values: vec![Value::Int(1), Value::Str("alice".to_string())],
        })
    );
}

#[test]
fn insert_une_seule_valeur() {
    assert_eq!(
        parse("INSERT INTO t VALUES (42);"),
        Ok(Statement::Insert {
            table: "t".to_string(),
            columns: None,
            values: vec![Value::Int(42)],
        })
    );
}

// --- SELECT -----------------------------------------------------------------

#[test]
fn select_etoile() {
    assert_eq!(
        parse("SELECT * FROM users;"),
        Ok(select(Selection::All, "users", None))
    );
}

#[test]
fn select_liste_de_colonnes() {
    assert_eq!(
        parse("SELECT id, name FROM users;"),
        Ok(select(
            Selection::Columns(names(&["id", "name"])),
            "users",
            None
        ))
    );
}

#[test]
fn select_avec_where() {
    assert_eq!(
        parse("SELECT * FROM users WHERE id = 42;"),
        Ok(select(
            Selection::All,
            "users",
            Some(cmp(col("id"), CompareOp::Eq, int(42)))
        ))
    );
}

#[test]
fn select_avec_where_sur_une_chaine() {
    assert_eq!(
        parse("SELECT * FROM users WHERE name = 'bob';"),
        Ok(select(
            Selection::All,
            "users",
            Some(cmp(col("name"), CompareOp::Eq, txt("bob")))
        ))
    );
}

// --- ponctuation et casse ---------------------------------------------------

#[test]
fn le_point_virgule_final_est_facultatif() {
    assert_eq!(parse("SELECT * FROM users"), parse("SELECT * FROM users;"));
    assert_eq!(
        parse("CREATE TABLE t (a INTEGER)"),
        parse("CREATE TABLE t (a INTEGER);")
    );
}

#[test]
fn les_mots_cles_sont_insensibles_a_la_casse() {
    assert_eq!(
        parse("select * from users where id = 1"),
        parse("SELECT * FROM users WHERE id = 1")
    );
}

#[test]
fn la_casse_des_identifiants_est_conservee() {
    assert_eq!(
        parse("SELECT Name FROM Users"),
        Ok(select(Selection::Columns(names(&["Name"])), "Users", None))
    );
}

// --- opérateurs de comparaison ----------------------------------------------

#[test]
fn tous_les_operateurs_de_comparaison() {
    let cas = [
        ("=", CompareOp::Eq),
        ("!=", CompareOp::NotEq),
        ("<>", CompareOp::NotEq),
        ("<", CompareOp::Lt),
        ("<=", CompareOp::LtEq),
        (">", CompareOp::Gt),
        (">=", CompareOp::GtEq),
    ];
    for (symbole, attendu) in cas {
        let sql = format!("SELECT * FROM t WHERE a {symbole} 1");
        assert_eq!(
            parse(&sql),
            Ok(select(
                Selection::All,
                "t",
                Some(cmp(col("a"), attendu, int(1)))
            )),
            "pour l'opérateur {symbole}"
        );
    }
}

// --- priorités et associativité ---------------------------------------------

#[test]
fn where_avec_and() {
    assert_eq!(
        parse("SELECT * FROM t WHERE a = 1 AND b = 2"),
        Ok(select(
            Selection::All,
            "t",
            Some(and(
                cmp(col("a"), CompareOp::Eq, int(1)),
                cmp(col("b"), CompareOp::Eq, int(2)),
            ))
        ))
    );
}

#[test]
fn where_avec_or() {
    assert_eq!(
        parse("SELECT * FROM t WHERE a = 1 OR b = 2"),
        Ok(select(
            Selection::All,
            "t",
            Some(or(
                cmp(col("a"), CompareOp::Eq, int(1)),
                cmp(col("b"), CompareOp::Eq, int(2)),
            ))
        ))
    );
}

#[test]
fn and_est_prioritaire_sur_or() {
    // a = 1 AND b = 2 OR c = 3   se lit   (a = 1 AND b = 2) OR (c = 3)
    assert_eq!(
        parse("SELECT * FROM t WHERE a = 1 AND b = 2 OR c = 3"),
        Ok(select(
            Selection::All,
            "t",
            Some(or(
                and(
                    cmp(col("a"), CompareOp::Eq, int(1)),
                    cmp(col("b"), CompareOp::Eq, int(2)),
                ),
                cmp(col("c"), CompareOp::Eq, int(3)),
            ))
        ))
    );
}

#[test]
fn and_est_prioritaire_sur_or_aussi_a_gauche() {
    // a = 1 OR b = 2 AND c = 3   se lit   (a = 1) OR (b = 2 AND c = 3)
    assert_eq!(
        parse("SELECT * FROM t WHERE a = 1 OR b = 2 AND c = 3"),
        Ok(select(
            Selection::All,
            "t",
            Some(or(
                cmp(col("a"), CompareOp::Eq, int(1)),
                and(
                    cmp(col("b"), CompareOp::Eq, int(2)),
                    cmp(col("c"), CompareOp::Eq, int(3)),
                ),
            ))
        ))
    );
}

#[test]
fn or_est_associatif_a_gauche() {
    // a OR b OR c   ->   Or(Or(a, b), c)   et non   Or(a, Or(b, c))
    assert_eq!(
        parse("SELECT * FROM t WHERE a = 1 OR b = 2 OR c = 3"),
        Ok(select(
            Selection::All,
            "t",
            Some(or(
                or(
                    cmp(col("a"), CompareOp::Eq, int(1)),
                    cmp(col("b"), CompareOp::Eq, int(2)),
                ),
                cmp(col("c"), CompareOp::Eq, int(3)),
            ))
        ))
    );
}

#[test]
fn and_est_associatif_a_gauche() {
    assert_eq!(
        parse("SELECT * FROM t WHERE a = 1 AND b = 2 AND c = 3"),
        Ok(select(
            Selection::All,
            "t",
            Some(and(
                and(
                    cmp(col("a"), CompareOp::Eq, int(1)),
                    cmp(col("b"), CompareOp::Eq, int(2)),
                ),
                cmp(col("c"), CompareOp::Eq, int(3)),
            ))
        ))
    );
}

// --- parenthèses ------------------------------------------------------------

#[test]
fn les_parentheses_forcent_le_regroupement() {
    // Sans parenthèses, le AND l'emporterait. Avec, c'est le OR qui se noue.
    assert_eq!(
        parse("SELECT * FROM t WHERE (a = 1 OR b = 2) AND c = 3"),
        Ok(select(
            Selection::All,
            "t",
            Some(and(
                or(
                    cmp(col("a"), CompareOp::Eq, int(1)),
                    cmp(col("b"), CompareOp::Eq, int(2)),
                ),
                cmp(col("c"), CompareOp::Eq, int(3)),
            ))
        ))
    );
}

#[test]
fn les_parentheses_peuvent_etre_imbriquees() {
    assert_eq!(
        parse("SELECT * FROM t WHERE ((a = 1))"),
        Ok(select(
            Selection::All,
            "t",
            Some(cmp(col("a"), CompareOp::Eq, int(1)))
        ))
    );
}

#[test]
fn une_comparaison_peut_se_reduire_a_un_seul_terme() {
    // La grammaire l'autorise : `parse` ne juge que la syntaxe. Que `WHERE a`
    // ait un sens ou non, c'est l'affaire de l'exécution.
    assert_eq!(
        parse("SELECT * FROM t WHERE a"),
        Ok(select(Selection::All, "t", Some(col("a"))))
    );
}

#[test]
fn un_and_entre_parentheses_est_regroupe() {
    // Le pendant du test précédent : c'est le AND qui est isolé cette fois.
    // Sans parenthèses, `a = 1 AND b = 2 OR c = 3` donnerait le même arbre,
    // mais le parser doit savoir traiter les deux opérateurs dans une
    // parenthèse, pas seulement le OR.
    assert_eq!(
        parse("SELECT * FROM t WHERE (a = 1 AND b = 2) OR c = 3"),
        Ok(select(
            Selection::All,
            "t",
            Some(or(
                and(
                    cmp(col("a"), CompareOp::Eq, int(1)),
                    cmp(col("b"), CompareOp::Eq, int(2)),
                ),
                cmp(col("c"), CompareOp::Eq, int(3)),
            ))
        ))
    );
}

#[test]
fn une_parenthese_contient_une_expression_complete() {
    // Une parenthèse repart du sommet de la grammaire : tout ce qu'accepte
    // `expr` doit être accepté à l'intérieur.
    assert_eq!(
        parse("SELECT * FROM t WHERE (a = 1 AND b = 2)"),
        Ok(select(
            Selection::All,
            "t",
            Some(and(
                cmp(col("a"), CompareOp::Eq, int(1)),
                cmp(col("b"), CompareOp::Eq, int(2)),
            ))
        ))
    );
}

// --- les deux côtés d'une comparaison sont des `primary` ---------------------

#[test]
fn une_comparaison_accepte_une_colonne_a_droite() {
    // `comparison := primary ( op primary )?` — les deux côtés suivent la
    // même règle. Comparer deux colonnes est syntaxiquement valide.
    assert_eq!(
        parse("SELECT * FROM t WHERE a = b"),
        Ok(select(
            Selection::All,
            "t",
            Some(cmp(col("a"), CompareOp::Eq, col("b")))
        ))
    );
}

#[test]
fn une_comparaison_accepte_un_litteral_a_gauche() {
    assert_eq!(
        parse("SELECT * FROM t WHERE 1 = a"),
        Ok(select(
            Selection::All,
            "t",
            Some(cmp(int(1), CompareOp::Eq, col("a")))
        ))
    );
}

#[test]
fn une_parenthese_peut_entourer_un_operande() {
    // `primary := ident | value | "(" expr ")"` — la parenthèse est l'une des
    // formes de `primary`. Elle est donc acceptée partout où un opérande l'est,
    // des deux côtés de l'opérateur, et pas seulement en tête d'expression.
    assert_eq!(
        parse("SELECT * FROM t WHERE a = (1)"),
        Ok(select(
            Selection::All,
            "t",
            Some(cmp(col("a"), CompareOp::Eq, int(1)))
        ))
    );
    assert_eq!(
        parse("SELECT * FROM t WHERE (a) = 1"),
        Ok(select(
            Selection::All,
            "t",
            Some(cmp(col("a"), CompareOp::Eq, int(1)))
        ))
    );
}

// --- une vraie requête ------------------------------------------------------

#[test]
fn une_requete_complete() {
    assert_eq!(
        parse("SELECT id, name FROM users WHERE age >= 18 AND (name = 'bob' OR name = 'alice');"),
        Ok(select(
            Selection::Columns(names(&["id", "name"])),
            "users",
            Some(and(
                cmp(col("age"), CompareOp::GtEq, int(18)),
                or(
                    cmp(col("name"), CompareOp::Eq, txt("bob")),
                    cmp(col("name"), CompareOp::Eq, txt("alice")),
                ),
            ))
        ))
    );
}

// --- erreurs ----------------------------------------------------------------

#[test]
fn une_entree_vide_est_une_erreur() {
    assert_eq!(parse(""), Err(ParseError::UnexpectedEnd));
    assert_eq!(parse("   \n"), Err(ParseError::UnexpectedEnd));
}

#[test]
fn une_requete_tronquee_est_une_erreur() {
    assert_eq!(parse("SELECT * FROM"), Err(ParseError::UnexpectedEnd));
    assert_eq!(
        parse("SELECT * FROM t WHERE"),
        Err(ParseError::UnexpectedEnd)
    );
    assert_eq!(parse("CREATE TABLE t ("), Err(ParseError::UnexpectedEnd));
    assert_eq!(parse("INSERT INTO t"), Err(ParseError::UnexpectedEnd));
}

#[test]
fn une_parenthese_non_fermee_est_une_erreur() {
    assert_eq!(
        parse("SELECT * FROM t WHERE (a = 1"),
        Err(ParseError::UnexpectedEnd)
    );
}

#[test]
fn un_token_a_la_mauvaise_place_est_signale() {
    assert_eq!(
        parse("SELECT FROM users"),
        Err(ParseError::UnexpectedToken(Token::Keyword(Keyword::From)))
    );
}

#[test]
fn une_commande_inconnue_est_signalee() {
    // `DELETE` n'est pas un mot-clé de notre langage : le lexer en fait un
    // identifiant, et le parser ne sait pas quoi en faire.
    assert_eq!(
        parse("DELETE FROM users"),
        Err(ParseError::UnexpectedToken(Token::Ident(
            "DELETE".to_string()
        )))
    );
}

#[test]
fn du_texte_en_trop_apres_le_statement_est_une_erreur() {
    assert_eq!(
        parse("SELECT * FROM users foo"),
        Err(ParseError::UnexpectedToken(Token::Ident("foo".to_string())))
    );
    assert_eq!(
        parse("SELECT * FROM users;;"),
        Err(ParseError::UnexpectedToken(Token::Semicolon))
    );
}

#[test]
fn une_erreur_du_lexer_remonte_telle_quelle() {
    assert_eq!(
        parse("SELECT # FROM t"),
        Err(ParseError::Lex(LexError::UnexpectedChar { ch: '#', at: 7 }))
    );
    assert_eq!(
        parse("SELECT * FROM t WHERE name = 'bob"),
        Err(ParseError::Lex(LexError::UnterminatedString { at: 29 }))
    );
}
