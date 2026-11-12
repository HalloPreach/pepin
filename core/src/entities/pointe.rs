//! Pointes : danger immobile, mortel au contact.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::Environnement;

/// Marge (px) retirée sur les côtés et le haut de la zone dangereuse : la boîte
/// dessinée est un peu plus grosse que la zone qui blesse, pour rester équitable.
const MARGE_PARDON: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct Pointe {
    /// Boîte visuelle (une tuile de large, une demi-tuile de haut en général).
    pub boite: Aabb,
}

impl Pointe {
    /// Pointes de 16 x 8 px dont le coin haut-gauche est en `(x, y)`.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            boite: Aabb::new(x, y, 16.0, 8.0),
        }
    }

    /// Zone qui blesse vraiment : la boîte réduite de `MARGE_PARDON` sur les côtés et le haut.
    pub fn zone_danger(&self) -> Aabb {
        Aabb::new(
            self.boite.x + MARGE_PARDON,
            self.boite.y + MARGE_PARDON,
            self.boite.w - 2.0 * MARGE_PARDON,
            self.boite.h - MARGE_PARDON,
        )
    }
}

/// Les pointes n'ont rien à mettre à jour : on implémente quand même le trait
/// (corps vide) pour que `Entite` puisse traiter toutes les variantes pareil.
impl Actualisable for Pointe {
    fn actualiser(&mut self, _env: &Environnement<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_zone_de_danger_est_plus_petite_que_la_boite() {
        let p = Pointe::new(32.0, 40.0);
        let z = p.zone_danger();
        assert!(z.x > p.boite.x && z.droite() < p.boite.droite());
        assert!(z.y > p.boite.y);
        assert_eq!(z.bas(), p.boite.bas(), "colle au sol");
    }
}
