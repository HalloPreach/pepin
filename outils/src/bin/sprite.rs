//! L'outil `sprite` (carte 10.4a) : passer d'un sprite PNG à sa grille de texte, et
//! retour.
//!
//!   cargo run -p outils --bin sprite -- lire  assets/sprites/ennemis/corbeau0.png
//!       affiche la grille du PNG ;
//!   cargo run -p outils --bin sprite -- ecrire assets/sprites/ennemis/corbeau0.png < grille.txt
//!       refait le PNG depuis la grille lue sur l'entrée standard.
//!
//! L'écriture est canonique : `lire` puis `ecrire` redonne les mêmes octets.

use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultat = match args.as_slice() {
        [commande, fichier] if commande == "lire" => lire(fichier),
        [commande, fichier] if commande == "ecrire" => ecrire(fichier),
        _ => Err(
            "usage : sprite lire <fichier.png>  |  sprite ecrire <fichier.png> < grille.txt".into(),
        ),
    };
    match resultat {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("sprite : {e}");
            ExitCode::FAILURE
        }
    }
}

fn lire(fichier: &str) -> Result<(), String> {
    let octets = std::fs::read(fichier).map_err(|e| format!("{fichier} : {e}"))?;
    for ligne in outils::lire_png(&octets).map_err(|e| format!("{fichier} : {e}"))? {
        println!("{ligne}");
    }
    Ok(())
}

fn ecrire(fichier: &str) -> Result<(), String> {
    let mut texte = String::new();
    std::io::stdin()
        .read_to_string(&mut texte)
        .map_err(|e| e.to_string())?;
    // Les lignes vides (fin de fichier, par exemple) ne sont pas des rangées.
    let grille: Vec<String> = texte
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    let octets = outils::ecrire_png(&grille)?;
    std::fs::write(fichier, octets).map_err(|e| format!("{fichier} : {e}"))
}
