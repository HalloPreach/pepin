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
        let voulue = if joueur.vitesse.x.abs() > 8.0 {
            sens * AVANCE_X
        } else {
            0.0
        };
        self.avance += (voulue - self.avance) * 0.05;
        let haut = if joueur.vitesse.y < -40.0 {
            AVANCE_HAUT
        } else {
            0.0
        };
        let (cx, cy) = Self::cible(joueur, self.avance, haut);
        let cx = Self::borner(cx, largeur_niveau, LARGEUR_LOGIQUE as f32);
        let cy = Self::borner(cy, hauteur_niveau, HAUTEUR_LOGIQUE as f32);
        self.x += (cx - self.x) * LISSAGE_X;
        self.y += (cy - self.y) * LISSAGE_Y;
    }

    /// Position à afficher, interpolée entre deux pas puis arrondie à l'entier.
    ///
    /// `x_pepin` : abscisse interpolée de Pépin (coin gauche de sa boîte), celle que la scène
    /// dessine. Le décalage Pépin-caméra est arrondi UNE seule fois ; la caméra entière en est
    /// déduite (voir `camera_entiere`), si bien que Pépin ne tremble plus d'un pixel en courant.
    pub fn position_rendue(&self, alpha: f32, x_pepin: f32) -> (i32, i32) {
        let x = self.precedente.0 + (self.x - self.precedente.0) * alpha;
        let y = self.precedente.1 + (self.y - self.precedente.1) * alpha;
        (camera_entiere(x_pepin, x), y.round() as i32)
    }
}

