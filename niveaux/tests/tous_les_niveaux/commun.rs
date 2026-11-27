//! Ce que tous les modules de tests partagent : les types de `core` dont ils ont
//! besoin, la table du jeu (`niveaux()`) et les niveaux d'essai.
//!
//! Notion Rust : un dossier `tests/<nom>/main.rs` est UN seul binaire de test
//! (une seule édition de liens pour tous les modules, au lieu d'un binaire par
//! fichier de `tests/`). Ses modules se voient entre eux par `crate::...`.
//!
//! Les niveaux d'essai : la bibliothèque `niveaux` ne les met dans SA table
//! qu'avec la fonctionnalité `essais` (ou dans ses propres tests unitaires). Un
//! test d'intégration est compilé comme un crate à part, sans `cfg(test)` dans la
//! bibliothèque : on les charge donc ici, depuis les mêmes fichiers embarqués
//! (`FICHIERS_ESSAIS`), avec le même chargeur que le jeu.

use std::sync::OnceLock;

pub use niveaux::{niveaux, niveaux_monde_1, NB_NIVEAUX, NB_NIVEAUX_MONDE_1};
pub use pepin_core::faisabilite::{analyser, Options};
pub use pepin_core::level::Niveau;
pub use pepin_core::levels::GenreNiveau;
pub use pepin_core::zones::{InfosNiveau, Theme};

/// Les niveaux d'essai (hors parcours), rangés par clé.
pub fn essais() -> &'static [InfosNiveau] {
    static ESSAIS: OnceLock<Vec<InfosNiveau>> = OnceLock::new();
    ESSAIS.get_or_init(|| {
        let mut v: Vec<InfosNiveau> = niveaux::FICHIERS_ESSAIS
            .iter()
            .map(|f| {
                niveaux::charger_fichier(f.chemin, f.texte)
                    .unwrap_or_else(|e| panic!("niveau d'essai invalide : {e}"))
            })
            .collect();
        v.sort_by_key(|n| n.cle);
        v
    })
}

/// Le niveau de clé `cle` : dans le parcours, puis dans les essais.
pub fn par_cle(cle: &str) -> &'static InfosNiveau {
    niveaux()
        .iter()
        .chain(essais())
        .find(|n| n.cle == cle)
        .unwrap_or_else(|| panic!("aucun fichier de niveau n'a la clé « {cle} »"))
}

/// Les niveaux du jeu, puis les niveaux d'essai (hors de la table) : les
/// contrôles de grille s'appliquent aussi à eux. (Pas l'essai « horloge » : sa
/// liste vient telle quelle de l'ancien test de `core/src/zones.rs`.)
pub fn niveaux_et_essai() -> impl Iterator<Item = &'static InfosNiveau> {
    niveaux().iter().chain([
        par_cle("essai_vertical"),
        par_cle("essai_vent"),
        par_cle("essai_grele"),
        par_cle("essai_auvent"),
        par_cle("essai_ventilateur"),
        par_cle("essai_montgolfiere"),
        par_cle("essai_escargot"),
        par_cle("essai_chat"),
        par_cle("essai_nuage"),
        par_cle("essai_eau"),
        par_cle("essai_eau2"),
        par_cle("essai_ennemis_eau"),
        par_cle("essai_pieuvre"),
        par_cle("essai_balancement"),
    ])
}
