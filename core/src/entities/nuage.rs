//! Le Grand Nuage (lettre `O`) : le boss de « L'œil du cyclone ».
//!
//! Une machine à états PURE, en entiers (nombre de pas de 1/60 s), comme
//! l'éclair : le même pas donne toujours le même nuage, en natif comme en wasm.
//!
//! ```text
//!   prélude (90) ──► colère (son motif) ──► essoufflé (300) ──► colère (même motif)...
//!                                               │ Pépin lui saute sur la tête
//!                                               ▼
//!                                           touché (90) ──► colère (motif suivant)
//!                                               │ c'était son dernier point
//!                                               ▼
//!                                           déchirure (120) ──► vaincu (niveau fini)
//! ```
//!
//! - prélude : au début de l'arène (et après chaque mort de Pépin), il gronde ;
//!   rien ne tombe.
//! - colère : il lance un MOTIF d'éclairs, de plus en plus serré à chaque point
//!   perdu (`MOTIFS`). Chaque salve d'un motif frappe un groupe de colonnes,
//!   annoncées EXACTEMENT comme un éclair `e` : 48 pas d'annonce
//!   (`eclair::ANNONCE_PAS`), puis 12 pas de frappe (`eclair::FRAPPE_PAS`).
//! - essoufflé : plus d'éclair ; il laisse échapper un courant ascendant à côté
//!   de lui (`cote_du_courant`). Pépin y monte, puis lui retombe sur la tête :
//!   un point de vie en moins (`recoit`, `toucher`).
//! - touché : il clignote, rien ne tombe, puis il reprend avec le motif suivant.
//! - déchirure : à 0 point de vie, le ciel se déchire ; à la fin, il est vaincu
//!   et le niveau est fini (`est_vaincu`).
//!
//! Si Pépin meurt, le nuage reprend au prélude (`reprendre`) mais GARDE ses
//! points perdus : on ne recommence pas le combat depuis le début.
//!
//! Notions Rust : un `enum` pour les phases, des tableaux CONSTANTS de données
//! (`MOTIFS`, des tranches `&'static [...]` imbriquées), des fonctions pures
//! testées pas à pas, `Option` pour le courant (« un courant, ou rien »).

use super::Actualisable;
use crate::aabb::Aabb;
use crate::entities::contacts::MARGE_ECRASEMENT;
use crate::entities::eclair::{ANNONCE_PAS, FRAPPE_PAS};
use crate::physics::{CourantAscendant, Environnement};
use crate::player::Joueur;

/// Points de vie du Grand Nuage : trois sauts sur la tête.
pub const PV_NUAGE: u8 = 3;
/// Prélude : 1,5 s de grondement avant le premier motif (et après une mort).
pub const PRELUDE_PAS: u16 = 90;
/// Essoufflement : 5 s de courant ascendant, le temps de traverser l'arène et
/// de monter.
pub const ESSOUFFLE_PAS: u16 = 300;
/// Touché : 1,5 s de clignotement, sans danger.
pub const TOUCHE_PAS: u16 = 90;
/// Le ciel se déchire pendant 2 s, puis le niveau est fini.
pub const DECHIRURE_PAS: u16 = 120;
/// Taille du nuage (px) : trois cases de large, deux de haut.
pub const TAILLE_NUAGE: (f32, f32) = (48.0, 32.0);
/// Colonnes d'éclairs : de 8 colonnes à gauche de celle du `O` à 8 à droite.
pub const PORTEE_COLONNES: i8 = 8;
/// Nombre de colonnes d'éclairs (17).
pub const NB_COLONNES: usize = 2 * PORTEE_COLONNES as usize + 1;
/// Durée d'une salve : son annonce puis sa frappe (60 pas, comme un éclair).
pub const SALVE_PAS: u16 = ANNONCE_PAS + FRAPPE_PAS;

/// Une salve d'un motif : à `debut` (pas depuis le début du motif), les colonnes
/// de chaque bande `(de, a)` (bornes comprises, en colonnes depuis celle du `O` :
/// -8 tout à gauche, 0 sous le nuage, 8 tout à droite) sont annoncées pendant
/// 48 pas, puis frappées pendant 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Salve {
    pub debut: u16,
    pub bandes: &'static [(i8, i8)],
}

