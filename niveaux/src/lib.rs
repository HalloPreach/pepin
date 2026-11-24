//! Crate `niveaux` : les niveaux de PÉPIN écrits en fichiers RON
//! (`assets/levels/monde_1/toits.ron`...), lus, vérifiés puis convertis en
//! `InfosNiveau` du crate `core` (carte 10.1a).
//!
//! ```text
//!   assets/levels/**/*.ron  --(build.rs : include_str!)-->  FICHIERS
//!        --(charger : serde + ron, puis la grille par core)-->  InfosNiveau
//!        --(charger_table : clés, dossiers, ordre par core)-->  la table du jeu
//! ```
//!
//! `core` reste SANS dépendance : serde et ron ne servent qu'ici. Les vérifications
//! ne sont pas recopiées : la grille passe par le parseur de `core`
//! (`InfosNiveau::charger`) et la table par `core::levels::verifier_ordre`.
//!
//! Ajouter un niveau = ajouter un fichier : sa place vient de ses champs `monde`
//! et `numero`, aucune liste n'est tenue à la main. Format : `assets/levels/LISEZMOI.md`.
//!
//! Notions Rust : `include!` d'un fichier généré par `build.rs`, `OnceLock` (une
//! valeur calculée une seule fois, au premier appel, puis partagée), tri avec
//! `sort_by_key` sur un tuple, `?` qui convertit les erreurs.

pub mod erreur;
pub mod format;
pub mod source_rust;
#[cfg(test)]
mod tests;

pub use erreur::{ErreurChargement, GenreErreur};
pub use format::{ecrire, FichierNiveau};

use pepin_core::levels::{verifier_ordre, ErreurOrdre};
use pepin_core::zones::InfosNiveau;
use std::sync::OnceLock;

/// Un fichier de niveau embarqué : son chemin sous `assets/levels` et son texte.
#[derive(Debug, Clone, Copy)]
pub struct Fichier {
    pub chemin: &'static str,
    pub texte: &'static str,
}

// `FICHIERS` : la liste écrite par `build.rs` (tous les `.ron` de `assets/levels`).
include!(concat!(env!("OUT_DIR"), "/fichiers.rs"));

/// La table du jeu : les niveaux du parcours (rangés par monde puis numéro) et
/// les niveaux d'essai (hors parcours, rangés par clé).
#[derive(Debug, Clone)]
pub struct Table {
    pub niveaux: Vec<InfosNiveau>,
    pub essais: Vec<InfosNiveau>,
}

/// Lit UN niveau écrit en RON et le vérifie (syntaxe, champs, grille, aiguilles,
/// poissons). Le nom de fichier des erreurs est `(texte)` : voir `charger_fichier`.
pub fn charger(texte: &str) -> Result<InfosNiveau, ErreurChargement> {
    charger_fichier("(texte)", texte)
}

/// Comme `charger`, avec le nom du fichier dans les erreurs.
pub fn charger_fichier(fichier: &str, texte: &str) -> Result<InfosNiveau, ErreurChargement> {
    let erreur = |genre| ErreurChargement {
        fichier: fichier.to_string(),
        genre,
    };
    let lu: FichierNiveau = ron::from_str(texte).map_err(|e| {
        erreur(GenreErreur::Syntaxe {
            ligne: e.position.line,
            colonne: e.position.col,
            message: e.code.to_string(),
        })
    })?;
    let infos = lu.vers_infos();
    // La grille est lue par le parseur du jeu : mêmes règles, mêmes erreurs typées.
    infos.charger().map_err(|e| {
        erreur(GenreErreur::Grille {
            erreur: e,
            premiere_ligne: premiere_ligne_de_grille(texte, infos.ascii),
        })
    })?;
    Ok(infos)
}

/// Ligne du FICHIER (à partir de 1) où commence la grille (sa première ligne
/// non vide), pour qu'une erreur de grille se retrouve dans un éditeur.
fn premiere_ligne_de_grille(texte: &str, ascii: &str) -> usize {
    let Some(premiere) = ascii.lines().map(str::trim_end).find(|l| !l.is_empty()) else {
        return 0;
    };
    let debut = texte
        .lines()
        .position(|l| l.trim_start().starts_with("ascii"))
        .unwrap_or(0);
    texte
        .lines()
        .enumerate()
        .skip(debut)
        .find(|(_, l)| l.trim_end().ends_with(premiere))
        .map_or(0, |(i, _)| i + 1)
}

