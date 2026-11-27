//! La phase 2 de la Pieuvre du port : la RETRAITE et la MER (cartes 9.13a et
//! 9.36a).
//!
//! Décision d'AGA (27/11, carte 9.36a) : « je pense supprimer la p2 telle qu'elle
//! est actuellement, et juste faire une montée contre l'eau qui avance plus vite
//! avec des objets qui tombent du ciel envoyés par la pieuvre. on ne la tue pas à
//! la fin, comme le nuage juste elle finit par s'enfuir ».
//!
//! ```text
//!   phase 1 (inchangée) ── 3e coup ──► FURIE (2 s) ──► MONTÉE ── le sommet ──► fuite ──► vaincue
//!                                       elle se retire   la mer monte, les        elle plonge au
//!                                       au large, la     objets tombent           large, la mer
//!                                       bâche se déchire (`montee`)               redescend
//! ```
//!
//! - FURIE (`FURIE_PAS`, 2 s, sans danger hors de l'eau) : vaincue en phase 1,
//!   elle hurle et SE RETIRE AU LARGE ; l'écran tremble. La BÂCHE (le plafond de
//!   l'arène, `_` dans la grille : une boîte solide pendant la phase 1, pour qu'on
//!   ne monte pas avant) se déchire, et une TROMBE (un courant ascendant, la règle
//!   des bulles) monte du bassin, au-dessus du trou du milieu, jusqu'à la
//!   corniche du bas de la tour. À la fin de la furie, le QUAI de l'arène casse :
//!   toutes ses plateformes tombent à l'eau.
//! - MONTÉE : la mer monte (`surface_mer`), d'abord vite (la CRUE remplit l'arène
//!   jusqu'à la bâche), puis d'une tuile par seconde (`VITESSE_MER`), sans palier,
//!   jusque sous le sommet (`arret_mer`). Sa surface TUE au contact, BARQUE
//!   COMPRISE (`blesse_en_fuite` regarde seulement si les pieds passent sous la
//!   surface). Elle SUIT Pépin et ne descend jamais : voir `montee`.
//! - LA FUITE : Pépin au sommet, elle plonge et disparaît au large
//!   (`PLONGEE_PAS`, 2 s) pendant que la mer REDESCEND (`DESCENTE_MER`), puis
//!   elle est « vaincue » : le niveau est fini comme avant (bilan, gouttes,
//!   déblocage, sauvegarde). On ne la tue pas.
//!
//! Une mort pendant la phase 2 ramène à la dernière lanterne de la tour : celle
//! du bas s'allume d'elle-même dès la furie (`entities::contacts`), et
//! `montee::reprendre_a` remet la mer sous la lanterne.
//!
//! MODE FACILE : la mer monte 0,75 fois moins vite (`FACTEUR_FACILE`).
//!
//! Notions Rust : un sous-module qui ajoute des méthodes à la structure de son
//! parent (un second bloc `impl Pieuvre`) et voit ses champs privés ; une
//! fonction pure du temps (`surface_mer_a`).

use super::{PhasePieuvre, Pieuvre, PLONGEE_PAS};
use crate::aabb::{Aabb, Grille, Tuile, TAILLE_TUILE};
use crate::physics::CourantAscendant;

/// La furie : 2 s, elle hurle et se retire ; l'écran tremble ; le quai casse à
/// la fin.
pub const FURIE_PAS: u16 = 120;
/// La crue qui remplit l'arène (px par pas) : une tuile en 0,5 s.
pub const VITESSE_CRUE: f32 = TAILLE_TUILE / 30.0;
/// Carte 9.36a : la mer dans la tour (px par pas), une tuile par seconde (la
/// 9.13f : une tuile toutes les 1,3 s). C'est la menace principale ; réglée par
/// la mesure de difficulté (`CLE=pieuvre`).
pub const VITESSE_MER: f32 = TAILLE_TUILE / 70.0;
/// Mode facile : la mer monte 0,75 fois moins vite.
pub const FACTEUR_FACILE: f32 = 0.75;
/// La mer s'arrête 3 tuiles sous le sommet.
pub const SOUS_LE_SOMMET: f32 = 3.0 * TAILLE_TUILE;
/// À la fuite, la mer redescend de 6 tuiles pendant la plongée.
pub const DESCENTE_MER: f32 = 6.0 * TAILLE_TUILE;

impl Pieuvre {
    /// La phase 2 commence (le dernier coup de la phase 1) : plus de saisie.
    pub(super) fn commencer_la_fuite(&mut self) {
        self.fuite = true;
        self.saisie = None;
        self.remettre_la_tour();
    }

    /// La phase 2 a-t-elle commencé ?
    pub fn en_fuite(&self) -> bool {
        self.fuite
    }

    /// La bâche, solide tant qu'elle n'est pas déchirée (`Entite::solide`).
    pub fn bache_solide(&self) -> Option<Aabb> {
        self.bache.filter(|_| !self.fuite)
    }

    /// Le quai de l'arène (les plateformes du bassin) ne porte plus, de la fin de
    /// la furie à la fin du niveau.
    pub(super) fn cases_de_la_fuite(&self) -> Vec<(i32, i32)> {
        if self.quai_casse() {
            self.plateformes.iter().flatten().copied().collect()
        } else {
            Vec::new()
        }
    }

    fn quai_casse(&self) -> bool {
        self.fuite && self.phase != PhasePieuvre::Furie
    }

    /// La case `(tx, ty)` du quai est-elle tombée à l'eau ? (Le dessin la cache.)
    pub fn case_engloutie(&self, tx: i32, ty: i32) -> bool {
        self.quai_casse() && self.plateformes.iter().any(|p| p.contains(&(tx, ty)))
    }

