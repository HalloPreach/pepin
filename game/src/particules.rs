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

#[derive(Debug, Clone, Copy)]
struct Particule {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    /// Pas de vie restants.
    vie: u8,
    couleur: Couleur,
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
                couleur,
            });
        }
    }

    /// Un pas de simulation.
    pub fn actualiser(&mut self) {
        for p in &mut self.liste {
            p.vy += GRAVITE;
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
            draw_rectangle(x as f32, y as f32, 1.0, 1.0, couleur_mq(p.couleur));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::palette::PLUIE;

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
