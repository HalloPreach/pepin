//! La Pieuvre du port (lettre `8`) : le boss du Monde 2 (carte 9.12a).
//!
//! Demande d'AGA : « ses tentacules frappent par colonnes annoncées, on lui saute
//! dessus quand elle sort la tête. » Elle vit SOUS l'eau d'un bassin qui fait toute
//! la largeur de l'arène ; Pépin se tient sur des pontons et des caisses (ou en
//! barque, carte 9.2).
//!
//! Même modèle que le Grand Nuage (`super::nuage`) : une machine à états PURE, en
//! entiers (pas de 1/60 s), le même pas donne toujours la même pieuvre.
//!
//! ```text
//!   prélude (90) ──► colère (son motif) ──► remous (36) ──► tête sortie (120) ──► colère...
//!                                                                │ Pépin lui saute sur la tête
//!                                                                ▼
//!                                                            touchée (60) ──► colère (motif suivant)
//!                                                                │ c'était son dernier point
//!                                                                ▼
//!                                                            plongée (120) ──► vaincue (niveau fini)
//! ```
//!
//! - colère : un MOTIF de tentacules sur les 9 colonnes du bassin (2 tuiles de
//!   large chacune). Chaque tentacule est d'abord ANNONCÉ (72 pas, 1,2 s : remous
//!   et ombre sur l'eau, puis le bout du tentacule qui pointe), sans danger ; puis
//!   il se DRESSE de toute la hauteur (30 pas, 0,5 s) : mortel ; puis il redescend
//!   (12 pas), sans danger. Un motif par point perdu (`MOTIFS`), de plus en plus
//!   gros et rapide : 4 colonnes sur 9 par salve, puis 6, puis 7 en VAGUE qui
//!   balaie le bassin, aller, un souffle, puis retour (carte 9.12d : plus lente
//!   et lisible pour un humain, nettement sous l'orage à la mesure).
//! - remous : la tête remonte (l'eau bouillonne à sa place), sans danger.
//! - tête sortie : la tête est posée sur l'eau, au milieu ou d'un côté
//!   (`PLACES_TETE`), 2 s. Lui tomber dessus la touche (`recoit`, même règle que
//!   l'écrasement d'un ennemi) ; la toucher AUTREMENT est mortel (`blesse`).
//! - touchée : elle grimace, sans danger, puis replonge et reprend, plus vite.
//! - plongée : à 0 point de vie, elle replonge pour de bon et lâche une clé ; à la
//!   fin, elle est vaincue et le niveau est fini (`est_vaincue`), comme le cyclone.
//!
//! Si Pépin meurt, elle reprend au prélude (`reprendre`) mais GARDE ses points
//! perdus. MODE FACILE (carte 8.14, même mécanisme que le nuage) : `passer_en_facile`
//! lui donne `PV_PIEUVRE_FACILE` points de vie et seulement les deux premiers motifs.
//!
//! Notions Rust : on RÉUTILISE la `Salve` du Grand Nuage (`use super::nuage::Salve`)
//! au lieu de la recopier ; un `enum` à données (`EtatTentacule::Annonce(f32)`) ; des
//! tableaux de taille fixe (`[Aabb; NB_COLONNES]`) construits par le parseur.

use super::nuage::Salve;
use super::Actualisable;
use crate::aabb::Aabb;
use crate::entities::contacts::MARGE_ECRASEMENT;
use crate::physics::Environnement;
use crate::player::Joueur;

