//! Écran de pause : le niveau figé, assombri, avec un petit menu.

use pepin_core::palette::{ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, OR_PALE};

use super::{dessiner_menu, panneau, trame, Disposition};
use crate::contenu;
use crate::menu::Menu;
use crate::texte::{self, Style};

pub fn dessiner(anim: u32, libelles: &[&str], menu: &Menu, d: &Disposition) {
    trame();
    // Trois lignes depuis la carte 7.8b (le son est une icône en bas à gauche) : le
    // panneau finit 14 px sous la dernière (y = 128), en y = 142.
    panneau(40.0, 18.0, 240.0, 124.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    texte::dessiner_centre(
        contenu::TITRE_PAUSE,
        30.0,
        Style::simple(OR_PALE, 3).avec_ombre(ARDOISE_OMBRE),
    );
    dessiner_menu(libelles, menu, d, anim);
}
