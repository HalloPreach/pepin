//! Particules d'éclaboussures : de minuscules carrés de 1x1 pixel logique qui
//! giclent à l'atterrissage, au saut, quand on ramasse une goutte d'or, etc.
//!
//! Tout est déterministe (petit générateur congruentiel linéaire, pas de
//! hasard système) et avance à pas fixe, comme la physique. Le dessin arrondit
//! les positions à l'entier : jamais de sous-pixel.
//!
//! Notions Rust illustrées ici : `Vec::retain` (supprimer ce qui est mort),
//! `u32` avec `wrapping_mul` (arithmétique qui boucle au lieu de paniquer).

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::Couleur;

use crate::rendu::couleur_mq;

/// Nombre maximal de particules vivantes (évite toute explosion mémoire).
const MAX: usize = 256;
/// Gravité des particules, en pixels par pas au carré.
const GRAVITE: f32 = 0.12;
/// Vitesse de montée d'un brin de vent, en pixels par pas.
const VITESSE_SOUFFLE: f32 = 1.2;
/// Vitesse horizontale d'un brin de vent latéral, en pixels par pas.
const VITESSE_LATERALE: f32 = 1.8;
/// Longueur (px) d'une traînée de vent latéral (carte 5.15 : 3 px avant, trop
/// courts pour se voir sur un ciel clair).
pub const LONGUEUR_TRAINEE: u8 = 8;
/// Longueur (px) de la TÊTE d'une traînée, à l'avant (du côté où souffle le vent),
/// dans une couleur plus claire que le reste : sur une image fixe, la tête claire
/// et la queue sombre disent le sens du vent, comme une comète.
pub const LONGUEUR_TETE: u8 = 2;

#[derive(Debug, Clone, Copy)]
struct Particule {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    /// Pas de vie restants.
    vie: u8,
    /// Gravité propre à la particule (négative : elle monte, comme la fumée).
    gravite: f32,
    couleur: Couleur,
    /// Hauteur du trait dessiné (1 = un point ; 3 = un tiret vertical, pour le vent).
    hauteur: u8,
    /// Largeur du trait dessiné (1 = un point ; `LONGUEUR_TRAINEE` = une traînée
    /// horizontale, pour le vent latéral).
    largeur: u8,
    /// Couleur de la tête d'une traînée de vent latéral (ses `LONGUEUR_TETE` px de
    /// devant, du côté où elle avance) ; `None` : un trait d'une seule couleur.
    tete: Option<Couleur>,
}

/// L'ensemble des particules vivantes.
#[derive(Debug, Clone, Default)]
pub struct Particules {
    liste: Vec<Particule>,
    graine: u32,
}

impl Particules {
    pub fn new() -> Self {
        Self {
            liste: Vec::new(),
            graine: 0x1234_5678,
        }
    }

