//! Ventilateur : une hélice de toit dont le courant ascendant souffle 2 s
//! puis s'arrête 2 s. La reprise est annoncée comme les éclairs : 0,8 s de
//! clignotement qui accélère avant que l'hélice ne reprenne.
//!
//! Le cycle est une machine à états avec un `enum` (`PhaseVentilateur`) ; les
//! durées sont des ENTIERS (nombre de pas à 60 Hz), comme l'éclair :
//!
//!   arrêt (72) → annonce (48) → souffle (120) → arrêt ...
//!
//! `72 + 48 = 120` : le ventilateur « s'arrête 2 s », dont les 0,8 s finaux
//! sont l'annonce de la reprise. La période complète est de 240 pas (4 s :
//! 2 s de souffle, 2 s d'arrêt).
//!
//! La case (16 x 8 px, posée en bas de sa case de grille) est SOLIDE : on se
//! tient dessus et c'est là que le courant prend. Le courant lui-même n'existe
//! que pendant le souffle (`courant_actif` renvoie `None` sinon) : c'est le
//! jeu (`Monde`) qui le raccorde à l'environnement du joueur, pas le parseur.
//!
//! Notions Rust : `Option` (« un courant, ou rien »), une machine à états en
//! entiers exactement reproductible, une fonction pure `courant_actif` qui
//! n'EMPRUNTE que le ventilateur (`&self`).

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{CourantAscendant, Environnement, HZ};

/// Durée d'un souffle (2 s = 120 pas).
pub const SOUFFLE_PAS: u16 = 2 * HZ as u16;
/// Durée de l'annonce de la reprise (0,8 s = 48 pas, comme l'éclair).
pub const ANNONCE_PAS: u16 = (HZ as u16 * 4) / 5;
/// Durée de l'arrêt SANS son annonce (2 s au total : 72 + 48).
pub const ARRET_PAS: u16 = SOUFFLE_PAS - ANNONCE_PAS;
/// Période complète du cycle : 2 s de souffle + 2 s d'arrêt (240 pas).
pub const PERIODE_PAS: u16 = SOUFFLE_PAS + ANNONCE_PAS + ARRET_PAS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseVentilateur {
    /// L'hélice est arrêtée, aucun courant, aucun danger.
    Arret,
    /// Le voyant clignote (de plus en plus vite) : le souffle revient.
    Annonce,
    /// Le courant souffle : il porte vers le haut comme un courant `v`.
    Souffle,
}

#[derive(Debug, Clone)]
pub struct Ventilateur {
    /// Boîte de la CASE (16 x 8 px, bas de la case de grille) : solide, on se
    /// tient dessus.
    pub boite: Aabb,
    /// Zone du courant ascendant : du dessus de la case au premier solide
    /// au-dessus (ou au haut de la carte s'il n'y en a pas).
    pub zone: Aabb,
    phase: PhaseVentilateur,
    /// Pas restants avant de passer à la phase suivante.
    restant: u16,
}

impl Ventilateur {
    /// `decalage_pas` : délai avant la PREMIÈRE annonce (sert à désynchroniser
    /// plusieurs ventilateurs d'un même niveau) ; 0 = l'arrêt complet d'abord.
    pub fn nouveau(boite: Aabb, zone: Aabb, decalage_pas: u16) -> Self {
        Self {
            boite,
            zone,
            phase: PhaseVentilateur::Arret,
            // Un décalage court lève l'hélice plus tôt ; 0 garde l'arrêt
            // complet avant la première annonce.
            restant: if decalage_pas == 0 {
                ARRET_PAS
            } else {
                decalage_pas.min(ARRET_PAS)
            },
        }
    }

    pub fn phase(&self) -> PhaseVentilateur {
        self.phase
    }

    /// Le courant souffle-t-il en ce moment ?
    pub fn souffle(&self) -> bool {
        self.phase == PhaseVentilateur::Souffle
    }

    /// Le clignotement d'annonce est-il visible ?
    pub fn en_annonce(&self) -> bool {
        self.phase == PhaseVentilateur::Annonce
    }

    /// Avancement de l'annonce entre 0.0 (début) et 1.0 (souffle imminent),
    /// pour faire clignoter plus vite à l'écran (comme l'éclair). 0.0 sinon.
    pub fn progression_annonce(&self) -> f32 {
        if self.en_annonce() {
            1.0 - self.restant as f32 / ANNONCE_PAS as f32
        } else {
            0.0
        }
    }

    /// Le courant ascendant tant que l'hélice souffle (`None` en arrêt et
    /// pendant l'annonce) : c'est le jeu qui le passe à l'environnement du
    /// joueur.
    pub fn courant_actif(&self) -> Option<CourantAscendant> {
        self.souffle().then(|| CourantAscendant::new(self.zone))
    }
}

