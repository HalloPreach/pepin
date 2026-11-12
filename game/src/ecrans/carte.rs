//! Carte de présentation d'un niveau : numéro, nom, phrase d'ambiance et conseil.
//! Elle s'affiche par-dessus le niveau figé, juste après la transition.

use pepin_core::font::largeur_texte;
use pepin_core::palette::{
    ARDOISE_CLAIRE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME, CORAIL_CLAIR, OR_PALE,
};
use pepin_core::LARGEUR_LOGIQUE;

use super::{panneau, trame};
use crate::contenu;
use crate::texte::{self, couper_lignes, Style};

/// Durée d'affichage de la carte (pas) si le joueur ne la passe pas.
pub const DUREE_PAS: u16 = 180;
/// Largeur maximale d'une ligne de texte, en caractères (320 px / 6 px = 53 ; marge comprise).
const LIGNE_MAX: usize = 46;

/// Largeur utile du cadre de la carte, en pixels logiques.
const LARGEUR_CADRE: u32 = LARGEUR_LOGIQUE - 24;

/// Plus grande échelle (3, puis 2) à laquelle le nom du niveau tient dans le cadre.
fn echelle_nom(nom: &str) -> u32 {
    if largeur_texte(nom, 3) < LARGEUR_CADRE - 8 {
        3
    } else {
        2
    }
}

/// `indice` : 0 à 4 ; `nom` : titre du niveau ; `anim` : pas écoulés (clignotement).
pub fn dessiner(indice: usize, numero: u8, nom: &str, anim: u32) {
    trame();
    let l = LARGEUR_LOGIQUE as f32;
    panneau(12.0, 34.0, l - 24.0, 112.0, ARDOISE_NUIT, ARDOISE_CLAIRE);
    let ombre = |c| Style::simple(c, 1).avec_ombre(ARDOISE_OMBRE);

    texte::dessiner_centre(&contenu::etiquette_niveau(numero), 44.0, ombre(BRUME));
    texte::dessiner_centre(
        nom,
        56.0 + if echelle_nom(nom) == 3 { 0.0 } else { 4.0 },
        Style::simple(OR_PALE, echelle_nom(nom)).avec_ombre(ARDOISE_OMBRE),
    );
    let intro = contenu::intro(indice);
    let mut y = 88.0;
    for ligne in couper_lignes(intro.texte, LIGNE_MAX) {
        texte::dessiner_centre(&ligne, y, ombre(BLANC_CASSE));
        y += 10.0;
    }
    y += 4.0;
    for ligne in couper_lignes(intro.astuce, LIGNE_MAX) {
        texte::dessiner_centre(&ligne, y, ombre(CORAIL_CLAIR));
        y += 10.0;
    }
    if (anim / 30).is_multiple_of(2) {
        texte::dessiner_centre(contenu::CONTINUER, 134.0, ombre(BRUME));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::zones::NIVEAUX;

    #[test]
    fn toutes_les_cartes_tiennent_dans_leur_cadre() {
        for (i, n) in NIVEAUX.iter().enumerate() {
            let intro = contenu::intro(i);
            let a = couper_lignes(intro.texte, LIGNE_MAX);
            let b = couper_lignes(intro.astuce, LIGNE_MAX);
            assert!(a.len() <= 3, "{} : intro sur {} lignes", n.nom, a.len());
            assert!(b.len() <= 2, "{} : conseil sur {} lignes", n.nom, b.len());
            // Le texte commence en y = 88 ; la dernière ligne doit finir avant « continuer » (134).
            let fin = 88.0 + a.len() as f32 * 10.0 + 4.0 + b.len() as f32 * 10.0;
            assert!(fin <= 134.0, "{} : texte jusqu'à y = {fin}", n.nom);
            for ligne in a.iter().chain(&b) {
                assert!(largeur_texte(ligne, 1) < LARGEUR_CADRE - 8, "{}", ligne);
            }
            // Le nom, à l'échelle choisie, doit tenir dans le cadre.
            assert!(
                largeur_texte(n.nom, echelle_nom(n.nom)) < LARGEUR_CADRE - 8,
                "{}",
                n.nom
            );
        }
    }
}
