//! Les niveaux tels qu'ils sont ENCORE écrits en Rust (`core/src/levels/*.rs`) :
//! la source de l'exporteur (`src/bin/exporter.rs`) et la référence du test
//! d'équivalence. Ce module disparaît avec les fichiers `.rs` (carte 10.1b), quand
//! le jeu lira ses niveaux dans les fichiers RON.

use pepin_core::levels::{self, NIVEAUX};
use pepin_core::zones::InfosNiveau;

/// Les niveaux d'essai (hors de la table `NIVEAUX`), dans l'ordre des clés.
pub const ESSAIS: &[InfosNiveau] = &[
    levels::essai_auvent::INFOS,
    levels::essai_chat::INFOS,
    levels::essai_eau::INFOS,
    levels::essai_eau2::INFOS,
    levels::essai_ennemis_eau::INFOS,
    levels::essai_escargot::INFOS,
    levels::essai_grele::INFOS,
    levels::essai_horloge::INFOS,
    levels::essai_montgolfiere::INFOS,
    levels::essai_nuage::INFOS,
    levels::essai_pieuvre::INFOS,
    levels::essai_vent::INFOS,
    levels::essai_ventilateur::INFOS,
    levels::essai_vertical::INFOS,
];

/// Chaque niveau Rust avec son chemin de fichier RON sous `assets/levels` :
/// `monde_<N>/<cle>.ron` pour le parcours, `essais/<cle>.ron` pour les essais.
pub fn sources() -> Vec<(String, &'static InfosNiveau)> {
    let parcours = NIVEAUX
        .iter()
        .map(|n| (format!("monde_{}/{}.ron", n.monde, n.cle), n));
    let essais = ESSAIS.iter().map(|n| (format!("essais/{}.ron", n.cle), n));
    parcours.chain(essais).collect()
}

/// Marques qui encadrent la légende dans `assets/levels/LISEZMOI.md`.
pub const DEBUT_LEGENDE: &str = "<!-- legende : recopiee de core/src/level.rs par l'exporteur -->";
pub const FIN_LEGENDE: &str = "<!-- fin de la legende -->";

/// La légende de la grille, extraite du commentaire de `core/src/level.rs` (son
/// texte complet en paramètre) : le bloc ```` ```text ```` du haut du fichier.
/// Une seule source de vérité : la légende n'est jamais recopiée à la main.
pub fn legende(source_level_rs: &str) -> String {
    let mut dedans = false;
    let mut lignes = Vec::new();
    for l in source_level_rs.lines() {
        let Some(texte) = l.strip_prefix("//!") else {
            continue;
        };
        let texte = texte.strip_prefix(' ').unwrap_or(texte);
        if texte.starts_with("```") {
            if dedans {
                break;
            }
            dedans = true;
            continue;
        }
        if dedans {
            lignes.push(texte);
        }
    }
    format!("```text\n{}\n```", lignes.join("\n"))
}

/// `lisezmoi` dont la partie entre `DEBUT_LEGENDE` et `FIN_LEGENDE` est remplacée
/// par `legende` (`None` si les marques manquent).
pub fn avec_legende(lisezmoi: &str, legende: &str) -> Option<String> {
    let debut = lisezmoi.find(DEBUT_LEGENDE)? + DEBUT_LEGENDE.len();
    let fin = lisezmoi.find(FIN_LEGENDE)?;
    (debut <= fin).then(|| format!("{}\n{legende}\n{}", &lisezmoi[..debut], &lisezmoi[fin..]))
}
