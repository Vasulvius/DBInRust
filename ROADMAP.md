# minidb — feuille de route

Un moteur de base de données minimaliste inspiré de SQLite, écrit en Rust pour
apprendre les deux en même temps.

## Méthode

Une étape = une spécification exécutable (`tests/`) + une implémentation à
écrire. On ne passe à l'étape suivante que quand `cargo test` est vert et que
`cargo clippy` ne dit rien.

Après chaque étape :

```sh
cargo test
cargo clippy -- -D warnings   # le meilleur prof de Rust gratuit
cargo fmt
git commit
```

## Règles du jeu

- **Zéro dépendance externe** tant que possible. On écrit à la main ce que les
  crates feraient, puis on les découvre en sachant ce qu'elles remplacent.
- **Pas de `unsafe`.** Si le borrow checker refuse, c'est le design qu'il faut
  changer — et c'est justement là qu'on apprend.
- **Pas d'`unwrap()` hors des tests.** Les erreurs remontent.

## Les étapes

| #  | Étape                | Ce que tu construis                                    | Ce que ça t'apprend en Rust                      |
|----|----------------------|--------------------------------------------------------|--------------------------------------------------|
| 1  | REPL                 | boucle de lecture, méta-commandes `.exit` / `.help`     | `enum` à données, `&str` vs `String`, `match`     |
| 2  | Tokenizer            | SQL brut → `Vec<Token>`                                 | itérateurs, `Peekable`, découpage de `&str`       |
| 3  | Parser               | tokens → arbre syntaxique                               | descente récursive, `Box<T>`, `Result` et `?`     |
| 4  | Erreurs              | un type d'erreur unifié pour tout le moteur             | `Display`, `From`, `std::error::Error`            |
| 5  | Exécution en mémoire | `CREATE TABLE` / `INSERT` / `SELECT` en RAM             | `HashMap`, traits, premières vraies lifetimes     |
| 6  | Sérialisation        | une ligne ↔ une suite d'octets                          | `&[u8]`, `to_le_bytes`, découpage de slices       |
| 7  | Pager                | le fichier vu comme des pages de 4 Ko, avec cache       | `File`, `Seek`, ownership qui commence à piquer   |
| 8  | B-tree, feuilles     | nœud feuille, recherche binaire, éclatement             | indices plutôt que références                     |
| 9  | B-tree, arbre        | nœuds internes, arbre multi-niveaux                     | récursion sous contrainte du borrow checker       |
| 10 | `WHERE`              | évaluation d'expressions                                | pattern matching en profondeur                    |

Ensuite, au choix : machine virtuelle à bytecode (comme le vrai SQLite),
transactions et journal, index secondaires, jointures.

## État

- [x] **Étape 1 — REPL** (commit `006d2d9`)
      `classify` trie une ligne en méta-commande / SQL / vide, et `main.rs`
      fait tourner la boucle avec prompt, Ctrl-D et erreurs sur `stderr`.
      Au passage : le shadowing plutôt que `mut` sur un paramètre, les
      or-patterns dans un `match`, le tamponnage de `stdout` et pourquoi
      `flush()` existe, le type `!` de `break` et `panic!`.
- [ ] **Étape 2 — Tokenizer** ← en cours
      Spec dans `src/lexer.rs`, tests dans `tests/lexer.rs` (25 cas).

## Dettes assumées

Choses volontairement laissées de côté, à reprendre quand le besoin sera réel —
plutôt que de sur-concevoir maintenant.

| Quoi                                            | Prévu pour |
|-------------------------------------------------|------------|
| Positions dans le source pour les erreurs        | étape 4    |
| Nombres négatifs, flottants, commentaires `--`   | au besoin  |
| Identifiants entre guillemets (`"ma table"`)     | au besoin  |
| Arguments de méta-commandes (`.exit 0`)          | au besoin  |
