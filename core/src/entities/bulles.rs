//! Bulles qui soulèvent (carte 9.3, lettre `Y`) : une colonne de bulles qui
//! monte, sous l'eau ou au-dessus. Pépin qui y entre est soulevé comme par un
//! courant ascendant : c'en est un, TOUJOURS actif (`CourantAscendant`, la même
//! règle que la lettre `v`). Elle sert aux ascensions de la grotte : on y saute
//! (depuis le sol ou depuis la barque), on y plane, et on en sort de côté.
//!
//! La colonne : les cases `Y` de la grille, assemblées en rectangles (comme les
//! courants `v`), PROLONGÉES vers le bas à travers l'eau qui est juste dessous
//! (« sous l'eau ») : les bulles partent du fond du bassin. La barque, elle,
//! n'est pas soulevée (elle reste à la surface, voir `Joueur::actualiser`) : on
//! saute de la barque dans la colonne.
//!
//! Notions Rust : une structure qui ne change jamais (son mouvement n'est qu'un
//! dessin, calculé au rendu à partir du numéro du pas) et qui PRÊTE sa règle à
//! une autre (`courant` renvoie un `CourantAscendant` par valeur : `Copy`).

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{CourantAscendant, Environnement};

#[derive(Debug, Clone)]
pub struct Bulles {
    /// Colonne des bulles : là où elles soulèvent Pépin.
    pub zone: Aabb,
}

impl Bulles {
    pub fn new(zone: Aabb) -> Self {
        Self { zone }
    }

    /// Le courant ascendant de la colonne (accélération standard).
    pub fn courant(&self) -> CourantAscendant {
        CourantAscendant::new(self.zone)
    }
}

/// La colonne ne bouge pas : corps vide, comme la grêle.
impl Actualisable for Bulles {
    fn actualiser(&mut self, _env: &Environnement<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::player::{Direction, Entrees, Joueur};

    /// La règle centrale : Pépin qui plane dans la colonne MONTE (il tomberait
    /// doucement sans elle).
    #[test]
    fn pepin_qui_plane_dans_les_bulles_monte() {
        let g = GrilleTuiles::vide(10, 20);
        let bulles = Bulles::new(Aabb::new(64.0, 32.0, 16.0, 256.0));
        let courants = [bulles.courant()];
        let env = Environnement {
            courants: &courants,
            ..Environnement::simple(&g)
        };
        let mut j = Joueur::new(67.0, 200.0);
        let planer = Entrees {
            direction: Direction::Immobile,
            saut: true,
        };
        for _ in 0..60 {
            j.actualiser(&planer, &env);
        }
        assert!(j.boite.y < 150.0, "soulevé : y = {}", j.boite.y);
    }
}
