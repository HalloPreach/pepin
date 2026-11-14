//! Écran « comment jouer » : les commandes et quelques conseils.

use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, CORAIL_CLAIR, OR_PALE,
};
use pepin_core::LARGEUR_LOGIQUE;

use super::panneau;
use crate::contenu;
use crate::texte::{self, Style};

/// Colonne des touches / colonne des effets.
const X_TOUCHES: f32 = 16.0;
const X_EFFETS: f32 = 118.0;

pub fn dessiner() {
    panneau(
        4.0,
        4.0,
        LARGEUR_LOGIQUE as f32 - 8.0,
        172.0,
        ARDOISE_NUIT,
        ARDOISE_CLAIRE,
    );
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(
        contenu::TITRE_AIDE,
        12.0,
        Style::simple(OR_PALE, 2).avec_ombre(ARDOISE_OMBRE),
    );
    for (i, (touches, effet)) in contenu::COMMANDES_AIDE.iter().enumerate() {
        let y = 36.0 + i as f32 * 12.0;
        texte::dessiner(touches, X_TOUCHES, y, ombre(OR_PALE));
        texte::dessiner(effet, X_EFFETS, y, ombre(BLANC_CASSE));
    }
    // Six astuces depuis la carte 5.18 (au lieu de quatre) : commandes tous les
    // 12 px (au lieu de 13), astuces tous les 10 px à partir de y = 98 ; la
    // dernière finit en y = 156, avant la ligne « retour » (y = 162).
    for (i, astuce) in contenu::ASTUCES_AIDE.iter().enumerate() {
        texte::dessiner_centre(astuce, 98.0 + i as f32 * 10.0, ombre(BRUME));
    }
    texte::dessiner_centre(contenu::RETOUR_AIDE, 162.0, ombre(CORAIL_CLAIR));
}