impl Salve {
    /// La salve touche-t-elle la colonne `decalage` (de -8 à 8) ?
    pub fn couvre(&self, decalage: i8) -> bool {
        self.bandes
            .iter()
            .any(|&(de, a)| (de..=a).contains(&decalage))
    }
}

/// Les trois motifs, du plus large au plus serré : le 1er à 3 points de vie, le
/// 2e à 2, le 3e à 1. Les salves sont de plus en plus rapprochées (60, 40, puis
/// 30 pas : l'annonce de la suivante commence avant la fin de la précédente) et
/// laissent de moins en moins de place. Le test `chaque_motif_se_traverse`
/// vérifie qu'on peut toujours s'en sortir en courant.
pub const MOTIFS: [&[Salve]; PV_NUAGE as usize] = [
    // Motif 1 : une salve à la fois (gauche, droite, milieu, les deux côtés).
    &[
        Salve {
            debut: 0,
            bandes: &[(-8, -4)],
        },
        Salve {
            debut: 60,
            bandes: &[(4, 8)],
        },
        Salve {
            debut: 120,
            bandes: &[(-2, 2)],
        },
        Salve {
            debut: 180,
            bandes: &[(-8, -5), (5, 8)],
        },
    ],
    // Motif 2 : une salve toutes les 40 pas, deux ou trois bandes à la fois.
    &[
        Salve {
            debut: 0,
            bandes: &[(-8, -3)],
        },
        Salve {
            debut: 40,
            bandes: &[(3, 8)],
        },
        Salve {
            debut: 80,
            bandes: &[(-2, 2)],
        },
        Salve {
            debut: 120,
            bandes: &[(-8, -5), (1, 4)],
        },
        Salve {
            debut: 160,
            bandes: &[(-4, 0), (5, 8)],
        },
        Salve {
            debut: 200,
            bandes: &[(-8, -6), (-1, 1), (6, 8)],
        },
    ],
    // Motif 3 : une salve toutes les 30 pas, les bandes balaient l'arène.
    &[
        Salve {
            debut: 0,
            bandes: &[(-8, -5), (3, 8)],
        },
        Salve {
            debut: 30,
            bandes: &[(-4, -1)],
        },
        Salve {
            debut: 60,
            bandes: &[(-8, -6), (0, 3)],
        },
        Salve {
            debut: 90,
            bandes: &[(-5, -2), (4, 8)],
        },
        Salve {
            debut: 120,
            bandes: &[(-8, -6), (-1, 2)],
        },
        Salve {
            debut: 150,
            bandes: &[(-5, -3), (3, 6)],
        },
        Salve {
            debut: 180,
            bandes: &[(-8, -7), (-2, 1), (7, 8)],
        },
        Salve {
            debut: 210,
            bandes: &[(-8, -4), (2, 8)],
        },
    ],
];

/// Durée (pas) du motif `motif` : le début de sa dernière salve, plus une salve.
pub fn duree_motif(motif: usize) -> u16 {
    MOTIFS[motif].iter().map(|s| s.debut).max().unwrap_or(0) + SALVE_PAS
}

/// Ce que fait la colonne `decalage` au pas `t` du motif `motif` : rien, l'annonce
/// (avec son avancement de 0 à 1) ou la frappe. Si deux salves se chevauchent, la
/// frappe l'emporte sur l'annonce. Fonction pure.
pub fn etat_colonne(motif: usize, t: u16, decalage: i8) -> EtatColonne {
    let mut etat = EtatColonne::Calme;
    for s in MOTIFS[motif].iter().filter(|s| s.couvre(decalage)) {
        if t < s.debut || t >= s.debut + SALVE_PAS {
            continue;
        }
        let dans = t - s.debut;
        if dans >= ANNONCE_PAS {
            return EtatColonne::Frappe;
        }
        etat = EtatColonne::Annonce(dans as f32 / ANNONCE_PAS as f32);
    }
    etat
}

/// L'état d'une colonne d'éclair du nuage pendant un pas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatColonne {
    /// Rien ne tombe.
    Calme,
    /// Annoncée (sans danger) : avancement de l'annonce, de 0.0 à 1.0.
    Annonce(f32),
    /// Frappée : dangereuse.
    Frappe,
}

/// Phase du Grand Nuage (voir le schéma en tête de module).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseNuage {
    Prelude,
    Colere,
    Essouffle,
    Touche,
    Dechirure,
    Vaincu,
}

/// De quel côté du nuage souffle son courant ascendant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cote {
    Gauche,
    Droite,
}

