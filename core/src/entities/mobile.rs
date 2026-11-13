//! Plateformes mobiles : elles emportent le joueur. Deux trajets possibles :
//! - aller-retour en ligne droite entre deux points (chiffres `1` à `9` du niveau) ;
//! - tour d'horloge autour d'un pivot (lettre `h`) : c'est l'AIGUILLE d'une
//!   horloge. La plateforme suit un cercle dans le sens des aiguilles d'une
//!   montre, mais elle ne tourne pas sur elle-même : sa boîte reste un rectangle
//!   droit, donc son dessus reste HORIZONTAL et on s'y tient debout (comme la
//!   nacelle d'une grande roue).
//!
//! ```text
//!              midi (tour = 0)
//!                 ___
//!            ·         ·
//!   9 h    ·      +      ·   3 h (tour = 1/4)       + : le pivot `h`
//!  (3/4)    ·           ·                            ___ : la plateforme
//!              ·  ___  ·                             sens : midi -> 3 h -> 6 h -> 9 h
//!               6 h (1/2)
//! ```
//!
//! Notions Rust illustrées ici : un `enum` privé (`Trajet`) qui cache le genre
//! de trajet derrière la même structure publique ; `Option` pour une
//! information qui n'existe que pour un des genres (`extremites`, `cercle`) ;
//! une fonction PURE (`cos_sin_tour`) testée contre la bibliothèque standard.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, Vec2, DT};

/// Le trajet d'une plateforme mobile.
#[derive(Debug, Clone)]
enum Trajet {
    /// Allers-retours entre deux points (coin haut-gauche de la boîte).
    AllerRetour {
        a: (f32, f32),
        b: (f32, f32),
        /// Va-t-elle vers `b` (sinon vers `a`) ?
        vers_b: bool,
    },
    /// Tour d'horloge autour d'un pivot.
    Cercle(Cercle),
}

/// Le cercle d'une aiguille d'horloge : la plateforme a son CENTRE sur ce
/// cercle. Les positions se donnent en fraction de tour (`tour`) : 0 = midi (en
/// haut), 1/4 = 3 h (à droite), 1/2 = 6 h (en bas), 3/4 = 9 h (à gauche).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cercle {
    /// Centre du pivot (px).
    pub pivot: (f32, f32),
    /// Rayon (px) : du centre du pivot au centre de la plateforme.
    pub rayon: f32,
    /// Fraction de tour parcourue à chaque pas (sens horaire), toujours > 0.
    pas_tour: f32,
    /// Position actuelle, en fraction de tour, dans [0, 1).
    tour: f32,
    /// Coin haut-gauche de la plateforme AVANT le dernier pas : là où Pépin
    /// avait les pieds (voir `PlateformeMobile::boite_avant`).
    avant: (f32, f32),
}

impl Cercle {
    /// Position actuelle (fraction de tour dans [0, 1), 0 = midi).
    pub fn tour(&self) -> f32 {
        self.tour
    }

    /// Nombre de pas (1/60 s) d'un tour complet.
    pub fn pas_par_tour(&self) -> f32 {
        1.0 / self.pas_tour
    }

    /// Centre de la plateforme quand elle est à la fraction de tour `tour`.
    /// Sens horaire, axe Y vers le bas : midi est AU-DESSUS du pivot (y plus petit).
    pub fn centre_a(&self, tour: f32) -> (f32, f32) {
        let (c, s) = cos_sin_tour(tour);
        (self.pivot.0 + self.rayon * s, self.pivot.1 - self.rayon * c)
    }
}

