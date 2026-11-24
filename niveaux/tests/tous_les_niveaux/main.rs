//! Les tests de TOUS les niveaux du jeu (carte 10.1c).
//!
//! Avant la carte 10.1c, chaque niveau était un fichier Rust de `core`
//! (`core/src/levels/<clé>.rs`), et les tests qui lisaient les vrais niveaux
//! vivaient dans `core`. Les niveaux sont maintenant des fichiers RON
//! (`assets/levels/`), lus par ce crate : leurs tests sont donc ICI, un fichier
//! par sujet. `core` garde ses tests de logique pure, sur de faux niveaux écrits
//! dans les tests eux-mêmes ; il ne dépend toujours de rien.
//!
//!   ./docker-cargo.sh test -p niveaux                     (tests rapides)
//!   ./docker-cargo.sh carte <clé>                         (ceux d'une carte de niveau)
//!   ./docker-cargo.sh lot                                 (avec les tests de lot)

mod commun;
mod decor;
mod essais;
mod faisabilite;
mod grilles;
mod lettres;
mod par_niveau;
mod pieuvre_montee;
mod pigeons;
mod table;
mod variantes;
