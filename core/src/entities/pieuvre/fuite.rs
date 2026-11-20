//! La phase 2 de la Pieuvre du port, « fuite et montée » (carte 9.13a).
//!
//! Demande d'AGA : « J'aime pas la pieuvre, elle manque toujours de quelque chose
//! pour un boss final… Une phase deux ? ». Il a choisi : « Fuite et montée : elle
//! casse le quai, la mer monte, on grimpe sur des caisses et des mâts en fuyant
//! les tentacules qui montent par-dessous, et on l'achève en haut (caméra
//! verticale). »
//!
//! ```text
//!   phase 1 (inchangée) ── 3e coup ──► FURIE (2 s) ──► MONTÉE ── coup final ──► plongée ──► vaincue
//!                                       la bâche se       la mer monte, les
//!                                       déchire, la       tentacules jaillissent,
//!                                       trombe s'élève    la tête attend en haut
//! ```
//!
//! - FURIE (`FURIE_PAS`, 2 s, sans danger hors de l'eau) : elle hurle, l'écran
//!   tremble. La BÂCHE (le plafond de l'arène, `_` dans la grille : une boîte
//!   solide pendant la phase 1, pour qu'on ne monte pas avant) se déchire, et une
//!   TROMBE (un courant ascendant, la règle des bulles) monte du bassin, au-dessus
//!   du trou du milieu, jusqu'à la corniche du bas de la tour. À la fin de la
//!   furie, le QUAI de l'arène casse : toutes ses plateformes tombent à l'eau.
//! - MONTÉE : la mer monte (`surface_mer`), d'abord vite (la CRUE remplit l'arène
//!   jusqu'à la bâche), puis d'environ 1 tuile toutes les 1,3 s, avec un court
//!   palier de répit, annoncé, sous chaque corniche large de la tour. Sa surface
//!   en furie TUE au contact, BARQUE COMPRISE : pas de bateau en phase 2 (elle
//!   noierait la barque comme Pépin : `blesse_en_fuite` regarde seulement si les
//!   pieds passent sous la surface). Depuis la surface, un tentacule jaillit
//!   toutes les 2,2 s dans une colonne ANNONCÉE comme en phase 1 (1,4 s), monte
//!   de 4 tuiles puis redescend ; jamais sur la seule place d'un appui : il y a
//!   toujours une issue (`laisse_une_issue`).
//! - COUP FINAL : la tête attend sur la vigie, en haut de la tour (`&`). Un saut
//!   dessus donne la plongée, puis vaincue : le niveau est fini comme avant.
//!
//! Une mort pendant la phase 2 ramène au DÉBUT DE LA PHASE 2 : la lanterne de la
//! tour s'allume d'elle-même dès la furie (`entities::contacts`), et `reprendre`
//! remet la mer en bas.
//!
//! MODE FACILE : la mer monte 0,75 fois moins vite, les tentacules sont plus
//! espacés d'autant.
//!
//! Notions Rust : un sous-module qui ajoute des méthodes à la structure de son
//! parent (un second bloc `impl Pieuvre`) et voit ses champs privés ; une
//! fonction pure du temps (`surface_mer_a`) parcourue segment par segment.

use super::{EtatTentacule, PhasePieuvre, Pieuvre, ANNONCE_PAS, DRESSE_PAS, SALVE_PAS};
use crate::aabb::{Aabb, Grille, Tuile, TAILLE_TUILE};
use crate::physics::{CourantAscendant, JOUEUR_H, JOUEUR_L};

