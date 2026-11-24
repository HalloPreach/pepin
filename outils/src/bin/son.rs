//! L'outil `son` (carte 10.4e) : contrôler les fichiers OGG des sons.
//!
//!   cargo run -p outils --bin son -- controler assets/musiques/orage.ogg [reference.wav]
//!       décode le fichier et affiche : canaux, fréquence, commentaires Vorbis (il
//!       n'en faut AUCUN), taille, durée, raccord de la boucle (saut au point de
//!       boucle comparé aux sauts du son lui-même). Avec un WAV de référence : écart
//!       de longueur et rapport signal/bruit.
//!
//! Sortie en erreur si le fichier est stéréo, n'est pas à 22 050 Hz, porte un
//! commentaire, ou (avec référence) n'a pas la même longueur à 2 échantillons près.

use std::process::ExitCode;

use outils::son::{lire_ogg, lire_wav, raccord, signal_bruit_db, TAUX};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultat = match args.as_slice() {
        [commande, ogg] if commande == "controler" => controler(ogg, None),
        [commande, ogg, wav] if commande == "controler" => controler(ogg, Some(wav)),
        _ => Err("usage : son controler <fichier.ogg> [reference.wav]".into()),
    };
    match resultat {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("son : {e}");
            ExitCode::FAILURE
        }
    }
}

fn controler(chemin: &str, reference: Option<&String>) -> Result<(), String> {
    let octets = std::fs::read(chemin).map_err(|e| format!("{chemin} : {e}"))?;
    let ogg = lire_ogg(&octets)?;
    let n = ogg.echantillons.len();
    let r = raccord(&ogg.echantillons);
    print!(
        "{chemin} : {} octets, {} canal(aux), {} Hz, {n} échantillons ({:.3} s), raccord {} (sauts internes jusqu'à {}){}",
        octets.len(),
        ogg.canaux,
        ogg.taux,
        n as f64 / ogg.taux as f64,
        r.saut,
        r.saut_max_interieur,
        if r.sans_couture() { "" } else { " COUTURE" }
    );
    let mut erreurs = Vec::new();
    if ogg.canaux != 1 {
        erreurs.push("pas mono".to_string());
    }
    if ogg.taux != TAUX {
        erreurs.push(format!("fréquence {} au lieu de {TAUX}", ogg.taux));
    }
    if !ogg.commentaires.is_empty() {
        erreurs.push(format!("commentaires {:?}", ogg.commentaires));
    }
    if let Some(wav) = reference {
        let (taux, ech) = lire_wav(&std::fs::read(wav).map_err(|e| format!("{wav} : {e}"))?)?;
        let ecart = n as i64 - ech.len() as i64;
        print!(
            ", référence {} échantillons (écart {ecart}), signal/bruit {:.1} dB",
            ech.len(),
            signal_bruit_db(&ech, &ogg.echantillons)
        );
        if taux != ogg.taux || ecart.abs() > 2 {
            erreurs.push(format!("longueur ou fréquence différente de {wav}"));
        }
    }
    println!();
    if erreurs.is_empty() {
        Ok(())
    } else {
        Err(erreurs.join(" ; "))
    }
}