/// Points de vie de la Pieuvre : trois sauts sur la tête.
pub const PV_PIEUVRE: u8 = 3;
/// Points de vie en MODE FACILE (carte 8.14) : deux sauts suffisent.
pub const PV_PIEUVRE_FACILE: u8 = 2;
/// Prélude : 1,5 s de calme avant le premier motif (et après une mort).
pub const PRELUDE_PAS: u16 = 90;
/// Annonce d'un tentacule : 1,2 s (remous et ombre, puis le bout qui pointe).
/// Carte 9.12d (AGA : « la pieuvre est trop rapide, c'est plus dur que l'orage
/// pour un humain ») : 0,8 s auparavant, le robot réagit à l'image près, un humain
/// non (voir `un_humain_a_le_temps_de_changer_de_colonne`).
pub const ANNONCE_PAS: u16 = 72;
/// Moitié de l'annonce : avant, remous et ombre ; après, le bout du tentacule pointe.
pub const POINTE_PAS: u16 = ANNONCE_PAS / 2;
/// Tentacule dressé de toute la hauteur : 0,5 s, mortel.
pub const DRESSE_PAS: u16 = 30;
/// Tentacule qui redescend : 0,2 s, sans danger.
pub const REDESCENTE_PAS: u16 = 12;
/// Durée d'une salve : annonce, frappe, redescente.
pub const SALVE_PAS: u16 = ANNONCE_PAS + DRESSE_PAS + REDESCENTE_PAS;
/// La tête remonte : 0,6 s de remous à sa place, sans danger (c'est son annonce).
pub const REMOUS_PAS: u16 = 36;
/// La tête est sortie, posée sur l'eau : 2 s pour lui sauter dessus.
pub const TETE_PAS: u16 = 120;
/// Touchée : 1,5 s de grimace, sans danger (carte 9.12d : un vrai souffle après
/// chaque coup reçu, 1 s auparavant).
pub const TOUCHEE_PAS: u16 = 90;
/// Vaincue : elle replonge pendant 2 s, puis le niveau est fini.
pub const PLONGEE_PAS: u16 = 120;
/// Taille de la tête (px) : trois cases de large, deux de haut.
pub const TAILLE_TETE: (f32, f32) = (48.0, 32.0);
/// Colonnes de tentacules : de 4 colonnes à gauche de celle du `8` à 4 à droite.
pub const PORTEE_COLONNES: i8 = 4;
/// Nombre de colonnes de tentacules (9).
pub const NB_COLONNES: usize = 2 * PORTEE_COLONNES as usize + 1;
/// Largeur (px) d'une colonne : deux tuiles.
pub const LARGEUR_COLONNE: f32 = 32.0;
/// Marge (px) de chaque côté du tentacule dans sa colonne : son corps fait 24 px.
/// Entre deux colonnes voisines, il reste 8 px : moins que Pépin (10 px), on ne
/// s'abrite donc pas « entre deux » : la colonne annoncée est la colonne qui tue.
pub const MARGE_TENTACULE: f32 = 4.0;
/// Places de la tête, en colonnes depuis celle du `8` : au milieu après le motif 1,
/// à gauche après le 2, à droite après le 3 (il faut traverser l'arène).
pub const PLACES_TETE: [i8; 3] = [0, -3, 3];

/// Les trois motifs, un par point perdu, de plus en plus gros et rapides. Les
/// bandes sont en COLONNES depuis celle du `8` (-4 tout à gauche, 4 tout à droite).
///
/// Carte 9.12d (AGA, après avoir joué : « la pieuvre est trop rapide, c'est plus
/// dur que l'orage pour un humain ») : même principe que la 9.12c (« un peu moins
/// dure que le grand nuage, qui accélère au fur et à mesure qu'elle perd de la
/// vie et qui fait des attaques plus grosses »), mais partant de plus bas : une
/// salve toutes les 100, 78, puis 60 pas (66, 48, 30 auparavant), annonce de
/// 1,2 s, une salve de moins par motif, et un souffle de 140 pas entre l'aller et
/// le retour de la vague. Part du bassin frappée par salve : 44 %, 67 %, 78 %.
/// Réglés à la MESURE (`difficulte::tests::mesure_d_un_niveau_du_monde_2`,
/// CLE=pieuvre) : l'Habitué y meurt nettement moins qu'à l'orage.
/// - Motif 1 : 4 colonnes sur 9, une salve toutes les 100 pas : les bords, puis le
///   milieu, puis les bords... on fait la navette sur le platelage.
/// - Motif 2 : 6 colonnes sur 9, toutes les 78 pas : deux ou trois bandes, les
///   trous sautent d'un côté à l'autre, d'une colonne à la fois.
/// - Motif 3 : 7 colonnes sur 9, toutes les 60 pas : deux trous d'une colonne se
///   rejoignent au milieu ; ce trou de 2 colonnes glisse d'une colonne par salve
///   vers la droite (l'ALLER de la VAGUE), elle souffle, puis il revient vers la
///   gauche (le RETOUR), et la tête sort à droite : on retraverse l'arène.
///
/// Toujours une issue : le test `chaque_motif_se_traverse` (le raisonnement du
/// nuage, plus strict) vérifie qu'on peut toujours s'en sortir en courant, et
/// `un_humain_a_le_temps_de_changer_de_colonne` qu'un humain a le temps de voir
/// l'annonce avant de partir.
pub const MOTIFS: [&[Salve]; PV_PIEUVRE as usize] = [
    &[
        // Le bord gauche et deux colonnes à droite : au milieu.
        Salve {
            debut: 0,
            bandes: &[(-4, -3), (2, 2), (4, 4)],
        },
        // Le milieu : sur un bord.
        Salve {
            debut: 100,
            bandes: &[(-2, 1)],
        },
        // Les bords : au milieu.
        Salve {
            debut: 200,
            bandes: &[(-4, -3), (3, 4)],
        },
        // Le milieu : sur un bord (la tête sort ensuite au milieu).
        Salve {
            debut: 300,
            bandes: &[(-1, 2)],
        },
    ],
    &[
        Salve {
            debut: 0,
            bandes: &[(-4, -3), (0, 1), (3, 4)],
        },
        Salve {
            debut: 78,
            bandes: &[(-4, -2), (0, 2)],
        },
        Salve {
            debut: 156,
            bandes: &[(-4, -3), (-1, 1), (4, 4)],
        },
        Salve {
            debut: 234,
            bandes: &[(-4, -2), (1, 2), (4, 4)],
        },
        // La gauche et la droite : au milieu, puis à gauche (la tête y sort).
        Salve {
            debut: 312,
            bandes: &[(-4, -3), (0, 1), (3, 4)],
        },
    ],
    &[
        // Deux trous d'une colonne, à 2 colonnes au plus de chaque tentacule...
        Salve {
            debut: 0,
            bandes: &[(-4, -3), (-1, 1), (3, 4)],
        },
        // ... qui se rejoignent au milieu (100 pas : le temps d'y courir).
        Salve {
            debut: 100,
            bandes: &[(-4, -2), (1, 4)],
        },
        // L'aller : le trou glisse vers la droite, une colonne toutes les 60 pas...
        Salve {
            debut: 160,
            bandes: &[(-4, -1), (2, 4)],
        },
        Salve {
            debut: 220,
            bandes: &[(-4, 0), (3, 4)],
        },
        // ... elle souffle (140 pas), puis le retour, vers la gauche (la tête sort
        // ensuite à droite).
        Salve {
            debut: 360,
            bandes: &[(-4, -1), (2, 4)],
        },
        Salve {
            debut: 420,
            bandes: &[(-4, -2), (1, 4)],
        },
    ],
];

