//! Rendu : tout est dessiné dans une petite image de 320x180 pixels (la « cible »),
//! puis cette image est agrandie d'un facteur ENTIER (x1, x2, x3...) au centre de
//! la fenêtre, avec des bandes sombres si les proportions ne tombent pas juste.
//! Pas de flou : filtre « nearest », positions arrondies, jamais de demi-pixel.
//!
//! Notions Rust illustrées ici : fonctions pures testables (`facteur_entier`,
//! `placement`), struct qui possède une ressource graphique (`Cible`).

use macroquad::prelude::*;
use pepin_core::palette::{Couleur, ARDOISE_OMBRE};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

/// Convertit une couleur de palette en couleur macroquad opaque.
pub fn couleur_mq(c: Couleur) -> Color {
    let [r, g, b] = c.en_f32();
    Color::new(r, g, b, 1.0)
}

/// Plus grand facteur ENTIER (au moins 1) tel que l'image 320x180 tienne dans
/// une fenêtre de `largeur` x `hauteur` pixels physiques.
pub fn facteur_entier(largeur: u32, hauteur: u32) -> u32 {
    (largeur / LARGEUR_LOGIQUE)
        .min(hauteur / HAUTEUR_LOGIQUE)
        .max(1)
}

/// Où poser l'image agrandie : `(x, y, facteur)` en pixels physiques, centrée.
pub fn placement(largeur: u32, hauteur: u32) -> (i32, i32, u32) {
    let k = facteur_entier(largeur, hauteur);
    let x = (largeur as i32 - (LARGEUR_LOGIQUE * k) as i32) / 2;
    let y = (hauteur as i32 - (HAUTEUR_LOGIQUE * k) as i32) / 2;
    (x, y, k)
}

/// L'image 320x180 dans laquelle on dessine tout le jeu.
pub struct Cible {
    cible: RenderTarget,
}

impl Cible {
    /// Crée la cible en filtre « nearest ».
    pub fn new() -> Self {
        let cible = render_target(LARGEUR_LOGIQUE, HAUTEUR_LOGIQUE);
        cible.texture.set_filter(FilterMode::Nearest);
        Self { cible }
    }

    /// À appeler avant de dessiner : les coordonnées sont alors celles de l'écran logique
    /// (0,0) = coin haut-gauche, 320x180.
    pub fn commencer(&self) {
        let mut camera = Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            LARGEUR_LOGIQUE as f32,
            HAUTEUR_LOGIQUE as f32,
        ));
        camera.render_target = Some(self.cible.clone());
        set_camera(&camera);
    }

    /// Affiche la cible sur la fenêtre, agrandie d'un facteur entier et centrée.
    pub fn afficher(&self) {
        set_default_camera();
        clear_background(couleur_mq(ARDOISE_OMBRE));
        // Taille de la fenêtre en pixels PHYSIQUES (l'écran peut avoir un facteur d'échelle).
        let dpi = screen_dpi_scale();
        let (l, h) = (
            (screen_width() * dpi).round() as u32,
            (screen_height() * dpi).round() as u32,
        );
        let (x, y, k) = placement(l, h);
        // Les coordonnées de dessin sont logiques : on divise par le facteur d'écran.
        draw_texture_ex(
            &self.cible.texture,
            x as f32 / dpi,
            y as f32 / dpi,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (LARGEUR_LOGIQUE * k) as f32 / dpi,
                    (HAUTEUR_LOGIQUE * k) as f32 / dpi,
                )),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}

impl Default for Cible {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_facteur_est_toujours_entier_et_au_moins_un() {
        // Tailles de fenêtre courantes : (largeur, hauteur) -> facteur attendu.
        let cas = [
            (1280, 720, 4),
            (1920, 1080, 6),
            (2560, 1440, 8),
            (3840, 2160, 12),
            (1366, 768, 4),
            (1440, 900, 4),
            (390, 844, 1),
            (844, 390, 2),
            (320, 180, 1),
            (100, 50, 1),
            (0, 0, 1),
        ];
        for (l, h, k) in cas {
            assert_eq!(facteur_entier(l, h), k, "{l}x{h}");
        }
    }

