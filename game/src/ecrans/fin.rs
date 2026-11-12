//! Écran de fin : le soleil, le bilan de la partie et le choix de rejouer.

use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, CORAIL_CLAIR, OR, OR_PALE,
    PLUIE,
};
use pepin_core::zones::{Theme, GOUTTES_TOTAL};

use super::{dessiner_menu, panneau, Disposition};
use crate::chrono;
use crate::contenu;
use crate::decor;
use crate::menu::Menu;
use crate::particules::Particules;
use crate::texte::{self, couper_lignes, Style};

/// Largeur maximale (en caractères) du petit mot de fin : assez large pour que
/// « Il reste plein de gouttes d'or à dénicher. » tienne sur une seule ligne.
const LIGNE_MAX_MOT: usize = 42;

/// Les chiffres de la partie terminée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bilan {
    /// Durée en pas de simulation (1/60 s).
    pub pas: u32,
    pub gouttes: u32,
    pub morts: u32,
    pub score: u32,
}

/// Ajoute de temps en temps une gerbe d'étincelles dans les marges (sans hasard système).
pub fn confettis(particules: &mut Particules, anim: u32) {
    if !anim.is_multiple_of(9) {
        return;
    }
    let h = anim.wrapping_mul(2_654_435_761) >> 8;
    let x = if h & 1 == 0 { 4 + h % 30 } else { 286 + h % 30 };
    let y = 20 + (h >> 4) % 130;
    let couleur = [OR, CORAIL_CLAIR, PLUIE, OR_PALE][(h >> 12) as usize % 4];
    particules.etincelles(x as f32, y as f32, 5, couleur);
}

pub fn dessiner(
    bilan: &Bilan,
    anim: u32,
    particules: &Particules,
    libelles: &[&str],
    menu: &Menu,
    d: &Disposition,
) {
    decor::dessiner_fond(Theme::Soleil, (anim / 2) as i32, 0, anim);
    panneau(20.0, 4.0, 280.0, 166.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    texte::dessiner_centre(
        contenu::TITRE_FIN,
        10.0,
        Style::simple(OR_PALE, 3).avec_ombre(ARDOISE_OMBRE),
    );
    let total = GOUTTES_TOTAL as u32;
    let lignes = [
        (contenu::LIBELLE_TEMPS, chrono::temps_precis(bilan.pas)),
        (
            contenu::LIBELLE_GOUTTES,
            format!("{}/{}", bilan.gouttes, total),
        ),
        (contenu::LIBELLE_MORTS, bilan.morts.to_string()),
        (contenu::LIBELLE_SCORE, bilan.score.to_string()),
    ];
    for (i, (nom, valeur)) in lignes.iter().enumerate() {
        let y = 42.0 + i as f32 * 13.0;
        texte::dessiner(
            nom,
            56.0,
            y,
            Style::simple(BRUME, 1).avec_ombre(ARDOISE_OMBRE),
        );
        texte::dessiner_a_droite(
            valeur,
            264.0,
            y,
            Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE),
        );
    }
    let mot = contenu::message_fin(bilan.gouttes, total);
    for (i, ligne) in couper_lignes(mot, LIGNE_MAX_MOT).iter().enumerate() {
        texte::dessiner_centre(
            ligne,
            98.0 + i as f32 * 10.0,
            Style::simple(CORAIL_CLAIR, 1).avec_ombre(ARDOISE_OMBRE),
        );
    }
    dessiner_menu(libelles, menu, d, anim);
    particules.dessiner(0, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_mot_de_fin_tient_sur_une_ligne_sans_coupure_maladroite() {
        for gouttes in [0, 5, 8, 15] {
            let lignes = couper_lignes(contenu::message_fin(gouttes, 15), LIGNE_MAX_MOT);
            for l in &lignes {
                assert!(!l.starts_with([':', '!', '?', ';', '.']), "{l}");
            }
            assert!(lignes.len() <= 2, "{gouttes} gouttes : {lignes:?}");
        }
        let plein = couper_lignes(contenu::message_fin(0, 15), LIGNE_MAX_MOT);
        assert_eq!(plein, ["Il reste plein de gouttes d'or à dénicher."]);
    }
}