/// Durée (pas) du motif `motif` : le début de sa dernière salve, plus une salve.
pub fn duree_motif(motif: usize) -> u16 {
    MOTIFS[motif].iter().map(|s| s.debut).max().unwrap_or(0) + SALVE_PAS
}

/// Ce que fait le tentacule `decalage` (de -4 à 4) au pas `t` du motif `motif`.
/// Si deux salves se chevauchent, le tentacule dressé l'emporte. Fonction pure.
pub fn etat_tentacule(motif: usize, t: u16, decalage: i8) -> EtatTentacule {
    let mut etat = EtatTentacule::Calme;
    for s in MOTIFS[motif].iter().filter(|s| s.couvre(decalage)) {
        if t < s.debut || t >= s.debut + SALVE_PAS {
            continue;
        }
        let dans = t - s.debut;
        if dans < ANNONCE_PAS {
            etat = EtatTentacule::Annonce(dans as f32 / ANNONCE_PAS as f32);
        } else if dans < ANNONCE_PAS + DRESSE_PAS {
            return EtatTentacule::Dresse;
        } else {
            let fin = dans - ANNONCE_PAS - DRESSE_PAS;
            etat = EtatTentacule::Redescente(fin as f32 / REDESCENTE_PAS as f32);
        }
    }
    etat
}

/// L'état d'un tentacule pendant un pas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatTentacule {
    /// Rien ne sort de l'eau.
    Calme,
    /// Annoncé (sans danger) : avancement de l'annonce, de 0.0 à 1.0 (remous et
    /// ombre jusqu'à 0.5, le bout du tentacule ensuite).
    Annonce(f32),
    /// Dressé de toute la hauteur : mortel.
    Dresse,
    /// Il redescend (sans danger) : avancement, de 0.0 à 1.0.
    Redescente(f32),
}

/// Phase de la Pieuvre (voir le schéma en tête de module).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhasePieuvre {
    Prelude,
    Colere,
    Remous,
    Tete,
    Touchee,
    Plongee,
    Vaincue,
}