    #[test]
    fn l_image_agrandie_tient_dans_la_fenetre_et_est_centree() {
        for (l, h) in [(1280u32, 720u32), (1366, 768), (1440, 900), (2560, 1080)] {
            let (x, y, k) = placement(l, h);
            assert!(x >= 0 && y >= 0, "{l}x{h}");
            assert!(x as u32 + LARGEUR_LOGIQUE * k <= l);
            assert!(y as u32 + HAUTEUR_LOGIQUE * k <= h);
            // Bandes équilibrées à un pixel près.
            assert!((l as i32 - 2 * x - (LARGEUR_LOGIQUE * k) as i32).abs() <= 1);
        }
    }

    /// Fenêtre en pixels CSS + facteur d'écran -> pixels PHYSIQUES du canevas
    /// (c'est ce que voit le jeu depuis que `high_dpi` est activé dans `Conf`).
    fn physique(css_l: f32, css_h: f32, dpr: f32) -> (u32, u32) {
        ((css_l * dpr).round() as u32, (css_h * dpr).round() as u32)
    }

    #[test]
    fn le_facteur_sur_des_tailles_physiques_realistes() {
        // (largeur CSS, hauteur CSS, DPR, facteur attendu en pixels physiques)
        let cas = [
            // Téléphone en paysage, barre du navigateur affichée (844x340) : x5, et non x1.
            (844.0, 340.0, 3.0, 5),
            // Même téléphone, barre rentrée (844x390 = 2532x1170 physiques) : x6.
            (844.0, 390.0, 3.0, 6),
            // Portable à 100 %, 125 % et 150 % (1280x720 CSS) : toujours entier.
            (1280.0, 720.0, 1.0, 4),
            (1280.0, 720.0, 1.25, 5),
            (1280.0, 720.0, 1.5, 6),
            // Écran 4K à 200 %.
            (1920.0, 1080.0, 2.0, 12),
        ];
        for (cl, ch, dpr, k) in cas {
            let (l, h) = physique(cl, ch, dpr);
            assert_eq!(
                facteur_entier(l, h),
                k,
                "{cl}x{ch} @ {dpr} = {l}x{h} physiques"
            );
        }
        assert_eq!(physique(844.0, 390.0, 3.0), (2532, 1170));
        assert_eq!(facteur_entier(2532, 1170), 6);
        assert_eq!(facteur_entier(2532, 1020), 5);
        assert_eq!(facteur_entier(1280, 720), 4);
    }

    #[test]
    fn sur_telephone_le_jeu_n_est_plus_un_tiers_de_l_ecran() {
        // Avant `high_dpi`, le jeu voyait 844x340 (pixels CSS) : facteur 1, soit 180 px
        // CSS de haut. Maintenant il voit 2532x1020 : facteur 5, soit 900 px physiques,
        // 300 px CSS de haut.
        assert_eq!(
            facteur_entier(844, 340),
            1,
            "en pixels CSS : x1 (le défaut corrigé)"
        );
        assert_eq!(
            facteur_entier(844, 359),
            1,
            "en pixels CSS : x2 seulement dès 360 px"
        );
        let (l, h) = physique(844.0, 340.0, 3.0);
        let k = facteur_entier(l, h);
        let haut_css = (HAUTEUR_LOGIQUE * k) as f32 / 3.0;
        assert!(haut_css >= 300.0, "{haut_css} px CSS de haut");
    }

    #[test]
    fn le_placement_physique_est_entier_centre_et_sans_debordement() {
        for (l, h) in [
            (2532u32, 1020u32),
            (2532, 1170),
            (1600, 900),
            (1920, 1080),
            (1920, 1081),
            (2400, 1080),
        ] {
            let (x, y, k) = placement(l, h);
            assert!(k >= 1);
            assert!(x >= 0 && y >= 0, "{l}x{h}");
            assert!(x as u32 + LARGEUR_LOGIQUE * k <= l, "{l}x{h}");
            assert!(y as u32 + HAUTEUR_LOGIQUE * k <= h, "{l}x{h}");
            // Bandes équilibrées à un pixel physique près.
            assert!((l as i32 - 2 * x - (LARGEUR_LOGIQUE * k) as i32).abs() <= 1);
            assert!((h as i32 - 2 * y - (HAUTEUR_LOGIQUE * k) as i32).abs() <= 1);
        }
        assert_eq!(placement(2532, 1020), (466, 60, 5));
        assert_eq!(placement(1600, 900), (0, 0, 5));
    }
}
