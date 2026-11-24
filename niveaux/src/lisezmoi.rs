//! La légende de la grille dans `assets/levels/LISEZMOI.md` : recopiée du
//! commentaire de `core/src/level.rs`, jamais à la main (une seule source de
//! vérité). Le programme `src/bin/lisezmoi.rs` la recopie ; le test
//! `la_legende_du_lisezmoi_est_a_jour` vérifie qu'elle n'a pas pris de retard.
//!
//! (Ces fonctions vivaient dans `source_rust.rs`, avec l'exporteur des anciens
//! niveaux Rust ; les deux ont disparu à la carte 10.1c.)

/// Marques qui encadrent la légende dans `assets/levels/LISEZMOI.md`.
pub const DEBUT_LEGENDE: &str =
    "<!-- legende : recopiee de core/src/level.rs (cargo run -p niveaux --bin lisezmoi) -->";
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