#[derive(Debug, Clone)]
pub struct Pieuvre {
    /// Les 9 colonnes de tentacules (le corps du tentacule, 24 px de large), de la
    /// surface de l'eau jusqu'en haut de la carte, de -4 à 4.
    pub colonnes: [Aabb; NB_COLONNES],
    /// Les trois places de la tête (48 x 32, posée sur l'eau), dans l'ordre de
    /// `PLACES_TETE` : milieu, gauche, droite.
    pub places: [Aabb; 3],
    phase: PhasePieuvre,
    /// Pas écoulés depuis le début de la phase.
    ecoule: u16,
    pv: u8,
    pv_max: u8,
    /// Place de la tête (indice dans `places`), choisie quand elle remonte.
    place: usize,
}

impl Pieuvre {
    /// Une pieuvre neuve, au prélude, avec ses 3 points de vie.
    pub fn new(colonnes: [Aabb; NB_COLONNES], places: [Aabb; 3]) -> Self {
        Self {
            colonnes,
            places,
            phase: PhasePieuvre::Prelude,
            ecoule: 0,
            pv: PV_PIEUVRE,
            pv_max: PV_PIEUVRE,
            place: 0,
        }
    }

    /// Mode facile (carte 8.14) : `PV_PIEUVRE_FACILE` points de vie, donc les deux
    /// premiers motifs seulement. À appeler sur une pieuvre neuve.
    pub fn passer_en_facile(&mut self) {
        self.pv_max = PV_PIEUVRE_FACILE;
        self.pv = self.pv.min(PV_PIEUVRE_FACILE);
    }

    pub fn pv(&self) -> u8 {
        self.pv
    }

    pub fn pv_max(&self) -> u8 {
        self.pv_max
    }

    /// Le combat est-il en mode facile ? (Le dessin la pâlit, comme le nuage.)
    pub fn est_facile(&self) -> bool {
        self.pv_max < PV_PIEUVRE
    }

    pub fn phase(&self) -> PhasePieuvre {
        self.phase
    }

    /// Pas écoulés depuis le début de la phase en cours.
    pub fn ecoule(&self) -> u16 {
        self.ecoule
    }

    /// Le motif joué en colère : le nombre de points déjà perdus depuis `pv_max`.
    pub fn motif(&self) -> usize {
        usize::from(self.pv_max - self.pv.clamp(1, self.pv_max))
    }

    /// Durée de la phase en cours (`None` : vaincue, pour toujours).
    pub fn duree_phase(&self) -> Option<u16> {
        match self.phase {
            PhasePieuvre::Prelude => Some(PRELUDE_PAS),
            PhasePieuvre::Colere => Some(duree_motif(self.motif())),
            PhasePieuvre::Remous => Some(REMOUS_PAS),
            PhasePieuvre::Tete => Some(TETE_PAS),
            PhasePieuvre::Touchee => Some(TOUCHEE_PAS),
            PhasePieuvre::Plongee => Some(PLONGEE_PAS),
            PhasePieuvre::Vaincue => None,
        }
    }

    /// La place de la tête (sa boîte), de la remontée à la plongée ; `None` quand
    /// elle est sous l'eau.
    pub fn tete(&self) -> Option<Aabb> {
        match self.phase {
            PhasePieuvre::Remous
            | PhasePieuvre::Tete
            | PhasePieuvre::Touchee
            | PhasePieuvre::Plongee => Some(self.places[self.place]),
            _ => None,
        }
    }

    /// L'état du tentacule `i` (0 = le plus à gauche) : toujours `Calme` hors de la
    /// colère.
    pub fn etat_tentacule(&self, i: usize) -> EtatTentacule {
        if self.phase != PhasePieuvre::Colere || i >= NB_COLONNES {
            return EtatTentacule::Calme;
        }
        etat_tentacule(self.motif(), self.ecoule, i as i8 - PORTEE_COLONNES)
    }

    /// Pépin lui tombe-t-il sur la tête, sortie ? Même règle que l'écrasement d'un
    /// ennemi : il descend, et ses pieds ne dépassent pas de plus de
    /// `MARGE_ECRASEMENT` le dessus de la tête.
    pub fn recoit(&self, joueur: &Joueur) -> bool {
        let tete = self.places[self.place];
        self.phase == PhasePieuvre::Tete
            && joueur.vitesse.y > 0.0
            && joueur.boite.chevauche(&tete)
            && joueur.boite.bas() - tete.y <= MARGE_ECRASEMENT
    }

    /// `boite` meurt-elle ? Un tentacule dressé la touche, ou la tête sortie la
    /// touche (autrement qu'en tombant dessus : `recoit` est vérifié avant).
    pub fn blesse(&self, boite: &Aabb) -> bool {
        let tete = self.phase == PhasePieuvre::Tete && boite.chevauche(&self.places[self.place]);
        tete || (0..NB_COLONNES).any(|i| {
            self.etat_tentacule(i) == EtatTentacule::Dresse && self.colonnes[i].chevauche(boite)
        })
    }

