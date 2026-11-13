//! Caméra à défilement : suit Pépin avec lissage, regarde un peu en avant dans
//! le sens de la marche, un peu vers le haut quand il monte (le jeu grimpe vers
//! le soleil), et ne sort jamais du niveau.
//!
//! En hauteur, deux régimes (carte 5.2) :
//! - niveau d'UN écran de haut (11 rangées, 176 px) : la caméra verticale est figée,
//!   le bas du niveau sur le bas de l'écran. Rien n'a changé ici depuis le début du
//!   projet : le test `les_niveaux_d_un_ecran_gardent_la_meme_camera` le prouve en
//!   comparant, pas à pas, avec une copie de l'ancienne caméra ;
//! - niveau PLUS HAUT que l'écran (les niveaux verticaux de l'étape 5) : la caméra suit
//!   Pépin en hauteur avec une ZONE MORTE (voir `suivre_en_hauteur`).
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

// --- Niveaux plus hauts que l'écran ------------------------------------------------
//
// Les parts ci-dessous sont des fractions de la hauteur de l'écran (180 px), mesurées
// depuis le HAUT de l'écran jusqu'au CENTRE de Pépin.

/// Haut de la zone morte : 36 px. Un saut sur place depuis le cadrage « posé » (0,58)
/// monte le centre de Pépin à 0,26 (57 px plus haut) : il reste dans la zone, la caméra
/// ne bouge pas à chaque saut.
const ZONE_HAUT: f32 = 0.20;
/// Bas de la zone morte en chute ou en saut : 126 px.
const ZONE_BAS: f32 = 0.70;
/// Bas de la zone morte parapluie ouvert : 90 px. On plane pour descendre : la caméra
/// laisse voir où l'on va se poser (moitié basse de l'écran).
const ZONE_BAS_PLANE: f32 = 0.50;
/// Lissage vertical d'un niveau haut, plus vif que `LISSAGE_Y` : en chute libre
/// (6 px par pas), la caméra traîne 6 / 0,12 = 50 px derrière sa cible, pas 75.
const LISSAGE_Y_HAUT: f32 = 0.12;
/// Marge (px) toujours gardée entre la boîte de Pépin et le haut ou le bas de l'écran,
/// même en chute rapide (sauf au-dessus du niveau : là, la flèche de repère le montre).
const MARGE_ECRAN: f32 = 8.0;

/// Coin haut-gauche de la zone visible, en pixels du niveau.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    /// Position au pas précédent, pour l'interpolation du rendu.
    precedente: (f32, f32),
    /// Décalage d'anticipation actuel (lissé lui aussi).
    avance: f32,
    /// Niveau plus haut que l'écran : la caméra suit Pépin en hauteur.
    en_hauteur: bool,
    /// Niveau haut seulement : ordonnée VISÉE (avant lissage). Pépin posé la recadre ;
    /// en l'air elle ne bouge que s'il sort de la zone morte.
    visee_y: f32,
}

impl Camera {
    /// Caméra posée d'emblée sur Pépin (pas de glissement au lancement du niveau).
    pub fn new(joueur: &Joueur, largeur_niveau: f32, hauteur_niveau: f32) -> Self {
        let mut c = Self {
            x: 0.0,
            y: 0.0,
            precedente: (0.0, 0.0),
            avance: 0.0,
            en_hauteur: false,
            visee_y: 0.0,
        };
        c.recentrer(joueur, largeur_niveau, hauteur_niveau);
        c
    }

    /// Le niveau est-il plus haut que l'écran (caméra verticale mobile) ?
    pub fn suit_en_hauteur(hauteur_niveau: f32) -> bool {
        hauteur_niveau > HAUTEUR_LOGIQUE as f32
    }

    /// Saute directement à la position idéale (début de niveau, réapparition lointaine).
    pub fn recentrer(&mut self, joueur: &Joueur, largeur_niveau: f32, hauteur_niveau: f32) {
        let (cx, cy) = Self::cible(joueur, 0.0, 0.0);
        self.x = Self::borner(cx, largeur_niveau, LARGEUR_LOGIQUE as f32);
        self.y = Self::borner(cy, hauteur_niveau, HAUTEUR_LOGIQUE as f32);
        self.en_hauteur = Self::suit_en_hauteur(hauteur_niveau);
        // Même cadrage que Pépin posé (`cible` sans anticipation) : y est déjà bornée.
        self.visee_y = self.y;
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
        self.x += (cx - self.x) * LISSAGE_X;
        self.en_hauteur = Self::suit_en_hauteur(hauteur_niveau);
        if self.en_hauteur {
            self.suivre_en_hauteur(joueur, hauteur_niveau);
        } else {
            // Niveau d'un écran : exactement le calcul d'avant la carte 5.2 (la cible
            // bornée vaut toujours le bas du niveau, la caméra ne bouge pas).
            let cy = Self::borner(cy, hauteur_niveau, HAUTEUR_LOGIQUE as f32);
            self.y += (cy - self.y) * LISSAGE_Y;
        }
    }

