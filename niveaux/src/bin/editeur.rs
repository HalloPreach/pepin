//! Prépare les données de l'éditeur de niveaux (carte 10.3a), pour le build de
//! QA seulement (`./build.sh --qa`, jamais le jeu publié) :
//!
//!   ./docker-cargo.sh run -p niveaux --bin editeur -- DOSSIER
//!
//! écrit dans DOSSIER :
//! - `legende.json` : les lettres de la grille, les thèmes et les genres
//!   (`niveaux::legende`, la seule source : l'éditeur n'en recopie aucune) ;
//! - `niveaux/<monde_N ou essais>/<cle>.ron` : une copie de chaque fichier de
//!   niveau, que l'éditeur peut ouvrir. Copiée depuis `assets/levels` (et non
//!   depuis le texte embarqué, qui n'a plus ses commentaires `//` depuis la carte
//!   10.1c) : l'éditeur les montre et les garde à l'export ;
//! - `niveaux/index.json` : la liste de ces fichiers (un serveur de fichiers
//!   statiques ne sait pas lister un dossier) ;
//! - `sprites/<famille>/<clé>.png` (carte 10.4b) : le PNG de chaque vignette de la
//!   légende (`niveaux::legende::VIGNETTES`), copié depuis `assets/sprites`.

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
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../assets/levels")
            .join(fichier.chemin);
        fs::copy(&source, &destination).unwrap();
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
    // Carte 10.4b : les PNG des vignettes, copiés tels quels depuis
    // `assets/sprites` (seulement ceux d'une lettre de la légende).
    let mut vignettes = Vec::new();
    for l in niveaux::legende::LETTRES {
        let Some((chemin, _)) = niveaux::legende::vignette(l.lettre) else {
            continue;
        };
        if vignettes.contains(&chemin) {
            continue;
        }
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../assets/sprites")
            .join(format!("{chemin}.png"));
        let destination = dossier.join("sprites").join(format!("{chemin}.png"));
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(&source, &destination).unwrap();
        vignettes.push(chemin);
    }
    println!(
        "éditeur : legende.json ({} lettres, {} vignettes) et {} niveaux écrits dans {}",
        niveaux::legende::LETTRES.len(),
        vignettes.len(),
        chemins.len(),
        dossier.display()
    );
    ExitCode::SUCCESS
}
