//! Bannière du mode facile (carte 8.14) : « TROP DUR ? », puis ce que le mode facile
//! change pour CE boss (`facile_texte` du fichier du niveau, carte 9.34 : « MODE
//! FACILE : LE NUAGE N'A QUE 2 VIES »), avec OUI / NON, par-dessus le niveau figé
//! (après une mort au boss) ou la carte des niveaux. La logique est dans
//! `crate::facile`.

use pepin_core::palette::{ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, OR, OR_PALE};

use super::{panneau, trame};
use crate::contenu;
use crate::facile::Proposition;
use crate::texte::{self, couper_lignes, Style};

/// Largeur d'une ligne de `facile_texte`, en caractères (celle des intros) ; le
/// chargeur des niveaux vérifie qu'il tient en `LIGNES_MAX_FACILE` lignes.
const LIGNE_MAX: usize = niveaux::intro::LIGNE_MAX_INTRO;

/// La bannière et ses deux boutons (pixels logiques).
const BANNIERE: (f32, f32, f32, f32) = (20.0, 46.0, 280.0, 84.0);
const BOUTON_OUI: (f32, f32, f32, f32) = (70.0, 102.0, 80.0, 20.0);
const BOUTON_NON: (f32, f32, f32, f32) = (170.0, 102.0, 80.0, 20.0);

const _: () = assert!(BOUTON_OUI.1 + BOUTON_OUI.3 <= BANNIERE.1 + BANNIERE.3 - 4.0);

fn dedans((rx, ry, rl, rh): (f32, f32, f32, f32), x: f32, y: f32) -> bool {
    x >= rx && x < rx + rl && y >= ry && y < ry + rh
}

/// Le bouton sous le point : `Some(true)` pour OUI, `Some(false)` pour NON.
pub fn bouton_sous(x: f32, y: f32) -> Option<bool> {
    if dedans(BOUTON_OUI, x, y) {
        Some(true)
    } else if dedans(BOUTON_NON, x, y) {
        Some(false)
    } else {
        None
    }
}

/// Bouton à texte centré (échelle 2), cadre or s'il est choisi.
fn bouton((x, y, w, h): (f32, f32, f32, f32), libelle: &str, choisi: bool) {
    let (fond, bord, c) = if choisi {
        (ARDOISE_OMBRE, OR, OR_PALE)
    } else {
        (ARDOISE_NUIT, ARDOISE, BRUME)
    };
    panneau(x, y, w, h - 2.0, fond, bord);
    texte::dessiner_centre_sur(libelle, x + w / 2.0, y + 2.0, Style::simple(c, 2));
}

/// Ordonnées (depuis le haut de la bannière) des lignes de `facile_texte`, puis de
/// la ligne « pas de médaille » : une ligne, la mise en page de la carte 8.14 (28,
/// puis 40) ; deux lignes, serrées entre le titre (fini en 23) et les boutons (56).
fn ordonnees(lignes: usize) -> (Vec<f32>, f32) {
    if lignes <= 1 {
        (vec![28.0], 40.0)
    } else {
        let ys: Vec<f32> = (0..lignes).map(|k| 25.0 + 10.0 * k as f32).collect();
        let fin = ys[lignes - 1] + 11.0;
        (ys, fin)
    }
}

/// Dessine la bannière (sur un fond assombri) ; `facile_texte` : la phrase du boss
/// (son fichier de niveau).
pub fn dessiner(p: &Proposition, facile_texte: &str) {
    trame();
    let (_, y, _, _) = BANNIERE;
    panneau(BANNIERE.0, y, BANNIERE.2, BANNIERE.3, ARDOISE_NUIT, OR);
    let ombre = |c, e| Style::simple(c, e).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(contenu::TROP_DUR, y + 8.0, ombre(OR_PALE, 2));
    let lignes = couper_lignes(facile_texte, LIGNE_MAX);
    let (ys, y_medaille) = ordonnees(lignes.len());
    for (ligne, dy) in lignes.iter().zip(ys) {
        texte::dessiner_centre(ligne, y + dy, ombre(BLANC_CASSE, 1));
    }
    texte::dessiner_centre(
        contenu::FACILE_SANS_MEDAILLE,
        y + y_medaille,
        ombre(BRUME, 1),
    );
    bouton(BOUTON_OUI, contenu::OUI, p.oui);
    bouton(BOUTON_NON, contenu::NON, !p.oui);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deux lignes (la Pieuvre) tiennent entre le titre et les boutons ; une ligne
    /// (le Grand Nuage) garde la mise en page de la carte 8.14, au pixel près.
    #[test]
    fn les_lignes_tiennent_dans_la_banniere() {
        assert_eq!(ordonnees(1), (vec![28.0], 40.0));
        let fin_titre = 8.0 + 2.0 * pepin_core::font::HAUTEUR_GLYPHE as f32 + 1.0;
        for n in 1..=niveaux::facile::LIGNES_MAX_FACILE {
            let (ys, y_medaille) = ordonnees(n);
            assert!(ys[0] > fin_titre, "{n} lignes");
            let bas = y_medaille + pepin_core::font::HAUTEUR_GLYPHE as f32 + 1.0;
            assert!(BANNIERE.1 + bas < BOUTON_OUI.1, "{n} lignes : {bas}");
        }
    }
}