/// Charge et vérifie une table complète de fichiers : chaque fichier (`charger`),
/// son dossier (`monde_<N>` pour le parcours, `essais` pour les essais), sa clé
/// (le nom du fichier, unique), puis l'ORDRE de la table rangée par
/// (`monde`, `numero`) avec `core::levels::verifier_ordre`.
pub fn charger_table(fichiers: &[(&str, &str)]) -> Result<Table, ErreurChargement> {
    let mut niveaux: Vec<(InfosNiveau, &str)> = Vec::new();
    let mut essais: Vec<(InfosNiveau, &str)> = Vec::new();
    for &(chemin, texte) in fichiers {
        let infos = charger_fichier(chemin, texte)?;
        let erreur = |genre| ErreurChargement {
            fichier: chemin.to_string(),
            genre,
        };
        let (dossier, nom) = chemin.rsplit_once('/').unwrap_or(("", chemin));
        if nom.strip_suffix(".ron") != Some(infos.cle) {
            return Err(erreur(GenreErreur::CleDifferenteDuFichier {
                cle: infos.cle.to_string(),
            }));
        }
        if let Some((_, autre)) = niveaux
            .iter()
            .chain(essais.iter())
            .find(|(n, _)| n.cle == infos.cle)
        {
            return Err(erreur(GenreErreur::CleEnDouble {
                cle: infos.cle.to_string(),
                autre_fichier: autre.to_string(),
            }));
        }
        if dossier == "essais" {
            if infos.numero != 0 {
                return Err(erreur(GenreErreur::EssaiNumerote {
                    numero: infos.numero,
                }));
            }
            essais.push((infos, chemin));
        } else if let Some(monde) = dossier
            .strip_prefix("monde_")
            .and_then(|m| m.parse::<u8>().ok())
        {
            if infos.monde != monde {
                return Err(erreur(GenreErreur::MondeHorsDeSonDossier {
                    dossier: monde,
                    monde: infos.monde,
                }));
            }
            niveaux.push((infos, chemin));
        } else {
            return Err(erreur(GenreErreur::DossierInconnu));
        }
    }
    // La place de chaque niveau vient de ses champs, pas de l'ordre des fichiers.
    niveaux.sort_by_key(|(n, _)| (n.monde, n.numero));
    essais.sort_by_key(|(n, _)| n.cle);
    let rangee: Vec<InfosNiveau> = niveaux.iter().map(|(n, _)| *n).collect();
    verifier_ordre(&rangee).map_err(|e| {
        let indice = match e {
            ErreurOrdre::NumeroFaux { indice, .. }
            | ErreurOrdre::MondeDesordonne { indice, .. }
            | ErreurOrdre::NormalApresLaFin { indice }
            | ErreurOrdre::EpilogueAvantLaFin { indice } => Some(indice),
            ErreurOrdre::AucunNiveauNormal | ErreurOrdre::BonusMultiple => None,
        };
        ErreurChargement {
            fichier: indice.map_or("(table)".to_string(), |i| niveaux[i].1.to_string()),
            genre: GenreErreur::Ordre(e),
        }
    })?;
    Ok(Table {
        niveaux: rangee,
        essais: essais.into_iter().map(|(n, _)| n).collect(),
    })
}

/// La table du jeu, lue dans les fichiers embarqués : chargée et vérifiée UNE
/// fois (au premier appel), puis gardée pour toute la partie.
///
/// Panique avec un message lisible si un fichier est invalide : cela ne peut pas
/// arriver dans un jeu publié, car les tests de ce crate chargent chaque fichier.
pub fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let fichiers: Vec<(&str, &str)> = FICHIERS.iter().map(|f| (f.chemin, f.texte)).collect();
        charger_table(&fichiers).unwrap_or_else(|e| panic!("niveau invalide : {e}"))
    })
}

/// Les niveaux du parcours, dans l'ordre (l'équivalent de `core::levels::NIVEAUX`).
pub fn niveaux() -> &'static [InfosNiveau] {
    &table().niveaux
}

/// Les niveaux d'essai (hors parcours), rangés par clé.
pub fn essais() -> &'static [InfosNiveau] {
    &table().essais
}
