//! Écran « PLUS DE VIES » (carte 6.2) : Pépin a perdu sa dernière vie. Le niveau
//! figé reste derrière, assombri ; le panneau dit ce qui va se passer (le niveau
//! recommence, avec `VIES_DEPART` vies). Il se passe d'un appui (Entrée, toucher),
//! accepté après le court délai de l'histoire ; jamais tout seul (carte 9.31).

use pepin_core::font::HAUTEUR_GLYPHE;
use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, CORAIL, CORAIL_CLAIR,
};
use pepin_core::vies::VIES_DEPART;
use pepin_core::LARGEUR_LOGIQUE;

use super::{panneau, trame};
use crate::contenu;
use crate::sprites::Banque;
use crate::texte::{self, Style};

/// Valeur portée par `Ecran::PlusDeVies` à l'ouverture. Depuis la carte 9.31, l'écran
/// ne se passe plus jamais seul : ce nombre ne déclenche plus rien.
pub const DUREE_PAS: u16 = 240;

/// Cadre du panneau `(x, y, largeur, hauteur)`.
const CADRE: (f32, f32, f32, f32) = (40.0, 34.0, 240.0, 112.0);
/// Ordonnées : titre (échelle 3), Pépin, les deux phrases, « continuer ».
const Y_TITRE: f32 = 44.0;
const Y_PEPIN: f32 = 72.0;
const Y_PHRASES: f32 = 96.0;
const Y_CONTINUER: f32 = 132.0;

/// Hauteur d'une ligne de texte à l'échelle 1, ombre comprise.
const LIGNE: f32 = HAUTEUR_GLYPHE as f32 + 1.0;

// La mise en page verticale tient, vérifiée À LA COMPILATION : titre (échelle 3),
// Pépin (16 px de haut), les deux phrases (12 px d'écart), « continuer », dans le
// cadre et sans se toucher. Les largeurs sont vérifiées par les tests.
const _: () = {
    let (_, y, _, h) = CADRE;
    assert!(Y_TITRE >= y + 4.0);
    assert!(Y_PEPIN >= Y_TITRE + 3.0 * LIGNE + 2.0);
    assert!(Y_PHRASES >= Y_PEPIN + 16.0 + 4.0);
    assert!(Y_CONTINUER >= Y_PHRASES + 12.0 + LIGNE + 4.0);
    assert!(Y_CONTINUER + LIGNE <= y + h - 4.0);
};

/// Les deux phrases sous le titre : ce qui va se passer.
pub fn phrases() -> [String; 2] {
    [
        contenu::recommencer_avec_vies(VIES_DEPART),
        contenu::GOUTTES_A_REPRENDRE.to_string(),
    ]
}

/// `anim` : pas écoulés depuis l'ouverture (clignotement de « continuer »).
pub fn dessiner(banque: &Banque, anim: u32) {
    trame();
    let (x, y, l, h) = CADRE;
    panneau(x, y, l, h, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(
        contenu::TITRE_PLUS_DE_VIES,
        Y_TITRE,
        Style::simple(CORAIL, 3).avec_ombre(ARDOISE_OMBRE),
    );
    // Pépin, assis sous la pluie : sa dernière image de réapparition (le petit tas
    // d'étincelles) serait triste à l'écran ; il regarde le ciel, prêt à repartir.
    banque.dessiner(
        "pepin_ciel",
        (LARGEUR_LOGIQUE as f32 - 16.0) / 2.0,
        Y_PEPIN,
        false,
    );
    for (i, phrase) in phrases().iter().enumerate() {
        let couleur = if i == 0 { BLANC_CASSE } else { CORAIL_CLAIR };
        texte::dessiner_centre(phrase, Y_PHRASES + i as f32 * 12.0, ombre(couleur));
    }
    // Rien tant qu'un appui serait ignoré, puis un clignotement doux (carte 9.31).
    if super::histoire::invite_visible(anim) {
        texte::dessiner_centre(contenu::CONTINUER, Y_CONTINUER, ombre(BRUME));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::font::largeur_texte;

    /// Les textes tiennent en largeur dans le panneau (8 px de marge de chaque
    /// côté), et le panneau dans l'écran. La hauteur : `const _` plus haut.
    #[test]
    fn l_ecran_plus_de_vies_tient_dans_son_cadre() {
        let (x, _, l, _) = CADRE;
        let utile = l as u32 - 16;
        assert!(largeur_texte(contenu::TITRE_PLUS_DE_VIES, 3) <= utile);
        for p in phrases() {
            assert!(largeur_texte(&p, 1) <= utile, "{p}");
        }
        assert!(largeur_texte(contenu::CONTINUER, 1) <= utile);
        assert_eq!(x * 2.0 + l, LARGEUR_LOGIQUE as f32, "centré");
    }

    /// La phrase suit la constante : régler `VIES_DEPART` change le texte.
    #[test]
    fn la_phrase_donne_le_nombre_de_vies_du_depart() {
        let [p, _] = phrases();
        assert!(p.contains(&VIES_DEPART.to_string()), "{p}");
    }
}