/// Abscisse entière de la caméra pour le dessin, à partir de la position de Pépin et de celle
/// de la caméra (toutes deux en pixels du niveau, avec fractions).
///
/// Arrondir Pépin et la caméra chacun de leur côté fait osciller l'écart affiché d'un pixel
/// (156, 156, 155, 156, 155...). On arrondit donc l'écart une seule fois, puis on en déduit la
/// caméra : `cx = arrondi(x_pepin) - arrondi(x_pepin - x_camera)`. La scène dessine Pépin à
/// `arrondi(x_pepin) - cx`, soit exactement cet écart arrondi. Affichage seulement : la
/// simulation n'est pas touchée.
pub fn camera_entiere(x_pepin: f32, x_camera: f32) -> i32 {
    x_pepin.round() as i32 - (x_pepin - x_camera).round() as i32
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
        // Pépin posé pile sur la caméra (écart entier) : le résultat est la caméra arrondie.
        assert_eq!(c.position_rendue(0.0, 100.0).0, 100);
        assert_eq!(c.position_rendue(1.0, 110.0).0, 110);
        assert_eq!(c.position_rendue(0.5, 105.0).0, 105);
    }

    #[test]
    fn la_camera_entiere_garde_l_ecart_arrondi_une_seule_fois() {
        // Pépin à 1000,4 et caméra à 840,3 : l'écart est 160,1 -> 160 ; Pépin arrondi 1000.
        let cx = camera_entiere(1000.4, 840.3);
        assert_eq!(cx, 840);
        assert_eq!(1000.4_f32.round() as i32 - cx, 160);
        // Caméra entière : on la retrouve telle quelle.
        assert_eq!(camera_entiere(512.7, 400.0), 400);
    }

    /// Fait courir Pépin à vitesse constante (pas de 0,x px) et rend la suite des positions
    /// écran de Pépin et des abscisses de caméra, comme l'appel de `app.rs`.
    fn course(pas_px: f32, images: usize, vitesse: f32) -> Vec<(i32, i32)> {
        let mut j = Joueur::new(1000.0, 100.0);
        let mut c = Camera::new(&j, LARGEUR, HAUTEUR);
        let mut suite = Vec::new();
        for k in 1..=images {
            j.placer(1000.0 + k as f32 * pas_px, 100.0);
            j.vitesse.x = vitesse;
            c.suivre(&j, LARGEUR, HAUTEUR);
            let (cx, _) = c.position_rendue(1.0, j.boite.x);
            suite.push((j.boite.x.round() as i32 - cx, cx));
        }
        suite
    }

    #[test]
    fn en_courant_pepin_ne_fait_plus_d_aller_retour_a_l_ecran() {
        // Régime établi : le décalage Pépin-caméra se stabilise. Avant la correction, la
        // position écran faisait 10 allers-retours d'un pixel en 35 images.
        for pas_px in [0.4_f32, 0.8, 1.0, 1.6] {
            let suite = course(pas_px, 400, pas_px * 60.0);
            let ecran: Vec<i32> = suite.iter().skip(250).map(|&(e, _)| e).collect();
            let mut sens = 0;
            let mut retours = 0;
            for w in ecran.windows(2) {
                let d = (w[1] - w[0]).signum();
                if d != 0 {
                    if sens != 0 && d != sens {
                        retours += 1;
                    }
                    sens = d;
                }
            }
            assert_eq!(retours, 0, "pas {pas_px} : {ecran:?}");
        }
    }

    #[test]
    fn la_position_ecran_suit_l_ecart_reel_sans_bruit_d_arrondi() {
        // Pendant la mise en vitesse, l'écart réel Pépin-caméra monte puis redescend (la
        // caméra rattrape Pépin) : c'est du vrai mouvement. L'écran n'a le droit de changer de
        // sens qu'aux mêmes endroits que cet écart réel, jamais à cause de l'arrondi.
        let mut j = Joueur::new(1000.0, 100.0);
        let mut c = Camera::new(&j, LARGEUR, HAUTEUR);
        let (mut reel, mut ecran) = (Vec::new(), Vec::new());
        for k in 1..=300 {
            j.placer(1000.0 + k as f32 * 1.6, 100.0);
            j.vitesse.x = 96.0;
            c.suivre(&j, LARGEUR, HAUTEUR);
            let (cx, _) = c.position_rendue(1.0, j.boite.x);
            reel.push(j.boite.x - c.x);
            ecran.push(j.boite.x.round() as i32 - cx);
        }
        let retours = |suite: &[f32]| {
            let (mut sens, mut n) = (0.0_f32, 0);
            for w in suite.windows(2) {
                let d = (w[1] - w[0]).signum() * ((w[1] - w[0]).abs() > 1e-4) as i32 as f32;
                if d != 0.0 {
                    if sens != 0.0 && d != sens {
                        n += 1;
                    }
                    sens = d;
                }
            }
            n
        };
        let ecran_f: Vec<f32> = ecran.iter().map(|&e| e as f32).collect();
        assert!(
            retours(&ecran_f) <= retours(&reel),
            "écran {} retour(s) pour {} réel(s) : {ecran:?}",
            retours(&ecran_f),
            retours(&reel)
        );
        // Et l'écran reste à moins d'un demi-pixel de l'écart réel.
        for (r, e) in reel.iter().zip(&ecran) {
            assert!((r - *e as f32).abs() <= 0.5 + 1e-3, "réel {r} / écran {e}");
        }
    }

    #[test]
    fn le_decor_defile_sans_retour_en_arriere() {
        for pas_px in [0.4_f32, 0.8, 1.6] {
            let suite = course(pas_px, 400, pas_px * 60.0);
            assert!(
                suite.windows(2).all(|w| w[1].1 >= w[0].1),
                "pas {pas_px} : la caméra entière recule"
            );
        }
    }

    #[test]
    fn la_camera_verticale_ne_change_pas() {
        // Niveau de 11 rangées (176 px) : caméra verticale figée à -4, quelle que soit la
        // position de Pépin ; l'arrondi vertical est celui d'avant.
        let mut j = Joueur::new(500.0, 100.0);
        let mut c = Camera::new(&j, LARGEUR, HAUTEUR);
        for h in [20.0_f32, 60.0, 140.0, 20.0] {
            j.placer(500.0, h);
            c.suivre(&j, LARGEUR, HAUTEUR);
            assert_eq!(c.position_rendue(1.0, j.boite.x).1, -4);
        }
        // Niveau plus haut que l'écran : l'arrondi de y est celui de la caméra.
        c.y = 12.6;
        c.precedente.1 = 12.6;
        assert_eq!(c.position_rendue(1.0, 500.0).1, 13);
    }
}
