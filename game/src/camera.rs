//! Caméra à défilement : suit Pépin avec lissage, regarde un peu en avant dans
//! le sens de la marche, un peu vers le haut quand il monte (le jeu grimpe vers
//! le soleil), et ne sort jamais du niveau.
//!
//! La caméra est « pure » (aucun appel graphique) : elle se teste. Elle avance
//! à pas fixe, comme la physique ; le rendu interpole entre deux pas puis ARRONDIT
//! à l'entier (zéro sous-pixel, donc zéro scintillement).

use pepin_core::player::Joueur;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

/// Distance (px) regardée en avant de Pépin.
const AVANCE_X: f32 = 28.0;
/// Décalage vertical (px) vers le haut quand Pépin monte.
const AVANCE_HAUT: f32 = 24.0;
/// Part de la distance restante rattrapée à chaque pas (0 = figée, 1 = collée).
const LISSAGE_X: f32 = 0.10;
const LISSAGE_Y: f32 = 0.08;
/// Position de Pépin dans l'écran : un peu sous le centre pour voir plus haut.
const PART_Y: f32 = 0.58;

/// Coin haut-gauche de la zone visible, en pixels du niveau.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    /// Position au pas précédent, pour l'interpolation du rendu.
    precedente: (f32, f32),
    /// Décalage d'anticipation actuel (lissé lui aussi).
    avance: f32,
}

impl Camera {
    /// Caméra posée d'emblée sur Pépin (pas de glissement au lancement du niveau).
    pub fn new(joueur: &Joueur, largeur_niveau: f32, hauteur_niveau: f32) -> Self {
        let mut c = Self {
            x: 0.0,
            y: 0.0,
            precedente: (0.0, 0.0),
            avance: 0.0,
        };
        c.recentrer(joueur, largeur_niveau, hauteur_niveau);
        c
    }

    /// Saute directement à la position idéale (début de niveau, réapparition lointaine).
    pub fn recentrer(&mut self, joueur: &Joueur, largeur_niveau: f32, hauteur_niveau: f32) {
        let (cx, cy) = Self::cible(joueur, 0.0, 0.0);
        self.x = Self::borner(cx, largeur_niveau, LARGEUR_LOGIQUE as f32);
        self.y = Self::borner(cy, hauteur_niveau, HAUTEUR_LOGIQUE as f32);
        self.precedente = (self.x, self.y);
        self.avance = 0.0;
    }

    /// Point visé : Pépin légèrement décentré, plus l'anticipation.
    fn cible(joueur: &Joueur, avance: f32, haut: f32) -> (f32, f32) {
        let (px, py) = joueur.boite.centre();
        (
            px + avance - LARGEUR_LOGIQUE as f32 / 2.0,
            py - HAUTEUR_LOGIQUE as f32 * PART_Y - haut,
        )
    }

    /// Garde la caméra dans le niveau. Si le niveau est plus petit que l'écran,
    /// on aligne son bas sur le bas de l'écran (le ciel remplit le reste).
    fn borner(valeur: f32, taille_niveau: f32, taille_ecran: f32) -> f32 {
        let max = taille_niveau - taille_ecran;
        if max <= 0.0 {
            max
        } else {
            valeur.clamp(0.0, max)
        }
    }

    /// Un pas de simulation : se rapproche de la cible.
    pub fn suivre(&mut self, joueur: &Joueur, largeur_niveau: f32, hauteur_niveau: f32) {
        self.precedente = (self.x, self.y);
        let sens = if joueur.regarde_droite { 1.0 } else { -1.0 };
        let voulue = if joueur.vitesse.x.abs() > 8.0 { sens * AVANCE_X } else { 0.0 };
        self.avance += (voulue - self.avance) * 0.05;
        let haut = if joueur.vitesse.y < -40.0 { AVANCE_HAUT } else { 0.0 };
        let (cx, cy) = Self::cible(joueur, self.avance, haut);
        let cx = Self::borner(cx, largeur_niveau, LARGEUR_LOGIQUE as f32);
        let cy = Self::borner(cy, hauteur_niveau, HAUTEUR_LOGIQUE as f32);
        self.x += (cx - self.x) * LISSAGE_X;
        self.y += (cy - self.y) * LISSAGE_Y;
    }

    /// Position à afficher, interpolée entre deux pas puis arrondie à l'entier.
    pub fn position_rendue(&self, alpha: f32) -> (i32, i32) {
        let x = self.precedente.0 + (self.x - self.precedente.0) * alpha;
        let y = self.precedente.1 + (self.y - self.precedente.1) * alpha;
        (x.round() as i32, y.round() as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LARGEUR: f32 = 3000.0;
    const HAUTEUR: f32 = 176.0;

    #[test]
    fn la_camera_ne_sort_jamais_du_niveau() {
        let mut j = Joueur::new(5.0, 100.0);
        let mut c = Camera::new(&j, LARGEUR, HAUTEUR);
        assert!(c.x >= 0.0);
        for _ in 0..200 {
            c.suivre(&j, LARGEUR, HAUTEUR);
            assert!(c.x >= 0.0 && c.x <= LARGEUR - 320.0);
        }
        j.placer(LARGEUR - 5.0, 100.0);
        for _ in 0..600 {
            c.suivre(&j, LARGEUR, HAUTEUR);
            assert!(c.x <= LARGEUR - 320.0 + 0.001, "x = {}", c.x);
        }
        assert!((c.x - (LARGEUR - 320.0)).abs() < 1.0);
    }

    #[test]
    fn niveau_plus_bas_que_l_ecran_aligne_le_bas() {
        let j = Joueur::new(500.0, 100.0);
        let c = Camera::new(&j, LARGEUR, HAUTEUR);
        assert_eq!(c.y, HAUTEUR - 180.0);
    }

    #[test]
    fn le_suivi_est_lisse_et_converge() {
        let mut j = Joueur::new(200.0, 100.0);
        let mut c = Camera::new(&j, LARGEUR, HAUTEUR);
        j.placer(1000.0, 100.0);
        c.suivre(&j, LARGEUR, HAUTEUR);
        let (cible, _) = Camera::cible(&j, 0.0, 0.0);
        assert!(c.x < cible - 100.0, "un seul pas ne rattrape pas 800 px");
        for _ in 0..300 {
            c.suivre(&j, LARGEUR, HAUTEUR);
        }
        assert!((c.x - cible).abs() < 2.0);
    }

    #[test]
    fn l_anticipation_regarde_du_cote_de_la_marche() {
        let mut j = Joueur::new(1000.0, 100.0);
        j.vitesse.x = 96.0;
        let mut c = Camera::new(&j, LARGEUR, HAUTEUR);
        let (sans, _) = Camera::cible(&j, 0.0, 0.0);
        for _ in 0..200 {
            c.suivre(&j, LARGEUR, HAUTEUR);
        }
        assert!(c.x > sans + 10.0, "x = {} / sans = {}", c.x, sans);
    }

    #[test]
    fn la_position_rendue_est_entiere_et_interpolee() {
        let j = Joueur::new(1000.0, 100.0);
        let mut c = Camera::new(&j, LARGEUR, HAUTEUR);
        c.precedente = (100.0, 0.0);
        c.x = 110.0;
        assert_eq!(c.position_rendue(0.0).0, 100);
        assert_eq!(c.position_rendue(1.0).0, 110);
        assert_eq!(c.position_rendue(0.5).0, 105);
    }
}
