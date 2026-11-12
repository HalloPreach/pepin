//! Navigation dans les menus : touches de menu, clic de souris et toucher.
//!
//! C'est volontairement séparé de `entrees.rs` : les `Commandes` pilotent Pépin
//! pendant la partie (à chaque pas de simulation), alors que la `Navigation`
//! pilote les menus (une fois par image affichée, sur des appuis ponctuels).
//!
//! Sur un navigateur, un toucher est aussi vu comme un clic de souris par
//! macroquad : une seule lecture « pointeur » suffit donc pour la souris ET
//! le tactile. Les coordonnées sont converties en pixels LOGIQUES (320x180).
//!
//! Notions Rust illustrées ici : `Option` pour « il n'y a peut-être pas de
//! clic », fonction pure (`vers_logique`) testée sans fenêtre, `Default`.

use macroquad::prelude::{
    is_key_pressed, is_mouse_button_pressed, mouse_position, screen_dpi_scale, screen_height,
    screen_width, KeyCode, MouseButton,
};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::rendu::placement;

/// Ce que le joueur a demandé dans un menu pendant cette image.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Navigation {
    pub haut: bool,
    pub bas: bool,
    pub valider: bool,
    pub retour: bool,
    /// Touche M : couper ou remettre le son.
    pub son: bool,
    /// Un clic ou un toucher vient d'avoir lieu ici (pixels logiques).
    pub clic: Option<(f32, f32)>,
    /// Le pointeur a bougé et se trouve ici (pixels logiques) : surbrillance.
    pub survol: Option<(f32, f32)>,
}

/// Convertit une position de pointeur (pixels physiques de la fenêtre) en pixels
/// logiques de l'image 320x180 agrandie. `None` si le pointeur est dans les
/// bandes sombres, hors de l'image.
pub fn vers_logique(px: f32, py: f32, largeur: u32, hauteur: u32) -> Option<(f32, f32)> {
    let (x0, y0, k) = placement(largeur, hauteur);
    let k = k as f32;
    let (lx, ly) = ((px - x0 as f32) / k, (py - y0 as f32) / k);
    let dedans =
        (0.0..LARGEUR_LOGIQUE as f32).contains(&lx) && (0.0..HAUTEUR_LOGIQUE as f32).contains(&ly);
    dedans.then_some((lx, ly))
}

/// Lit le clavier et le pointeur. Garde en mémoire la dernière position du
/// pointeur pour ne signaler un survol que quand la souris bouge vraiment
/// (sinon la souris immobile écraserait les choix faits au clavier).
#[derive(Debug, Default)]
pub struct LecteurNavigation {
    dernier: Option<(f32, f32)>,
}

fn une_pressee(touches: &[KeyCode]) -> bool {
    touches.iter().any(|&t| is_key_pressed(t))
}

impl LecteurNavigation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lit les appuis de cette image.
    pub fn lire(&mut self) -> Navigation {
        let dpi = screen_dpi_scale();
        let (l, h) = (
            (screen_width() * dpi).round() as u32,
            (screen_height() * dpi).round() as u32,
        );
        let (mx, my) = mouse_position();
        let position = vers_logique(mx * dpi, my * dpi, l, h);
        let bouge = self.dernier != Some((mx, my));
        self.dernier = Some((mx, my));
        Navigation {
            haut: une_pressee(&[KeyCode::Up, KeyCode::W, KeyCode::Z]),
            bas: une_pressee(&[KeyCode::Down, KeyCode::S]),
            valider: une_pressee(&[KeyCode::Enter, KeyCode::KpEnter, KeyCode::Space]),
            retour: une_pressee(&[KeyCode::Escape, KeyCode::Backspace]),
            son: is_key_pressed(KeyCode::M),
            clic: if is_mouse_button_pressed(MouseButton::Left) {
                position
            } else {
                None
            },
            survol: if bouge { position } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_centre_de_la_fenetre_est_le_centre_de_l_image() {
        // 1280x720 = facteur 4, sans bandes : (640, 360) -> (160, 90).
        assert_eq!(vers_logique(640.0, 360.0, 1280, 720), Some((160.0, 90.0)));
        assert_eq!(vers_logique(0.0, 0.0, 1280, 720), Some((0.0, 0.0)));
    }

    #[test]
    fn les_bandes_sombres_sont_hors_de_l_image() {
        // 1440x900 = facteur 4 : image de 1280x720, bandes de 80 px de chaque côté.
        assert_eq!(vers_logique(10.0, 450.0, 1440, 900), None);
        assert_eq!(vers_logique(1430.0, 450.0, 1440, 900), None);
        assert_eq!(vers_logique(720.0, 450.0, 1440, 900), Some((160.0, 90.0)));
    }

    #[test]
    fn le_bord_droit_et_bas_sont_exclus() {
        assert_eq!(vers_logique(1280.0, 100.0, 1280, 720), None);
        assert_eq!(vers_logique(100.0, 720.0, 1280, 720), None);
    }

    #[test]
    fn une_petite_fenetre_garde_un_facteur_un() {
        // 390x844 (téléphone en portrait) : facteur 1, image centrée verticalement.
        let (x0, y0, k) = placement(390, 844);
        assert_eq!(k, 1);
        let (lx, ly) = vers_logique(x0 as f32 + 5.0, y0 as f32 + 7.0, 390, 844).unwrap();
        assert_eq!((lx, ly), (5.0, 7.0));
    }
}
