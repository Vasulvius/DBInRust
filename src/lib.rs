//! minidb — un moteur de base de données minimaliste inspiré de SQLite.
//!
//! Tout le code utile vit dans cette bibliothèque ; `src/main.rs` n'est qu'une
//! fine couche d'entrée/sortie par-dessus. C'est ce découpage qui rend le
//! moteur testable : les tests d'intégration de `tests/` ne peuvent voir que
//! ce qui est exposé ici.

pub mod repl;
