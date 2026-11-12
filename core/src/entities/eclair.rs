//! Éclairs temporisés : repos -> flash d'annonce (~0,8 s) -> frappe -> repos...
//!
//! Notions Rust : une machine à états avec un `enum` (`PhaseEclair`) ; les durées
//! sont des ENTIERS (nombre de pas à 60 Hz) pour un comportement exactement
//! reproductible, sans erreur d'arrondi de flottants.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, HZ};

/// Durée de l'annonce (flash / ombre au sol) : 0,8 s = 48 pas.
pub const ANNONCE_PAS: u16 = (HZ as u16 * 4) / 5;
/// Durée de la frappe dangereuse : 0,2 s = 12 pas.
pub const FRAPPE_PAS: u16 = (HZ as u16) / 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseEclair {
    /// Rien ne se passe, aucun danger.
    Repos,
    /// Le flash prévient le joueur, toujours sans danger.
    Annonce,
    /// L'éclair frappe : dangereux.
    Frappe,
}

#[derive(Debug, Clone)]
pub struct Eclair {
    /// Colonne frappée (aussi la zone du flash d'annonce).
    pub zone: Aabb,
    phase: PhaseEclair,
    /// Pas restants avant de passer à la phase suivante.
    restant: u16,
    /// Durée du repos entre deux coups.
    repos_pas: u16,
}

impl Eclair {
    /// `repos_pas` : pause entre deux coups ; `decalage_pas` : délai avant la
    /// première annonce (sert à désynchroniser plusieurs éclairs d'un même niveau).
    pub fn new(zone: Aabb, repos_pas: u16, decalage_pas: u16) -> Self {
        Self {
            zone,
            phase: PhaseEclair::Repos,
            restant: decalage_pas.max(1),
            repos_pas: repos_pas.max(1),
        }
    }

    pub fn phase(&self) -> PhaseEclair {
        self.phase
    }

    /// Le flash d'annonce est-il visible ?
    pub fn en_annonce(&self) -> bool {
        self.phase == PhaseEclair::Annonce
    }

    /// L'éclair blesse-t-il en ce moment ?
    pub fn frappe(&self) -> bool {
        self.phase == PhaseEclair::Frappe
    }

    /// Avancement de l'annonce entre 0.0 (début) et 1.0 (frappe imminente), pour
    /// faire clignoter plus vite à l'écran. 0.0 hors annonce.
    pub fn progression_annonce(&self) -> f32 {
        if self.en_annonce() {
            1.0 - self.restant as f32 / ANNONCE_PAS as f32
        } else {
            0.0
        }
    }
}

impl Actualisable for Eclair {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.restant = self.restant.saturating_sub(1);
        if self.restant == 0 {
            // Chaque variante désigne la phase suivante ET sa durée.
            let (phase, duree) = match self.phase {
                PhaseEclair::Repos => (PhaseEclair::Annonce, ANNONCE_PAS),
                PhaseEclair::Annonce => (PhaseEclair::Frappe, FRAPPE_PAS),
                PhaseEclair::Frappe => (PhaseEclair::Repos, self.repos_pas),
            };
            self.phase = phase;
            self.restant = duree;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    fn pas(e: &mut Eclair, n: u16) {
        let g = GrilleTuiles::vide(4, 4);
        let env = Environnement::simple(&g);
        for _ in 0..n {
            e.actualiser(&env);
        }
    }

    #[test]
    fn l_annonce_dure_huit_dixiemes_de_seconde() {
        assert_eq!(ANNONCE_PAS, 48);
        assert_eq!(FRAPPE_PAS, 12);
    }

    #[test]
    fn le_cycle_repos_annonce_frappe_repos() {
        let mut e = Eclair::new(Aabb::new(0.0, 0.0, 16.0, 64.0), 100, 10);
        pas(&mut e, 9);
        assert_eq!(e.phase(), PhaseEclair::Repos);
        pas(&mut e, 1);
        assert_eq!(e.phase(), PhaseEclair::Annonce);
        assert!(!e.frappe(), "l'annonce n'est jamais dangereuse");
        pas(&mut e, ANNONCE_PAS - 1);
        assert_eq!(e.phase(), PhaseEclair::Annonce);
        pas(&mut e, 1);
        assert_eq!(e.phase(), PhaseEclair::Frappe);
        assert!(e.frappe());
        pas(&mut e, FRAPPE_PAS - 1);
        assert!(e.frappe());
        pas(&mut e, 1);
        assert_eq!(e.phase(), PhaseEclair::Repos);
        // Le repos suivant dure bien `repos_pas`, puis le cycle recommence.
        pas(&mut e, 99);
        assert_eq!(e.phase(), PhaseEclair::Repos);
        pas(&mut e, 1);
        assert_eq!(e.phase(), PhaseEclair::Annonce);
    }

    #[test]
    fn la_progression_de_l_annonce_va_de_0_a_1() {
        let mut e = Eclair::new(Aabb::new(0.0, 0.0, 16.0, 64.0), 100, 1);
        assert_eq!(e.progression_annonce(), 0.0);
        pas(&mut e, 1);
        assert!(e.progression_annonce() < 0.05);
        pas(&mut e, ANNONCE_PAS - 2);
        assert!(e.progression_annonce() > 0.95);
    }
}
