//! Écran « comment jouer » : les commandes et quelques conseils.

use pepin_core::font::HAUTEUR_GLYPHE;
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

/// Mise en page verticale. Sept astuces depuis la carte 6.2 (six à la carte 5.18,
/// quatre avant) : commandes tous les 11 px à partir de y = 34 (au lieu de 12 px à
/// partir de 36), astuces tous les 10 px à partir de y = 92 (au lieu de 98) ; la
/// dernière finit en y = 160, avant la ligne « retour » (y = 163 au lieu de 162).
/// Vérifiée à la compilation (`const _` plus bas).
const Y_COMMANDES: f32 = 34.0;
const PAS_COMMANDES: f32 = 11.0;
const Y_ASTUCES: f32 = 92.0;
const PAS_ASTUCES: f32 = 10.0;
const Y_RETOUR: f32 = 163.0;
/// Panneau : de y = 4 à y = 176.
const Y_PANNEAU: f32 = 4.0;
const H_PANNEAU: f32 = 172.0;

/// Hauteur d'une ligne de texte à l'échelle 1, ombre comprise.
const LIGNE: f32 = HAUTEUR_GLYPHE as f32 + 1.0;

// La mise en page tient, vérifiée À LA COMPILATION (un réglage qui la casse ne
// compile plus) : titre (échelle 2, de 12 à 28), commandes, astuces et « retour » se
// suivent sans se toucher (au moins 2 px entre deux lignes, ombre comprise) et
// restent dans le panneau.
const _: () = {
    let fin_titre = 12.0 + 2.0 * LIGNE;
    assert!(Y_COMMANDES >= fin_titre + 4.0);
    assert!(PAS_COMMANDES >= LIGNE + 2.0);
    assert!(PAS_ASTUCES >= LIGNE + 2.0);
    let n_cmd = contenu::COMMANDES_AIDE.len() as f32;
    let fin_commandes = Y_COMMANDES + (n_cmd - 1.0) * PAS_COMMANDES + LIGNE;
    assert!(Y_ASTUCES >= fin_commandes + 4.0);
    let n_ast = contenu::ASTUCES_AIDE.len() as f32;
    let fin_astuces = Y_ASTUCES + (n_ast - 1.0) * PAS_ASTUCES + LIGNE;
    assert!(Y_RETOUR >= fin_astuces + 3.0);
    assert!(Y_RETOUR + LIGNE <= Y_PANNEAU + H_PANNEAU - 3.0);
};

pub fn dessiner() {
    panneau(
        4.0,
        Y_PANNEAU,
        LARGEUR_LOGIQUE as f32 - 8.0,
        H_PANNEAU,
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
        let y = Y_COMMANDES + i as f32 * PAS_COMMANDES;
        texte::dessiner(touches, X_TOUCHES, y, ombre(OR_PALE));
        texte::dessiner(effet, X_EFFETS, y, ombre(BLANC_CASSE));
    }
    for (i, astuce) in contenu::ASTUCES_AIDE.iter().enumerate() {
        texte::dessiner_centre(astuce, Y_ASTUCES + i as f32 * PAS_ASTUCES, ombre(BRUME));
    }
    texte::dessiner_centre(contenu::RETOUR_AIDE, Y_RETOUR, ombre(CORAIL_CLAIR));
}