impl Actualisable for Ventilateur {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.restant = self.restant.saturating_sub(1);
        if self.restant == 0 {
            // Chaque variante désigne la phase suivante ET sa durée.
            let (phase, duree) = match self.phase {
                PhaseVentilateur::Arret => (PhaseVentilateur::Annonce, ANNONCE_PAS),
                PhaseVentilateur::Annonce => (PhaseVentilateur::Souffle, SOUFFLE_PAS),
                PhaseVentilateur::Souffle => (PhaseVentilateur::Arret, ARRET_PAS),
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
    use crate::player::{Entrees, Joueur};

    fn ventilateur() -> Ventilateur {
        // Case à la colonne 4 (x = 64) ; zone du courant de y = 32 à y = 128.
        Ventilateur::nouveau(
            Aabb::new(64.0, 128.0, 16.0, 8.0),
            Aabb::new(64.0, 32.0, 16.0, 96.0),
            0,
        )
    }

    fn pas(v: &mut Ventilateur, n: u16) {
        let g = GrilleTuiles::vide(4, 4);
        let env = Environnement::simple(&g);
        for _ in 0..n {
            v.actualiser(&env);
        }
    }

    /// Passe le ventilateur au milieu de son souffle (au cas où).
    fn au_souffle(mut v: Ventilateur) -> Ventilateur {
        for _ in 0..PERIODE_PAS {
            if v.souffle() {
                break;
            }
            pas(&mut v, 1);
        }
        assert!(v.souffle(), "le ventilateur doit être en souffle");
        v
    }

    #[test]
    fn le_cycle_est_exact() {
        // 72 + 48 = 2 s d'arrêt, 120 pas de souffle, période 240 (4 s).
        assert_eq!((ARRET_PAS, ANNONCE_PAS, SOUFFLE_PAS, PERIODE_PAS), (72, 48, 120, 240));
        let mut v = ventilateur();
        assert_eq!(v.phase(), PhaseVentilateur::Arret);
        pas(&mut v, ARRET_PAS - 1);
        assert_eq!(v.phase(), PhaseVentilateur::Arret);
        pas(&mut v, 1);
        assert_eq!(v.phase(), PhaseVentilateur::Annonce);
        pas(&mut v, ANNONCE_PAS - 1);
        assert_eq!(v.phase(), PhaseVentilateur::Annonce);
        pas(&mut v, 1);
        assert_eq!(v.phase(), PhaseVentilateur::Souffle);
        pas(&mut v, SOUFFLE_PAS - 1);
        assert_eq!(v.phase(), PhaseVentilateur::Souffle);
        pas(&mut v, 1);
        assert_eq!(v.phase(), PhaseVentilateur::Arret);
        // Le cycle complet reprend exactement : 240 pas de phase en phase.
        for _ in 0..3 {
            pas(&mut v, PERIODE_PAS);
            assert_eq!(v.phase(), PhaseVentilateur::Arret);
        }
    }

    #[test]
    fn l_annonce_precede_toujours_la_reprise() {
        // La seule phase qui précède le souffle est l'annonce : le clignotement
        // annonce la reprise, et la progression va de 0.0 à 1.0 pendant qu'elle
        // s'accélère.
        let mut v = ventilateur();
        pas(&mut v, ARRET_PAS);
        assert_eq!(v.phase(), PhaseVentilateur::Annonce);
        assert!(v.progression_annonce() < 0.1);
        pas(&mut v, ANNONCE_PAS / 2);
        assert!(v.progression_annonce() >= 0.4 && v.progression_annonce() < 0.9);
        pas(&mut v, ANNONCE_PAS / 2);
        assert_eq!(v.phase(), PhaseVentilateur::Souffle);
        assert_eq!(v.progression_annonce(), 0.0);
        // Hors annonce, aucun clignotement et aucun courant.
        pas(&mut v, SOUFFLE_PAS);
        assert_eq!(v.phase(), PhaseVentilateur::Arret);
        assert!(!v.en_annonce());
        assert!(v.courant_actif().is_none());
    }

    #[test]
    fn la_poussee_n_existe_que_pendant_le_souffle() {
        let v = au_souffle(ventilateur());
        assert!(v.courant_actif().is_some(), "courant pendant le souffle");
        let mut v2 = ventilateur();
        pas(&mut v2, ARRET_PAS + ANNONCE_PAS / 2); // 72 + 24 = 96 : au milieu de l'annonce
        assert!(v2.courant_actif().is_none(), "pas de courant pendant l'annonce");
        pas(&mut v2, ANNONCE_PAS / 2); // 96 + 24 = 120 : on rentre dans le souffle
        assert!(v2.courant_actif().is_some(), "le courant reprend à la reprise");
        pas(&mut v2, SOUFFLE_PAS); // 120 + 120 = 240 : on rentre dans l'arrêt
        assert_eq!(v2.phase(), PhaseVentilateur::Arret);
        assert!(v2.courant_actif().is_none(), "pas de courant pendant l'arrêt");
    }

    /// Le joueur est posé dans la zone du courant, à l'intérieur d'une grille LARGES
    /// (8 x 8 = 128 px : les côtés gauche/droite sont des murs, il faut que ni le
    /// joueur ni la colonne du ventilateur n'y soient coincés).
    fn joueur_dans_la_zone() -> (GrilleTuiles, Joueur) {
        let g = GrilleTuiles::vide(8, 8);
        // Zone du courant : x 64..80, y 32..128. Le joueur (boîte 10x14) est en
        // (70, 100) : dans la zone et dans la grille, avec de la marge pour monter.
        (g, Joueur::new(70.0, 100.0))
    }

    #[test]
    fn la_montee_est_portee_pendant_le_souffle() {
        let v = au_souffle(ventilateur());
        let (g, mut j) = joueur_dans_la_zone();
        let c = v.courant_actif().unwrap();
        let courants = [c];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
            vents: &[],
        };
        let y0 = j.boite.y;
        for _ in 0..30 {
            j.actualiser(&Entrees::default(), &env);
        }
        assert!(
            j.boite.y < y0 - 20.0,
            "le courant porte vers le haut (y : {y0} -> {})",
            j.boite.y
        );
        assert!(j.vitesse.y < 0.0, "montée : vitesse vers le haut");
    }

    /// Un plafond solide (rangée du haut) arrête la montée : « arrêt en haut ».
    #[test]
    fn la_montee_s_arrete_au_plafond() {
        let v = au_souffle(Ventilateur::nouveau(
            Aabb::new(64.0, 128.0, 16.0, 8.0),
            Aabb::new(64.0, 16.0, 16.0, 112.0),
            0,
        ));
        // Grille 8 x 8 : un plafond solide sur la rangée du haut (y 0..16), le reste
        // est vide. La zone du courant (y 16..128) remonte juste sous ce plafond.
        let lignes = vec![
            "########", // plafond (rangée du haut)
            "........",
            "........",
            "........",
            "........",
            "........",
            "........",
            "........",
        ];
        let g = GrilleTuiles::depuis_texte(&lignes);
        let mut j = Joueur::new(70.0, 100.0);
        let c = v.courant_actif().unwrap();
        let courants = [c];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
            vents: &[],
        };
        for _ in 0..SOUFFLE_PAS {
            j.actualiser(&Entrees::default(), &env);
        }
        assert_eq!(j.boite.y, 16.0, "la montée s'arrête au plafond");
    }

