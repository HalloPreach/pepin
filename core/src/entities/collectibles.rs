//! Collectibles et repères : gouttes d'or (à ramasser) et lanternes (checkpoints).

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, JOUEUR_H, JOUEUR_L};

/// Goutte d'or facultative : 3 par niveau, 15 au total.
#[derive(Debug, Clone)]
pub struct GoutteDor {
    /// Numéro de la goutte dans son niveau (0, 1 ou 2).
    pub index: u8,
    pub boite: Aabb,
    ramassee: bool,
}

impl GoutteDor {
    /// Goutte de 8 x 8 px dont le coin haut-gauche est en `(x, y)`.
    pub fn new(index: u8, x: f32, y: f32) -> Self {
        Self {
            index,
            boite: Aabb::new(x, y, 8.0, 8.0),
            ramassee: false,
        }
    }

    pub fn est_ramassee(&self) -> bool {
        self.ramassee
    }

    /// Ramasse la goutte. Renvoie `true` seulement la PREMIÈRE fois : le score
    /// ne peut donc pas être crédité deux fois pour la même goutte.
    pub fn ramasser(&mut self) -> bool {
        let premiere_fois = !self.ramassee;
        self.ramassee = true;
        premiere_fois
    }
}

impl Actualisable for GoutteDor {
    fn actualiser(&mut self, _env: &Environnement<'_>) {}
}

/// Lanterne : une fois allumée, le joueur y réapparaît après une mort.
#[derive(Debug, Clone)]
pub struct Lanterne {
    pub boite: Aabb,
    active: bool,
}

impl Lanterne {
    /// Lanterne de 8 x 16 px dont le coin haut-gauche est en `(x, y)`.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            boite: Aabb::new(x, y, 8.0, 16.0),
            active: false,
        }
    }

    pub fn est_active(&self) -> bool {
        self.active
    }

    /// Allume la lanterne ; `true` si elle vient de s'allumer (pas déjà allumée).
    pub fn activer(&mut self) -> bool {
        let nouvelle = !self.active;
        self.active = true;
        nouvelle
    }

    pub fn eteindre(&mut self) {
        self.active = false;
    }

    /// Position `(x, y)` (coin haut-gauche) où poser le joueur : centré sur la
    /// lanterne, les pieds au niveau de son bas.
    pub fn point_de_reapparition(&self) -> (f32, f32) {
        (
            self.boite.x + (self.boite.w - JOUEUR_L) / 2.0,
            self.boite.bas() - JOUEUR_H,
        )
    }
}

impl Actualisable for Lanterne {
    fn actualiser(&mut self, _env: &Environnement<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_goutte_ne_se_ramasse_qu_une_fois() {
        let mut g = GoutteDor::new(1, 10.0, 10.0);
        assert!(!g.est_ramassee());
        assert!(g.ramasser());
        assert!(g.est_ramassee());
        assert!(!g.ramasser(), "deuxième ramassage : rien de neuf");
    }

    #[test]
    fn la_lanterne_s_allume_une_fois_et_donne_un_point_de_respawn() {
        let mut l = Lanterne::new(100.0, 48.0);
        assert!(!l.est_active());
        assert!(l.activer());
        assert!(!l.activer());
        l.eteindre();
        assert!(!l.est_active());

        let (x, y) = l.point_de_reapparition();
        assert_eq!(x + JOUEUR_L / 2.0, l.boite.centre().0, "centré");
        assert_eq!(y + JOUEUR_H, l.boite.bas(), "pieds au bas de la lanterne");
    }
}