#[derive(Debug, Clone)]
pub struct GrandNuage {
    /// Le corps du nuage (48 x 32) : on lui saute sur la tête (son dessus).
    pub boite: Aabb,
    /// Les 17 colonnes d'éclairs, de la plus à gauche (-8) à la plus à droite (8).
    pub colonnes: [Aabb; NB_COLONNES],
    /// Les zones de ses deux courants ascendants : à gauche, puis à droite.
    pub courants: [Aabb; 2],
    phase: PhaseNuage,
    /// Pas écoulés depuis le début de la phase.
    ecoule: u16,
    pv: u8,
}

impl GrandNuage {
    /// Un nuage neuf, au prélude, avec ses 3 points de vie.
    pub fn new(boite: Aabb, colonnes: [Aabb; NB_COLONNES], courants: [Aabb; 2]) -> Self {
        Self {
            boite,
            colonnes,
            courants,
            phase: PhaseNuage::Prelude,
            ecoule: 0,
            pv: PV_NUAGE,
        }
    }

    pub fn phase(&self) -> PhaseNuage {
        self.phase
    }

    /// Points de vie restants (de 3 à 0).
    pub fn pv(&self) -> u8 {
        self.pv
    }

    /// Pas écoulés depuis le début de la phase en cours.
    pub fn ecoule(&self) -> u16 {
        self.ecoule
    }

    /// Le motif joué en colère : 0 à 3 points de vie, 1 à 2, 2 à 1.
    pub fn motif(&self) -> usize {
        usize::from(PV_NUAGE - self.pv.clamp(1, PV_NUAGE))
    }

    /// Durée de la phase en cours (`None` : vaincu, pour toujours).
    pub fn duree_phase(&self) -> Option<u16> {
        match self.phase {
            PhaseNuage::Prelude => Some(PRELUDE_PAS),
            PhaseNuage::Colere => Some(duree_motif(self.motif())),
            PhaseNuage::Essouffle => Some(ESSOUFFLE_PAS),
            PhaseNuage::Touche => Some(TOUCHE_PAS),
            PhaseNuage::Dechirure => Some(DECHIRURE_PAS),
            PhaseNuage::Vaincu => None,
        }
    }

    /// Côté du courant de l'essoufflement : à gauche après les motifs 1 et 3, à
    /// droite après le motif 2 (il faut traverser l'arène).
    pub fn cote_du_courant(&self) -> Cote {
        if self.motif().is_multiple_of(2) {
            Cote::Gauche
        } else {
            Cote::Droite
        }
    }

    /// Le courant ascendant, pendant l'essoufflement seulement (sinon `None`) :
    /// c'est le jeu qui le passe à l'environnement du joueur.
    pub fn courant_actif(&self) -> Option<CourantAscendant> {
        (self.phase == PhaseNuage::Essouffle).then(|| {
            let zone = match self.cote_du_courant() {
                Cote::Gauche => self.courants[0],
                Cote::Droite => self.courants[1],
            };
            CourantAscendant::new(zone)
        })
    }

    /// Nombre de pas d'essoufflement restants, CELUI-CI COMPRIS (0 hors de
    /// l'essoufflement) : le courant agit pendant ce pas et les
    /// `essoufflement_restant() - 1` suivants. Lecture seule : le jeu s'en sert pour
    /// que les brins de vent visibles meurent avec le courant.
    pub fn essoufflement_restant(&self) -> u16 {
        if self.phase == PhaseNuage::Essouffle {
            ESSOUFFLE_PAS - self.ecoule
        } else {
            0
        }
    }

    /// L'état de la colonne `i` (0 = la plus à gauche) en ce moment : toujours
    /// `Calme` hors de la colère.
    pub fn etat_colonne(&self, i: usize) -> EtatColonne {
        if self.phase != PhaseNuage::Colere || i >= NB_COLONNES {
            return EtatColonne::Calme;
        }
        etat_colonne(self.motif(), self.ecoule, i as i8 - PORTEE_COLONNES)
    }

    /// Une colonne qui frappe en ce moment touche-t-elle `boite` ?
    pub fn foudroie(&self, boite: &Aabb) -> bool {
        (0..NB_COLONNES).any(|i| {
            self.etat_colonne(i) == EtatColonne::Frappe && self.colonnes[i].chevauche(boite)
        })
    }

