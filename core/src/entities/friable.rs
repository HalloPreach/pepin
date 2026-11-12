//! Plateforme qui s'effrite : solide, puis ~0,5 s après le premier contact elle
//! disparaît, puis elle revient quand plus personne n'est dessus.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::Environnement;

/// Délai entre le contact et la disparition : 0,5 s = 30 pas.
pub const EFFRITEMENT_PAS: u16 = 30;
/// Durée d'absence avant de pouvoir revenir : 3 s = 180 pas.
pub const ABSENCE_PAS: u16 = 180;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatFriable {
    /// Intacte et solide.
    Stable,
    /// Touchée : s'effrite (tremble à l'écran), encore solide, `n` pas avant de tomber.
    Effritement(u16),
    /// Disparue (non solide). `n` pas avant de pouvoir revenir ; `0` = prête à revenir.
    Absente(u16),
}

#[derive(Debug, Clone)]
pub struct PlateformeFriable {
    pub boite: Aabb,
    pub etat: EtatFriable,
}

impl PlateformeFriable {
    pub fn new(boite: Aabb) -> Self {
        Self {
            boite,
            etat: EtatFriable::Stable,
        }
    }

    /// Le joueur se pose dessus. Ne fait rien si elle s'effrite déjà ou est absente :
    /// le compte à rebours ne repart PAS à zéro à chaque pas passé dessus.
    pub fn toucher(&mut self) {
        if self.etat == EtatFriable::Stable {
            self.etat = EtatFriable::Effritement(EFFRITEMENT_PAS);
        }
    }

    /// Boîte solide si la plateforme est là (`Stable` ou `Effritement`), sinon `None`.
    pub fn solide(&self) -> Option<Aabb> {
        match self.etat {
            EtatFriable::Absente(_) => None,
            _ => Some(self.boite),
        }
    }

    /// Revient si le délai est écoulé ET si `occupant` (la boîte du joueur) ne la
    /// chevauche pas : une plateforme qui réapparaîtrait DANS le joueur le coincerait.
    pub fn reapparaitre_si_libre(&mut self, occupant: &Aabb) {
        if self.etat == EtatFriable::Absente(0) && !self.boite.chevauche(occupant) {
            self.etat = EtatFriable::Stable;
        }
    }
}

impl Actualisable for PlateformeFriable {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.etat = match self.etat {
            EtatFriable::Stable => EtatFriable::Stable,
            EtatFriable::Effritement(n) if n <= 1 => EtatFriable::Absente(ABSENCE_PAS),
            EtatFriable::Effritement(n) => EtatFriable::Effritement(n - 1),
            EtatFriable::Absente(n) => EtatFriable::Absente(n.saturating_sub(1)),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    fn plateforme() -> PlateformeFriable {
        PlateformeFriable::new(Aabb::new(32.0, 64.0, 32.0, 8.0))
    }

    fn pas(p: &mut PlateformeFriable, n: u16) {
        let g = GrilleTuiles::vide(4, 4);
        let env = Environnement::simple(&g);
        for _ in 0..n {
            p.actualiser(&env);
        }
    }

    #[test]
    fn reste_stable_sans_contact() {
        let mut p = plateforme();
        pas(&mut p, 500);
        assert_eq!(p.etat, EtatFriable::Stable);
        assert!(p.solide().is_some());
    }

    #[test]
    fn disparait_une_demi_seconde_apres_le_contact() {
        assert_eq!(EFFRITEMENT_PAS, 30);
        let mut p = plateforme();
        p.toucher();
        pas(&mut p, EFFRITEMENT_PAS - 1);
        assert!(p.solide().is_some(), "encore là juste avant 0,5 s");
        pas(&mut p, 1);
        assert!(p.solide().is_none(), "disparue à 0,5 s");
    }

    #[test]
    fn toucher_plusieurs_fois_ne_relance_pas_le_decompte() {
        let mut p = plateforme();
        p.toucher();
        pas(&mut p, 10);
        p.toucher();
        assert_eq!(p.etat, EtatFriable::Effritement(EFFRITEMENT_PAS - 10));
    }

    #[test]
    fn revient_apres_l_absence_si_personne_dessus() {
        let mut p = plateforme();
        p.toucher();
        pas(&mut p, EFFRITEMENT_PAS + ABSENCE_PAS);
        assert_eq!(p.etat, EtatFriable::Absente(0));

        // Un joueur est dans la zone : elle attend.
        let dedans = Aabb::new(40.0, 60.0, 10.0, 14.0);
        p.reapparaitre_si_libre(&dedans);
        assert!(p.solide().is_none());

        // Il est parti : elle revient.
        p.reapparaitre_si_libre(&Aabb::new(200.0, 0.0, 10.0, 14.0));
        assert_eq!(p.etat, EtatFriable::Stable);
    }
}
