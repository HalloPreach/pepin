//! Recopie la légende de la grille (le bloc ```` ```text ```` du haut de
//! `core/src/level.rs`) dans `assets/levels/LISEZMOI.md`, entre ses deux marques.
//! À relancer quand la légende de `core` change (le test
//! `la_legende_du_lisezmoi_est_a_jour` le rappelle).
//!
//!   ./docker-cargo.sh run -p niveaux --bin lisezmoi

use niveaux::lisezmoi::{avec_legende, legende};
use std::fs;
use std::path::PathBuf;

fn main() {
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let source = fs::read_to_string(racine.join("core/src/level.rs")).unwrap();
    let chemin_lisezmoi = racine.join("assets/levels/LISEZMOI.md");
    let lisezmoi = fs::read_to_string(&chemin_lisezmoi).unwrap_or_default();
    match avec_legende(&lisezmoi, &legende(&source)) {
        Some(texte) => {
            fs::write(&chemin_lisezmoi, texte).unwrap();
            println!("légende recopiée dans assets/levels/LISEZMOI.md");
        }
        None => println!("LISEZMOI.md sans marques de légende : légende non recopiée"),
    }
}