    /// Suivi vertical d'un niveau plus haut que l'écran, en trois temps :
    /// 1. visée : Pépin POSÉ est recadré à `PART_Y` (on voit plus haut que lui : le jeu
    ///    grimpe) ; EN L'AIR, la visée ne bouge pas tant que son centre reste dans la
    ///    zone morte `[ZONE_HAUT, ZONE_BAS]` de l'écran (`ZONE_BAS_PLANE` parapluie
    ///    ouvert) : un saut ne fait pas tanguer l'écran, une chute est suivie ;
    /// 2. la visée est bornée au niveau, puis la caméra s'en rapproche (lissage) ;
    /// 3. garde-fou : la boîte de Pépin reste à `MARGE_ECRAN` px au moins des bords
    ///    haut et bas de l'écran, quelle que soit sa vitesse ; seules les bornes du
    ///    niveau passent avant (au-dessus du niveau, la flèche de repère le montre).
    fn suivre_en_hauteur(&mut self, joueur: &Joueur, hauteur_niveau: f32) {
        let h = HAUTEUR_LOGIQUE as f32;
        let (_, centre) = joueur.boite.centre();
        if joueur.au_sol {
            self.visee_y = centre - h * PART_Y;
        } else {
            let bas = if joueur.plane {
                ZONE_BAS_PLANE
            } else {
                ZONE_BAS
            };
            // `clamp(min, max)` : la visée la plus basse garde Pépin sous `ZONE_HAUT`,
            // la plus haute le garde au-dessus du bas de la zone.
            self.visee_y = self.visee_y.clamp(centre - h * bas, centre - h * ZONE_HAUT);
        }
        self.visee_y = Self::borner(self.visee_y, hauteur_niveau, h);
        self.y += (self.visee_y - self.y) * LISSAGE_Y_HAUT;
        let (haut, bas) = (joueur.boite.y, joueur.boite.bas());
        let y = self.y.clamp(bas + MARGE_ECRAN - h, haut - MARGE_ECRAN);
        self.y = Self::borner(y, hauteur_niveau, h);
    }

    /// Position à afficher, interpolée entre deux pas puis arrondie à l'entier.
    ///
    /// `pepin` : position interpolée de Pépin (coin haut-gauche de sa boîte), celle que la
    /// scène dessine. Le décalage Pépin-caméra est arrondi UNE seule fois ; la caméra
    /// entière en est déduite (voir `camera_entiere`), si bien que Pépin ne tremble plus
    /// d'un pixel en courant. En hauteur, même règle pour un niveau haut ; un niveau d'un
    /// écran garde l'arrondi simple d'avant (sa caméra verticale est entière et figée).
    pub fn position_rendue(&self, alpha: f32, pepin: (f32, f32)) -> (i32, i32) {
        let x = self.precedente.0 + (self.x - self.precedente.0) * alpha;
        let y = self.precedente.1 + (self.y - self.precedente.1) * alpha;
        let y_entier = if self.en_hauteur {
            camera_entiere(pepin.1, y)
        } else {
            y.round() as i32
        };
        (camera_entiere(pepin.0, x), y_entier)
    }
}

