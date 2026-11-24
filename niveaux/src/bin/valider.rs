//! Valide des fichiers de niveau (carte 10.1a), pour un humain ou un agent qui
//! vient d'écrire ou de modifier un `.ron`.
//!
//!   ./docker-cargo.sh run -p niveaux --bin valider -- assets/levels/monde_1/toits.ron
//!       vérifie ce fichier seul (syntaxe, champs, grille, aiguilles, poissons) ;
//!   ./docker-cargo.sh run -p niveaux --bin valider
//!       vérifie TOUT `assets/levels` lu sur le disque, comme le jeu : chaque
//!       fichier, plus les dossiers, les clés et l'ordre des niveaux.
//!
//! Code de sortie 0 si tout est valide, 1 sinon (le message dit où est l'erreur).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let resultat = if arguments.is_empty() {
        valider_tout()
    } else {
        arguments.iter().try_for_each(|chemin| {
            let texte = fs::read_to_string(chemin).map_err(|e| format!("{chemin} : {e}"))?;
            niveaux::charger_fichier(chemin, &texte)
                .map(|infos| println!("{chemin} : valide (« {} »)", infos.nom))
                .map_err(|e| e.to_string())
        })
    };
    match resultat {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("ERREUR : {message}");
            ExitCode::FAILURE
        }
    }
}

/// Toute l'arborescence `assets/levels`, lue sur le disque.
fn valider_tout() -> Result<(), String> {
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/levels");
    let mut fichiers = Vec::new();
    lister(&racine, &racine, &mut fichiers);
    fichiers.sort();
    let paires: Vec<(&str, &str)> = fichiers
        .iter()
        .map(|(c, t)| (c.as_str(), t.as_str()))
        .collect();
    let table = niveaux::charger_table(&paires).map_err(|e| e.to_string())?;
    println!(
        "assets/levels : valide ({} niveaux du parcours, {} niveaux d'essai)",
        table.niveaux.len(),
        table.essais.len()
    );
    Ok(())
}

/// (chemin relatif avec des `/`, texte) de chaque `.ron` sous `dossier`.
fn lister(racine: &Path, dossier: &Path, fichiers: &mut Vec<(String, String)>) {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            lister(racine, &chemin, fichiers);
        } else if chemin.extension().is_some_and(|e| e == "ron") {
            let relatif: Vec<String> = chemin
                .strip_prefix(racine)
                .unwrap()
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            let texte = fs::read_to_string(&chemin).unwrap_or_default();
            fichiers.push((relatif.join("/"), texte));
        }
    }
}
