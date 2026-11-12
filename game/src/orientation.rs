//! Écran « Tourne ton téléphone » : sur un appareil tactile tenu en PORTRAIT, le jeu
//! (320x180, fait pour le paysage) serait réduit à une bande étroite. On le met donc
//! en pause derrière un message en pixel art, et il reprend tout seul au retour en
//! paysage (rien n'est perdu : la simulation et le chrono ne bougent plus entre-temps).
//!
//! Tout est dessiné dans la cible 320x180 existante : sprites en grilles de texte
//! (comme `sprites`), couleurs de la palette seulement, police bitmap 5x7, aucun
//! dégradé ni transparence. En portrait, l'agrandissement entier de `rendu` donne
//! le plus grand facteur qui tient sur la largeur : les pixels restent gros et nets.
//!
//! Notions Rust illustrées ici : fonction de décision pure et testée, constantes
//! `&[&str]` pour les dessins, itérateurs `enumerate` sur des grilles.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::{ARDOISE, ARDOISE_NUIT, ARDOISE_OMBRE, BLANC_CASSE, BRUME};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::rendu::couleur_mq;
use crate::sprites::{self, Image};
use crate::texte::{self, Style};

/// Première ligne du message.
pub const LIGNE_1: &str = "Tourne ton téléphone";
/// Deuxième ligne, juste en dessous.
pub const LIGNE_2: &str = "pour jouer en paysage";

/// Durée d'affichage de chaque image du téléphone (secondes).
const DUREE_IMAGE: f64 = 0.9;
/// Taille d'un pixel de sprite, en pixels logiques (entier : net).
const ECHELLE_SPRITE: u32 = 2;
/// Échelle du texte (pixels logiques par pixel de glyphe).
const ECHELLE_TEXTE: u32 = 2;

/// Faut-il afficher l'écran « Tourne ton téléphone » (et mettre le jeu en pause) ?
/// Oui seulement si la fenêtre est plus haute que large ET que l'appareil est
/// tactile (un toucher a déjà été reçu) : une fenêtre étroite de navigateur de
/// bureau, sans toucher, garde le jeu normal.
pub fn doit_afficher(largeur: f32, hauteur: f32, tactile: bool) -> bool {
    tactile && hauteur > largeur
}

/// Quelle image du téléphone montrer à cet instant (0 = portrait, 1 = paysage) ?
pub fn image_active(temps_s: f64) -> usize {
    if !temps_s.is_finite() || temps_s < 0.0 {
        return 0;
    }
    ((temps_s / DUREE_IMAGE) as u64 % 2) as usize
}

// --- Sprites ---------------------------------------------------------------
// Même code couleur que `sprites` : `.` transparent, `0`-`9` / `A`-`F` = indice
// dans la palette (2 ardoise, 3 ardoise claire, 4 ardoise pâle, 6 nuage, B or,
// D pluie, E pluie sombre).

/// Téléphone tenu droit : 12x20.
#[rustfmt::skip]
pub const TELEPHONE_PORTRAIT: Image = &[
    ".6666666666.",
    "622233332226",
    "622222222226",
    "62DDDDDDDD26",
    "62DDDDDDDD26",
    "62DDDDBBDD26",
    "62DDDDBBDD26",
    "62DDDDDDDD26",
    "62DDDDDDDD26",
    "62D66DDDDD26",
    "62D666DDDD26",
    "62DDDDDDDD26",
    "624444444426",
    "62EEEEEEEE26",
    "62EEEEEEEE26",
    "62EEEEEEEE26",
    "62EEEEEEEE26",
    "622222222226",
    "622223322226",
    ".6666666666.",
];

/// Le même téléphone tourné d'un quart de tour : 20x12.
#[rustfmt::skip]
pub const TELEPHONE_PAYSAGE: Image = &[
    ".666666666666666666.",
    "62222222222222222226",
    "622EEEE4DDDDDDDDD226",
    "622EEEE4D66DDDDDD226",
    "622EEEE4D66DDDDDD236",
    "632EEEE4D6DDDDDDD236",
    "632EEEE4DDDDDBBDD236",
    "622EEEE4DDDDDBBDD236",
    "622EEEE4DDDDDDDDD226",
    "622EEEE4DDDDDDDDD226",
    "62222222222222222226",
    ".666666666666666666.",
];

