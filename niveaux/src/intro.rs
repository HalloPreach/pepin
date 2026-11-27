//! La carte de présentation d'un niveau (carte 11.3) : ses deux textes,
//! `intro_texte` (la phrase d'ambiance) et `intro_astuce` (le conseil), sont écrits
//! dans le fichier du niveau, plus dans le code du jeu. Le chargeur refuse un
//! niveau dont l'intro est vide, ou trop longue pour l'écran d'intro.
//!
//! Les limites viennent de la mise en page de la carte (`game/src/ecrans/carte.rs`,
//! qui lit `LIGNE_MAX_INTRO` ici) : des lignes de 46 caractères au plus, coupées
//! entre les mots ; la phrase commence en y = 88, chaque ligne prend 10 pixels, 4 de
//! plus entre la phrase et le conseil, et tout doit finir avant « continuer » (y = 134).
//! Donc : phrase sur 3 lignes au plus, conseil sur 2, et 4 lignes en tout (la ligne
//! « sans perte de vie » du bonus comprise). L'éditeur applique les MÊMES règles
//! (`editeur/ron.js`, `lignesIntro`).

use pepin_core::vies::coute_des_vies;
use pepin_core::zones::InfosNiveau;

/// Largeur d'une ligne de la carte, en caractères.
pub const LIGNE_MAX_INTRO: usize = 46;
/// Lignes au plus pour la phrase d'ambiance.
pub const LIGNES_MAX_TEXTE: usize = 3;
/// Lignes au plus pour le conseil.
pub const LIGNES_MAX_ASTUCE: usize = 2;
/// Lignes au plus pour la phrase, le conseil et la ligne du bonus, ensemble.
pub const LIGNES_MAX_INTRO: usize = 4;

/// Ce qui ne va pas dans l'intro d'un niveau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurIntro {
    /// Le champ (`intro_texte` ou `intro_astuce`) est vide.
    Vide { champ: &'static str },
    /// Le champ prend `lignes` lignes à l'écran, `max` au plus.
    TropLongue {
        champ: &'static str,
        lignes: usize,
        max: usize,
    },
    /// Phrase, conseil (et ligne du bonus) prennent `lignes` lignes ensemble.
    TropLongueEnTout { lignes: usize },
}

/// Nombre de lignes de `texte` à l'écran : coupé entre les mots, comme
/// `game::texte::couper_lignes` (un mot plus long qu'une ligne en prend une seule).
pub fn nb_lignes(texte: &str) -> usize {
    let mut lignes = 0;
    let mut longueur = 0;
    for mot in texte.split_whitespace() {
        let n = mot.chars().count();
        if lignes > 0 && longueur + 1 + n <= LIGNE_MAX_INTRO {
            longueur += 1 + n;
        } else {
            lignes += 1;
            longueur = n;
        }
    }
    lignes
}

/// Vérifie l'intro d'un niveau : présente, et à la taille de l'écran d'intro.
pub fn verifier(infos: &InfosNiveau) -> Result<(), ErreurIntro> {
    // La ligne « sans perte de vie » du niveau qui ne coûte aucune vie (le bonus).
    let mut total = usize::from(!coute_des_vies(infos.genre));
    for (champ, texte, max) in [
        ("intro_texte", infos.intro_texte, LIGNES_MAX_TEXTE),
        ("intro_astuce", infos.intro_astuce, LIGNES_MAX_ASTUCE),
    ] {
        let lignes = nb_lignes(texte);
        if lignes == 0 {
            return Err(ErreurIntro::Vide { champ });
        }
        if lignes > max {
            return Err(ErreurIntro::TropLongue { champ, lignes, max });
        }
        total += lignes;
    }
    if total > LIGNES_MAX_INTRO {
        return Err(ErreurIntro::TropLongueEnTout { lignes: total });
    }
    Ok(())
}