/// La furie : 2 s, elle hurle et l'écran tremble ; le quai casse à la fin.
pub const FURIE_PAS: u16 = 120;
/// La crue qui remplit l'arène (px par pas) : une tuile en 0,5 s.
pub const VITESSE_CRUE: f32 = TAILLE_TUILE / 30.0;
/// La mer dans la tour (px par pas) : une tuile toutes les 1,3 s.
pub const VITESSE_MER: f32 = TAILLE_TUILE / 78.0;
/// Mode facile : la mer monte 0,75 fois moins vite.
pub const FACTEUR_FACILE: f32 = 0.75;
/// Un palier de répit : la mer s'arrête 2,5 s.
pub const PALIER_PAS: u16 = 150;
/// Annonce d'un palier : la dernière seconde de montée avant lui.
pub const ANNONCE_PALIER_PAS: u16 = 60;
/// Largeur (en tuiles) d'une corniche LARGE, qui a son palier.
pub const CORNICHE_LARGE: usize = 8;
/// Le palier : la mer s'arrête 3 tuiles sous la corniche.
pub const SOUS_LA_CORNICHE: f32 = 3.0 * TAILLE_TUILE;
/// Hauteur d'un tentacule de la montée au-dessus de la surface : 4 tuiles.
pub const HAUTEUR_TENTACULE_MONTEE: f32 = 4.0 * TAILLE_TUILE;
/// Un tentacule de la montée toutes les 2,2 s (entre 2 et 2,5 s).
pub const RYTHME_TENTACULE_PAS: u16 = 132;

