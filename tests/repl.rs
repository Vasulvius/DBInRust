use minidb::repl::{Input, MetaCommand, classify};

// --- méta-commandes ---------------------------------------------------------

#[test]
fn exit_est_reconnu() {
    assert_eq!(classify(".exit"), Input::Meta(MetaCommand::Exit));
}

#[test]
fn quit_est_un_alias_de_exit() {
    assert_eq!(classify(".quit"), Input::Meta(MetaCommand::Exit));
}

#[test]
fn help_est_reconnu() {
    assert_eq!(classify(".help"), Input::Meta(MetaCommand::Help));
}

#[test]
fn les_noms_de_meta_commandes_ignorent_la_casse() {
    assert_eq!(classify(".EXIT"), Input::Meta(MetaCommand::Exit));
    assert_eq!(classify(".Help"), Input::Meta(MetaCommand::Help));
}

#[test]
fn une_meta_commande_inconnue_conserve_le_texte_saisi() {
    assert_eq!(
        classify(".tables"),
        Input::Meta(MetaCommand::Unknown(".tables".to_string()))
    );
}

#[test]
fn une_meta_commande_avec_argument_nest_pas_reconnue() {
    // On ne gère pas encore les arguments : `.exit 0` n'est pas `.exit`.
    assert_eq!(
        classify(".exit 0"),
        Input::Meta(MetaCommand::Unknown(".exit 0".to_string()))
    );
}

#[test]
fn un_point_seul_est_une_meta_commande_inconnue() {
    assert_eq!(
        classify("."),
        Input::Meta(MetaCommand::Unknown(".".to_string()))
    );
}

// --- SQL --------------------------------------------------------------------

#[test]
fn une_ligne_quelconque_est_du_sql() {
    assert_eq!(
        classify("select * from users;"),
        Input::Sql("select * from users;".to_string())
    );
}

#[test]
fn le_sql_est_conserve_tel_quel_casse_comprise() {
    // `classify` ne fait que trier. Normaliser le SQL, ce sera le travail du
    // tokenizer à l'étape 2.
    assert_eq!(
        classify("SELECT Name FROM Users"),
        Input::Sql("SELECT Name FROM Users".to_string())
    );
}

// --- espaces et lignes vides ------------------------------------------------

#[test]
fn les_espaces_autour_sont_ignores() {
    assert_eq!(classify("   .exit  "), Input::Meta(MetaCommand::Exit));
    assert_eq!(
        classify("\tselect 1;\n"),
        Input::Sql("select 1;".to_string())
    );
}

#[test]
fn une_ligne_vide_ne_produit_rien() {
    assert_eq!(classify(""), Input::Empty);
    assert_eq!(classify("      "), Input::Empty);
    assert_eq!(classify("\n"), Input::Empty);
}