    #[test]
    fn quand_le_ventilateur_s_arrete_pepin_redescend() {
        let v = au_souffle(ventilateur());
        let (g, mut j) = joueur_dans_la_zone();
        let c = v.courant_actif().unwrap();
        let courants = [c];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
            vents: &[],
        };
        for _ in 0..30 {
            j.actualiser(&Entrees::default(), &env);
        }
        let y_apres_montee = j.boite.y;
        // Le ventilateur s'arrête : plus de courant, la gravité reprend le dessus.
        let env_vide = Environnement::simple(&g);
        for _ in 0..30 {
            j.actualiser(&Entrees::default(), &env_vide);
        }
        assert!(
            j.boite.y > y_apres_montee,
            "sans courant, Pépin redescend ({y_apres_montee} -> {})",
            j.boite.y
        );
        assert!(j.vitesse.y > 0.0, "il tombe");
    }

    /// Le courant n'a pas de composante horizontale : porté sans glisser, Pépin
    /// garde son abscisse (aucune commande).
    #[test]
    fn pepin_est_porte_sans_glisser() {
        let v = au_souffle(ventilateur());
        let (g, mut j) = joueur_dans_la_zone();
        let c = v.courant_actif().unwrap();
        let courants = [c];
        let env = Environnement {
            grille: &g,
            solides: &[],
            courants: &courants,
            vents: &[],
        };
        let x0 = j.boite.x;
        for _ in 0..30 {
            j.actualiser(&Entrees::default(), &env);
        }
        assert_eq!(j.boite.x, x0, "porté verticalement, sans glisser");
        assert!(j.boite.x >= v.zone.x && j.boite.x + j.boite.w <= v.zone.droite());
    }

    #[test]
    fn le_decalage_raccourcit_le_premier_arret() {
        let mut v = Ventilateur::nouveau(Aabb::new(0.0, 0.0, 16.0, 8.0), Aabb::new(0.0, 0.0, 16.0, 8.0), 12);
        pas(&mut v, 11);
        assert_eq!(v.phase(), PhaseVentilateur::Arret);
        pas(&mut v, 1);
        assert_eq!(v.phase(), PhaseVentilateur::Annonce);
        // 0 : l'arrêt complet (72 pas) avant la première annonce.
        let mut v0 = ventilateur();
        pas(&mut v0, ARRET_PAS - 1);
        assert_eq!(v0.phase(), PhaseVentilateur::Arret);
    }
}
