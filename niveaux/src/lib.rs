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
pub mod intro;
pub mod legende;
pub mod lisezmoi;
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
            message: avec_conseil(e.code.to_string()),
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
    // L'intro de la carte du niveau (carte 11.3) : présente, et à la taille de l'écran.
    intro::verifier(&infos).map_err(|e| erreur(GenreErreur::Intro(e)))?;
    Ok(infos)
}

/// Le message de `ron`, plus un conseil quand l'erreur vient d'un fichier écrit
/// avant la carte 11.3 : son champ `intro` est devenu `intro_texte` et
/// `intro_astuce` (ron ne dit que « champ inconnu » ou « champ manquant »).
fn avec_conseil(message: String) -> String {
    if message.contains("`intro`")
        || message.contains("`intro_texte`")
        || message.contains("`intro_astuce`")
    {
        format!("{message} (l'intro du niveau s'écrit en deux champs, après duree_cible_s : intro_texte, la phrase, puis intro_astuce, le conseil)")
    } else {
        message
    }
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

/// Les fichiers embarqués que le jeu charge : le parcours, plus les niveaux
/// d'essai dans les tests et le build de QA (fonctionnalité `essais`).
pub fn fichiers_embarques() -> Vec<(&'static str, &'static str)> {
    let essais: &[Fichier] = if cfg!(any(test, feature = "essais")) {
        FICHIERS_ESSAIS
    } else {
        &[]
    };
    FICHIERS
        .iter()
        .chain(essais)
        .map(|f| (f.chemin, f.texte))
        .collect()
}

/// La table du jeu, lue dans les fichiers embarqués : chargée et vérifiée UNE
/// fois (au premier appel), puis gardée pour toute la partie. `Err` : le premier
/// fichier invalide, avec son erreur lisible (fichier, ligne, colonne).
///
/// Le build de QA s'en sert pour AFFICHER l'erreur au lieu de planter. Dans le
/// jeu publié, l'erreur ne peut pas arriver : le test
/// `tous_les_fichiers_embarques_sont_valides` charge chaque fichier, et
/// `./build.sh` refuse de construire le jeu si ce test échoue.
pub fn table_verifiee() -> Result<&'static Table, &'static ErreurChargement> {
    static TABLE: OnceLock<Result<Table, ErreurChargement>> = OnceLock::new();
    TABLE
        .get_or_init(|| charger_table(&fichiers_embarques()))
        .as_ref()
}

/// La table du jeu (voir `table_verifiee`). Panique avec le message lisible si un
/// fichier est invalide.
pub fn table() -> &'static Table {
    table_verifiee().unwrap_or_else(|e| panic!("niveau invalide : {e}"))
}

/// Les niveaux du parcours, dans l'ordre (rangés par `monde` puis `numero`).
pub fn niveaux() -> &'static [InfosNiveau] {
    &table().niveaux
}

/// Les niveaux d'essai (hors parcours), rangés par clé.
pub fn essais() -> &'static [InfosNiveau] {
    &table().essais
}

/// Les niveaux du Monde 1 : le début du parcours (`NB_NIVEAUX_MONDE_1` niveaux).
pub fn niveaux_monde_1() -> &'static [InfosNiveau] {
    &niveaux()[..NB_NIVEAUX_MONDE_1]
}

/// Le niveau de clé `cle`, cherché dans le parcours puis dans les essais.
/// Panique si la clé est inconnue : à n'appeler qu'avec une clé écrite dans le
/// code (un test, un niveau d'essai nommé), jamais avec une clé lue ailleurs
/// (pour celles-là : `pepin_core::levels::indice_de_cle`).
pub fn par_cle(cle: &str) -> &'static InfosNiveau {
    niveaux()
        .iter()
        .chain(essais())
        .find(|n| n.cle == cle)
        .unwrap_or_else(|| panic!("aucun fichier de niveau n'a la clé « {cle} »"))
}