    /// Un saut sur la tête : un point en moins ; touchée, ou, au dernier point, la
    /// plongée. Sans effet si la tête n'est pas sortie.
    pub fn toucher(&mut self) {
        if self.phase != PhasePieuvre::Tete {
            return;
        }
        self.pv = self.pv.saturating_sub(1);
        let phase = if self.pv == 0 {
            PhasePieuvre::Plongee
        } else {
            PhasePieuvre::Touchee
        };
        self.entrer(phase);
    }

    /// Pépin est mort : elle reprend au prélude, avec ses points perdus. Sans effet
    /// quand elle plonge déjà pour de bon (ou une fois vaincue).
    pub fn reprendre(&mut self) {
        if !matches!(self.phase, PhasePieuvre::Plongee | PhasePieuvre::Vaincue) {
            self.entrer(PhasePieuvre::Prelude);
        }
    }

    /// La tête sort tout de suite, à la place `place` (robot de faisabilité, qui ne
    /// connaît pas le temps : un joueur patient attend qu'elle sorte ; tests).
    pub fn sortir_la_tete(&mut self, place: usize) {
        if self.pv > 0 {
            self.entrer(PhasePieuvre::Tete);
            self.place = place.min(self.places.len() - 1);
        }
    }

    /// Est-elle vaincue (le niveau est fini) ?
    pub fn est_vaincue(&self) -> bool {
        self.phase == PhasePieuvre::Vaincue
    }

    /// Avancement de la plongée finale, de 0.0 à 1.0 (1.0 une fois vaincue).
    pub fn progression_plongee(&self) -> f32 {
        match self.phase {
            PhasePieuvre::Plongee => self.ecoule as f32 / PLONGEE_PAS as f32,
            PhasePieuvre::Vaincue => 1.0,
            _ => 0.0,
        }
    }

    /// Décale la pieuvre de `pas` pas dans le temps (en avance si positif, en retard
    /// dans la phase en cours si négatif), comme `GrandNuage::decaler` : le robot de
    /// QA vérifie qu'un plan tient à une ou deux images près.
    pub fn decaler(&mut self, pas: i32) {
        if pas >= 0 {
            for _ in 0..pas {
                self.avancer();
            }
        } else {
            self.ecoule = self.ecoule.saturating_sub(pas.unsigned_abs() as u16);
        }
    }

    fn avancer(&mut self) {
        let Some(duree) = self.duree_phase() else {
            return; // vaincue : plus rien ne bouge
        };
        self.ecoule = self.ecoule.saturating_add(1);
        if self.ecoule < duree {
            return;
        }
        let suivante = match self.phase {
            PhasePieuvre::Prelude | PhasePieuvre::Tete | PhasePieuvre::Touchee => {
                PhasePieuvre::Colere
            }
            PhasePieuvre::Colere => {
                // La tête remonte à la place du motif qui vient de finir.
                self.place = self.motif() % PLACES_TETE.len();
                PhasePieuvre::Remous
            }
            PhasePieuvre::Remous => PhasePieuvre::Tete,
            PhasePieuvre::Plongee | PhasePieuvre::Vaincue => PhasePieuvre::Vaincue,
        };
        self.entrer(suivante);
    }

    fn entrer(&mut self, phase: PhasePieuvre) {
        self.phase = phase;
        self.ecoule = 0;
    }
}

impl Actualisable for Pieuvre {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.avancer();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    /// Une pieuvre dont le `8` est en (9, 9) : surface de l'eau à y = 144.
    fn pieuvre() -> Pieuvre {
        let colonnes = std::array::from_fn(|i| {
            Aabb::new(16.0 + 32.0 * i as f32 + MARGE_TENTACULE, 0.0, 24.0, 144.0)
        });
        let places = PLACES_TETE.map(|k| {
            let milieu = 160.0 + 32.0 * f32::from(k);
            Aabb::new(milieu - 24.0, 112.0, TAILLE_TETE.0, TAILLE_TETE.1)
        });
        Pieuvre::new(colonnes, places)
    }

    fn pas(p: &mut Pieuvre, k: u32) {
        let g = GrilleTuiles::vide(4, 4);
        let env = Environnement::simple(&g);
        for _ in 0..k {
            p.actualiser(&env);
        }
    }

