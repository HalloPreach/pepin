//! Module `timestep` : boucle à pas fixe 60 Hz, découplée du rendu.
//!
//! Principe (« Fix Your Timestep ») : le rendu tourne à la fréquence de l'écran
//! (60, 144 Hz...) mais la simulation avance TOUJOURS par pas de 1/60 s. À chaque
//! image on ajoute le temps réel écoulé dans un accumulateur, puis on exécute
//! autant de pas que cet accumulateur en contient.
//!
//! Notions Rust illustrées ici : struct avec état privé, `&mut self`,
//! `Default`, constantes associées, tests unitaires.

use crate::physics::HZ;

/// Durée d'un pas en secondes, en `f64` pour limiter l'accumulation d'erreurs.
const DT_F64: f64 = 1.0 / HZ as f64;
/// Nombre maximal de pas exécutés pour UNE image affichée.
pub const MAX_PAS_PAR_IMAGE: u32 = 5;
/// Temps réel maximal pris en compte pour une image (secondes). Si l'onglet
/// était en arrière-plan 10 s, on ne simule pas 10 s d'un coup.
pub const DT_REEL_MAX: f64 = 0.25;
/// Tolérance pour que 6 × (1/60) donne bien 0,1 s malgré les arrondis binaires.
const EPSILON: f64 = 1e-9;

/// Accumulateur de temps. Ses champs sont privés : on ne peut le manipuler
/// qu'à travers ses méthodes, donc impossible de le mettre dans un état incohérent.
#[derive(Debug, Clone, Default)]
pub struct PasFixe {
    accumulateur: f64,
}

impl PasFixe {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reçoit le temps réel écoulé (secondes) et renvoie le nombre de pas de
    /// simulation à exécuter pour cette image.
    ///
    /// Protection anti-spirale de la mort : si le rendu est trop lent, simuler
    /// plus de pas rend l'image suivante encore plus lente, etc. On plafonne
    /// donc à `MAX_PAS_PAR_IMAGE` et on ABANDONNE le retard restant (le jeu
    /// ralentit au lieu de se figer).
    pub fn avancer(&mut self, dt_reel: f64) -> u32 {
        // Une durée négative ou NaN/infinie est ignorée : `is_finite` couvre NaN et ±inf.
        let dt = if dt_reel.is_finite() && dt_reel > 0.0 {
            dt_reel.min(DT_REEL_MAX)
        } else {
            0.0
        };
        self.accumulateur += dt;

        let mut pas = 0;
        while self.accumulateur + EPSILON >= DT_F64 && pas < MAX_PAS_PAR_IMAGE {
            self.accumulateur -= DT_F64;
            pas += 1;
        }
        if self.accumulateur < 0.0 {
            self.accumulateur = 0.0; // l'epsilon a pu nous faire passer légèrement sous 0
        }
        if self.accumulateur + EPSILON >= DT_F64 {
            self.accumulateur = 0.0; // plafond atteint : on jette le retard
        }
        pas
    }

    /// Fraction de pas restante dans l'accumulateur, entre 0 et 1 : sert au
    /// rendu pour interpoler entre l'état précédent et l'état courant.
    pub fn alpha(&self) -> f32 {
        ((self.accumulateur / DT_F64) as f32).clamp(0.0, 1.0)
    }

    /// Remet l'accumulateur à zéro (changement de niveau, reprise de pause...).
    pub fn reinitialiser(&mut self) {
        self.accumulateur = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_pas_par_image_a_60_hz() {
        let mut p = PasFixe::new();
        let total: u32 = (0..60).map(|_| p.avancer(1.0 / 60.0)).sum();
        assert_eq!(total, 60);
    }

    #[test]
    fn plusieurs_pas_si_le_rendu_est_lent() {
        let mut p = PasFixe::new();
        assert_eq!(p.avancer(1.0 / 30.0), 2);
        assert_eq!(p.avancer(0.1), 6.min(MAX_PAS_PAR_IMAGE));
    }

    #[test]
    fn pas_fixe_independant_du_taux_d_affichage() {
        // 1 seconde simulée à 144 Hz, 30 Hz et 240 Hz : toujours 60 pas.
        for hz in [30.0_f64, 144.0, 240.0] {
            let mut p = PasFixe::new();
            let images = hz as u32;
            let total: u32 = (0..images).map(|_| p.avancer(1.0 / hz)).sum();
            assert!((59..=60).contains(&total), "{hz} Hz -> {total} pas");
        }
    }

    #[test]
    fn le_reste_est_conserve_entre_les_images() {
        let mut p = PasFixe::new();
        // 3 × 5 ms = 15 ms < 16,67 ms : aucun pas, puis le 4e appel en déclenche un.
        assert_eq!(p.avancer(0.005), 0);
        assert_eq!(p.avancer(0.005), 0);
        assert_eq!(p.avancer(0.005), 0);
        assert!(p.alpha() > 0.8 && p.alpha() < 1.0);
        assert_eq!(p.avancer(0.005), 1);
    }

    #[test]
    fn plafond_anti_spirale() {
        let mut p = PasFixe::new();
        assert_eq!(p.avancer(5.0), MAX_PAS_PAR_IMAGE);
        // Le retard est abandonné : l'image suivante, normale, ne rattrape rien.
        assert_eq!(p.avancer(0.0), 0);
        assert_eq!(p.avancer(1.0 / 60.0), 1);
    }

    #[test]
    fn durees_invalides_ignorees() {
        let mut p = PasFixe::new();
        assert_eq!(p.avancer(-1.0), 0);
        assert_eq!(p.avancer(f64::NAN), 0);
        assert_eq!(p.avancer(f64::INFINITY), 0);
        assert_eq!(p.alpha(), 0.0);
    }
}
