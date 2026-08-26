use std::error::Error;

use minidb::lexer::{Keyword, LexError, Token, tokenize};
use minidb::parser::{ParseError, Statement, parse};

// --- Display sur les tokens -------------------------------------------------

#[test]
fn un_mot_cle_saffiche_en_majuscules() {
    assert_eq!(Keyword::Select.to_string(), "SELECT");
    assert_eq!(Keyword::From.to_string(), "FROM");
    assert_eq!(Keyword::Integer.to_string(), "INTEGER");
}

#[test]
fn un_token_saffiche_comme_dans_le_source() {
    // `to_string()` est offert gratuitement par `Display` : tout type qui
    // l'implémente gagne `ToString` sans rien écrire de plus.
    assert_eq!(Token::Keyword(Keyword::Select).to_string(), "SELECT");
    assert_eq!(Token::Ident("users".to_string()).to_string(), "users");
    assert_eq!(Token::Int(42).to_string(), "42");
    assert_eq!(Token::Str("bob".to_string()).to_string(), "'bob'");
}

#[test]
fn la_ponctuation_et_les_operateurs_saffichent_tels_quels() {
    assert_eq!(Token::Comma.to_string(), ",");
    assert_eq!(Token::Semicolon.to_string(), ";");
    assert_eq!(Token::LParen.to_string(), "(");
    assert_eq!(Token::RParen.to_string(), ")");
    assert_eq!(Token::Star.to_string(), "*");
    assert_eq!(Token::Eq.to_string(), "=");
    assert_eq!(Token::NotEq.to_string(), "!=");
    assert_eq!(Token::Lt.to_string(), "<");
    assert_eq!(Token::LtEq.to_string(), "<=");
    assert_eq!(Token::Gt.to_string(), ">");
    assert_eq!(Token::GtEq.to_string(), ">=");
}

// --- Display sur les erreurs ------------------------------------------------

#[test]
fn les_messages_du_lexer() {
    assert_eq!(
        LexError::UnexpectedChar { ch: '#', at: 7 }.to_string(),
        "unexpected character '#' at position 7"
    );
    assert_eq!(
        LexError::UnterminatedString { at: 29 }.to_string(),
        "unterminated string starting at position 29"
    );
    assert_eq!(
        LexError::NumberOutOfRange {
            text: "99999999999999999999".to_string(),
            at: 4
        }
        .to_string(),
        "number out of range at position 4: 99999999999999999999"
    );
}

#[test]
fn les_messages_du_parser() {
    assert_eq!(
        ParseError::UnexpectedToken(Token::Keyword(Keyword::From)).to_string(),
        "unexpected token: FROM"
    );
    assert_eq!(
        ParseError::UnexpectedToken(Token::RParen).to_string(),
        "unexpected token: )"
    );
    assert_eq!(
        ParseError::UnexpectedEnd.to_string(),
        "unexpected end of input"
    );
}

#[test]
fn une_erreur_lexicale_enveloppee_delegue_son_message() {
    // `ParseError::Lex` ne rajoute pas sa propre couche de texte : il laisse
    // parler l'erreur qu'il transporte. Sinon l'utilisateur lirait deux fois
    // la même chose une fois la chaîne des causes déroulée.
    let inner = LexError::UnexpectedChar { ch: '#', at: 7 };
    assert_eq!(
        ParseError::Lex(inner).to_string(),
        "unexpected character '#' at position 7"
    );
}

// --- les positions ----------------------------------------------------------

#[test]
fn un_caractere_inconnu_porte_sa_position() {
    assert_eq!(
        tokenize("select # from t"),
        Err(LexError::UnexpectedChar { ch: '#', at: 7 })
    );
    assert_eq!(
        tokenize("a ! b"),
        Err(LexError::UnexpectedChar { ch: '!', at: 2 })
    );
}

#[test]
fn la_position_est_un_decalage_en_octets_pas_en_caracteres() {
    // `é` occupe deux octets en UTF-8. Le `#` est le 5e caractère, mais il
    // commence au 6e octet — et c'est l'octet qui fait foi.
    assert_eq!(
        tokenize("'é' #"),
        Err(LexError::UnexpectedChar { ch: '#', at: 5 })
    );
}

#[test]
fn une_chaine_non_fermee_pointe_son_apostrophe_ouvrante() {
    assert_eq!(
        tokenize("'abc"),
        Err(LexError::UnterminatedString { at: 0 })
    );
    assert_eq!(
        tokenize("x = 'abc"),
        Err(LexError::UnterminatedString { at: 4 })
    );
}

#[test]
fn un_nombre_hors_bornes_pointe_son_premier_chiffre() {
    assert_eq!(
        tokenize("a = 99999999999999999999"),
        Err(LexError::NumberOutOfRange {
            text: "99999999999999999999".to_string(),
            at: 4
        })
    );
}

// --- de bout en bout --------------------------------------------------------

#[test]
fn les_messages_rendus_par_parse() {
    // Ce que ton REPL affichera désormais, au lieu d'un vidage de structure.
    assert_eq!(
        parse("SELECT * FROM users WHERE name = 'bob")
            .unwrap_err()
            .to_string(),
        "unterminated string starting at position 33"
    );
    assert_eq!(
        parse("SELECT FROM users").unwrap_err().to_string(),
        "unexpected token: FROM"
    );
    assert_eq!(
        parse("SELECT * FROM t WHERE (a = 1")
            .unwrap_err()
            .to_string(),
        "unexpected end of input"
    );
    assert_eq!(
        parse("SELECT # FROM t").unwrap_err().to_string(),
        "unexpected character '#' at position 7"
    );
}

// --- std::error::Error ------------------------------------------------------

#[test]
fn source_expose_la_cause_sous_jacente() {
    let err = parse("SELECT # FROM t").unwrap_err();
    let cause = err.source().expect("ParseError::Lex doit exposer sa cause");
    assert_eq!(cause.to_string(), "unexpected character '#' at position 7");
}

#[test]
fn les_erreurs_sans_cause_nont_pas_de_source() {
    assert!(ParseError::UnexpectedEnd.source().is_none());
    assert!(ParseError::UnexpectedToken(Token::Star).source().is_none());
    assert!(LexError::UnterminatedString { at: 0 }.source().is_none());
}

#[test]
fn une_erreur_devient_un_objet_trait() {
    let err: Box<dyn Error> = Box::new(parse("SELECT FROM users").unwrap_err());
    assert_eq!(err.to_string(), "unexpected token: FROM");
}

#[test]
fn le_point_dinterrogation_convertit_vers_box_dyn_error() {
    // C'est le vrai bénéfice d'implémenter `Error` : la bibliothèque standard
    // fournit un `From<E> for Box<dyn Error>` pour tout `E: Error + 'static`.
    // Le `?` s'en sert tout seul, sans que tu écrives la moindre conversion.
    fn run(sql: &str) -> Result<Statement, Box<dyn Error>> {
        Ok(parse(sql)?)
    }

    assert!(run("SELECT * FROM t").is_ok());
    assert_eq!(
        run("SELECT FROM t").unwrap_err().to_string(),
        "unexpected token: FROM"
    );
}