    /// La règle centrale : tentacules annoncés puis mortels, et la tête sortie ne se
    /// touche qu'en lui tombant dessus (un point de moins à chaque fois, puis la
    /// plongée) ; la toucher de côté est mortel, et rien ne la touche sous l'eau.
    #[test]
    fn la_tete_ne_se_touche_qu_en_lui_tombant_dessus() {
        let mut p = pieuvre();
        // Prélude, puis le 1er motif : sa 1re salve frappe le bord gauche et deux
        // colonnes à droite, dont la colonne 2 (corps de x = 212 à 236), annoncée 72
        // pas (sans danger), dressée 30 (mortelle), puis elle redescend. La colonne
        // -1 (x = 116 à 140) est libre.
        let sur_la_colonne = Aabb::new(218.0, 114.0, 10.0, 14.0);
        pas(&mut p, PRELUDE_PAS as u32);
        assert_eq!(p.phase(), PhasePieuvre::Colere);
        assert!(matches!(p.etat_tentacule(6), EtatTentacule::Annonce(_)));
        assert!(!p.blesse(&sur_la_colonne));
        pas(&mut p, ANNONCE_PAS as u32);
        assert_eq!(p.etat_tentacule(6), EtatTentacule::Dresse);
        assert!(p.blesse(&sur_la_colonne));
        assert!(!p.blesse(&Aabb::new(122.0, 114.0, 10.0, 14.0)));
        pas(&mut p, DRESSE_PAS as u32);
        assert!(matches!(p.etat_tentacule(6), EtatTentacule::Redescente(_)));
        assert!(!p.blesse(&sur_la_colonne));
        // Sous l'eau, la tête ne se touche pas.
        let mut j = Joueur::new(150.0, 100.0);
        j.vitesse.y = 60.0;
        assert!(!p.recoit(&j));
        // Fin du motif : remous (sans danger), puis la tête sort au milieu.
        pas(&mut p, (duree_motif(0) - ANNONCE_PAS - DRESSE_PAS) as u32);
        assert_eq!(
            (p.phase(), p.tete()),
            (PhasePieuvre::Remous, Some(p.places[0]))
        );
        let tete = p.places[0];
        assert!(!p.blesse(&tete));
        pas(&mut p, REMOUS_PAS as u32);
        assert_eq!(p.phase(), PhasePieuvre::Tete);
        // De côté (Pépin qui marche dedans), ou par-dessous : mortel, pas de point.
        let mut de_cote = Joueur::new(tete.x - 6.0, tete.y + 10.0);
        de_cote.vitesse.x = 90.0;
        assert!(!p.recoit(&de_cote) && p.blesse(&de_cote.boite));
        let mut monte = Joueur::new(tete.x + 19.0, tete.y - 12.0);
        monte.vitesse.y = -100.0;
        assert!(!p.recoit(&monte) && p.blesse(&monte.boite));
        // En lui tombant dessus (pieds 2 px dans le dessus) : touchée.
        let mut tombe = Joueur::new(tete.x + 19.0, tete.y - 12.0);
        tombe.vitesse.y = 120.0;
        assert!(p.recoit(&tombe));
        p.toucher();
        assert_eq!((p.phase(), p.pv()), (PhasePieuvre::Touchee, 2));
        assert!(!p.blesse(&tombe.boite), "touchée, elle ne blesse plus");
        // Une mort de Pépin : elle reprend au prélude mais GARDE le point perdu.
        p.reprendre();
        assert_eq!(
            (p.phase(), p.pv(), p.motif()),
            (PhasePieuvre::Prelude, 2, 1)
        );
        // Les deux derniers coups (la tête sort à gauche, puis à droite), puis la
        // plongée : vaincue, le niveau est fini.
        for (place, pv) in [(1, 1), (2, 0)] {
            pas(&mut p, 1);
            while p.phase() != PhasePieuvre::Tete {
                pas(&mut p, 1);
            }
            assert_eq!(p.tete(), Some(p.places[place]));
            p.toucher();
            assert_eq!(p.pv(), pv);
        }
        assert_eq!(p.phase(), PhasePieuvre::Plongee);
        p.reprendre();
        assert_eq!(
            p.phase(),
            PhasePieuvre::Plongee,
            "la plongée ne s'annule pas"
        );
        pas(&mut p, PLONGEE_PAS as u32);
        assert!(p.est_vaincue());
        // Chaque coup la rend plus rapide : `chaque_point_perdu_frappe_plus_gros_et_plus_vite`
        // (depuis la carte 9.12d, le 3e motif dure plus longtemps : il a un souffle
        // entre l'aller et le retour de la vague).
        // Mode facile : 2 points de vie, seulement les deux premiers motifs.
        let mut f = pieuvre();
        f.passer_en_facile();
        assert_eq!((f.pv(), f.pv_max(), f.motif()), (2, 2, 0));
    }