    /// Pépin lui tombe-t-il sur la tête pendant l'essoufflement ? Même règle que
    /// l'écrasement d'un ennemi : il descend, et ses pieds ne dépassent pas de plus
    /// de `MARGE_ECRASEMENT` le dessus du nuage. Hors de l'essoufflement (ou de
    /// côté, ou par-dessous), rien : le corps du nuage ne blesse pas, seuls ses
    /// éclairs sont dangereux.
    pub fn recoit(&self, joueur: &Joueur) -> bool {
        self.phase == PhaseNuage::Essouffle
            && joueur.vitesse.y > 0.0
            && joueur.boite.chevauche(&self.boite)
            && joueur.boite.bas() - self.boite.y <= MARGE_ECRASEMENT
    }

    /// Un saut sur la tête : un point de vie en moins ; touché, ou, au dernier
    /// point, le ciel se déchire. Sans effet hors de l'essoufflement.
    pub fn toucher(&mut self) {
        if self.phase != PhaseNuage::Essouffle {
            return;
        }
        self.pv = self.pv.saturating_sub(1);
        let phase = if self.pv == 0 {
            PhaseNuage::Dechirure
        } else {
            PhaseNuage::Touche
        };
        self.entrer(phase);
    }

    /// Pépin est mort : le nuage reprend au prélude, avec ses points perdus. Sans
    /// effet quand le ciel se déchire déjà (ou une fois vaincu).
    pub fn reprendre(&mut self) {
        if !matches!(self.phase, PhaseNuage::Dechirure | PhaseNuage::Vaincu) {
            self.entrer(PhaseNuage::Prelude);
        }
    }

    /// Passe tout de suite à l'essoufflement (robot de faisabilité, qui ne connaît
    /// pas le temps : un joueur patient attend l'essoufflement ; tests).
    pub fn essouffler(&mut self) {
        if self.pv > 0 {
            self.entrer(PhaseNuage::Essouffle);
        }
    }

    /// Le nuage est-il vaincu (le ciel s'est déchiré : le niveau est fini) ?
    pub fn est_vaincu(&self) -> bool {
        self.phase == PhaseNuage::Vaincu
    }

    /// Avancement de la déchirure du ciel, de 0.0 à 1.0 (1.0 une fois vaincu, 0.0
    /// avant).
    pub fn progression_dechirure(&self) -> f32 {
        match self.phase {
            PhaseNuage::Dechirure => self.ecoule as f32 / DECHIRURE_PAS as f32,
            PhaseNuage::Vaincu => 1.0,
            _ => 0.0,
        }
    }

    /// Décale le nuage de `pas` pas dans le temps : en avance (`pas` positif : il
    /// avance vraiment, d'une phase à l'autre s'il le faut) ou en retard (négatif :
    /// il recule dans la phase en cours, sans remonter avant son début). Sert au
    /// robot de QA, qui vérifie qu'un plan tient si le nuage a une ou deux images
    /// d'avance ou de retard sur Pépin.
    pub fn decaler(&mut self, pas: i32) {
        if pas >= 0 {
            for _ in 0..pas {
                self.avancer();
            }
        } else {
            self.ecoule = self.ecoule.saturating_sub(pas.unsigned_abs() as u16);
        }
    }

    /// Un pas de temps (le cœur de `actualiser`, qui n'a pas besoin du niveau).
    fn avancer(&mut self) {
        let Some(duree) = self.duree_phase() else {
            return; // vaincu : plus rien ne bouge
        };
        self.ecoule = self.ecoule.saturating_add(1);
        if self.ecoule < duree {
            return;
        }
        // Chaque phase désigne la suivante. Après l'essoufflement manqué comme après
        // un coup reçu, il repart en colère : `motif()` lit les points restants.
        let suivante = match self.phase {
            PhaseNuage::Prelude | PhaseNuage::Essouffle | PhaseNuage::Touche => PhaseNuage::Colere,
            PhaseNuage::Colere => PhaseNuage::Essouffle,
            PhaseNuage::Dechirure | PhaseNuage::Vaincu => PhaseNuage::Vaincu,
        };
        self.entrer(suivante);
    }

    fn entrer(&mut self, phase: PhaseNuage) {
        self.phase = phase;
        self.ecoule = 0;
    }
}

