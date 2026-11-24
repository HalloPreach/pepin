//! Exporteur (carte 10.1a) : écrit un fichier `.ron` par niveau Rust actuel
//! (`assets/levels/monde_<N>/<cle>.ron` et `assets/levels/essais/<cle>.ron`),
//! puis recopie la légende de la grille dans `assets/levels/LISEZMOI.md`.
//!
//! Le format est STABLE : relancer l'exporteur redonne les mêmes octets.
//!
//!   ./docker-cargo.sh run -p niveaux --bin exporter

use niveaux::source_rust::{avec_legende, legende, sources};
use std::fs;
use std::path::PathBuf;

fn main() {
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let dossier = racine.join("assets/levels");
    let mut ecrits = 0;
    for (chemin, infos) in sources() {
        let destination = dossier.join(&chemin);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(&destination, niveaux::ecrire(infos)).unwrap();
        ecrits += 1;
    }
    println!("{ecrits} fichiers de niveau écrits sous assets/levels/");

    let source = fs::read_to_string(racine.join("core/src/level.rs")).unwrap();
    let chemin_lisezmoi = dossier.join("LISEZMOI.md");
    let lisezmoi = fs::read_to_string(&chemin_lisezmoi).unwrap_or_default();
    match avec_legende(&lisezmoi, &legende(&source)) {
        Some(texte) => {
            fs::write(&chemin_lisezmoi, texte).unwrap();
            println!("légende recopiée dans assets/levels/LISEZMOI.md");
        }
        None => println!("LISEZMOI.md sans marques de légende : légende non recopiée"),
    }
}