/// Cosinus et sinus d'une fraction de tour (`tour = 1` : 360°), SANS `f32::sin`.
///
/// Pourquoi ne pas appeler `f32::sin_cos` ? Le jeu tourne en WebAssembly dans le
/// navigateur, et ses tests tournent en natif. La bibliothèque mathématique n'est
/// pas la même des deux côtés : un sinus peut différer d'un dernier bit. Les
/// opérations de base (+, -, ×, ÷), elles, sont exactement arrondies partout
/// (norme IEEE 754). Calculer le sinus avec elles seules garantit le MÊME résultat
/// au bit près : le robot de QA, qui calcule son parcours en natif, est rejoué
/// image par image dans le navigateur sans le moindre écart.
///
/// Méthode : on ramène l'angle au premier quart de tour (`a` entre 0 et π/2),
/// où les séries de Taylor convergent vite (erreur < 1e-7 avec 6 termes), puis on
/// replace le résultat dans son quart de tour par symétrie.
/// Fonction pure.
pub fn cos_sin_tour(tour: f32) -> (f32, f32) {
    let quarts = (tour - tour.floor()) * 4.0; // dans [0, 4]
    let q = quarts.floor();
    let a = (quarts - q) * std::f32::consts::FRAC_PI_2;
    let a2 = a * a;
    // Forme de Horner : sin a = a (1 - a²/6 (1 - a²/20 (1 - a²/42 (...)))).
    let s = a
        * (1.0
            - a2 / 6.0
                * (1.0 - a2 / 20.0 * (1.0 - a2 / 42.0 * (1.0 - a2 / 72.0 * (1.0 - a2 / 110.0)))));
    let c = 1.0
        - a2 / 2.0
            * (1.0
                - a2 / 12.0
                    * (1.0
                        - a2 / 30.0 * (1.0 - a2 / 56.0 * (1.0 - a2 / 90.0 * (1.0 - a2 / 132.0)))));
    // `q` vaut 4 seulement si l'arrondi a donné `tour - floor = 1.0` : c'est midi.
    match q as u32 % 4 {
        0 => (c, s),
        1 => (-s, c),
        2 => (-c, -s),
        _ => (s, -c),
    }
}

#[derive(Debug, Clone)]
pub struct PlateformeMobile {
    /// Boîte actuelle (solide).
    pub boite: Aabb,
    trajet: Trajet,
    /// Vitesse en px/s (le long du cercle pour une aiguille).
    vitesse: f32,
    /// Déplacement effectué pendant le DERNIER pas (sert à porter le joueur).
    pub deplacement: Vec2,
}

impl PlateformeMobile {
    /// Plateforme en aller-retour : `boite` donne la taille ; elle est placée en
    /// `a` immédiatement.
    pub fn new(boite: Aabb, a: (f32, f32), b: (f32, f32), vitesse: f32) -> Self {
        Self {
            boite: Aabb::new(a.0, a.1, boite.w, boite.h),
            trajet: Trajet::AllerRetour { a, b, vers_b: true },
            vitesse,
            deplacement: Vec2::default(),
        }
    }

    /// Aiguille d'horloge : plateforme de taille `(largeur, hauteur)` dont le
    /// centre tourne autour de `pivot` à `rayon` px, à `vitesse` px/s le long du
    /// cercle, dans le sens horaire. Elle part de la fraction de tour `tour`
    /// (0 = midi). `rayon` et `vitesse` doivent être strictement positifs.
    pub fn aiguille(
        taille: (f32, f32),
        pivot: (f32, f32),
        rayon: f32,
        vitesse: f32,
        tour: f32,
    ) -> Self {
        assert!(
            rayon > 0.0 && vitesse > 0.0,
            "aiguille sans rayon ou immobile"
        );
        let mut cercle = Cercle {
            pivot,
            rayon,
            // Longueur du cercle : 2πr ; on en parcourt `vitesse × DT` par pas.
            pas_tour: vitesse * DT / (std::f32::consts::TAU * rayon),
            tour: tour - tour.floor(),
            avant: (0.0, 0.0),
        };
        let (x, y) = coin(&cercle, cercle.tour, taille);
        cercle.avant = (x, y);
        Self {
            boite: Aabb::new(x, y, taille.0, taille.1),
            trajet: Trajet::Cercle(cercle),
            vitesse,
            deplacement: Vec2::default(),
        }
    }

    /// Les deux extrémités du trajet d'une plateforme en aller-retour (positions
    /// du coin haut-gauche), pour l'analyse de faisabilité ; `None` pour une
    /// aiguille.
    pub fn extremites(&self) -> Option<((f32, f32), (f32, f32))> {
        match self.trajet {
            Trajet::AllerRetour { a, b, .. } => Some((a, b)),
            Trajet::Cercle(_) => None,
        }
    }

    /// Le cercle d'une aiguille d'horloge ; `None` pour une plateforme en aller-retour.
    pub fn cercle(&self) -> Option<&Cercle> {
        match &self.trajet {
            Trajet::Cercle(c) => Some(c),
            Trajet::AllerRetour { .. } => None,
        }
    }

