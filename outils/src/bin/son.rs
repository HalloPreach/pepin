//! L'outil `son` (carte 10.4e) : contrôler un fichier son du jeu avant de le poser
//! dans `assets/musiques` ou `assets/sons`.
//!
//!   cargo run -p outils --bin son -- controler assets/musiques/orage.wav
//!       lit le fichier et affiche : taille, durée, raccord de la boucle (saut au
//!       point de boucle comparé aux sauts du son lui-même).
//!
//! Sortie en erreur si le fichier n'est pas un WAV PCM 16 bits mono à 22 050 Hz
//! sans métadonnées (voir `outils::son::lire_wav`).

use std::process::ExitCode;

use outils::son::{lire_wav, raccord, TAUX};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultat = match args.as_slice() {
        [commande, fichiers @ ..] if commande == "controler" && !fichiers.is_empty() => {
            fichiers.iter().try_for_each(|f| controler(f))
        }
        _ => Err("usage : son controler <fichier.wav>...".into()),
    };
    match resultat {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("son : {e}");
            ExitCode::FAILURE
        }
    }
}

fn controler(chemin: &str) -> Result<(), String> {
    let octets = std::fs::read(chemin).map_err(|e| format!("{chemin} : {e}"))?;
    let ech = lire_wav(&octets).map_err(|e| format!("{chemin} : {e}"))?;
    if ech.is_empty() {
        return Err(format!("{chemin} : aucun échantillon"));
    }
    let n = ech.len();
    let r = raccord(&ech);
    println!(
        "{chemin} : {} octets, {n} échantillons ({:.3} s), raccord {} (sauts internes jusqu'à {}){}",
        octets.len(),
        n as f64 / TAUX as f64,
        r.saut,
        r.saut_max_interieur,
        if r.sans_couture() { "" } else { " COUTURE" }
    );
    Ok(())
}