impl Actualisable for GrandNuage {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.avancer();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    /// Un nuage posé en (128, 48), au-dessus d'un sol à y = 144 : colonnes de 12 px
    /// du haut de la carte au sol, courants de 32 px à une case de lui.
    pub(crate) fn nuage() -> GrandNuage {
        let colonnes =
            std::array::from_fn(|i| Aabb::new(16.0 * (i as f32 + 1.0) + 2.0, 0.0, 12.0, 144.0));
        let courants = [
            Aabb::new(80.0, 16.0, 32.0, 128.0),
            Aabb::new(192.0, 16.0, 32.0, 128.0),
        ];
        GrandNuage::new(Aabb::new(128.0, 48.0, 48.0, 32.0), colonnes, courants)
    }

    fn pas(n: &mut GrandNuage, k: u32) {
        let g = GrilleTuiles::vide(4, 4);
        let env = Environnement::simple(&g);
        for _ in 0..k {
            n.actualiser(&env);
        }
    }

    /// Pépin qui tombe sur la tête du nuage (pieds 2 px dans son dessus).
    fn pepin_sur_la_tete(n: &GrandNuage) -> Joueur {
        let mut j = Joueur::new(n.boite.x + 19.0, n.boite.y - 12.0);
        j.vitesse.y = 36.0;
        j
    }

    #[test]
    fn les_motifs_durent_de_4_a_5_secondes() {
        assert_eq!(SALVE_PAS, 60);
        assert_eq!(
            [duree_motif(0), duree_motif(1), duree_motif(2)],
            [240, 260, 270]
        );
        assert_eq!(NB_COLONNES, 17);
    }

    #[test]
    fn de_plus_en_plus_serres() {
        // Les salves se rapprochent : 60, 40, puis 30 pas entre deux débuts.
        for (motif, ecart) in [(0, 60), (1, 40), (2, 30)] {
            for paire in MOTIFS[motif].windows(2) {
                assert_eq!(paire[1].debut - paire[0].debut, ecart, "motif {motif}");
            }
        }
        // Et chaque motif frappe plus de colonnes, au total, que le précédent.
        let frappes = |m: usize| -> usize {
            MOTIFS[m]
                .iter()
                .map(|s| (-8..=8).filter(|&c| s.couvre(c)).count())
                .sum()
        };
        assert!(frappes(0) < frappes(1) && frappes(1) < frappes(2));
        // Aucune bande ne sort des 17 colonnes.
        for s in MOTIFS.iter().flat_map(|m| m.iter()) {
            for &(de, a) in s.bandes {
                assert!(-PORTEE_COLONNES <= de && de <= a && a <= PORTEE_COLONNES);
            }
        }
    }

    #[test]
    fn une_colonne_est_annoncee_48_pas_puis_frappee_12() {
        // Motif 1, 1re salve (colonnes -8 à -4, au pas 0).
        assert_eq!(etat_colonne(0, 0, -8), EtatColonne::Annonce(0.0));
        assert!(matches!(etat_colonne(0, 47, -4), EtatColonne::Annonce(p) if p > 0.95));
        assert_eq!(etat_colonne(0, 48, -6), EtatColonne::Frappe);
        assert_eq!(etat_colonne(0, 59, -6), EtatColonne::Frappe);
        assert_eq!(etat_colonne(0, 60, -6), EtatColonne::Calme);
        // La colonne -3 n'est pas dans la salve.
        assert_eq!(etat_colonne(0, 50, -3), EtatColonne::Calme);
        // Motif 2 : la frappe de la 1re salve (48-59) chevauche l'annonce de la 2e (40-87).
        assert_eq!(etat_colonne(1, 50, -5), EtatColonne::Frappe);
        assert!(matches!(etat_colonne(1, 50, 5), EtatColonne::Annonce(_)));
    }

