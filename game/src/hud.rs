//! Interface en jeu : gouttes d'or ramassées (en haut à gauche) et chrono (en haut à droite).

use pepin_core::palette::{ARDOISE_OMBRE, BLANC_CASSE};
use pepin_core::zones::GOUTTES_TOTAL;
use pepin_core::LARGEUR_LOGIQUE;

use crate::chrono;
use crate::sprites::Banque;
use crate::texte::{self, Style};

/// Dessine le HUD. `gouttes` : gouttes d'or de la partie ; `pas` : durée de la partie en pas.
pub fn dessiner(banque: &Banque, gouttes: u32, pas: u32) {
    let style = Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE);
    banque.dessiner("goutte0", 6.0, 5.0, false);
    texte::dessiner(&format!("{gouttes}/{GOUTTES_TOTAL}"), 17.0, 6.0, style);
    texte::dessiner_a_droite(
        &chrono::temps_court(pas),
        LARGEUR_LOGIQUE as f32 - 6.0,
        6.0,
        style,
    );
}
