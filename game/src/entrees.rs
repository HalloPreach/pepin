//! Entrées : une abstraction `Commandes` indépendante du périphérique.
//!
//! Le jeu ne sait pas si l'on joue au clavier, à la manette ou au doigt : il lit
//! des `Commandes` (gauche, droite, saut, pause). Chaque périphérique est une
//! `SourceEntrees` ; `Sources` en fusionne plusieurs. Pour ajouter le tactile
//! plus tard, il suffira d'écrire une nouvelle structure qui implémente le trait.
//!
//! Notions Rust illustrées ici : `trait` (comportement commun à plusieurs types),
//! objet-trait `Box<dyn SourceEntrees>` (liste de sources de types différents),
//! `Copy` pour une petite structure passée par valeur.

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};
use pepin_core::player::{Direction, Entrees};

/// Ce que le joueur demande pendant une image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Commandes {
    pub gauche: bool,
    pub droite: bool,
    /// Saut MAINTENU (en l'air, maintenir ouvre le parapluie).
    pub saut: bool,
    /// Demande de pause : vrai seulement à l'image de l'appui (front montant).
    pub pause: bool,
}

impl Commandes {
    /// Direction horizontale. Gauche ET droite en même temps s'annulent.
    pub fn direction(&self) -> Direction {
        match (self.gauche, self.droite) {
            (true, false) => Direction::Gauche,
            (false, true) => Direction::Droite,
            _ => Direction::Immobile,
        }
    }

    /// Les entrées que comprend la simulation du cœur.
    pub fn vers_entrees(&self) -> Entrees {
        Entrees {
            direction: self.direction(),
            saut: self.saut,
        }
    }

    /// Réunit deux jeux de commandes : une commande est active si l'un des deux
    /// périphériques la demande (clavier ET tactile peuvent coexister).
    pub fn fusionner(self, autre: Commandes) -> Commandes {
        Commandes {
            gauche: self.gauche || autre.gauche,
            droite: self.droite || autre.droite,
            saut: self.saut || autre.saut,
            pause: self.pause || autre.pause,
        }
    }
}

/// Un périphérique d'entrée (clavier, tactile, manette...).
pub trait SourceEntrees {
    /// Lit l'état du périphérique, une fois par image affichée.
    fn lire(&mut self) -> Commandes;
}

/// Le clavier : flèches, Q/A/D (WASD et ZQSD), Espace, Haut, W, Z, Échap, P.
pub struct Clavier;

/// Une touche de la liste est-elle enfoncée ?
fn une_enfoncee(touches: &[KeyCode]) -> bool {
    touches.iter().any(|&t| is_key_down(t))
}

impl SourceEntrees for Clavier {
    fn lire(&mut self) -> Commandes {
        const SAUT: [KeyCode; 4] = [KeyCode::Space, KeyCode::Up, KeyCode::W, KeyCode::Z];
        // `is_key_pressed` rattrape un appui très bref, relâché avant l'image suivante.
        let saut = une_enfoncee(&SAUT) || SAUT.iter().any(|&t| is_key_pressed(t));
        Commandes {
            gauche: une_enfoncee(&[KeyCode::Left, KeyCode::A, KeyCode::Q]),
            droite: une_enfoncee(&[KeyCode::Right, KeyCode::D]),
            saut,
            pause: is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P),
        }
    }
}

/// Toutes les sources actives, fusionnées.
#[derive(Default)]
pub struct Sources {
    liste: Vec<Box<dyn SourceEntrees>>,
}

impl Sources {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un périphérique (la boîte prend la possession de la source).
    pub fn ajouter(&mut self, source: Box<dyn SourceEntrees>) {
        self.liste.push(source);
    }

    /// Lit chaque source et fusionne leurs commandes.
    pub fn lire(&mut self) -> Commandes {
        self.liste
            .iter_mut()
            .fold(Commandes::default(), |acc, s| acc.fusionner(s.lire()))
    }
}

/// Raccourci de mise au point : touches 1 à 5 pour sauter à un niveau.
pub fn raccourci_niveau() -> Option<usize> {
    [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
    ]
    .iter()
    .position(|&k| is_key_pressed(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gauche() -> Commandes {
        Commandes {
            gauche: true,
            ..Commandes::default()
        }
    }

    /// Fausse source : renvoie toujours les mêmes commandes.
    struct Fixe(Commandes);
    impl SourceEntrees for Fixe {
        fn lire(&mut self) -> Commandes {
            self.0
        }
    }

    #[test]
    fn gauche_et_droite_ensemble_s_annulent() {
        let c = Commandes {
            gauche: true,
            droite: true,
            ..Commandes::default()
        };
        assert_eq!(c.direction(), Direction::Immobile);
        assert_eq!(gauche().direction(), Direction::Gauche);
    }

    #[test]
    fn la_conversion_garde_direction_et_saut() {
        let c = Commandes {
            droite: true,
            saut: true,
            ..Commandes::default()
        };
        let e = c.vers_entrees();
        assert_eq!(e.direction, Direction::Droite);
        assert!(e.saut);
    }

    #[test]
    fn la_fusion_est_un_ou_logique() {
        let saut = Commandes {
            saut: true,
            pause: true,
            ..Commandes::default()
        };
        let f = gauche().fusionner(saut);
        assert!(f.gauche && f.saut && f.pause && !f.droite);
    }

    #[test]
    fn plusieurs_sources_se_fusionnent() {
        let mut s = Sources::new();
        assert_eq!(s.lire(), Commandes::default(), "aucune source = rien");
        s.ajouter(Box::new(Fixe(gauche())));
        s.ajouter(Box::new(Fixe(Commandes {
            saut: true,
            ..Commandes::default()
        })));
        let c = s.lire();
        assert!(c.gauche && c.saut && !c.droite);
    }
}