    #[test]
    fn le_combat_pas_a_pas() {
        let mut n = nuage();
        assert_eq!((n.phase(), n.pv(), n.motif()), (PhaseNuage::Prelude, 3, 0));
        assert!(n.courant_actif().is_none());
        // Prélude : 90 pas sans rien.
        pas(&mut n, 89);
        assert_eq!(n.phase(), PhaseNuage::Prelude);
        assert!((0..NB_COLONNES).all(|i| n.etat_colonne(i) == EtatColonne::Calme));
        pas(&mut n, 1);
        assert_eq!((n.phase(), n.ecoule()), (PhaseNuage::Colere, 0));
        // 1re salve du motif 1 : annoncée tout de suite, frappe au 48e pas.
        assert!(matches!(n.etat_colonne(0), EtatColonne::Annonce(_)));
        pas(&mut n, 48);
        assert_eq!(n.etat_colonne(0), EtatColonne::Frappe);
        assert!(n.foudroie(&Aabb::new(20.0, 130.0, 10.0, 14.0)));
        assert!(!n.foudroie(&Aabb::new(150.0, 130.0, 10.0, 14.0)));
        // Fin du motif (240 pas) : essoufflé, courant à GAUCHE.
        pas(&mut n, 240 - 48 - 1);
        assert_eq!(n.phase(), PhaseNuage::Colere);
        pas(&mut n, 1);
        assert_eq!(n.phase(), PhaseNuage::Essouffle);
        assert_eq!(n.cote_du_courant(), Cote::Gauche);
        assert_eq!(n.courant_actif().map(|c| c.zone), Some(n.courants[0]));
        assert!(!n.foudroie(&Aabb::new(20.0, 130.0, 10.0, 14.0)));
        assert_eq!(n.essoufflement_restant(), ESSOUFFLE_PAS);
        pas(&mut n, 10);
        assert_eq!(n.essoufflement_restant(), ESSOUFFLE_PAS - 10);
        // Un saut sur la tête : touché, plus de courant, 2 points.
        assert!(n.recoit(&pepin_sur_la_tete(&n)));
        n.toucher();
        assert_eq!((n.phase(), n.pv(), n.motif()), (PhaseNuage::Touche, 2, 1));
        assert!(n.courant_actif().is_none());
        assert_eq!(n.essoufflement_restant(), 0);
        assert!(
            !n.recoit(&pepin_sur_la_tete(&n)),
            "touché : on ne retape pas"
        );
        // Touché 90 pas, puis le motif 2 ; son courant est à DROITE.
        pas(&mut n, TOUCHE_PAS as u32);
        assert_eq!((n.phase(), n.motif()), (PhaseNuage::Colere, 1));
        pas(&mut n, duree_motif(1) as u32);
        assert_eq!(n.phase(), PhaseNuage::Essouffle);
        assert_eq!(n.courant_actif().map(|c| c.zone), Some(n.courants[1]));
        n.toucher();
        pas(&mut n, TOUCHE_PAS as u32 + duree_motif(2) as u32);
        assert_eq!(
            (n.phase(), n.pv(), n.motif()),
            (PhaseNuage::Essouffle, 1, 2)
        );
        assert_eq!(n.cote_du_courant(), Cote::Gauche);
        // Dernier point : le ciel se déchire 120 pas, puis il est vaincu.
        n.toucher();
        assert_eq!((n.phase(), n.pv()), (PhaseNuage::Dechirure, 0));
        assert_eq!(n.progression_dechirure(), 0.0);
        pas(&mut n, 60);
        assert_eq!(n.progression_dechirure(), 0.5);
        assert!(!n.est_vaincu());
        pas(&mut n, 60);
        assert!(n.est_vaincu());
        assert_eq!(n.progression_dechirure(), 1.0);
        pas(&mut n, 1000);
        assert!(n.est_vaincu(), "vaincu pour toujours");
    }

    #[test]
    fn essoufflement_manque_le_meme_motif_recommence() {
        let mut n = nuage();
        pas(&mut n, PRELUDE_PAS as u32 + duree_motif(0) as u32);
        assert_eq!(n.phase(), PhaseNuage::Essouffle);
        pas(&mut n, ESSOUFFLE_PAS as u32);
        assert_eq!((n.phase(), n.motif(), n.pv()), (PhaseNuage::Colere, 0, 3));
    }

    #[test]
    fn la_tete_seulement_et_pendant_l_essoufflement() {
        let mut n = nuage();
        let j = pepin_sur_la_tete(&n);
        assert!(!n.recoit(&j), "en prélude, le nuage ne craint rien");
        n.essouffler();
        assert!(n.recoit(&j));
        // Qui monte (dans le courant) ne tape pas.
        let mut monte = j.clone();
        monte.vitesse.y = -100.0;
        assert!(!n.recoit(&monte));
        // Par-dessous ou de côté (pieds trop bas dans le nuage) : rien.
        let mut dessous = Joueur::new(n.boite.x + 19.0, n.boite.bas() - 6.0);
        dessous.vitesse.y = 36.0;
        assert!(!n.recoit(&dessous));
        // Toucher hors de l'essoufflement ne fait rien.
        let mut calme = nuage();
        calme.toucher();
        assert_eq!((calme.pv(), calme.phase()), (3, PhaseNuage::Prelude));
    }

