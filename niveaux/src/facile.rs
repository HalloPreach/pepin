//! La ligne de la bannière du mode facile (carte 9.34), `facile_texte` : ce que le
//! mode facile change POUR LE BOSS de ce niveau (« MODE FACILE : LE NUAGE N'A QUE 2
//! VIES »). C'est une donnée du niveau, comme son intro (carte 11.3) : un nouveau
//! boss apporte sa phrase dans son fichier, le code du jeu ne tient aucune liste.
//!
//! Les règles, vérifiées au chargement (et par l'éditeur, `editeur/ron.js`,
//! `erreursFacile`) :
//! - un niveau dont la grille a un boss (Grand Nuage `O`, Pieuvre `8`) DOIT avoir
//!   son `facile_texte`, sinon la bannière ne saurait que dire ;
//! - un niveau sans boss ne l'a pas (il n'y a pas de mode facile à proposer) ;
//! - la bannière coupe la phrase entre les mots en lignes de `LIGNE_MAX_INTRO` (46)
//!   caractères, comme la carte du niveau, et n'a la place que pour
//!   `LIGNES_MAX_FACILE` lignes (`game/src/ecrans/facile.rs`).

use pepin_core::zones::InfosNiveau;

use crate::intro::nb_lignes;

/// Lignes au plus de `facile_texte` dans la bannière.
pub const LIGNES_MAX_FACILE: usize = 2;

/// Ce qui ne va pas dans le `facile_texte` d'un niveau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurFacile {
    /// Le niveau a un boss, mais pas de `facile_texte`.
    Manquant,
    /// Le niveau n'a pas de boss, mais un `facile_texte`.
    SansBoss,
    /// Le texte prend `lignes` lignes dans la bannière, `LIGNES_MAX_FACILE` au plus.
    TropLong { lignes: usize },
}

/// Vérifie le `facile_texte` du niveau, qui a un boss ou non (`a_un_boss` : lu dans
/// sa grille par l'appelant).
pub fn verifier(infos: &InfosNiveau, a_un_boss: bool) -> Result<(), ErreurFacile> {
    let lignes = nb_lignes(infos.facile_texte);
    match (a_un_boss, lignes) {
        (true, 0) => Err(ErreurFacile::Manquant),
        (false, 0) => Ok(()),
        (false, _) => Err(ErreurFacile::SansBoss),
        (true, n) if n > LIGNES_MAX_FACILE => Err(ErreurFacile::TropLong { lignes: n }),
        (true, _) => Ok(()),
    }
}