    /// Coin haut-gauche qu'aurait cette aiguille à la fraction de tour `tour`
    /// (`None` pour une plateforme en aller-retour).
    pub fn coin_a(&self, tour: f32) -> Option<(f32, f32)> {
        self.cercle()
            .map(|c| coin(c, tour, (self.boite.w, self.boite.h)))
    }

    /// Vitesse (px/s), le long du trajet.
    pub fn vitesse(&self) -> f32 {
        self.vitesse
    }

    /// Fait avancer une aiguille d'horloge de `pas` pas d'un coup sur son cercle
    /// (en arrière si `pas` est négatif), comme si le temps avait passé : sert au
    /// robot de QA pour essayer un plan avec l'aiguille en avance ou en retard
    /// d'une image. Le déplacement mémorisé est remis à zéro (un saut, pas un
    /// mouvement). Sans effet sur une plateforme en aller-retour.
    pub fn tourner(&mut self, pas: i32) {
        let taille = (self.boite.w, self.boite.h);
        if let Trajet::Cercle(c) = &mut self.trajet {
            let t = c.tour + pas as f32 * c.pas_tour;
            c.tour = t - t.floor();
            let (x, y) = coin(c, c.tour, taille);
            c.avant = (x, y);
            self.boite.x = x;
            self.boite.y = y;
            self.deplacement = Vec2::default();
        }
    }

    /// Boîte de la plateforme AVANT le dernier pas : c'est sur elle que Pépin
    /// avait les pieds (`contacts::porter_par_plateformes`).
    /// - Aller-retour : recalculée en reculant du dernier déplacement (calcul
    ///   d'origine, gardé tel quel : les niveaux existants ne bougent pas d'un bit).
    /// - Aiguille : la place exacte, retenue au pas précédent. Reculer du
    ///   déplacement pourrait tomber un cheveu à côté (arrondi de flottant) et
    ///   Pépin, les pieds pile sur l'ancienne place, ne serait plus « posé dessus ».
    pub fn boite_avant(&self) -> Aabb {
        match &self.trajet {
            Trajet::AllerRetour { .. } => {
                self.boite.decale(-self.deplacement.x, -self.deplacement.y)
            }
            Trajet::Cercle(c) => Aabb::new(c.avant.0, c.avant.1, self.boite.w, self.boite.h),
        }
    }
}

/// Coin haut-gauche d'une boîte de taille `taille` centrée sur le cercle à `tour`.
fn coin(cercle: &Cercle, tour: f32, taille: (f32, f32)) -> (f32, f32) {
    let (cx, cy) = cercle.centre_a(tour);
    (cx - taille.0 / 2.0, cy - taille.1 / 2.0)
}

impl Actualisable for PlateformeMobile {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        let taille = (self.boite.w, self.boite.h);
        let avant = (self.boite.x, self.boite.y);
        // Chaque branche donne le déplacement de ce pas ET la nouvelle place.
        let (deplacement, apres) = match &mut self.trajet {
            Trajet::AllerRetour { a, b, vers_b } => {
                let (cx, cy) = if *vers_b { *b } else { *a };
                let (dx, dy) = (cx - avant.0, cy - avant.1);
                let distance = (dx * dx + dy * dy).sqrt();
                let pas = self.vitesse * DT;
                let (mx, my) = if distance <= pas {
                    // Arrivée : on se pose pile sur la cible et on repart dans l'autre sens.
                    *vers_b = !*vers_b;
                    (dx, dy)
                } else {
                    (dx / distance * pas, dy / distance * pas)
                };
                ((mx, my), (avant.0 + mx, avant.1 + my))
            }
            Trajet::Cercle(c) => {
                // Un cran de plus sur le cadran ; après un tour complet on repart de 0.
                // La place se recalcule depuis le tour (et non en ajoutant des
                // petits déplacements) : aucune erreur ne s'accumule, la plateforme
                // reste sur son cercle tour après tour.
                c.tour += c.pas_tour;
                if c.tour >= 1.0 {
                    c.tour -= 1.0;
                }
                c.avant = avant;
                let (x, y) = coin(c, c.tour, taille);
                ((x - avant.0, y - avant.1), (x, y))
            }
        };
        self.boite.x = apres.0;
        self.boite.y = apres.1;
        self.deplacement = Vec2::new(deplacement.0, deplacement.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    #[test]
    fn va_et_vient_entre_les_deux_points() {
        let g = GrilleTuiles::vide(20, 5);
        let env = Environnement::simple(&g);
        let mut p = PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (16.0, 40.0),
            (96.0, 40.0),
            60.0,
        );
        assert_eq!((p.boite.x, p.boite.y), (16.0, 40.0));
        let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
        for _ in 0..600 {
            p.actualiser(&env);
            min_x = min_x.min(p.boite.x);
            max_x = max_x.max(p.boite.x);
            assert_eq!(p.boite.y, 40.0);
        }
        assert!((min_x - 16.0).abs() < 1e-3, "min = {min_x}");
        assert!((max_x - 96.0).abs() < 1e-3, "max = {max_x}");
    }

