//! Screen shake : une très légère secousse de la caméra (1 à 2 pixels) à la mort.
//!
//! Le décalage est toujours en pixels ENTIERS (pas de sous-pixel) et se calcule
//! sans hasard, d'après le nombre de pas restants : c'est reproductible et testable.
//! La secousse s'atténue linéairement jusqu'à zéro. Elle peut être désactivée
//! (option « réduire les effets » pour les joueurs sensibles au mouvement).

/// Une secousse en cours.
#[derive(Debug, Clone, Copy)]
pub struct Secousse {
    /// Désactivée = ne bouge jamais.
    actif: bool,
    /// Pas restants.
    restant: u8,
    /// Durée totale de la secousse en cours (pour l'atténuation).
    duree: u8,
    /// Amplitude maximale en pixels.
    force: u8,
}

impl Secousse {
    /// `actif` à faux : toutes les demandes de secousse sont ignorées.
    pub fn new(actif: bool) -> Self {
        Self {
            actif,
            restant: 0,
            duree: 1,
            force: 0,
        }
    }

    /// Lance une secousse de `force` pixels pendant `duree` pas.
    pub fn declencher(&mut self, force: u8, duree: u8) {
        if self.actif && duree > 0 {
            self.restant = duree;
            self.duree = duree;
            self.force = force;
        }
    }

    /// Un pas de simulation.
    pub fn actualiser(&mut self) {
        self.restant = self.restant.saturating_sub(1);
    }

    /// Décalage `(dx, dy)` entier à ajouter à la caméra.
    pub fn decalage(&self) -> (i32, i32) {
        if self.restant == 0 {
            return (0, 0);
        }
        // Amplitude qui décroît avec le temps restant (arrondie vers le haut).
        let a = (self.force as u32 * self.restant as u32).div_ceil(self.duree as u32) as i32;
        let dx = if self.restant.is_multiple_of(2) {
            a
        } else {
            -a
        };
        let dy = if self.restant % 4 < 2 {
            a / 2
        } else {
            -(a / 2)
        };
        (dx, dy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn au_repos_aucun_decalage() {
        assert_eq!(Secousse::new(true).decalage(), (0, 0));
    }

    #[test]
    fn la_secousse_reste_dans_la_force_puis_s_arrete() {
        let mut s = Secousse::new(true);
        s.declencher(2, 12);
        let mut a_bouge = false;
        for _ in 0..12 {
            let (dx, dy) = s.decalage();
            assert!(dx.abs() <= 2 && dy.abs() <= 2);
            a_bouge |= (dx, dy) != (0, 0);
            s.actualiser();
        }
        assert!(a_bouge);
        assert_eq!(s.decalage(), (0, 0), "terminée après la durée");
    }

    #[test]
    fn la_secousse_s_attenue() {
        let mut s = Secousse::new(true);
        s.declencher(4, 20);
        let debut = s.decalage().0.abs();
        for _ in 0..16 {
            s.actualiser();
        }
        assert!(s.decalage().0.abs() < debut);
    }

    #[test]
    fn desactivee_elle_ne_bouge_jamais() {
        let mut s = Secousse::new(false);
        s.declencher(3, 10);
        assert_eq!(s.decalage(), (0, 0));
    }
}
