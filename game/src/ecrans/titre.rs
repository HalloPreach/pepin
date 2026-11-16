//! Écran titre : le ciel d'orage défile derrière Pépin qui court sur les toits.

use pepin_core::font::HAUTEUR_GLYPHE;
use pepin_core::palette::{ARDOISE_OMBRE, BLANC_CASSE, BRUME, OR_PALE};
use pepin_core::zones::Theme;
use pepin_core::LARGEUR_LOGIQUE;

use super::{dessiner_menu, image, rect, Disposition};
use crate::contenu;
use crate::decor;
use crate::menu::Menu;
use crate::sauvegarde::Sauvegarde;
use crate::skins::Skin;
use crate::sprites::Banque;
use crate::texte::{self, Style};

/// Hauteur du dessus des toits. 154 depuis la ligne HISTOIRE (carte 8.4, 148 avant) :
/// le menu de cinq lignes finit juste au-dessus (test de `ecrans`).
pub const SOL: f32 = 154.0;
/// Ordonnées du titre (échelle 5) et du sous-titre (échelle 1). Remontés de 2 px
/// (carte 8.4) pour faire place à la cinquième ligne du menu.
const Y_TITRE: f32 = 12.0;
const Y_SOUS_TITRE: f32 = 54.0;
/// Bas du sous-titre, ombre comprise : le menu commence en dessous (test de `ecrans`).
#[cfg(test)]
pub const FIN_SOUS_TITRE: f32 = Y_SOUS_TITRE + HAUTEUR_GLYPHE as f32 + 1.0;
/// Bas du grand titre, ombre comprise : le menu à six lignes commence en dessous.
#[cfg(test)]
pub const FIN_TITRE: f32 = Y_TITRE + (HAUTEUR_GLYPHE * 5 + 5) as f32;

// Le titre (7 rangées à l'échelle 5, ombre décalée de 5 px) finit avant le sous-titre.
const _: () = assert!(Y_TITRE + (HAUTEUR_GLYPHE * 5 + 5) as f32 <= Y_SOUS_TITRE);

/// Dessine l'écran titre. `anim` : pas écoulés depuis son ouverture.
pub fn dessiner(
    banque: &Banque,
    skin: Skin,
    anim: u32,
    libelles: &[&str],
    menu: &Menu,
    d: &Disposition,
) {
    // Le décor défile : on a l'impression que Pépin court vers la droite.
    decor::dessiner_fond(Theme::Toits, (anim * 2) as i32, 0, anim);
    let decalage = ((anim * 2) % 16) as f32;
    for k in 0..=20 {
        let x = k as f32 * 16.0 - decalage;
        banque.dessiner_tuile(Theme::Toits, "toit_haut", x, SOL);
        banque.dessiner_tuile(Theme::Toits, "toit_plein", x, SOL + 16.0);
    }
    // À gauche du menu (jusqu'à 5 lignes : cartes 7.4 et 8.4), sous le skin choisi.
    banque.dessiner(
        image(&skin.images().course, anim, 5),
        14.0,
        SOL - 16.0,
        false,
    );
    decor::dessiner_avant(Theme::Toits, anim, false);

    // Remontés à la carte 7.4 pour la cinquième ligne du menu (CARTE).
    texte::dessiner_centre(
        contenu::TITRE,
        Y_TITRE,
        Style::simple(OR_PALE, 5).avec_ombre(ARDOISE_OMBRE),
    );
    // Six lignes (carte 8.8) : le menu prend la place du sous-titre.
    if libelles.len() <= super::LIGNES_TITRE_MAX {
        texte::dessiner_centre(
            contenu::SOUS_TITRE,
            Y_SOUS_TITRE,
            Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE),
        );
    }
    dessiner_menu(libelles, menu, d, anim);
    // Bandeau sombre derrière le rappel des touches : lisible malgré les briques.
    rect(0.0, 164.0, LARGEUR_LOGIQUE as f32, 16.0, ARDOISE_OMBRE);
    texte::dessiner_centre(contenu::AIDE_MENU, 168.0, Style::simple(BRUME, 1));
}

/// Le porte-monnaie de gouttes (en haut à gauche : la goutte d'or et le nombre, carte 7.5b)
/// et, dès qu'une partie a été finie, le meilleur score (en haut à droite), au-dessus du
/// titre (carte 7.2).
pub fn dessiner_poche(banque: &Banque, s: &Sauvegarde) {
    let style = Style::simple(BLANC_CASSE, 1).avec_ombre(ARDOISE_OMBRE);
    // Sprite 10x10, deux pixels plus haut que le texte, comme dans le HUD.
    banque.dessiner("goutte0", 4.0, 1.0, false);
    texte::dessiner(&s.gouttes_en_poche.to_string(), 16.0, 3.0, style);
    if s.parties_finies > 0 {
        let record = format!("{} : {}", contenu::LIBELLE_RECORD, s.meilleur_score);
        texte::dessiner_a_droite(&record, LARGEUR_LOGIQUE as f32 - 4.0, 3.0, style);
    }
}
