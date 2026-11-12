//! Champignon-trampoline : rebondit très haut le joueur qui tombe dessus.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::Environnement;

/// Vitesse de rebond (px/s). Hauteur atteinte = v² / 2g = 430² / 1800 ≈ 103 px
/// (≈ 6,4 tuiles), presque le double d'un saut normal (≈ 57 px).
pub const VITESSE_CHAMPIGNON: f32 = 430.0;
/// Pas pendant lesquels le chapeau reste « écrasé » à l'écran après un rebond.
pub const COMPRESSION_PAS: u8 = 8;

#[derive(Debug, Clone)]
pub struct Champignon {
    pub boite: Aabb,
    /// Pas d'animation restants (0 = chapeau normal).
    compression: u8,
}

impl Champignon {
    /// Champignon de 16 x 8 px dont le coin haut-gauche est en `(x, y)`.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            boite: Aabb::new(x, y, 16.0, 8.0),
            compression: 0,
        }
    }

    /// Déclenche l'animation d'écrasement du chapeau.
    pub fn rebondir(&mut self) {
        self.compression = COMPRESSION_PAS;
    }

    /// Le chapeau est-il aplati en ce moment ?
    pub fn est_comprime(&self) -> bool {
        self.compression > 0
    }
}

impl Actualisable for Champignon {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.compression = self.compression.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::physics::hauteur_saut_theorique;

    #[test]
    fn le_rebond_depasse_nettement_un_saut() {
        let h = VITESSE_CHAMPIGNON * VITESSE_CHAMPIGNON / (2.0 * crate::physics::GRAVITE);
        assert!(h > 1.5 * hauteur_saut_theorique(), "h = {h}");
    }

    #[test]
    fn l_animation_se_termine() {
        let g = GrilleTuiles::vide(4, 4);
        let env = Environnement::simple(&g);
        let mut c = Champignon::new(0.0, 0.0);
        assert!(!c.est_comprime());
        c.rebondir();
        assert!(c.est_comprime());
        for _ in 0..COMPRESSION_PAS {
            c.actualiser(&env);
        }
        assert!(!c.est_comprime());
    }
}
