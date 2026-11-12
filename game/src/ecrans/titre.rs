//! Écran titre : le ciel d'orage défile derrière Pépin qui court sur les toits.

use pepin_core::palette::{ARDOISE_OMBRE, BLANC_CASSE, BRUME, OR_PALE};
use pepin_core::zones::Theme;
use pepin_core::LARGEUR_LOGIQUE;

use super::{dessiner_menu, image, rect, Disposition};
use crate::contenu;
use crate::decor;
use crate::menu::Menu;
use crate::sprites::{pepin, Banque};
use crate::texte::{self, Style};

/// Hauteur du dessus des toits.
const SOL: f32 = 148.0;

/// Dessine l'écran titre. `anim` : pas écoulés depuis son ouverture.
pub fn dessiner(banque: &Banque, anim: u32, libelles: &[&str], menu: &Menu, d: &Disposition) {
    // Le décor défile : on a l'impression que Pépin court vers la droite.
    decor::dessiner_fond(Theme::Toits, (anim * 2) as i32, 0, anim);
    let decalage = ((anim * 2) % 16) as f32;
    for k in 0..=20 {
        let x = k as f32 * 16.0 - decalage;
        banque.dessiner_tuile(Theme::Toits, "toit_haut", x, SOL);
        banque.dessiner_tuile(Theme::Toits, "toit_plein", x, SOL + 16.0);
    }
    banque.dessiner(image(&pepin::COURSE, anim, 5), 152.0, SOL - 16.0, false);
    decor::dessiner_avant(Theme::Toits, anim);

    texte::dessiner_centre(
        contenu::TITRE,
        14.0,
        Style::simple(OR_PALE, 5).avec_ombre(ARDOISE_OMBRE),
    );
    texte::dessiner_centre(
        contenu::SOUS_TITRE,
        56.0,
        Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE),
    );
    dessiner_menu(libelles, menu, d, anim);
    // Bandeau sombre derrière le rappel des touches : lisible malgré les briques.
    rect(0.0, 164.0, LARGEUR_LOGIQUE as f32, 16.0, ARDOISE_OMBRE);
    texte::dessiner_centre(contenu::AIDE_MENU, 168.0, Style::simple(BRUME, 1));
}