    #[test]
    fn une_mort_ramene_au_prelude_sans_rendre_les_points() {
        let mut n = nuage();
        n.essouffler();
        n.toucher();
        pas(&mut n, TOUCHE_PAS as u32 + 100);
        assert_eq!((n.phase(), n.pv()), (PhaseNuage::Colere, 2));
        n.reprendre();
        assert_eq!((n.phase(), n.ecoule(), n.pv()), (PhaseNuage::Prelude, 0, 2));
        pas(&mut n, PRELUDE_PAS as u32);
        assert_eq!(
            (n.phase(), n.motif()),
            (PhaseNuage::Colere, 1),
            "le motif 2, pas le 1"
        );
        // Pendant la déchirure, une mort ne change rien.
        n.essouffler();
        n.toucher();
        n.essouffler();
        n.toucher();
        assert_eq!(n.phase(), PhaseNuage::Dechirure);
        n.reprendre();
        assert_eq!(n.phase(), PhaseNuage::Dechirure);
    }

    #[test]
    fn decaler_en_avance_ou_en_retard() {
        let mut n = nuage();
        pas(&mut n, PRELUDE_PAS as u32 + 10);
        assert_eq!(n.ecoule(), 10);
        // En avance : comme autant de pas réels.
        let mut avance = n.clone();
        avance.decaler(2);
        let mut reel = n.clone();
        pas(&mut reel, 2);
        assert_eq!(
            (avance.phase(), avance.ecoule()),
            (reel.phase(), reel.ecoule())
        );
        // En retard : il recule dans la phase, jamais avant son début.
        n.decaler(-3);
        assert_eq!((n.phase(), n.ecoule()), (PhaseNuage::Colere, 7));
        n.decaler(-20);
        assert_eq!((n.phase(), n.ecoule()), (PhaseNuage::Colere, 0));
        // Une avance passe d'une phase à l'autre.
        n.decaler(i32::from(duree_motif(0)) + 5);
        assert_eq!((n.phase(), n.ecoule()), (PhaseNuage::Essouffle, 5));
    }

    /// Chaque motif se traverse en courant, d'où qu'on parte au début du calme
    /// qui le précède (prélude ou touché, 90 pas). Calcul à rebours : `sur[q]` dit
    /// si, à la place `q` et au pas `t`, on peut tenir jusqu'à la fin du motif en
    /// se déplaçant d'au plus 1,5 px par pas (un peu moins que la course, 1,6 px
    /// par pas, pour la mise en vitesse). Places au demi-pixel ; Pépin fait 10 px
    /// de large, chaque colonne d'éclair 12 px au milieu de ses 16.
    #[test]
    fn chaque_motif_se_traverse() {
        const DEMI: usize = 2;
        let pepin = 10 * DEMI;
        let places = NB_COLONNES * 16 * DEMI - pepin + 1;
        for motif in 0..MOTIFS.len() {
            let mut sur = vec![true; places];
            for t in (0..PRELUDE_PAS + duree_motif(motif)).rev() {
                let precedent = sur.clone();
                for (q, s) in sur.iter_mut().enumerate() {
                    let (de, a) = (q.saturating_sub(3), (q + 3).min(places - 1));
                    *s = precedent[de..=a].iter().any(|&p| p);
                }
                if t >= PRELUDE_PAS {
                    for i in 0..NB_COLONNES {
                        let etat = etat_colonne(motif, t - PRELUDE_PAS, i as i8 - PORTEE_COLONNES);
                        if etat != EtatColonne::Frappe {
                            continue;
                        }
                        let gauche = (16 * i + 2) * DEMI;
                        let droite = gauche + 12 * DEMI;
                        // Pépin (de q à q + 10 px) chevauche la colonne (de gauche à droite).
                        for (q, s) in sur.iter_mut().enumerate() {
                            if q < droite && q + pepin > gauche {
                                *s = false;
                            }
                        }
                    }
                }
            }
            if let Some(q) = sur.iter().position(|&s| !s) {
                panic!(
                    "motif {} : impossible de s'en sortir en partant de x = {}",
                    motif + 1,
                    q as f32 / DEMI as f32
                );
            }
        }
    }
}