/// Flèche courbe « tourner dans ce sens » : 12x5, couleur or.
#[rustfmt::skip]
pub const FLECHE_TOUR: Image = &[
    "..BBBBBBB...",
    ".BB.....BB..",
    "BB.......BB.",
    "B.......BBBB",
    "B........BB.",
];

/// Les deux images de l'animation, dans l'ordre : portrait puis paysage.
pub const IMAGES_TELEPHONE: [Image; 2] = [TELEPHONE_PORTRAIT, TELEPHONE_PAYSAGE];

// --- Dessin ----------------------------------------------------------------

/// Dessine une grille de pixels dont le coin haut-gauche est en (`x`, `y`) logiques,
/// chaque pixel de sprite valant `echelle` x `echelle` pixels logiques.
fn dessiner_image(image: Image, x: i32, y: i32, echelle: u32) {
    let e = echelle as f32;
    for (j, ligne) in image.iter().enumerate() {
        for (i, car) in ligne.chars().enumerate() {
            if let Ok(Some(couleur)) = sprites::pixel(car) {
                draw_rectangle(
                    x as f32 + i as f32 * e,
                    y as f32 + j as f32 * e,
                    e,
                    e,
                    couleur_mq(couleur),
                );
            }
        }
    }
}

/// Cadre de 2 pixels logiques tout autour de l'écran, fait de quatre barres pleines.
fn dessiner_cadre() {
    let (l, h) = (LARGEUR_LOGIQUE as f32, HAUTEUR_LOGIQUE as f32);
    let c = couleur_mq(ARDOISE);
    for (x, y, w, hh) in [
        (3.0, 3.0, l - 6.0, 2.0),
        (3.0, h - 5.0, l - 6.0, 2.0),
        (3.0, 3.0, 2.0, h - 6.0),
        (l - 5.0, 3.0, 2.0, h - 6.0),
    ] {
        draw_rectangle(x, y, w, hh, c);
    }
}