    /// Nombre de particules vivantes.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.liste.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.liste.is_empty()
    }

    /// Position `(x, y)` de chaque brin de vent ascendant vivant (tiret vertical
    /// de 3 px), pour les tests des effets.
    #[cfg(test)]
    pub fn brins_de_souffle(&self) -> Vec<(f32, f32)> {
        self.liste
            .iter()
            .filter(|p| p.hauteur == 3 && p.largeur == 1)
            .map(|p| (p.x, p.y))
            .collect()
    }

    /// Nombre pseudo-aléatoire entre 0.0 et 1.0.
    fn hasard(&mut self) -> f32 {
        self.graine = self
            .graine
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.graine >> 8) & 0xFFFF) as f32 / 65535.0
    }

    /// Fait gicler `n` particules de `couleur` depuis le point `(x, y)` (niveau).
    /// `force` règle la hauteur du jet.
    pub fn eclabousser(&mut self, x: f32, y: f32, n: usize, couleur: Couleur, force: f32) {
        for _ in 0..n {
            if self.liste.len() >= MAX {
                return;
            }
            let angle = self.hasard() * 2.0 - 1.0; // -1 (gauche) .. 1 (droite)
            let haut = 0.4 + self.hasard() * 0.9;
            let vie = 14 + (self.hasard() * 14.0) as u8;
            self.liste.push(Particule {
                x,
                y,
                vx: angle * 1.1,
                vy: -haut * force,
                vie,
                gravite: GRAVITE,
                couleur,
                hauteur: 1,
                largeur: 1,
                tete: None,
            });
        }
    }

    /// Petits nuages de poussière qui dérivent du côté `sens` (-1.0 ou 1.0) :
    /// presque pas de gravité, vie un peu plus longue que les éclaboussures.
    pub fn poussiere(&mut self, x: f32, y: f32, n: usize, couleur: Couleur, sens: f32) {
        for _ in 0..n {
            if self.liste.len() >= MAX {
                return;
            }
            let vx = sens * (0.15 + self.hasard() * 0.45);
            let vy = -(0.05 + self.hasard() * 0.3);
            let vie = 12 + (self.hasard() * 12.0) as u8;
            self.liste.push(Particule {
                x,
                y,
                vx,
                vy,
                vie,
                gravite: 0.004,
                couleur,
                hauteur: 1,
                largeur: 1,
                tete: None,
            });
        }
    }

    /// Étincelles qui partent dans toutes les directions et retombent doucement.
    pub fn etincelles(&mut self, x: f32, y: f32, n: usize, couleur: Couleur) {
        for _ in 0..n {
            if self.liste.len() >= MAX {
                return;
            }
            let vx = (self.hasard() * 2.0 - 1.0) * 1.3;
            let vy = (self.hasard() * 2.0 - 1.0) * 1.3 - 0.3;
            let vie = 10 + (self.hasard() * 10.0) as u8;
            self.liste.push(Particule {
                x,
                y,
                vx,
                vy,
                vie,
                gravite: 0.03,
                couleur,
                hauteur: 1,
                largeur: 1,
                tete: None,
            });
        }
    }

    /// Un brin de vent qui monte dans un courant ascendant (colonne de vent visible) : un tiret
    /// vertical de 3 px. Il naît à une hauteur tirée au hasard entre `haut` (bord haut de la zone)
    /// et `bas` (bord bas), et disparaît au plus tard en atteignant le bord haut : le vent se voit
    /// sur toute la colonne sans déborder de la zone où il agit.
    pub fn souffle(&mut self, x: f32, haut: f32, bas: f32, couleur: Couleur) {
        self.souffle_borne(x, haut, bas, couleur, u8::MAX);
    }

    /// Comme `souffle`, mais le brin vit au plus `vie_max` pas : un vent qui va
    /// s'arrêter (celui d'un ventilateur) n'en laisse aucun lui survivre. Avec
    /// `vie_max = u8::MAX`, c'est exactement `souffle` (mêmes tirages, même vie).
    pub fn souffle_borne(&mut self, x: f32, haut: f32, bas: f32, couleur: Couleur, vie_max: u8) {
        if self.liste.len() >= MAX {
            return;
        }
        let vx = (self.hasard() * 2.0 - 1.0) * 0.15;
        let y = bas - 2.0 - self.hasard() * (bas - haut - 4.0).max(0.0);
        let restant = ((y - haut) / VITESSE_SOUFFLE).floor().max(1.0);
        let vie = ((24.0 + (self.hasard() * 16.0).floor())
            .min(restant)
            .min(255.0) as u8)
            .min(vie_max)
            .max(1);
        self.liste.push(Particule {
            x,
            y,
            vx,
            vy: -VITESSE_SOUFFLE,
            vie,
            gravite: 0.0,
            couleur,
            hauteur: 3,
            largeur: 1,
            tete: None,
        });
    }

    /// Une traînée de vent latéral qui traverse une zone de vent de `sens` (-1 vers
    /// la gauche, 1 vers la droite) : un trait horizontal de `LONGUEUR_TRAINEE` px,
    /// sa tête (à l'avant) en `tete`, le reste en `traine`. Elle naît à une place
    /// et une hauteur tirées au hasard dans la zone, et s'efface avant d'en sortir :
    /// le vent se voit sur toute la zone sans déborder.
    ///
    /// Les couleurs viennent de la table d'ambiance (`decor::ambiance`, champ
    /// `vent`) : contrastées sur le ciel du niveau (carte 5.15).
    #[allow(clippy::too_many_arguments)]
    pub fn souffle_lateral(
        &mut self,
        x_min: f32,
        x_max: f32,
        haut: f32,
        bas: f32,
        sens: i8,
        traine: Couleur,
        tete: Couleur,
    ) {
        if self.liste.len() >= MAX {
            return;
        }
        let longueur = f32::from(LONGUEUR_TRAINEE);
        // Place du bord gauche du trait : de `x_min` à `x_max - longueur`.
        let course = (x_max - x_min - longueur).max(0.0);
        let x = x_min + self.hasard() * course;
        let y = haut + 1.0 + self.hasard() * (bas - haut - 2.0).max(0.0);
        // Vie bornée par le bord opposé de la zone (la traînée s'efface avant de sortir).
        let reste = if sens > 0 {
            (x_max - longueur - x) / VITESSE_LATERALE
        } else {
            (x - x_min) / VITESSE_LATERALE
        };
        let vie = (18.0 + (self.hasard() * 18.0).floor())
            .min(reste.floor().max(1.0))
            .min(255.0) as u8;
        self.liste.push(Particule {
            x,
            y,
            vx: sens as f32 * VITESSE_LATERALE,
            vy: 0.0,
            vie,
            gravite: 0.0,
            couleur: traine,
            hauteur: 1,
            largeur: LONGUEUR_TRAINEE,
            tete: Some(tete),
        });
    }

    /// Un pas de simulation.
    pub fn actualiser(&mut self) {
        for p in &mut self.liste {
            p.vy += p.gravite;
            p.x += p.vx;
            p.y += p.vy;
            p.vie -= 1;
        }
        self.liste.retain(|p| p.vie > 0);
    }

    /// Dessine les particules (coordonnées niveau) avec la caméra entière `(cx, cy)`.
    pub fn dessiner(&self, cx: i32, cy: i32) {
        for p in &self.liste {
            let (x, y) = (p.x.round() as i32 - cx, p.y.round() as i32 - cy);
            draw_rectangle(
                x as f32,
                y as f32,
                p.largeur as f32,
                p.hauteur as f32,
                couleur_mq(p.couleur),
            );
            // Traînée de vent : la tête par-dessus, à l'avant.
            if let Some(tete) = p.tete {
                let x_tete = x + decalage_tete(p.vx, p.largeur);
                draw_rectangle(
                    x_tete as f32,
                    y as f32,
                    f32::from(LONGUEUR_TETE),
                    p.hauteur as f32,
                    couleur_mq(tete),
                );
            }
        }
    }

    /// Les traînées de vent latéral vivantes : `(x, y, vx, couleur, tête)`, pour
    /// les tests des effets.
    #[cfg(test)]
    pub fn trainees(&self) -> Vec<(f32, f32, f32, Couleur, Couleur)> {
        self.liste
            .iter()
            .filter_map(|p| p.tete.map(|t| (p.x, p.y, p.vx, p.couleur, t)))
            .collect()
    }
}