    /// Part des colonnes frappées par une salve, en moyenne sur le motif (de 0 à 1).
    fn part_frappee(salves: &[Salve], nb_colonnes: usize, portee: i8) -> f32 {
        let frappees: usize = salves
            .iter()
            .map(|s| (-portee..=portee).filter(|&d| s.couvre(d)).count())
            .sum();
        frappees as f32 / (salves.len() * nb_colonnes) as f32
    }

    /// Carte 9.12c (AGA : « un peu moins dure que le grand nuage mais sur le même
    /// principe, qui accélère au fur et à mesure qu'elle perd de la vie et qui fait
    /// des attaques plus grosses ») : à chaque point perdu, la pieuvre frappe PLUS
    /// GROS (part du bassin frappée par salve) et PLUS VITE (écart entre salves).
    /// La difficulté elle-même se MESURE (`difficulte::tests::mesure_des_deux_boss`) :
    /// la part frappée du nuage est affichée pour comparaison, pas imposée. Le
    /// rythme : l'écart le PLUS COURT entre deux salves (carte 9.12d : le souffle
    /// entre l'aller et le retour de la vague du 3e motif n'est pas son rythme).
    #[test]
    fn chaque_point_perdu_frappe_plus_gros_et_plus_vite() {
        use super::super::nuage;
        let mut avant: Option<(f32, u16)> = None;
        for (m, salves) in MOTIFS.iter().enumerate() {
            let pieuvre = part_frappee(salves, NB_COLONNES, PORTEE_COLONNES);
            let grand_nuage =
                part_frappee(nuage::MOTIFS[m], nuage::NB_COLONNES, nuage::PORTEE_COLONNES);
            let ecart = salves
                .windows(2)
                .map(|p| p[1].debut - p[0].debut)
                .min()
                .unwrap();
            println!(
                "motif {} : pieuvre {:.0} % toutes les {ecart} pas, Grand Nuage {:.0} %",
                m + 1,
                100.0 * pieuvre,
                100.0 * grand_nuage
            );
            if let Some((part, ecart_avant)) = avant {
                assert!(pieuvre > part, "motif {} : pas plus gros", m + 1);
                assert!(ecart < ecart_avant, "motif {} : pas plus vite", m + 1);
            }
            avant = Some((pieuvre, ecart));
        }
    }

