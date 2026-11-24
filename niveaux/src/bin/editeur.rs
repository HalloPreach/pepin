//! Prépare les données de l'éditeur de niveaux (carte 10.3a), pour le build de
//! QA seulement (`./build.sh --qa`, jamais le jeu publié) :
//!
//!   ./docker-cargo.sh run -p niveaux --bin editeur -- DOSSIER
//!
//! écrit dans DOSSIER :
//! - `legende.json` : les lettres de la grille, les thèmes et les genres
//!   (`niveaux::legende`, la seule source : l'éditeur n'en recopie aucune) ;
//! - `niveaux/<monde_N ou essais>/<cle>.ron` : une copie de chaque fichier de
//!   niveau embarqué, que l'éditeur peut ouvrir ;
//! - `niveaux/index.json` : la liste de ces fichiers (un serveur de fichiers
//!   statiques ne sait pas lister un dossier).

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(dossier) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("Usage : editeur DOSSIER");
        return ExitCode::FAILURE;
    };
    let niveaux_dir = dossier.join("niveaux");
    let mut chemins = Vec::new();
    for fichier in niveaux::FICHIERS.iter().chain(niveaux::FICHIERS_ESSAIS) {
        let destination = niveaux_dir.join(fichier.chemin);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(&destination, fichier.texte).unwrap();
        chemins.push(format!("{:?}", fichier.chemin));
    }
    fs::write(
        niveaux_dir.join("index.json"),
        format!("[\n  {}\n]\n", chemins.join(",\n  ")),
    )
    .unwrap();
    fs::write(
        dossier.join("legende.json"),
        niveaux::legende::legende_json(),
    )
    .unwrap();
    println!(
        "éditeur : legende.json ({} lettres) et {} niveaux écrits dans {}",
        niveaux::legende::LETTRES.len(),
        chemins.len(),
        dossier.display()
    );
    ExitCode::SUCCESS
}
