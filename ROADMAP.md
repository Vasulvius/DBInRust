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
| 5  | Exécution en mémoire | `CREATE TABLE` / `INSERT` / `SELECT` en RAM             | `HashMap`, `Default`, itérateurs sur deux niveaux |
| 6  | `WHERE`              | évaluation d'expressions sur une ligne                  | pattern matching en profondeur, récursion          |
| 7  | Sérialisation        | une ligne ↔ une suite d'octets                          | `&[u8]`, `to_le_bytes`, découpage de slices       |
| 8  | Pager                | le fichier vu comme des pages de 4 Ko, avec cache       | `File`, `Seek`, ownership qui commence à piquer   |
| 9  | B-tree, feuilles     | nœud feuille, recherche binaire, éclatement             | indices plutôt que références                     |
| 10 | B-tree, arbre        | nœuds internes, arbre multi-niveaux                     | récursion sous contrainte du borrow checker       |

Ensuite, au choix : machine virtuelle à bytecode (comme le vrai SQLite),
transactions et journal, index secondaires, jointures.

## État

- [x] **Étape 1 — REPL** (commit `006d2d9`)
      `classify` trie une ligne en méta-commande / SQL / vide, et `main.rs`
      fait tourner la boucle avec prompt, Ctrl-D et erreurs sur `stderr`.
      Au passage : le shadowing plutôt que `mut` sur un paramètre, les
      or-patterns dans un `match`, le tamponnage de `stdout` et pourquoi
      `flush()` existe, le type `!` de `break` et `panic!`.
- [x] **Étape 2 — Tokenizer**
      `tokenize` découpe le SQL en `Vec<Token>` ; un dispatcher à cinq bras où
      chaque classe de caractère délègue à une méthode qui consomme ce qu'elle
      prend. Seuls les espaces sont consommés en ligne — et ils ne produisent
      aucun token.
      Au passage : `Peekable` et le `peek` sans consommation, `?` compris comme
      un **opérateur** qui fait passer une expression de `Result<T, E>` à `T`,
      les gardes dans un `match`, et pourquoi un double emballage
      `Option<Result<_, _>>` est le symptôme d'un dispatcher qui ne tranche pas.
- [x] **Étape 3 — Parser**
      `parse(sql) -> Result<Statement, ParseError>` construit l'arbre par
      descente récursive : une méthode par règle de grammaire, quatre niveaux
      d'expression (`or` → `and` → `compare` → `primary`), et la seule vraie
      récursion dans le bras `LParen` de `primary`.
      Au passage : `Box` motivé par la taille infinie d'un `enum` récursif, la
      priorité encodée par la chaîne d'appels et non par une table,
      l'associativité à gauche obtenue en repliant dans une boucle,
      `From<LexError>` qui fait convertir `?` tout seul, et le trio
      `next`/`expect`/`eat` plus les extracteurs qui rendent chaque règle
      courte.
      Deux signaux réappris : un `Option<Result<_, _>>` (ou un paramètre
      `Option` d'accumulateur) trahit une répétition écrite en récursion, et un
      chemin d'erreur mort trahit une fonction qui répond pour un cas que son
      appelant a déjà écarté.
- [x] **Étape 4 — Erreurs**
      `Display` sur `Keyword`, `Token`, `LexError` et `ParseError` ;
      `std::error::Error` sur les deux erreurs, avec `source()` qui expose la
      cause ; positions en octets dans `LexError`. Le REPL affiche désormais
      `unterminated string starting at position 33` au lieu d'un `{:?}`.
      Au passage : premiers traits écrits à la main plutôt que dérivés,
      `Display` (humain) vs `Debug` (programmeur) comme choix de conception,
      `ToString` **dérivé** de `Display` et jamais l'inverse — l'appeler depuis
      `fmt` fait déborder la pile —, et `Box<dyn Error>` que `?` sait viser
      tout seul dès qu'`Error` est implémenté.
      La leçon annoncée depuis l'étape 2 a coûté 70 erreurs de compilation :
      voilà le prix d'un type d'erreur enrichi après coup.
- [ ] **Étape 5 — Exécution en mémoire** ← en cours
      Spec dans `src/exec.rs`, tests dans `tests/exec.rs` (32 cas).
      `Database::execute(sql) -> Result<Output, DbError>`.

## Dettes assumées

Choses volontairement laissées de côté, à reprendre quand le besoin sera réel —
plutôt que de sur-concevoir maintenant.

| Quoi                                            | Prévu pour |
|-------------------------------------------------|------------|
| Positions dans les erreurs du **parser**         | plus tard  |
| Affichage humain d'une erreur (extrait + curseur)| plus tard  |
| Nombres négatifs, flottants, commentaires `--`   | au besoin  |
| Identifiants entre guillemets (`"ma table"`)     | au besoin  |
| Arguments de méta-commandes (`.exit 0`)          | au besoin  |