/// Dessine tout l'écran dans la cible logique. `temps_s` : horloge en secondes
/// (fait alterner les deux images du téléphone).
pub fn dessiner(temps_s: f64) {
    draw_rectangle(
        0.0,
        0.0,
        LARGEUR_LOGIQUE as f32,
        HAUTEUR_LOGIQUE as f32,
        couleur_mq(ARDOISE_NUIT),
    );
    dessiner_cadre();

    // Le téléphone et la flèche, centrés dans une boîte de 40x40 pixels logiques.
    let image = IMAGES_TELEPHONE[image_active(temps_s)];
    let (l, h) = (
        (sprites::largeur(image) as u32 * ECHELLE_SPRITE) as i32,
        (sprites::hauteur(image) as u32 * ECHELLE_SPRITE) as i32,
    );
    let (centre_x, centre_y) = (LARGEUR_LOGIQUE as i32 / 2, 62);
    dessiner_image(image, centre_x - l / 2, centre_y - h / 2, ECHELLE_SPRITE);
    dessiner_image(FLECHE_TOUR, centre_x + 32, centre_y - 20, ECHELLE_SPRITE);

    // Le message, ombré pour rester net sur le fond.
    let ombre = |c| Style::simple(c, ECHELLE_TEXTE).avec_ombre(ARDOISE_OMBRE);
    texte::dessiner_centre(LIGNE_1, 100.0, ombre(BLANC_CASSE));
    texte::dessiner_centre(LIGNE_2, 122.0, ombre(BRUME));
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::font;

    #[test]
    fn portrait_et_tactile_affichent_l_ecran() {
        // Téléphone en portrait : on affiche, que ce soit en CSS ou en pixels physiques.
        assert!(doit_afficher(390.0, 844.0, true));
        assert!(doit_afficher(780.0, 1688.0, true));
    }

    #[test]
    fn paysage_n_affiche_rien() {
        assert!(!doit_afficher(844.0, 390.0, true));
        assert!(!doit_afficher(1920.0, 1080.0, true));
        assert!(!doit_afficher(1920.0, 1080.0, false));
    }

    #[test]
    fn portrait_sans_toucher_n_affiche_rien() {
        // Fenêtre étroite d'un navigateur de bureau, souris et clavier seulement.
        assert!(!doit_afficher(600.0, 900.0, false));
        assert!(!doit_afficher(390.0, 844.0, false));
    }

    #[test]
    fn un_ecran_carre_reste_en_jeu() {
        assert!(!doit_afficher(800.0, 800.0, true));
    }

    #[test]
    fn les_deux_images_alternent_dans_le_temps() {
        assert_eq!(image_active(0.0), 0);
        assert_eq!(image_active(DUREE_IMAGE * 0.99), 0);
        assert_eq!(image_active(DUREE_IMAGE * 1.01), 1);
        assert_eq!(image_active(DUREE_IMAGE * 2.01), 0);
        assert_eq!(image_active(-1.0), 0);
        assert_eq!(image_active(f64::NAN), 0);
    }

    #[test]
    fn les_sprites_sont_rectangulaires_et_dans_la_palette() {
        sprites::verifier("téléphone portrait", TELEPHONE_PORTRAIT).unwrap();
        sprites::verifier("téléphone paysage", TELEPHONE_PAYSAGE).unwrap();
        sprites::verifier("flèche", FLECHE_TOUR).unwrap();
    }

    #[test]
    fn les_sprites_ont_les_bonnes_dimensions() {
        assert_eq!(
            (
                sprites::largeur(TELEPHONE_PORTRAIT),
                sprites::hauteur(TELEPHONE_PORTRAIT)
            ),
            (12, 20)
        );
        assert_eq!(
            (
                sprites::largeur(TELEPHONE_PAYSAGE),
                sprites::hauteur(TELEPHONE_PAYSAGE)
            ),
            (20, 12)
        );
        assert_eq!(
            (sprites::largeur(FLECHE_TOUR), sprites::hauteur(FLECHE_TOUR)),
            (12, 5)
        );
    }

    #[test]
    fn le_paysage_est_le_portrait_tourne_d_un_quart_de_tour() {
        let portrait: Vec<Vec<char>> = TELEPHONE_PORTRAIT
            .iter()
            .map(|l| l.chars().collect())
            .collect();
        let (hauteur, largeur) = (portrait.len(), portrait[0].len());
        for (r, ligne) in TELEPHONE_PAYSAGE.iter().enumerate() {
            for (c, car) in ligne.chars().enumerate() {
                assert_eq!(car, portrait[hauteur - 1 - c][r], "ligne {r}, colonne {c}");
            }
        }
        assert_eq!(TELEPHONE_PAYSAGE.len(), largeur);
    }

    #[test]
    fn le_message_tient_dans_l_ecran_logique_et_les_glyphes_existent() {
        for ligne in [LIGNE_1, LIGNE_2] {
            let largeur = font::largeur_texte(ligne, ECHELLE_TEXTE);
            assert!(largeur + 16 <= LARGEUR_LOGIQUE, "{ligne} : {largeur} px");
            for car in ligne.chars() {
                assert!(font::glyphe(car).is_some(), "glyphe manquant : {car:?}");
            }
        }
    }

    #[test]
    fn le_contenu_tient_dans_la_hauteur() {
        // Dernière ligne de texte : y = 122, 7 pixels de glyphe + l'ombre.
        let bas = 122 + (font::hauteur_texte(ECHELLE_TEXTE) + ECHELLE_TEXTE) as i32;
        assert!(bas < HAUTEUR_LOGIQUE as i32 - 6, "bas du texte : {bas}");
    }
}
