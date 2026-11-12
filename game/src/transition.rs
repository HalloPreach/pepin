//! Transition entre deux niveaux : des volets d'ardoise qui descendent de haut
//! en bas (colonne après colonne), cachent l'écran, puis se relèvent.
//!
//! Pas de fondu : la palette n'a pas de transparence, donc on « ferme » l'écran
//! avec des rectangles pleins, comme sur les consoles 8 bits. La machine à
//! états est pure (testable) ; seule `dessiner` appelle macroquad.
//!
//! Notions Rust illustrées ici : `enum` comme machine à états, méthode qui
//! renvoie un `bool` pour signaler « maintenant ! » à l'appelant.

use macroquad::prelude::draw_rectangle;
use pepin_core::palette::ARDOISE_OMBRE;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::rendu::couleur_mq;

/// Durée (pas) de la fermeture, puis de l'ouverture.
const DUREE: u8 = 24;
/// Largeur d'un volet en pixels.
const LARGEUR_VOLET: i32 = 16;
/// Nombre de volets pour couvrir la largeur de l'écran.
const NB_VOLETS: i32 = (LARGEUR_LOGIQUE as i32 + LARGEUR_VOLET - 1) / LARGEUR_VOLET;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Etape {
    Aucune,
    Fermeture(u8),
    Ouverture(u8),
}

/// La transition en cours (ou pas).
#[derive(Debug, Clone, Copy)]
pub struct Transition {
    etape: Etape,
}

impl Transition {
    pub fn new() -> Self {
        Self {
            etape: Etape::Aucune,
        }
    }

    /// Une transition est-elle en cours ? (la simulation du monde est alors figée)
    pub fn est_active(&self) -> bool {
        self.etape != Etape::Aucune
    }

    /// Commence la fermeture de l'écran (sans effet si une transition est déjà en cours).
    pub fn demarrer(&mut self) {
        if !self.est_active() {
            self.etape = Etape::Fermeture(0);
        }
    }

    /// Un pas de simulation. Renvoie `true` à l'instant précis où l'écran est
    /// entièrement couvert : c'est le moment de charger le niveau suivant.
    pub fn actualiser(&mut self) -> bool {
        match self.etape {
            Etape::Aucune => false,
            Etape::Fermeture(n) if n + 1 >= DUREE => {
                self.etape = Etape::Ouverture(0);
                true
            }
            Etape::Fermeture(n) => {
                self.etape = Etape::Fermeture(n + 1);
                false
            }
            Etape::Ouverture(n) if n + 1 >= DUREE => {
                self.etape = Etape::Aucune;
                false
            }
            Etape::Ouverture(n) => {
                self.etape = Etape::Ouverture(n + 1);
                false
            }
        }
    }

    /// Part de l'écran couverte, de 0.0 (rien) à 1.0 (tout).
    pub fn progression(&self) -> f32 {
        match self.etape {
            Etape::Aucune => 0.0,
            Etape::Fermeture(n) => (n + 1) as f32 / DUREE as f32,
            Etape::Ouverture(n) => 1.0 - (n + 1) as f32 / DUREE as f32,
        }
    }

    /// Dessine les volets dans la cible logique.
    pub fn dessiner(&self) {
        let p = self.progression();
        if p <= 0.0 {
            return;
        }
        for colonne in 0..NB_VOLETS {
            let h = hauteur_volet(p, colonne);
            if h > 0 {
                draw_rectangle(
                    (colonne * LARGEUR_VOLET) as f32,
                    0.0,
                    LARGEUR_VOLET as f32,
                    h as f32,
                    couleur_mq(ARDOISE_OMBRE),
                );
            }
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::new()
    }
}

/// Hauteur (px) du volet `colonne` quand l'écran est couvert à la part `p`.
/// Les volets de gauche partent les premiers : un balayage en diagonale.
fn hauteur_volet(p: f32, colonne: i32) -> i32 {
    let retard = 0.5 * colonne as f32 / (NB_VOLETS - 1) as f32;
    let part = (p * 1.5 - retard).clamp(0.0, 1.0);
    (part * HAUTEUR_LOGIQUE as f32).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn au_repos_rien_n_est_couvert() {
        let t = Transition::new();
        assert!(!t.est_active());
        assert_eq!(t.progression(), 0.0);
    }

    #[test]
    fn le_cycle_signale_une_seule_fois_l_ecran_couvert() {
        let mut t = Transition::new();
        t.demarrer();
        let mut signaux = 0;
        let mut pas = 0;
        while t.est_active() {
            if t.actualiser() {
                signaux += 1;
                assert!(t.progression() > 0.9, "couvert au moment du signal");
            }
            pas += 1;
            assert!(pas < 1000, "la transition doit finir");
        }
        assert_eq!(signaux, 1);
        assert_eq!(pas, 2 * DUREE as u32);
    }

    #[test]
    fn demarrer_pendant_une_transition_ne_la_relance_pas() {
        let mut t = Transition::new();
        t.demarrer();
        for _ in 0..5 {
            t.actualiser();
        }
        let avant = t.progression();
        t.demarrer();
        assert_eq!(t.progression(), avant);
    }

    #[test]
    fn a_pleine_couverture_tous_les_volets_sont_complets() {
        for c in 0..NB_VOLETS {
            assert_eq!(hauteur_volet(1.0, c), HAUTEUR_LOGIQUE as i32);
            assert_eq!(hauteur_volet(0.0, c), 0);
        }
        // Balayage : le premier volet descend avant le dernier.
        assert!(hauteur_volet(0.4, 0) > hauteur_volet(0.4, NB_VOLETS - 1));
    }
}
