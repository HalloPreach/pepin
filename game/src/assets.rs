//! Le dossier `assets/` dans son ensemble (carte 10.4f) : tests seulement.
//!
//! Chaque dossier a déjà ses tests « aucun fichier manquant ni orphelin » (sprites,
//! fonds, sons, niveaux : voir `assets/LISEZMOI.md`). Mais chaque `build.rs` ne liste
//! que les fichiers de SON extension : un `.ogg` oublié dans `sons/`, un `.jpg` dans
//! `fonds/` ou un dossier inconnu seraient ignorés en silence (et copiés nulle part,
//! ou servis pour rien). Ici, on vérifie que chaque fichier de `assets/` est un fichier
//! que le jeu lit, et que le LISEZMOI général décrit chaque dossier.
//!
//! Notions Rust illustrées ici : parcours récursif d'un dossier (`std::fs::read_dir`)
//! dans un test, chemin du dépôt par `env!("CARGO_MANIFEST_DIR")`.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    /// Chaque dossier de premier niveau d'`assets/`, et l'extension des fichiers que le
    /// jeu y lit (sous-dossiers compris).
    const DOSSIERS: [(&str, &str); 5] = [
        ("sprites", "png"),
        ("fonds", "png"),
        ("levels", "ron"),
        ("musiques", "wav"),
        ("sons", "wav"),
    ];

    fn racine() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets")
    }

    /// Tous les fichiers sous `dossier`, en chemin relatif à `assets/`.
    fn fichiers(dossier: &Path, sortie: &mut Vec<String>) {
        for entree in std::fs::read_dir(dossier).unwrap() {
            let chemin = entree.unwrap().path();
            if chemin.is_dir() {
                fichiers(&chemin, sortie);
            } else {
                let relatif = chemin.strip_prefix(racine()).unwrap();
                sortie.push(relatif.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    #[test]
    fn chaque_fichier_d_assets_est_lu_par_le_jeu() {
        let mut tous = Vec::new();
        fichiers(&racine(), &mut tous);
        assert!(tous.len() > 400, "{} fichiers seulement", tous.len());
        for f in &tous {
            // Les LISEZMOI sont pour les humains : celui d'`assets/` et un par dossier.
            if f == "LISEZMOI.md" || (f.ends_with("/LISEZMOI.md") && f.matches('/').count() == 1) {
                continue;
            }
            let (dossier, reste) = f.split_once('/').unwrap_or(("", f));
            let Some((_, ext)) = DOSSIERS.iter().find(|(d, _)| *d == dossier) else {
                panic!("assets/{f} : dossier inconnu (voir DOSSIERS et assets/LISEZMOI.md)");
            };
            assert!(
                reste.ends_with(&format!(".{ext}")),
                "assets/{f} : le jeu ne lit que des .{ext} dans assets/{dossier}"
            );
        }
    }

    #[test]
    fn chaque_dossier_d_assets_est_decrit() {
        let lisezmoi = include_str!("../../assets/LISEZMOI.md");
        for (d, _) in DOSSIERS {
            assert!(
                lisezmoi.contains(&format!("| `{d}/` |")),
                "assets/LISEZMOI.md ne décrit pas assets/{d}/"
            );
        }
        for entree in std::fs::read_dir(racine()).unwrap() {
            let chemin = entree.unwrap().path();
            if chemin.is_dir() {
                let nom = chemin.file_name().unwrap().to_string_lossy().into_owned();
                assert!(
                    DOSSIERS.iter().any(|(d, _)| *d == nom),
                    "assets/{nom}/ : dossier inconnu"
                );
            }
        }
    }
}