    #[test]
    fn le_deplacement_memorise_correspond_a_la_vitesse() {
        let g = GrilleTuiles::vide(20, 5);
        let env = Environnement::simple(&g);
        let mut p = PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (0.0, 80.0),
            (0.0, 0.0),
            30.0,
        );
        p.actualiser(&env);
        assert_eq!(p.deplacement.x, 0.0);
        assert!((p.deplacement.y - (-30.0 * DT)).abs() < 1e-4, "monte");
        assert_eq!(p.extremites(), Some(((0.0, 80.0), (0.0, 0.0))));
        assert!(p.cercle().is_none() && p.coin_a(0.5).is_none());
    }

    // --- Aiguilles d'horloge (carte 5.10) ---

    /// Notre sinus « maison » colle à celui de la bibliothèque standard (à 3e-7
    /// près) sur 1000 positions du tour, et donne les valeurs exactes aux quatre
    /// quarts du cadran.
    #[test]
    fn cos_sin_tour_suit_la_bibliotheque_standard() {
        for k in 0..1000 {
            let tour = k as f32 / 1000.0;
            let (c, s) = cos_sin_tour(tour);
            let (s_ref, c_ref) = (f64::from(tour) * std::f64::consts::TAU).sin_cos();
            assert!((f64::from(c) - c_ref).abs() < 3e-7, "cos({tour}) = {c}");
            assert!((f64::from(s) - s_ref).abs() < 3e-7, "sin({tour}) = {s}");
        }
        assert_eq!(cos_sin_tour(0.0), (1.0, 0.0));
        assert_eq!(cos_sin_tour(0.25), (-0.0, 1.0));
        assert_eq!(cos_sin_tour(0.5), (-1.0, -0.0));
        assert_eq!(cos_sin_tour(0.75), (0.0, -1.0));
        // Hors de [0, 1) : on ramène au même tour.
        assert_eq!(cos_sin_tour(1.25), cos_sin_tour(0.25));
        assert_eq!(cos_sin_tour(-0.75), cos_sin_tour(0.25));
    }

    /// Aiguille de 48 px de rayon autour du pivot (200, 100), plateforme 32 x 8.
    fn aiguille(tour: f32) -> PlateformeMobile {
        PlateformeMobile::aiguille((32.0, 8.0), (200.0, 100.0), 48.0, 36.0, tour)
    }

    fn centre(p: &PlateformeMobile) -> (f32, f32) {
        (p.boite.x + p.boite.w / 2.0, p.boite.y + p.boite.h / 2.0)
    }

    /// Les quatre heures du cadran : midi au-dessus du pivot, 3 h à droite, 6 h
    /// en dessous, 9 h à gauche (l'axe Y pointe vers le bas).
    #[test]
    fn aiguille_placee_sur_le_cadran() {
        let pos = |t| centre(&aiguille(t));
        assert_eq!(pos(0.0), (200.0, 52.0));
        assert_eq!(pos(0.25), (248.0, 100.0));
        assert_eq!(pos(0.5), (200.0, 148.0));
        assert_eq!(pos(0.75), (152.0, 100.0));
        // Un tour de plus, c'est la même place.
        assert_eq!(pos(1.0), pos(0.0));
        assert_eq!(aiguille(0.25).coin_a(0.5), Some((184.0, 144.0)));
        assert_eq!(aiguille(0.0).extremites(), None);
    }

    /// Pendant deux tours complets : le centre reste sur le cercle (rayon 48 à
    /// 1e-3 près), la boîte garde sa taille (le dessus reste horizontal), la
    /// vitesse le long du cercle est 36 px/s, le sens est horaire (midi -> 3 h
    /// -> 6 h -> 9 h) et un tour dure 2π × 48 / 36 s.
    #[test]
    fn aiguille_tourne_sur_son_cercle_dans_le_sens_horaire() {
        let g = GrilleTuiles::vide(30, 20);
        let env = Environnement::simple(&g);
        let mut p = aiguille(0.0);
        let pas_par_tour = p.cercle().unwrap().pas_par_tour();
        let attendu = std::f32::consts::TAU * 48.0 / 36.0 * 60.0;
        assert!((pas_par_tour - attendu).abs() < 0.01, "{pas_par_tour}");
        // Premier pas : de midi vers 3 h, donc vers la DROITE.
        p.actualiser(&env);
        assert!(p.deplacement.x > 0.59 && p.deplacement.y.abs() < 0.01);
        let mut quarts = Vec::new();
        let mut precedent = p.cercle().unwrap().tour();
        for _ in 0..(2.0 * pas_par_tour) as u32 {
            p.actualiser(&env);
            let (cx, cy) = centre(&p);
            let r = ((cx - 200.0).powi(2) + (cy - 100.0).powi(2)).sqrt();
            assert!((r - 48.0).abs() < 1e-3, "rayon {r}");
            assert_eq!((p.boite.w, p.boite.h), (32.0, 8.0));
            let v = (p.deplacement.x.powi(2) + p.deplacement.y.powi(2)).sqrt() * 60.0;
            assert!((v - 36.0).abs() < 0.05, "vitesse {v}");
            let tour = p.cercle().unwrap().tour();
            assert!((0.0..1.0).contains(&tour));
            // Le quart du cadran franchi pendant ce pas (0 = midi, 1 = 3 h...).
            let q = (tour * 4.0).floor() as u8;
            if q != (precedent * 4.0).floor() as u8 {
                quarts.push(q);
            }
            precedent = tour;
        }
        assert_eq!(quarts, vec![1, 2, 3, 0, 1, 2, 3, 0], "sens horaire");
    }

    /// La phase de départ : `tour = 0.25` démarre à 3 h, et la plateforme
    /// descend aussitôt (de 3 h vers 6 h, côté droit du cadran).
    #[test]
    fn aiguille_partie_de_trois_heures_descend() {
        let g = GrilleTuiles::vide(30, 20);
        let env = Environnement::simple(&g);
        let mut p = aiguille(1.25);
        assert_eq!(p.cercle().unwrap().tour(), 0.25);
        let b0 = p.boite;
        p.actualiser(&env);
        assert!(p.deplacement.y > 0.59 && p.deplacement.x.abs() < 0.01);
        // La boîte d'avant le pas est retenue EXACTEMENT (pas de recul arrondi).
        assert_eq!(p.boite_avant(), b0);
    }

    /// `tourner` : 30 pas d'un coup donnent (presque) la même place que 30 pas
    /// d'`actualiser` ; en arrière, on revient au départ. Sans effet sur une
    /// plateforme en aller-retour.
    #[test]
    fn tourner_d_un_coup_comme_pas_a_pas() {
        let g = GrilleTuiles::vide(30, 20);
        let env = Environnement::simple(&g);
        let mut pas_a_pas = aiguille(0.0);
        for _ in 0..30 {
            pas_a_pas.actualiser(&env);
        }
        let mut d_un_coup = aiguille(0.0);
        d_un_coup.tourner(30);
        assert!((d_un_coup.boite.x - pas_a_pas.boite.x).abs() < 1e-3);
        assert!((d_un_coup.boite.y - pas_a_pas.boite.y).abs() < 1e-3);
        assert_eq!(d_un_coup.deplacement, Vec2::default());
        assert_eq!(d_un_coup.boite_avant(), d_un_coup.boite);
        d_un_coup.tourner(-30);
        assert!((d_un_coup.boite.x - aiguille(0.0).boite.x).abs() < 1e-3);
        assert!((d_un_coup.boite.y - aiguille(0.0).boite.y).abs() < 1e-3);
        // En retard d'un pas sur midi : juste avant midi, à gauche du pivot.
        let mut avant_midi = aiguille(0.0);
        avant_midi.tourner(-1);
        assert!(avant_midi.cercle().unwrap().tour() > 0.99);
        assert!(avant_midi.boite.x < aiguille(0.0).boite.x);
        let mut p = PlateformeMobile::new(
            Aabb::new(0.0, 0.0, 32.0, 8.0),
            (16.0, 40.0),
            (96.0, 40.0),
            60.0,
        );
        p.tourner(10);
        assert_eq!((p.boite.x, p.boite.y), (16.0, 40.0));
    }
}
