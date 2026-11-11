//! Dessin de texte pixel par pixel avec la police bitmap 5x7 de `pepin_core::font`.
//!
//! Tout est dessiné en coordonnées logiques (320x180) : chaque pixel allumé
//! du glyphe devient un carré de `echelle` x `echelle` pixels logiques.

use pepin_core::font::{self, ESPACEMENT, HAUTEUR_GLYPHE, LARGEUR_GLYPHE};
use pepin_core::palette::Couleur;
use pepin_core::LARGEUR_LOGIQUE;
use macroquad::prelude::{draw_rectangle, Color};

/// Apparence d'un texte : échelle entière, couleur de palette, ombre portée facultative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// Facteur d'agrandissement entier (1 = glyphes de 5x7 pixels logiques).
    pub echelle: u32,
    /// Couleur du texte (issue de `palette.rs`).
    pub couleur: Couleur,
    /// Couleur de l'ombre portée, décalée d'un pixel de glyphe vers le bas-droite ; `None` = pas d'ombre.
    pub ombre: Option<Couleur>,
}

impl Style {
    /// Style sans ombre, à l'échelle donnée (au minimum 1).
    pub const fn simple(couleur: Couleur, echelle: u32) -> Self {
        Self { echelle: if echelle == 0 { 1 } else { echelle }, couleur, ombre: None }
    }

    /// Même style avec une ombre portée de la couleur donnée.
    pub const fn avec_ombre(self, ombre: Couleur) -> Self {
        Self { ombre: Some(ombre), ..self }
    }
}

/// Convertit une couleur de palette en couleur macroquad opaque.
fn couleur_mq(c: Couleur) -> Color {
    let [r, g, b] = c.en_f32();
    Color::new(r, g, b, 1.0)
}

/// Dessine le texte une fois, avec son coin haut-gauche en (`x`, `y`).
fn dessiner_passe(texte: &str, x: f32, y: f32, echelle: u32, couleur: Couleur) {
    let e = echelle as f32;
    let c = couleur_mq(couleur);
    let pas = (LARGEUR_GLYPHE + ESPACEMENT) as f32 * e;
    for (i, car) in texte.chars().enumerate() {
        let dessin = font::glyphe_ou_inconnu(car);
        let x0 = x + i as f32 * pas;
        for gy in 0..HAUTEUR_GLYPHE {
            for gx in 0..LARGEUR_GLYPHE {
                if font::pixel_allume(dessin, gx, gy) {
                    draw_rectangle(x0 + gx as f32 * e, y + gy as f32 * e, e, e, c);
                }
            }
        }
    }
}

/// Dessine `texte` avec son coin haut-gauche en (`x`, `y`) (pixels logiques).
/// L'ombre éventuelle est dessinée d'abord, puis le texte par-dessus.
pub fn dessiner(texte: &str, x: f32, y: f32, style: Style) {
    // On aligne sur la grille de pixels logiques pour éviter tout flou.
    let (x, y) = (x.round(), y.round());
    if let Some(ombre) = style.ombre {
        let d = style.echelle as f32;
        dessiner_passe(texte, x + d, y + d, style.echelle, ombre);
    }
    dessiner_passe(texte, x, y, style.echelle, style.couleur);
}

/// Abscisse du coin gauche d'un texte centré horizontalement autour de `centre_x`.
pub fn x_centre(texte: &str, centre_x: f32, echelle: u32) -> f32 {
    centre_x - font::largeur_texte(texte, echelle) as f32 / 2.0
}

/// Dessine `texte` centré horizontalement autour de `centre_x`, ligne du haut en `y`.
pub fn dessiner_centre_sur(texte: &str, centre_x: f32, y: f32, style: Style) {
    dessiner(texte, x_centre(texte, centre_x, style.echelle), y, style);
}

/// Dessine `texte` centré horizontalement sur tout l'écran logique, ligne du haut en `y`.
pub fn dessiner_centre(texte: &str, y: f32, style: Style) {
    dessiner_centre_sur(texte, LARGEUR_LOGIQUE as f32 / 2.0, y, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::palette::{BLANC_CASSE, ARDOISE_OMBRE};

    #[test]
    fn le_centrage_utilise_la_largeur_reelle_du_texte() {
        // « AB » à l'échelle 2 : (5 + 1 + 5) * 2 = 22 pixels, donc départ à 160 - 11.
        assert_eq!(x_centre("AB", 160.0, 2), 149.0);
        assert_eq!(x_centre("", 160.0, 2), 160.0);
    }

    #[test]
    fn l_echelle_nulle_est_ramenee_a_un() {
        assert_eq!(Style::simple(BLANC_CASSE, 0).echelle, 1);
        assert_eq!(Style::simple(BLANC_CASSE, 3).avec_ombre(ARDOISE_OMBRE).ombre, Some(ARDOISE_OMBRE));
    }
}
