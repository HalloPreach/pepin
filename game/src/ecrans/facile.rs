//! Bannière du mode facile (carte 8.14) : « TROP DUR ? MODE FACILE : LE NUAGE N'A
//! QUE 2 VIES », avec OUI / NON, par-dessus le niveau figé (après une mort au boss)
//! ou la carte des niveaux. La logique est dans `crate::facile`.

use pepin_core::palette::{ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, OR, OR_PALE};

use super::{panneau, trame};
use crate::contenu;
use crate::facile::Proposition;
use crate::texte::{self, Style};

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

/// Dessine la bannière (sur un fond assombri).
pub fn dessiner(p: &Proposition) {
    trame();
    let (_, y, _, _) = BANNIERE;
    panneau(BANNIERE.0, y, BANNIERE.2, BANNIERE.3, ARDOISE_NUIT, OR);
    let ombre = |c, e| Style::simple(c, e).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(contenu::TROP_DUR, y + 8.0, ombre(OR_PALE, 2));
    texte::dessiner_centre(
        contenu::MODE_FACILE_PROPOSE,
        y + 28.0,
        ombre(BLANC_CASSE, 1),
    );
    texte::dessiner_centre(contenu::FACILE_SANS_MEDAILLE, y + 40.0, ombre(BRUME, 1));
    bouton(BOUTON_OUI, contenu::OUI, p.oui);
    bouton(BOUTON_NON, contenu::NON, !p.oui);
}