    /// La surface du bassin : là où part la mer.
    fn depart_mer(&self) -> f32 {
        self.colonnes[0].bas()
    }

    /// Vitesse de la mer dans la tour (px par pas), mode facile compris.
    pub(super) fn vitesse_mer(&self) -> f32 {
        if self.est_facile() {
            VITESSE_MER * FACTEUR_FACILE
        } else {
            VITESSE_MER
        }
    }

    /// Pas de la montée où finit la crue (la mer arrive sous la bâche).
    pub fn fin_de_crue(&self) -> u16 {
        let haut = self.bache.map_or(self.depart_mer(), |b| b.bas());
        ((self.depart_mer() - haut) / VITESSE_CRUE).ceil() as u16
    }

    /// Ordonnée (px) de la mer au pas `t` de son horloge. Fonction pure : la crue
    /// jusqu'à la bâche, puis la montée régulière (`vitesse_mer`), jusqu'à
    /// `arret_mer`. Elle ne descend jamais quand `t` grandit.
    pub fn surface_mer_a(&self, t: u16) -> f32 {
        let crue = self.fin_de_crue();
        if t < crue {
            return self.depart_mer() - f32::from(t) * VITESSE_CRUE;
        }
        let y = self.bache.map_or(self.depart_mer(), |b| b.bas());
        (y - f32::from(t - crue) * self.vitesse_mer()).max(self.arret_mer)
    }

    /// La mer en ce moment (`None` hors de la phase 2). Elle tue au contact,
    /// pendant la furie et la montée.
    pub fn surface_mer(&self) -> Option<f32> {
        if !self.fuite {
            return None;
        }
        Some(match self.phase {
            PhasePieuvre::Montee => self.surface_mer_a(self.horloge_mer()),
            // La fuite (carte 9.36a) : la mer redescend de `DESCENTE_MER`
            // depuis là où elle était, pendant la plongée.
            PhasePieuvre::Plongee | PhasePieuvre::Vaincue => {
                let k = (f32::from(self.ecoule()) / f32::from(PLONGEE_PAS)).min(1.0);
                let k = if self.phase == PhasePieuvre::Vaincue {
                    1.0
                } else {
                    k
                };
                self.arret_mer + k * DESCENTE_MER
            }
            _ => self.depart_mer(),
        })
    }

    /// La fuite commence : la mer se fige où elle est (elle redescendra d'ici).
    pub(super) fn figer_la_mer(&mut self) {
        if let Some(s) = self.surface_mer() {
            self.arret_mer = s;
        }
    }

    /// La place de la trombe : au-dessus du trou du milieu (la place du milieu de
    /// la tête), du bassin jusqu'à la corniche du bas de la tour (2 tuiles au-dessus
    /// de la bâche). `None` sans phase 2.
    pub fn zone_trombe(&self) -> Option<Aabb> {
        let bache = self.bache?;
        self.sommet?;
        let milieu = self.places[0];
        let haut = bache.y - 2.0 * TAILLE_TUILE;
        Some(Aabb::new(
            milieu.x,
            haut,
            milieu.w,
            self.depart_mer() - haut,
        ))
    }

    /// La trombe, pendant la phase 2 (un courant ascendant : la règle des bulles).
    pub fn trombe(&self) -> Option<CourantAscendant> {
        let en_cours =
            self.fuite && matches!(self.phase, PhasePieuvre::Furie | PhasePieuvre::Montee);
        self.zone_trombe()
            .filter(|_| en_cours)
            .map(CourantAscendant::new)
    }

    /// Phase 2 : la mer en furie touche-t-elle `boite` (ses pieds sous la
    /// surface) ? Pendant la furie et la montée seulement : à la fuite, elle
    /// redescend, calmée.
    pub(super) fn blesse_en_fuite(&self, boite: &Aabb) -> bool {
        matches!(self.phase, PhasePieuvre::Furie | PhasePieuvre::Montee)
            && self.surface_mer().is_some_and(|mer| boite.bas() > mer)
    }

    /// Les appuis de la tour au-dessus de la bâche : `(dessus, gauche, droite)` en
    /// px, dans les colonnes du bassin.
    pub fn appuis(&self, grille: &dyn Grille) -> Vec<(f32, f32, f32)> {
        let t = TAILLE_TUILE;
        let x0 = ((self.colonnes[0].x - super::MARGE_TENTACULE) / t) as i32;
        let x1 = ((self.colonnes[self.colonnes.len() - 1].droite() + super::MARGE_TENTACULE) / t)
            as i32
            - 1;
        let bas = self.bache.map_or(0, |b| (b.y / t) as i32);
        let solide = |x: i32, y: i32| grille.tuile(x, y) == Tuile::Solide;
        let mut appuis = Vec::new();
        for y in 1..bas {
            let mut x = x0;
            while x <= x1 {
                if solide(x, y) && !solide(x, y - 1) {
                    let debut = x;
                    while x <= x1 && solide(x, y) && !solide(x, y - 1) {
                        x += 1;
                    }
                    appuis.push((y as f32 * t, debut as f32 * t, x as f32 * t));
                } else {
                    x += 1;
                }
            }
        }
        appuis
    }

    /// Pour le robot de faisabilité, qui ne connaît pas le temps : la pieuvre est
    /// déjà en montée ; la mer ne monte pas pour lui et rien ne tombe, et entrer
    /// dans le sommet compte comme un coup (un joueur patient de la phase 1
    /// attend la furie ; la vraie montée est vérifiée par le pilote et les tests
    /// du niveau). Sans tour, rien ne change.
    pub fn pour_le_robot(&mut self) {
        if self.sommet.is_some() {
            self.entrer(PhasePieuvre::Montee);
            self.tour.pour_le_robot();
        }
    }
}
