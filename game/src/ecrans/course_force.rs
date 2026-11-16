//! Choix de la force du fantôme (carte 8.7) : par-dessus la carte des niveaux tramée,
//! le titre « COURSE CONTRE LE ROBOT », le nom du niveau, les trois forces (DÉBUTANT,
//! HABITUÉ, EXPERT) et CARTE, puis, pour la force choisie, le meilleur résultat déjà
//! obtenu contre elle et (EXPERT) la prime de la première victoire si elle reste à
//! gagner.

use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, ARDOISE_PALE, BLANC_CASSE, BRUME, OR, OR_PALE,
};
use pepin_core::zones::InfosNiveau;
use pepin_core::LARGEUR_LOGIQUE;

use super::carte::titre_niveau;
use super::{dessiner_menu, panneau, trame, Disposition};
use crate::contenu;
use crate::course::{self, Force};
use crate::defis;
use crate::menu::{Choix, Menu};
use crate::sauvegarde::Sauvegarde;
use crate::texte::{self, Style};

/// Dessine l'écran (la carte des niveaux est déjà dessinée dessous). `choix` : les
/// lignes du menu (pour savoir quelle force est choisie).
pub fn dessiner(
    infos: &InfosNiveau,
    s: &Sauvegarde,
    choix: &[Choix],
    libelles: &[&str],
    menu: &Menu,
    d: &Disposition,
    anim: u32,
) {
    trame();
    let l = LARGEUR_LOGIQUE as f32;
    panneau(12.0, 6.0, l - 24.0, 168.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let ombre = |c, e| Style::simple(c, e).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(contenu::TITRE_COURSE, 13.0, ombre(OR_PALE, 2));
    texte::dessiner_centre(&titre_niveau(infos.nom), 34.0, ombre(BLANC_CASSE, 1));
    dessiner_menu(libelles, menu, d, anim);
    // Le meilleur résultat contre la force choisie (rien sur la ligne CARTE).
    let Some(Choix::Force(force)) = choix.get(menu.selection()).copied() else {
        return;
    };
    let y = 134.0;
    match s.meilleure_course(infos.cle, force.rang()) {
        Some(e) => {
            let ligne = format!(
                "{} {}",
                contenu::MEILLEUR_RESULTAT,
                course::texte_resultat(e)
            );
            let c = if e < 0 { OR } else { BRUME };
            texte::dessiner_centre(&ligne, y, ombre(c, 1));
        }
        None => texte::dessiner_centre(contenu::PAS_ENCORE_COURU, y, ombre(ARDOISE_PALE, 1)),
    }
    if force == Force::Expert
        && !s
            .bonus_credites
            .contains(&defis::bonus_course_expert(infos.cle))
    {
        let prime = contenu::prime_a_gagner(defis::GAIN_COURSE_EXPERT);
        texte::dessiner_centre(&prime, y + 14.0, ombre(OR_PALE, 1));
    }
}