impl Pieuvre {
    /// La phase 2 commence (le dernier coup de la phase 1) : plus de saisie.
    pub(super) fn commencer_la_fuite(&mut self) {
        self.fuite = true;
        self.saisie = None;
        self.tentacule_montee = None;
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
            let mut cases: Vec<(i32, i32)> = self.plateformes.iter().flatten().copied().collect();
            // Carte 9.13b : les plateformes arrachées de la tour.
            cases.extend(self.cases_arrachees());
            cases
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
    fn vitesse_mer(&self) -> f32 {
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

    /// Ordonnée (px) de la mer au pas `t` de la montée. Fonction pure : la crue
    /// jusqu'à la bâche, puis la montée lente, arrêtée `PALIER_PAS` à chaque
    /// palier, jusqu'à `arret_mer`.
    pub fn surface_mer_a(&self, t: u16) -> f32 {
        let crue = self.fin_de_crue();
        if t < crue {
            return self.depart_mer() - f32::from(t) * VITESSE_CRUE;
        }
        let v = self.vitesse_mer();
        let mut y = self.bache.map_or(self.depart_mer(), |b| b.bas());
        let mut reste = f32::from(t - crue);
        for &p in self.paliers.iter().filter(|&&p| p > self.arret_mer) {
            let d = (y - p).max(0.0) / v;
            if reste < d {
                return y - reste * v;
            }
            reste -= d;
            y = p;
            if reste < f32::from(PALIER_PAS) {
                return p;
            }
            reste -= f32::from(PALIER_PAS);
        }
        (y - reste * v).max(self.arret_mer)
    }

    /// La mer en ce moment (`None` hors de la phase 2). Elle tue au contact.
    pub fn surface_mer(&self) -> Option<f32> {
        if !self.fuite {
            return None;
        }
        Some(match self.phase {
            // Carte 9.13b : l'horloge de la mer s'arrête pendant une rencontre
            // et recule après chaque coup (`montee`).
            PhasePieuvre::Montee => self.surface_mer_a(self.horloge_mer()),
            // Après le coup final, elle reste où elle était (`arret_mer`).
            PhasePieuvre::Plongee | PhasePieuvre::Vaincue => self.arret_mer,
            _ => self.depart_mer(),
        })
    }

    /// Le coup final : la mer se fige où elle est.
    pub(super) fn figer_la_mer(&mut self) {
        if let Some(s) = self.surface_mer() {
            self.arret_mer = s;
        }
    }

    /// La mer est-elle à un palier au pas `t` de la montée ?
    fn au_palier(&self, t: u16) -> bool {
        let s = self.surface_mer_a(t);
        t >= self.fin_de_crue()
            && self.paliers.iter().any(|&p| (s - p).abs() < 0.01)
            && (self.surface_mer_a(t.saturating_sub(1)) - s).abs() < 0.01
    }

    /// Annonce d'un palier : `Some(progression)` (de 0 à 1) pendant la dernière
    /// seconde avant que la mer s'arrête, puis `Some(1.0)` tant qu'elle est
    /// arrêtée ; `None` sinon. Le dessin la montre aux murs.
    pub fn annonce_palier(&self) -> Option<f32> {
        if self.phase != PhasePieuvre::Montee {
            return None;
        }
        let t = self.horloge_mer();
        if self.au_palier(t) {
            return Some(1.0);
        }
        (1..=ANNONCE_PALIER_PAS)
            .find(|&k| self.au_palier(t.saturating_add(k)))
            .map(|k| 1.0 - f32::from(k) / f32::from(ANNONCE_PALIER_PAS))
    }

    /// La place de la trombe : au-dessus du trou du milieu (la place du milieu de
    /// la tête), du bassin jusqu'à la corniche du bas de la tour (2 tuiles au-dessus
    /// de la bâche). `None` sans phase 2.
    pub fn zone_trombe(&self) -> Option<Aabb> {
        let bache = self.bache?;
        self.tete_finale?;
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

    /// Le tentacule de la montée en ce moment : sa colonne et son état.
    pub fn tentacule_de_montee(&self) -> Option<(usize, EtatTentacule)> {
        if !self.fuite || self.phase != PhasePieuvre::Montee {
            return None;
        }
        let (i, debut) = self.tentacule_montee?;
        let dans = self.ecoule.checked_sub(debut)?;
        let etat = if dans < ANNONCE_PAS {
            EtatTentacule::Annonce(f32::from(dans) / f32::from(ANNONCE_PAS))
        } else if dans < ANNONCE_PAS + DRESSE_PAS {
            EtatTentacule::Dresse
        } else if dans < SALVE_PAS {
            let fin = dans - ANNONCE_PAS - DRESSE_PAS;
            EtatTentacule::Redescente(
                f32::from(fin) / f32::from(SALVE_PAS - ANNONCE_PAS - DRESSE_PAS),
            )
        } else {
            return None;
        };
        Some((i, etat))
    }

    /// Le corps du tentacule de la colonne `i` dressé au-dessus de la mer `mer`.
    pub fn boite_tentacule_montee(&self, i: usize, mer: f32) -> Aabb {
        let c = self.colonnes[i];
        Aabb::new(
            c.x,
            mer - HAUTEUR_TENTACULE_MONTEE,
            c.w,
            HAUTEUR_TENTACULE_MONTEE,
        )
    }

    /// Phase 2 : la mer, ou le tentacule dressé de la montée, touche-t-il `boite` ?
    pub(super) fn blesse_en_fuite(&self, boite: &Aabb) -> bool {
        let Some(mer) = self.surface_mer() else {
            return false;
        };
        if boite.bas() > mer {
            return true;
        }
        matches!(
            self.tentacule_de_montee(),
            Some((i, EtatTentacule::Dresse)) if self.boite_tentacule_montee(i, mer).chevauche(boite)
        )
    }

    /// Rythme des tentacules de la montée (plus espacés en mode facile).
    fn rythme_montee(&self) -> u16 {
        if self.est_facile() {
            (f32::from(RYTHME_TENTACULE_PAS) / FACTEUR_FACILE) as u16
        } else {
            RYTHME_TENTACULE_PAS
        }
    }

    /// Un pas des tentacules de la montée : quand le précédent a fini et que le
    /// rythme le permet (jamais pendant un palier : c'est le répit), un nouveau
    /// tentacule s'annonce, dans la colonne de Pépin si elle laisse une issue,
    /// sinon dans la plus proche qui en laisse une.
    pub(super) fn avancer_montee(&mut self, grille: &dyn Grille) {
        if !self.fuite || self.phase != PhasePieuvre::Montee {
            return;
        }
        let t = self.ecoule;
        // L'horloge de la mer (carte 9.13b) : les paliers et l'issue se jugent sur
        // elle ; pendant la garde d'une tête, ses tentacules à elle frappent.
        let h = self.horloge_mer();
        let prochain = match self.tentacule_montee {
            Some((_, debut)) => debut.saturating_add(self.rythme_montee()),
            None => self.fin_de_crue(),
        };
        if t < prochain
            || self.sous_garde()
            || self.au_palier(h)
            || self.au_palier(h.saturating_add(SALVE_PAS))
        {
            return;
        }
        let x = self.pepin.map_or(self.depart_mer(), |p| p.x + p.w / 2.0);
        if let Some(i) = self.colonne_de_montee(grille, x, h) {
            self.tentacule_montee = Some((i, t));
        }
    }

    /// La colonne d'un tentacule lancé au pas `t` de la montée, Pépin au milieu
    /// `x` : la sienne, ou la plus proche qui laisse une issue (`None` : aucune).
    pub fn colonne_de_montee(&self, grille: &dyn Grille, x: f32, t: u16) -> Option<usize> {
        let milieu = |i: usize| self.colonnes[i].x + self.colonnes[i].w / 2.0;
        let mut ordre: Vec<usize> = (0..self.colonnes.len()).collect();
        ordre.sort_by(|&a, &b| (milieu(a) - x).abs().total_cmp(&(milieu(b) - x).abs()));
        ordre
            .into_iter()
            .find(|&i| self.laisse_une_issue(grille, i, t))
    }

    /// Règle d'équité (« jamais sur la seule case d'appui obligée ») : un
    /// tentacule lancé au pas `t` dans la colonne `i` laisse-t-il, sur CHAQUE
    /// appui à sa portée, une place sûre assez large pour Pépin ? Un appui : une
    /// suite de cases solides de la grille, libres au-dessus, sur la même rangée.
    /// À sa portée : Pépin debout dessus toucherait le tentacule dressé, la mer
    /// prise au début et à la fin de la frappe.
    pub fn laisse_une_issue(&self, grille: &dyn Grille, i: usize, t: u16) -> bool {
        let haut_mer = self.surface_mer_a(t.saturating_add(ANNONCE_PAS + DRESSE_PAS));
        let bas_mer = self.surface_mer_a(t.saturating_add(ANNONCE_PAS));
        let corps = self.boite_tentacule_montee(i, haut_mer);
        let (g, d) = (corps.x, corps.droite());
        self.appuis(grille)
            .into_iter()
            .filter(|&(y, _, _)| y > haut_mer - HAUTEUR_TENTACULE_MONTEE && y - JOUEUR_H < bas_mer)
            .filter(|&(_, a, b)| a < d && b > g)
            .all(|(_, a, b)| g - a > JOUEUR_L || b - d > JOUEUR_L)
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
    /// déjà en montée, la tête en haut de la tour ; la mer ne monte pas pour lui
    /// (un joueur patient de la phase 1 attend la furie, et la mer laisse le temps
    /// de grimper : `levels::pieuvre::tests`). Sans tour, rien ne change.
    pub fn pour_le_robot(&mut self) {
        if self.tete_finale.is_some() {
            self.entrer(PhasePieuvre::Montee);
            // Carte 9.13b : les rencontres gagnées, les trappes ouvertes.
            self.gagner_toute_la_tour();
        }
    }
}

/// Les paliers de la mer dans une tour : 3 tuiles sous chaque corniche LARGE
/// (au moins `CORNICHE_LARGE` cases solides de suite, libres au-dessus), au-dessus
/// de la corniche du bas (`plus_bas`, en rangées). De la plus basse à la plus haute.
pub fn paliers_de_la_tour(
    solide: impl Fn(usize, usize) -> bool,
    largeur: usize,
    plus_bas: usize,
) -> Vec<f32> {
    let mut paliers = Vec::new();
    for y in (1..plus_bas).rev() {
        let mut suite = 0;
        let large = (0..largeur).any(|x| {
            suite = if solide(x, y) && !solide(x, y - 1) {
                suite + 1
            } else {
                0
            };
            suite >= CORNICHE_LARGE
        });
        if large {
            paliers.push(y as f32 * TAILLE_TUILE + SOUS_LA_CORNICHE);
        }
    }
    paliers
}