/// Décalage (px) de la tête d'une traînée de `largeur` px depuis son bord gauche :
/// à droite si elle avance vers la droite (`vx > 0`), à gauche sinon. Pure.
fn decalage_tete(vx: f32, largeur: u8) -> i32 {
    if vx > 0.0 {
        i32::from(largeur.saturating_sub(LONGUEUR_TETE))
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::palette::{OR_PALE, PLUIE};

    #[test]
    fn le_souffle_lateral_traverse_sa_zone() {
        // Zone de 60 px de large (x 10..70), dans les deux sens : chaque traînée
        // (tout son trait, tête comprise) reste dans la zone pendant toute sa vie,
        // avance dans le sens du vent sans monter ni descendre, et meurt au bord.
        let longueur = f32::from(LONGUEUR_TRAINEE);
        for sens in [1i8, -1] {
            let mut p = Particules::new();
            for _ in 0..40 {
                p.souffle_lateral(10.0, 70.0, 0.0, 48.0, sens, PLUIE, OR_PALE);
            }
            assert_eq!(p.len(), 40);
            let depart = p.trainees();
            for (x, y, vx, couleur, tete) in &depart {
                assert!(*x >= 10.0 && x + longueur <= 70.0, "naît hors zone : {x}");
                assert!((1.0..=47.0).contains(y));
                assert_eq!(vx.signum(), f32::from(sens));
                assert_eq!((*couleur, *tete), (PLUIE, OR_PALE));
            }
            for _ in 0..60 {
                p.actualiser();
                for &(x, y, _, _, _) in &p.trainees() {
                    assert!(
                        x >= 10.0 - 0.01 && x + longueur <= 70.0 + 0.01,
                        "sens {sens} : la traînée déborde : {x}"
                    );
                    assert!(
                        depart.iter().any(|d| d.1 == y),
                        "elle ne change pas de hauteur"
                    );
                }
            }
            // Toutes les traînées meurent avant de sortir de la zone (vie bornée).
            assert!(p.is_empty(), "sens {sens}");
        }
    }

    /// La tête (couleur claire) est à l'AVANT de la traînée : à droite quand le
    /// vent souffle vers la droite, à gauche sinon. C'est ce qui donne le sens du
    /// vent sur une image fixe.
    #[test]
    fn la_tete_de_la_trainee_est_a_l_avant() {
        assert_eq!(
            decalage_tete(VITESSE_LATERALE, LONGUEUR_TRAINEE),
            i32::from(LONGUEUR_TRAINEE - LONGUEUR_TETE)
        );
        assert_eq!(decalage_tete(-VITESSE_LATERALE, LONGUEUR_TRAINEE), 0);
        // Une queue au moins deux fois plus longue que la tête : le trait se lit.
        assert!(LONGUEUR_TRAINEE >= 3 * LONGUEUR_TETE);
    }

    #[test]
    fn les_particules_naissent_puis_meurent() {
        let mut p = Particules::new();
        p.eclabousser(10.0, 10.0, 8, PLUIE, 2.0);
        assert_eq!(p.len(), 8);
        for _ in 0..40 {
            p.actualiser();
        }
        assert!(p.is_empty(), "toutes les particules ont une vie bornée");
    }

    #[test]
    fn poussiere_etincelles_et_souffle_ont_une_vie_bornee() {
        let mut p = Particules::new();
        p.poussiere(0.0, 0.0, 5, PLUIE, 1.0);
        p.etincelles(0.0, 0.0, 5, PLUIE);
        p.souffle(0.0, -48.0, 0.0, PLUIE);
        assert_eq!(p.len(), 11);
        for _ in 0..60 {
            p.actualiser();
        }
        assert!(p.is_empty());
    }

    #[test]
    fn le_souffle_borne_ne_vit_pas_plus_que_demande() {
        // Même tirage que `souffle` : avec u8::MAX, la vie est identique.
        let (mut a, mut b) = (Particules::new(), Particules::new());
        a.souffle(0.0, -96.0, 0.0, PLUIE);
        b.souffle_borne(0.0, -96.0, 0.0, PLUIE, u8::MAX);
        assert_eq!(a.liste[0].vie, b.liste[0].vie);
        assert_eq!(a.liste[0].y, b.liste[0].y);
        // Vie bornée à 5 : le brin est effacé au 5e pas au plus tard.
        let mut p = Particules::new();
        p.souffle_borne(0.0, -96.0, 0.0, PLUIE, 5);
        assert!((1..=5).contains(&p.liste[0].vie));
        for _ in 0..5 {
            p.actualiser();
        }
        assert!(p.is_empty(), "le brin borné à 5 pas ne survit pas au 5e");
        // Vie 0 demandée : 1 au minimum (il s'efface au premier pas).
        p.souffle_borne(0.0, -96.0, 0.0, PLUIE, 0);
        p.actualiser();
        assert!(p.is_empty());
    }

    #[test]
    fn la_poussiere_derive_du_cote_demande() {
        let mut p = Particules::new();
        p.poussiere(100.0, 0.0, 6, PLUIE, -1.0);
        for _ in 0..8 {
            p.actualiser();
        }
        assert!(p.liste.iter().all(|q| q.x < 100.0));
    }

    #[test]
    fn le_souffle_monte() {
        let mut p = Particules::new();
        p.souffle(0.0, -1000.0, 100.0, PLUIE);
        let y0 = p.liste[0].y;
        for _ in 0..10 {
            p.actualiser();
        }
        assert!(p.liste[0].y < y0 - 8.0);
    }

    #[test]
    fn le_souffle_couvre_toute_la_zone_et_n_en_sort_pas() {
        // Zone de 96 px de haut (y de 100 à 196) : les brins naissent à toutes les hauteurs,
        // sont des tirets de 3 px, et ne dépassent jamais le bord haut de la zone.
        let mut p = Particules::new();
        let (haut, bas) = (100.0_f32, 196.0_f32);
        let mut naissances = Vec::new();
        for _ in 0..200 {
            p.souffle(10.0, haut, bas, PLUIE);
            naissances.push(p.liste.last().unwrap().y);
        }
        let (min, max) = naissances
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &y| (a.min(y), b.max(y)));
        assert!(max <= bas - 2.0 && max > bas - 20.0, "max = {max}");
        assert!(min >= haut && min < haut + 30.0, "min = {min}");
        assert!(p.liste.iter().all(|q| q.hauteur == 3));
        let mut plus_haut = f32::MAX;
        for _ in 0..60 {
            p.actualiser();
            for q in &p.liste {
                plus_haut = plus_haut.min(q.y);
            }
        }
        assert!(
            plus_haut >= haut - VITESSE_SOUFFLE,
            "sorti de la zone : {plus_haut}"
        );
        assert!(p.is_empty());
    }

    #[test]
    fn les_autres_particules_restent_des_points() {
        let mut p = Particules::new();
        p.eclabousser(0.0, 0.0, 3, PLUIE, 1.0);
        p.poussiere(0.0, 0.0, 3, PLUIE, 1.0);
        p.etincelles(0.0, 0.0, 3, PLUIE);
        assert!(p.liste.iter().all(|q| q.hauteur == 1));
    }

    #[test]
    fn le_nombre_de_particules_est_plafonne() {
        let mut p = Particules::new();
        for _ in 0..100 {
            p.eclabousser(0.0, 0.0, 50, PLUIE, 2.0);
        }
        assert_eq!(p.len(), MAX);
    }

    #[test]
    fn le_jet_est_deterministe() {
        let (mut a, mut b) = (Particules::new(), Particules::new());
        a.eclabousser(1.0, 2.0, 5, PLUIE, 2.0);
        b.eclabousser(1.0, 2.0, 5, PLUIE, 2.0);
        a.actualiser();
        b.actualiser();
        let pos = |p: &Particules| p.liste.iter().map(|q| (q.x, q.y)).collect::<Vec<_>>();
        assert_eq!(pos(&a), pos(&b));
    }
}