    /// Toujours une issue (le raisonnement du Grand Nuage, `nuage::tests::
    /// chaque_motif_se_traverse`, en plus strict). Calcul à rebours : `sur[q]` dit
    /// si, à la place `q` et au pas `t`, on peut tenir jusqu'à la fin du motif en se
    /// déplaçant d'au plus 1,5 px par pas (un peu moins que la course, 1,6 px par
    /// pas). Places au demi-pixel ; Pépin fait 10 px de large, chaque tentacule
    /// 24 px au milieu de sa colonne de 32. Deux garanties :
    /// - d'où qu'on parte au début du calme qui précède le motif (le plus court :
    ///   « touchée », 90 pas), on s'en sort ;
    /// - au DÉBUT de chaque frappe, de TOUTE place que cette frappe épargne, on
    ///   tient jusqu'à la fin du motif : la salve suivante a toujours une colonne
    ///   sûre qu'on atteint en courant (aucune impasse, salve après salve).
    ///
    /// Le modèle est celui du nuage (un sol plat où l'on court) : depuis la carte
    /// 9.12c, l'arène a justement un platelage où l'on court, et ses trous d'une
    /// case se sautent à la même vitesse (1,6 px par pas en l'air) ; pas en barque
    /// (lente). La preuve sur la vraie physique est le
    /// robot de QA (`le_robot_bat_la_pieuvre_sans_mourir`), et la mesure de
    /// difficulté (tous les profils finissent).
    #[test]
    fn chaque_motif_se_traverse() {
        const DEMI: usize = 2;
        let pepin = 10 * DEMI;
        let largeur = LARGEUR_COLONNE as usize;
        let marge = MARGE_TENTACULE as usize;
        let corps = largeur - 2 * marge;
        let places = NB_COLONNES * largeur * DEMI - pepin + 1;
        let calme = TOUCHEE_PAS;
        for (motif, salves) in MOTIFS.iter().enumerate() {
            let debuts: Vec<u16> = salves
                .iter()
                .map(|s| calme + s.debut + ANNONCE_PAS)
                .collect();
            let mut sur = vec![true; places];
            for t in (0..calme + duree_motif(motif)).rev() {
                let precedent = sur.clone();
                for (q, s) in sur.iter_mut().enumerate() {
                    let (de, a) = (q.saturating_sub(3), (q + 3).min(places - 1));
                    *s = precedent[de..=a].iter().any(|&p| p);
                }
                let mut frappee = vec![false; places];
                if t >= calme {
                    for i in 0..NB_COLONNES {
                        let etat = etat_tentacule(motif, t - calme, i as i8 - PORTEE_COLONNES);
                        if etat != EtatTentacule::Dresse {
                            continue;
                        }
                        let gauche = (largeur * i + marge) * DEMI;
                        let droite = gauche + corps * DEMI;
                        // Pépin (de q à q + 10 px) chevauche le tentacule.
                        for (q, s) in sur.iter_mut().enumerate() {
                            if q < droite && q + pepin > gauche {
                                *s = false;
                                frappee[q] = true;
                            }
                        }
                    }
                }
                if debuts.contains(&t) {
                    if let Some(q) = (0..places).find(|&q| !frappee[q] && !sur[q]) {
                        panic!(
                            "motif {}, frappe au pas {} : impasse en x = {}",
                            motif + 1,
                            t - calme,
                            q as f32 / DEMI as f32
                        );
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

    /// Carte 9.12d (AGA : « la pieuvre est trop rapide, c'est plus dur que l'orage
    /// pour un humain ») : le robot réagit à l'image près, un humain non. Aucune
    /// salve n'oblige à partir moins de 0,6 s (36 pas) après le début de son
    /// annonce. Pour chaque salve, Pépin est au milieu d'une colonne que la salve
    /// d'avant épargnait (n'importe laquelle pour la première) ; si celle-ci la
    /// frappe, il court (1,6 px par pas) vers une colonne qu'elle épargne : il part
    /// au plus tôt au début de l'annonce, ou quand le dernier tentacule dressé sur
    /// son chemin redescend ; il doit être sorti du tentacule (le bord de sa boîte
    /// passe celui du corps, 15 px de moins que la distance des milieux) quand la
    /// colonne tue. Le temps entre le départ possible et le dernier moment pour
    /// partir est sa marge : au moins 36 pas, vers la meilleure colonne.
    #[test]
    fn un_humain_a_le_temps_de_changer_de_colonne() {
        const REACTION_PAS: f32 = 36.0;
        const COURSE: f32 = 1.6;
        let colonnes = || -PORTEE_COLONNES..=PORTEE_COLONNES;
        for (motif, salves) in MOTIFS.iter().enumerate() {
            for (j, salve) in salves.iter().enumerate() {
                let tue = salve.debut + ANNONCE_PAS;
                let departs = colonnes().filter(|&c| j == 0 || !salves[j - 1].couvre(c));
                for c in departs.filter(|&c| salve.couvre(c)) {
                    let marge = colonnes()
                        .filter(|&d| !salve.couvre(d))
                        .map(|d| {
                            // Le chemin, de la colonne voisine de `c` jusqu'à `d` : un
                            // tentacule d'une autre salve dressé dessus pendant
                            // l'annonce retarde le départ.
                            let (de, a) = if d > c { (c + 1, d) } else { (d, c - 1) };
                            let depart = salves
                                .iter()
                                .filter(|k| (de..=a).any(|e| k.couvre(e)))
                                .map(|k| {
                                    (k.debut + ANNONCE_PAS, k.debut + ANNONCE_PAS + DRESSE_PAS)
                                })
                                .filter(|&(dresse, fin)| dresse < tue && fin > salve.debut)
                                .map(|(_, fin)| fin)
                                .fold(salve.debut, u16::max);
                            let trajet =
                                (LARGEUR_COLONNE * f32::from((d - c).abs()) - 15.0) / COURSE;
                            f32::from(tue) - trajet - f32::from(depart)
                        })
                        .fold(f32::MIN, f32::max);
                    assert!(
                        marge >= REACTION_PAS,
                        "motif {}, salve {} (pas {}) : depuis la colonne {c}, {marge:.0} pas \
                         pour réagir",
                        motif + 1,
                        j + 1,
                        salve.debut
                    );
                }
            }
        }
    }
}