/// Coordonnée entière de la caméra pour le dessin, à partir de la position de Pépin et de
/// celle de la caméra (toutes deux en pixels du niveau, avec fractions), sur un axe.
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
        assert_eq!(c.position_rendue(0.0, (100.0, 100.0)).0, 100);
        assert_eq!(c.position_rendue(1.0, (110.0, 100.0)).0, 110);
        assert_eq!(c.position_rendue(0.5, (105.0, 100.0)).0, 105);
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
            let (cx, _) = c.position_rendue(1.0, (j.boite.x, j.boite.y));
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
            let (cx, _) = c.position_rendue(1.0, (j.boite.x, j.boite.y));
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
            assert_eq!(c.position_rendue(1.0, (j.boite.x, j.boite.y)).1, -4);
        }
        // Niveau d'un écran : arrondi simple de y (celui de la caméra seule).
        c.y = 12.6;
        c.precedente.1 = 12.6;
        assert_eq!(c.position_rendue(1.0, (500.0, 100.0)).1, 13);
    }

    // --- Niveaux d'un écran : la même caméra qu'avant la carte 5.2 -------------------

    /// Copie de la caméra d'AVANT la carte 5.2 (commit abd1483), réduite à ce qui calcule
    /// une position. C'est la référence du test suivant : ne pas la « corriger ».
    mod ancienne {
        use super::super::camera_entiere;
        use pepin_core::player::Joueur;
        use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

        const AVANCE_X: f32 = 28.0;
        const AVANCE_HAUT: f32 = 24.0;
        const LISSAGE_X: f32 = 0.10;
        const LISSAGE_Y: f32 = 0.08;
        const PART_Y: f32 = 0.58;

        pub struct Camera {
            pub x: f32,
            pub y: f32,
            pub precedente: (f32, f32),
            pub avance: f32,
        }

        impl Camera {
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

            pub fn recentrer(&mut self, joueur: &Joueur, largeur_niveau: f32, hauteur_niveau: f32) {
                let (cx, cy) = Self::cible(joueur, 0.0, 0.0);
                self.x = Self::borner(cx, largeur_niveau, LARGEUR_LOGIQUE as f32);
                self.y = Self::borner(cy, hauteur_niveau, HAUTEUR_LOGIQUE as f32);
                self.precedente = (self.x, self.y);
                self.avance = 0.0;
            }

            fn cible(joueur: &Joueur, avance: f32, haut: f32) -> (f32, f32) {
                let (px, py) = joueur.boite.centre();
                (
                    px + avance - LARGEUR_LOGIQUE as f32 / 2.0,
                    py - HAUTEUR_LOGIQUE as f32 * PART_Y - haut,
                )
            }

            fn borner(valeur: f32, taille_niveau: f32, taille_ecran: f32) -> f32 {
                let max = taille_niveau - taille_ecran;
                if max <= 0.0 {
                    max
                } else {
                    valeur.clamp(0.0, max)
                }
            }

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

            pub fn position_rendue(&self, alpha: f32, x_pepin: f32) -> (i32, i32) {
                let x = self.precedente.0 + (self.x - self.precedente.0) * alpha;
                let y = self.precedente.1 + (self.y - self.precedente.1) * alpha;
                (camera_entiere(x_pepin, x), y.round() as i32)
            }
        }
    }

    /// Entrées pseudo-aléatoires mais FIXES (générateur congruentiel, même graine = mêmes
    /// entrées) : courir, s'arrêter, changer de sens, sauter plus ou moins longtemps, donc
    /// aussi planer (saut maintenu en descente) et tomber dans les trous.
    fn entrees_melangees(n: usize, graine: u64) -> Vec<Entrees> {
        let mut g = graine;
        let mut tirer = |m: u64| {
            g = g
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (g >> 33) % m
        };
        let mut v = Vec::with_capacity(n);
        while v.len() < n {
            let direction = match tirer(4) {
                0 => Direction::Gauche,
                1 => Direction::Immobile,
                _ => Direction::Droite,
            };
            let duree = 10 + tirer(80) as usize;
            let debut_saut = tirer(duree as u64) as usize;
            let duree_saut = tirer(70) as usize;
            for k in 0..duree {
                let saut = (debut_saut..debut_saut + duree_saut).contains(&k);
                v.push(Entrees { direction, saut });
            }
        }
        v.truncate(n);
        v
    }

    /// Les 32 bits exacts d'un état de caméra : `-0.0` et `0.0` sont différents ici.
    fn bits(x: f32, y: f32, precedente: (f32, f32), avance: f32) -> [u32; 5] {
        [x, y, precedente.0, precedente.1, avance].map(f32::to_bits)
    }

    /// La preuve demandée : sur chaque niveau du jeu (11 rangées), la nouvelle caméra et
    /// l'ancienne, menées par la VRAIE simulation (`Monde`, morts et recentrages compris,
    /// comme dans `app.rs`), sont identiques au bit près à chaque pas, et la position
    /// rendue (celle que la scène dessine) aussi, pour plusieurs `alpha`.
    #[test]
    fn les_niveaux_d_un_ecran_gardent_la_meme_camera() {
        use crate::monde::Monde;
        use pepin_core::entities::contacts::Contact;
        use pepin_core::zones::NIVEAUX;
        let (mut morts, mut plus_loin, mut sauts) = (0, 0.0_f32, 0);
        for (indice, infos) in NIVEAUX.iter().enumerate() {
            let mut m = Monde::new(indice);
            let (l, h) = (m.niveau.largeur_px(), m.niveau.hauteur_px());
            assert!(
                !Camera::suit_en_hauteur(h),
                "{} : un écran de haut",
                infos.nom
            );
            let mut nouvelle = Camera::new(&m.joueur, l, h);
            let mut vieille = ancienne::Camera::new(&m.joueur, l, h);
            for (pas, e) in entrees_melangees(5000, indice as u64 + 1)
                .iter()
                .enumerate()
            {
                m.avancer(e);
                if m.contacts.iter().any(|c| matches!(c, Contact::Mort(_))) {
                    morts += 1;
                    nouvelle.recentrer(&m.joueur, l, h);
                    vieille.recentrer(&m.joueur, l, h);
                }
                sauts += usize::from(m.evenements.saut);
                nouvelle.suivre(&m.joueur, l, h);
                vieille.suivre(&m.joueur, l, h);
                assert_eq!(
                    bits(nouvelle.x, nouvelle.y, nouvelle.precedente, nouvelle.avance),
                    bits(vieille.x, vieille.y, vieille.precedente, vieille.avance),
                    "{}, pas {pas}",
                    infos.nom
                );
                for alpha in [0.0, 0.25, 0.5, 0.75, 0.999, 1.0] {
                    let pepin = m.joueur_interpole(alpha);
                    assert_eq!(
                        nouvelle.position_rendue(alpha, pepin),
                        vieille.position_rendue(alpha, pepin.0),
                        "{}, pas {pas}, alpha {alpha}",
                        infos.nom
                    );
                }
                plus_loin = plus_loin.max(nouvelle.x);
            }
        }
        // Le test a bien vu la caméra défiler, Pépin sauter et mourir (recentrage).
        println!("morts : {morts}, sauts : {sauts}, caméra la plus à droite : {plus_loin}");
        assert!(morts >= 5 && sauts >= 100 && plus_loin > 300.0);
    }

    // --- Niveaux plus hauts que l'écran : mini-niveau vertical de test ---------------

    use pepin_core::aabb::GrilleTuiles;
    use pepin_core::physics::{Environnement, JOUEUR_H};
    use pepin_core::player::{Direction, Entrees};

    /// Mini-niveau vertical : 20 x 40 tuiles (640 px de haut, 3,6 écrans). Sol en bas
    /// (rangée 39, dessus à y = 624), une corniche à mi-hauteur (rangée 20, colonnes 2 à
    /// 9, dessus à y = 320) et une autre trois rangées plus haut (rangée 17, colonnes 12
    /// à 17, dessus à y = 272). Rien dans les colonnes 18 et 19 : un puits jusqu'au sol.
    fn tour() -> GrilleTuiles {
        let mut lignes = vec!["...................."; 40];
        lignes[17] = "............######..";
        lignes[20] = "..########..........";
        lignes[39] = "####################";
        GrilleTuiles::depuis_texte(&lignes)
    }
    const L_TOUR: f32 = 320.0;
    const H_TOUR: f32 = 640.0;
    const H_ECRAN: f32 = HAUTEUR_LOGIQUE as f32;

    /// Pépin posé (après quelques pas) sur le solide dont le dessus est en `sol_y`.
    fn pose(g: &GrilleTuiles, x: f32, sol_y: f32) -> Joueur {
        let env = Environnement::simple(g);
        let mut j = Joueur::new(x, sol_y - JOUEUR_H - 2.0);
        for _ in 0..10 {
            j.actualiser(&Entrees::default(), &env);
        }
        assert!(j.au_sol && (j.boite.bas() - sol_y).abs() < 0.01);
        j
    }

    /// Ordonnée du centre de Pépin À L'ÉCRAN (0 = haut de l'écran).
    fn centre_ecran(j: &Joueur, c: &Camera) -> f32 {
        j.boite.centre().1 - c.y
    }

    #[test]
    fn seul_un_niveau_plus_haut_que_l_ecran_suit_en_hauteur() {
        assert!(!Camera::suit_en_hauteur(176.0), "11 rangées");
        assert!(
            !Camera::suit_en_hauteur(180.0),
            "pile la hauteur de l'écran"
        );
        assert!(Camera::suit_en_hauteur(192.0), "12 rangées");
        // Au départ, en bas d'un niveau haut : la caméra est sur sa borne basse.
        let g = tour();
        let j = pose(&g, 150.0, 624.0);
        let c = Camera::new(&j, L_TOUR, H_TOUR);
        assert_eq!(c.y, H_TOUR - H_ECRAN);
        assert!(c.en_hauteur);
    }

    #[test]
    fn un_saut_sur_place_ne_fait_pas_bouger_la_camera() {
        let g = tour();
        let env = Environnement::simple(&g);
        let mut j = pose(&g, 80.0, 320.0);
        let mut c = Camera::new(&j, L_TOUR, H_TOUR);
        let depart = c.y.to_bits();
        // Cadrage « posé » : centre de Pépin à 0,58 de l'écran.
        assert!((centre_ecran(&j, &c) - H_ECRAN * PART_Y).abs() < 0.01);
        let mut plus_haut = j.boite.y;
        for pas in 0..90 {
            let saut = Entrees {
                direction: Direction::Immobile,
                saut: pas < 30,
            };
            j.actualiser(&saut, &env);
            c.suivre(&j, L_TOUR, H_TOUR);
            plus_haut = plus_haut.min(j.boite.y);
            assert_eq!(c.y.to_bits(), depart, "pas {pas} : la caméra a bougé");
        }
        assert!(j.au_sol, "retombé sur la corniche");
        assert!(j.boite.y - plus_haut > 50.0, "un vrai saut : {plus_haut}");
    }

    #[test]
    fn se_poser_trois_rangees_plus_haut_recadre_en_douceur() {
        let g = tour();
        let env = Environnement::simple(&g);
        let mut j = pose(&g, 80.0, 320.0);
        let mut c = Camera::new(&j, L_TOUR, H_TOUR);
        // Pépin se retrouve posé sur la corniche de la rangée 17 (48 px plus haut).
        j.placer(220.0, 272.0 - JOUEUR_H);
        let mut avant = c.y;
        for pas in 0..240 {
            j.actualiser(&Entrees::default(), &env);
            c.suivre(&j, L_TOUR, H_TOUR);
            let d = avant - c.y;
            // La caméra MONTE (y diminue), jamais plus de 48 x 0,12 px par pas.
            assert!(
                (-0.001..=48.0 * LISSAGE_Y_HAUT + 0.01).contains(&d),
                "pas {pas} : {d}"
            );
            avant = c.y;
        }
        assert!(j.au_sol);
        assert!((centre_ecran(&j, &c) - H_ECRAN * PART_Y).abs() < 0.1);
    }

    #[test]
    fn une_longue_chute_garde_pepin_a_l_ecran() {
        let g = tour();
        let env = Environnement::simple(&g);
        // En haut du puits de droite (colonnes 18-19), caméra calée en haut du niveau.
        let mut j = Joueur::new(292.0, 4.0);
        let mut c = Camera::new(&j, L_TOUR, H_TOUR);
        assert_eq!(c.y, 0.0);
        let mut avant = c.y;
        for pas in 0..240 {
            j.actualiser(&Entrees::default(), &env);
            c.suivre(&j, L_TOUR, H_TOUR);
            let (haut, bas) = (j.boite.y - c.y, j.boite.bas() - c.y);
            assert!(
                haut >= 0.0 && bas <= H_ECRAN,
                "pas {pas} : Pépin hors de l'écran ({haut}..{bas})"
            );
            assert!(
                c.y >= avant,
                "pas {pas} : la caméra remonte pendant la chute"
            );
            avant = c.y;
        }
        assert!(j.au_sol && j.boite.bas() == 624.0, "posé sur le sol du bas");
        // Posé en bas : la caméra finit sur sa borne basse.
        assert!((c.y - (H_TOUR - H_ECRAN)).abs() < 0.01, "y = {}", c.y);
    }

    #[test]
    fn parapluie_ouvert_on_voit_en_dessous() {
        let g = tour();
        let env = Environnement::simple(&g);
        // Même puits, d'abord en tombant, puis en planant : on mesure où se tient Pépin à
        // l'écran une fois le régime établi (après 90 pas).
        let regime = |saut: bool| {
            let mut j = Joueur::new(292.0, 4.0);
            let mut c = Camera::new(&j, L_TOUR, H_TOUR);
            let mut centres = Vec::new();
            for pas in 0..150 {
                let e = Entrees {
                    direction: Direction::Immobile,
                    saut,
                };
                j.actualiser(&e, &env);
                c.suivre(&j, L_TOUR, H_TOUR);
                if pas >= 90 && !j.au_sol {
                    centres.push(centre_ecran(&j, &c));
                }
            }
            assert!(!centres.is_empty());
            centres.into_iter().fold(0.0_f32, f32::max)
        };
        let plane = regime(true);
        let chute = regime(false);
        // En planant (0,6 px par pas), la caméra traîne de 5 px au plus derrière le bas de
        // la zone : au moins 85 px de vue sous Pépin.
        assert!(plane <= H_ECRAN * ZONE_BAS_PLANE + 6.0, "plané : {plane}");
        assert!(
            chute > plane,
            "en chute, Pépin est plus bas à l'écran : {chute}"
        );
    }

    #[test]
    fn au_dessus_du_niveau_la_camera_reste_en_haut() {
        let g = tour();
        let env = Environnement::simple(&g);
        let mut j = pose(&g, 80.0, 320.0);
        let mut c = Camera::new(&j, L_TOUR, H_TOUR);
        // Lancé très haut (comme par un champignon) : 140 px au-dessus du haut du niveau.
        j.placer(80.0, -140.0);
        for _ in 0..60 {
            c.suivre(&j, L_TOUR, H_TOUR);
            j.placer(80.0, -140.0);
            j.actualiser(&Entrees::default(), &env);
        }
        assert_eq!(c.y, 0.0, "borne haute du niveau");
        // Pépin est au-dessus du cadre : la scène montre la flèche de repère.
        let (_, cy) = c.position_rendue(1.0, (j.boite.x, j.boite.y));
        assert!(j.boite.bas() as i32 - cy <= 0);
    }

    /// Même vérification que pour la course, en hauteur : Pépin descend à vitesse
    /// constante (pas de 0,x px), la caméra le suit. Sa position à l'écran ne change de
    /// sens qu'aux mêmes endroits que l'écart réel Pépin-caméra (jamais à cause de
    /// l'arrondi), reste à moins d'un demi-pixel de lui, et le décor (la caméra entière)
    /// ne remonte jamais.
    #[test]
    fn en_hauteur_pepin_ne_tremble_pas_a_l_ecran() {
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
        for pas_px in [0.6_f32, 1.3, 2.7, 6.0] {
            let mut j = Joueur::new(150.0, 10.0);
            let mut c = Camera::new(&j, L_TOUR, H_TOUR);
            let (mut reel, mut ecran, mut cams) = (Vec::new(), Vec::new(), Vec::new());
            // Assez de pas pour descendre de 500 px : sortie de la zone morte, suivi établi.
            for k in 1..=(500.0 / pas_px) as usize {
                j.placer(150.0, 10.0 + k as f32 * pas_px);
                j.vitesse.y = pas_px * 60.0;
                c.suivre(&j, L_TOUR, H_TOUR);
                let (_, cy) = c.position_rendue(1.0, (j.boite.x, j.boite.y));
                reel.push(j.boite.y - c.y);
                ecran.push((j.boite.y.round() as i32 - cy) as f32);
                cams.push(cy);
            }
            // Le décor ne recule jamais une fois le suivi lancé (caméra entière à 3 px ou
            // plus) ; au tout début, quand la caméra ne bouge encore que de quelques
            // dixièmes de pixel par pas, l'écart arrondi une seule fois peut la faire
            // reculer d'UN pixel (même règle que l'horizontale : on préfère un Pépin
            // immobile à l'écran).
            assert!(
                cams.windows(2).all(|w| w[1] >= w[0] - 1),
                "pas {pas_px} : {cams:?}"
            );
            let lance = cams.iter().position(|&c| c >= 3).unwrap();
            assert!(
                cams[lance..].windows(2).all(|w| w[1] >= w[0]),
                "pas {pas_px} : {cams:?}"
            );
            assert!(
                retours(&ecran) <= retours(&reel),
                "pas {pas_px} : {ecran:?}"
            );
            for (r, e) in reel.iter().zip(&ecran) {
                assert!((r - e).abs() <= 0.5 + 1e-3, "réel {r} / écran {e}");
            }
        }
    }
}
