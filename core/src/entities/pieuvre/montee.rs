//! La montée de la phase 2, plus longue et pleine d'INTERACTIONS (carte 9.13b).
//!
//! Demande d'AGA, mot pour mot : « la montée est trop simple et trop courte, il
//! faudrait des interactions avec la pieuvre ». La tour double (70 rangées), et la
//! Pieuvre ne se contente plus d'attendre en haut :
//!
//! ```text
//!   bas de la tour ── RENCONTRE 1 ── lanterne de mi-hauteur ── RENCONTRE 2 ── vigie
//!     (crue)          (au tiers)                                (aux 2 tiers)  (coup final)
//!       balayeurs et arrachages en chemin, la mer qui monte toujours dessous
//! ```
//!
//! - RENCONTRE (`$` dans la grille, coin bas-gauche de la tête, comme `&`) : quand
//!   Pépin approche de sa corniche, la tête SURGIT (0,6 s) et s'agrippe. Une TRAPPE
//!   (`-` : ses tentacules tendus sur le trou de la corniche du dessus, une boîte
//!   solide) bloque le passage tant qu'elle tient. La mer s'arrête pendant la
//!   rencontre. Deux tentacules de garde frappent, chacun son tour, à gauche puis à
//!   droite de la tête, annoncés comme en phase 1 (1,4 s) : jamais les deux à la
//!   fois, il y a toujours une issue. Un saut sur la tête : elle hurle, lâche la
//!   corniche et replonge ; la trappe s'ouvre, la mer RECULE de 2 tuiles, puis
//!   reprend. Un point de vie en moins. Au coup final de la vigie, les mêmes
//!   gardes frappent pendant que la tête attend.
//! - BALAYEUR (`!`, contre un mur, juste au-dessus d'une corniche) : quand Pépin
//!   arrive sur la corniche, des remous au pied du mur, puis le bout du tentacule
//!   sort du mur (1 s en tout) ; puis il BALAIE la corniche d'un mur à l'autre, au
//!   ras du sol. On saute par-dessus son bout, ou on monte plus haut.
//! - ARRACHAGE (`%` : des cases solides) : quand Pépin arrive sous la plateforme,
//!   un tentacule s'y enroule (1,3 s d'annonce, l'eau de la saisie 9.12g), puis
//!   l'arrache : elle ne revient pas dans cette partie de la montée. Il reste
//!   toujours un autre chemin (les tests le vérifient).
//!
//! Une mort ramène à la dernière lanterne : celle du bas de la tour, ou celle de
//! mi-hauteur, juste après la 1re rencontre. Ce qui est AU-DESSUS de Pépin repart
//! à neuf (rencontres, arrachages), ce qui est dessous reste gagné ; la mer est
//! remise sous la lanterne.
//!
//! Notions Rust : un 3e bloc `impl Pieuvre` dans un 3e fichier ; une structure
//! `Default` (`#[derive(Default)]`) qu'on remplit après coup ; des `Option<u16>`
//! comme « horloges » (le pas où une chose a commencé, ou rien).

use super::{EtatTentacule, PhasePieuvre, Pieuvre, ANNONCE_PAS, DRESSE_PAS, SALVE_PAS};
use super::{ANNONCE_SAISIE_PAS, PLONGEE_SAISIE_PAS};
use crate::aabb::{Aabb, TAILLE_TUILE};

/// La tête d'une rencontre surgit de l'eau : 0,6 s, sans danger, pas encore frappable.
pub const SURGIT_PAS: u16 = 36;
/// Une rencontre commence quand les pieds de Pépin sont à moins de 7 tuiles sous
/// la corniche de la tête.
pub const APPROCHE: f32 = 7.0 * TAILLE_TUILE;
/// Un tentacule de garde toutes les 2,7 s (une salve, puis 0,6 s de calme).
pub const GARDE_PAS: u16 = SALVE_PAS + 36;
/// Un tentacule de garde se dresse jusqu'à 4 tuiles au-dessus de la corniche.
pub const HAUTEUR_GARDE: f32 = 4.0 * TAILLE_TUILE;
/// Largeur (px) d'un tentacule de garde, et l'écart qui le sépare de la tête.
pub const LARGEUR_GARDE: f32 = 24.0;
pub const ECART_GARDE: f32 = 4.0;
/// La tête lâchée replonge en 0,7 s.
pub const LACHEE_PAS: u16 = 42;
/// Après un coup, la mer recule de 2 tuiles…
pub const RECUL_MER: f32 = 2.0 * TAILLE_TUILE;
/// … en remontant son horloge de 4 pas par pas (2 tuiles en 0,65 s).
pub const RECUL_PAR_PAS: u16 = 4;
/// Le balayeur s'annonce 1 s (remous, puis le bout au mur à la moitié).
pub const ANNONCE_BALAI_PAS: u16 = 60;
/// Il glisse de 3 px par pas (180 px/s : Pépin court à 96 px/s, il faut sauter).
pub const VITESSE_BALAI: f32 = 3.0;
/// Il se retire en 0,5 s, sans danger.
pub const RETRAIT_BALAI_PAS: u16 = 30;
/// Repos d'un balayeur avant qu'il puisse repartir.
pub const REPOS_BALAI_PAS: u16 = 180;
/// Le bout du balayeur : 24 x 12, au ras de la corniche.
pub const BOUT_BALAI: (f32, f32) = (24.0, 12.0);
/// Un arrachage commence quand Pépin est 1 à 4 tuiles sous la plateforme…
pub const ARRACHAGE_SOUS: (f32, f32) = (TAILLE_TUILE, 4.0 * TAILLE_TUILE);
/// … et à moins de 5 tuiles d'elle de côté.
pub const ARRACHAGE_COTE: f32 = 5.0 * TAILLE_TUILE;
/// L'arrachée part en 0,5 s (elle ne porte plus dès ce moment).
pub const EMPORTEE_PAS: u16 = PLONGEE_SAISIE_PAS;

/// Une rencontre de la montée : la tête sur sa corniche, et la trappe qu'elle
/// tient fermée au-dessus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rencontre {
    pub tete: Aabb,
    pub trappe: Aabb,
}

/// Un balayeur : la corniche qu'il balaie (son dessus `y`), le mur d'où il sort
/// (`depart`, bord intérieur) et celui où il s'arrête (`arrivee`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Balayeur {
    pub y: f32,
    pub depart: f32,
    pub arrivee: f32,
}

impl Balayeur {
    /// Sort-il du mur de gauche ?
    pub fn de_gauche(&self) -> bool {
        self.depart < self.arrivee
    }

    /// Pas de glissade, d'un mur à l'autre.
    pub fn course_pas(&self) -> u16 {
        (((self.arrivee - self.depart).abs() - BOUT_BALAI.0) / VITESSE_BALAI).ceil() as u16
    }

    /// La boîte du bout après `s` pas de glissade.
    pub fn bout(&self, s: u16) -> Aabb {
        let avance = (f32::from(s) * VITESSE_BALAI).min((self.arrivee - self.depart).abs());
        let x = if self.de_gauche() {
            (self.depart + avance).min(self.arrivee - BOUT_BALAI.0)
        } else {
            (self.depart - BOUT_BALAI.0 - avance).max(self.arrivee)
        };
        Aabb::new(x, self.y - BOUT_BALAI.1, BOUT_BALAI.0, BOUT_BALAI.1)
    }
}

/// Ce que fait un balayeur pendant un pas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatBalai {
    /// Remous puis le bout au mur (avancement de 0 à 1) : sans danger.
    Annonce(f32),
    /// Il glisse : son bout (la boîte) tue.
    Balaie(Aabb),
    /// Il se retire (avancement de 0 à 1) : sans danger.
    Retrait(f32),
}

/// Ce que devient une plateforme arrachée.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatArrachage {
    /// Le tentacule s'enroule, elle tremble (de 0 à 1) : elle porte encore.
    Annonce(f32),
    /// Il l'emporte (de 0 à 1) : elle ne porte plus.
    Emportee(f32),
    /// Partie pour de bon.
    Partie,
}

/// Tout ce que la montée ajoute à la Pieuvre (carte 9.13b). Posé par le parseur
/// (`rencontres`, `balayeurs`, `arrachables`), le reste est l'état du combat.
#[derive(Debug, Clone, Default)]
pub struct Tour {
    /// Les rencontres, de la plus basse à la plus haute.
    pub rencontres: Vec<Rencontre>,
    pub balayeurs: Vec<Balayeur>,
    /// Les plateformes arrachables : leurs cases `(tx, ty)`.
    pub arrachables: Vec<Vec<(i32, i32)>>,
    /// Rencontres gagnées (les premières de `rencontres`).
    gagnees: usize,
    /// Pas (de la montée) où la garde de la tête en cours a commencé.
    garde: Option<u16>,
    /// La dernière tête lâchée et le pas de son coup (elle replonge).
    lachee: Option<(usize, u16)>,
    /// L'horloge de la mer (pas de `surface_mer_a`), arrêtée en rencontre.
    horloge_mer: u16,
    /// Pas d'horloge de la mer qu'il reste à reculer.
    recul: u16,
    /// Début de chaque balayage (ou rien).
    balayages: Vec<Option<u16>>,
    /// Début de chaque arrachage (ou rien).
    arrachages: Vec<Option<u16>>,
}

impl Pieuvre {
    /// L'horloge de la mer : `surface_mer_a(horloge_mer)` est la mer du moment.
    pub fn horloge_mer(&self) -> u16 {
        self.tour.horloge_mer
    }

    /// Rencontres gagnées.
    pub fn rencontres_gagnees(&self) -> usize {
        self.tour.gagnees
    }

    /// La rencontre en cours (la tête surgie ou agrippée), s'il y en a une.
    pub fn rencontre(&self) -> Option<&Rencontre> {
        self.tour.garde?;
        self.tour.rencontres.get(self.tour.gagnees)
    }

    /// Pas écoulés depuis le début de la garde en cours.
    fn dans_la_garde(&self) -> Option<u16> {
        let debut = self.tour.garde?;
        if self.phase() != PhasePieuvre::Montee {
            return None;
        }
        self.ecoule().checked_sub(debut)
    }

    /// La tête de la rencontre surgit : avancement de 0 à 1 (`None` ensuite).
    pub fn surgissement(&self) -> Option<f32> {
        self.rencontre()?;
        let d = self.dans_la_garde()?;
        (d < SURGIT_PAS).then(|| f32::from(d) / f32::from(SURGIT_PAS))
    }

    /// La tête de la rencontre agrippée, qu'on peut frapper.
    pub fn tete_de_rencontre(&self) -> Option<Aabb> {
        let r = self.rencontre()?;
        (self.dans_la_garde()? >= SURGIT_PAS).then_some(r.tete)
    }

    /// La tête lâchée qui replonge : sa boîte et l'avancement (de 0 à 1).
    pub fn tete_lachee(&self) -> Option<(Aabb, f32)> {
        let (k, coup) = self.tour.lachee?;
        let d = self.ecoule().checked_sub(coup)?;
        (d < LACHEE_PAS && self.phase() == PhasePieuvre::Montee).then(|| {
            (
                self.tour.rencontres[k].tete,
                f32::from(d) / f32::from(LACHEE_PAS),
            )
        })
    }

    /// La boîte solide de la pieuvre : la bâche en phase 1 ; en phase 2, la
    /// trappe de la prochaine rencontre (une seule à la fois suffit : on ne
    /// peut pas atteindre la suivante avant d'avoir ouvert celle-ci).
    pub fn solide(&self) -> Option<Aabb> {
        if !self.en_fuite() {
            return self.bache_solide();
        }
        self.tour
            .rencontres
            .get(self.tour.gagnees)
            .map(|r| r.trappe)
    }

    /// La tête gardée en ce moment : celle de la rencontre, ou celle de la vigie.
    fn tete_gardee(&self) -> Option<Aabb> {
        match self.tour.rencontres.get(self.tour.gagnees) {
            Some(r) => Some(r.tete),
            None => self.tete_finale,
        }
    }

    /// Les deux tentacules de garde : `(corps, état)` du seul qui frappe (gauche,
    /// puis droite, chacun son tour), du pied de la mer `mer` jusqu'à 4 tuiles
    /// au-dessus de la corniche.
    pub fn garde_en_cours(&self) -> Option<(Aabb, EtatTentacule)> {
        let d = self.dans_la_garde()?.checked_sub(SURGIT_PAS)?;
        let tete = self.tete_gardee()?;
        let mer = self.surface_mer()?;
        let cycle = self.rythme_garde();
        let (k, dans) = (d / cycle, d % cycle);
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
        let x = if k % 2 == 0 {
            tete.x - ECART_GARDE - LARGEUR_GARDE
        } else {
            tete.droite() + ECART_GARDE
        };
        let haut = tete.bas() - HAUTEUR_GARDE;
        Some((
            Aabb::new(x, haut, LARGEUR_GARDE, (mer - haut).max(0.0)),
            etat,
        ))
    }

    /// Rythme des gardes (plus lent en mode facile).
    fn rythme_garde(&self) -> u16 {
        if self.est_facile() {
            (f32::from(GARDE_PAS) / super::FACTEUR_FACILE) as u16
        } else {
            GARDE_PAS
        }
    }

    /// L'état du balayeur `k` (`None` : au repos).
    pub fn etat_balayeur(&self, k: usize) -> Option<EtatBalai> {
        if self.phase() != PhasePieuvre::Montee {
            return None;
        }
        let b = self.tour.balayeurs.get(k)?;
        let d = self.ecoule().checked_sub((*self.tour.balayages.get(k)?)?)?;
        let course = b.course_pas();
        Some(if d < ANNONCE_BALAI_PAS {
            EtatBalai::Annonce(f32::from(d) / f32::from(ANNONCE_BALAI_PAS))
        } else if d < ANNONCE_BALAI_PAS + course {
            EtatBalai::Balaie(b.bout(d - ANNONCE_BALAI_PAS))
        } else if d < ANNONCE_BALAI_PAS + course + RETRAIT_BALAI_PAS {
            let r = d - ANNONCE_BALAI_PAS - course;
            EtatBalai::Retrait(f32::from(r) / f32::from(RETRAIT_BALAI_PAS))
        } else {
            return None;
        })
    }

    /// L'état de la plateforme arrachable `k` (`None` : intacte).
    pub fn etat_arrachage(&self, k: usize) -> Option<EtatArrachage> {
        let d = self.ecoule().checked_sub((*self.tour.arrachages.get(k)?)?);
        let Some(d) = d.filter(|_| self.phase() == PhasePieuvre::Montee) else {
            // Après le coup final (ou une horloge remise à zéro) : partie.
            return Some(EtatArrachage::Partie);
        };
        Some(if d < ANNONCE_SAISIE_PAS {
            EtatArrachage::Annonce(f32::from(d) / f32::from(ANNONCE_SAISIE_PAS))
        } else if d < ANNONCE_SAISIE_PAS + EMPORTEE_PAS {
            EtatArrachage::Emportee(f32::from(d - ANNONCE_SAISIE_PAS) / f32::from(EMPORTEE_PAS))
        } else {
            EtatArrachage::Partie
        })
    }

    /// Les cases arrachées qui ne portent plus.
    pub(super) fn cases_arrachees(&self) -> Vec<(i32, i32)> {
        (0..self.tour.arrachables.len())
            .filter(|&k| {
                matches!(
                    self.etat_arrachage(k),
                    Some(EtatArrachage::Emportee(_) | EtatArrachage::Partie)
                )
            })
            .flat_map(|k| self.tour.arrachables[k].iter().copied())
            .collect()
    }

    /// Phase 2 : un tentacule de garde dressé, ou le bout d'un balayeur, touche-t-il
    /// `boite` ?
    pub(super) fn blesse_en_montee(&self, boite: &Aabb) -> bool {
        let garde = matches!(
            self.garde_en_cours(),
            Some((corps, EtatTentacule::Dresse)) if corps.chevauche(boite)
        );
        garde
            || (0..self.tour.balayeurs.len()).any(|k| {
                matches!(self.etat_balayeur(k), Some(EtatBalai::Balaie(b)) if b.chevauche(boite))
            })
    }

    /// Pépin peut-il frapper une tête en ce moment ? (Montée : la rencontre
    /// agrippée, sinon la vigie.)
    pub(super) fn tete_a_frapper(&self) -> Option<Aabb> {
        if self.tour.gagnees < self.tour.rencontres.len() {
            self.tete_de_rencontre()
        } else {
            self.tete_finale
        }
    }

    /// Un saut sur la tête d'une rencontre : elle lâche la corniche, la trappe
    /// s'ouvre, la mer recule. `false` si ce n'était pas une rencontre.
    pub(super) fn gagner_la_rencontre(&mut self) -> bool {
        if self.tete_de_rencontre().is_none() {
            return false;
        }
        let k = self.tour.gagnees;
        self.tour.lachee = Some((k, self.ecoule()));
        self.tour.gagnees += 1;
        self.tour.garde = None;
        // Le recul se compte en HAUTEUR : on cherche, en remontant l'horloge, le
        // dernier pas où la mer était 2 tuiles plus bas (la crue et les paliers
        // n'ont pas la vitesse de la montée).
        let h = self.tour.horloge_mer;
        let cible = self.surface_mer_a(h) + RECUL_MER;
        let avant = (0..=h)
            .rev()
            .find(|&t| self.surface_mer_a(t) >= cible - 0.01);
        self.tour.recul = h - avant.unwrap_or(0);
        // Jamais 0 avant le coup final (le HUD de 2 cases du mode facile).
        self.pv = self.pv.saturating_sub(1).max(1);
        true
    }

    /// Le début de la phase 2 : un point par rencontre, plus le coup final (au
    /// plus les cases du HUD : `pv_max`, 2 en mode facile).
    pub(super) fn pv_de_la_fuite(&self) -> u8 {
        (self.tour.rencontres.len() as u8 + 1).min(self.pv_max)
    }

    /// Pour le robot de faisabilité : toutes les rencontres gagnées.
    pub(super) fn gagner_toute_la_tour(&mut self) {
        self.tour.gagnees = self.tour.rencontres.len();
    }

    /// Un pas de la montée (après `avancer`) : la mer, les rencontres, les
    /// balayeurs, les arrachages, selon où est Pépin.
    pub(super) fn avancer_tour(&mut self) {
        if !self.en_fuite() || self.phase() != PhasePieuvre::Montee {
            return;
        }
        let t = self.ecoule();
        // La mer : elle recule après un coup, s'arrête pendant une rencontre.
        if self.tour.recul > 0 {
            let k = self.tour.recul.min(RECUL_PAR_PAS);
            self.tour.recul -= k;
            self.tour.horloge_mer = self.tour.horloge_mer.saturating_sub(k);
        } else if self.rencontre().is_none() {
            self.tour.horloge_mer = self.tour.horloge_mer.saturating_add(1);
        }
        let Some(pepin) = self.pepin else {
            return;
        };
        // La garde commence quand Pépin approche de la tête.
        if self.tour.garde.is_none() {
            if let Some(tete) = self.tete_gardee() {
                if pepin.bas() <= tete.bas() + APPROCHE {
                    self.tour.garde = Some(t);
                }
            }
        }
        let mer = self.surface_mer().unwrap_or(f32::MAX);
        // Un balayeur part quand Pépin arrive sur sa corniche (la mer dessous).
        self.tour.balayages.resize(self.tour.balayeurs.len(), None);
        for k in 0..self.tour.balayeurs.len() {
            let b = self.tour.balayeurs[k];
            if let Some(debut) = self.tour.balayages[k] {
                let fin = ANNONCE_BALAI_PAS + b.course_pas() + RETRAIT_BALAI_PAS;
                if t >= debut.saturating_add(fin + REPOS_BALAI_PAS) {
                    self.tour.balayages[k] = None;
                }
                continue;
            }
            let sur = pepin.bas() <= b.y + 1.0 && pepin.bas() >= b.y - 2.0 * TAILLE_TUILE;
            if sur && b.y < mer {
                self.tour.balayages[k] = Some(t);
            }
        }
        // Un arrachage part quand Pépin arrive sous la plateforme.
        self.tour
            .arrachages
            .resize(self.tour.arrachables.len(), None);
        let milieu = pepin.x + pepin.w / 2.0;
        for k in 0..self.tour.arrachables.len() {
            if self.tour.arrachages[k].is_some() {
                continue;
            }
            let cases = &self.tour.arrachables[k];
            let (g, d) = super::etendue(cases);
            let haut = cases.iter().map(|c| c.1).min().unwrap_or(0) as f32 * TAILLE_TUILE;
            let sous = pepin.bas() - haut;
            let cote = (g - milieu).max(milieu - d).max(0.0);
            if sous >= ARRACHAGE_SOUS.0 && sous <= ARRACHAGE_SOUS.1 && cote < ARRACHAGE_COTE {
                self.tour.arrachages[k] = Some(t);
            }
        }
    }

    /// Les tentacules de la montée se taisent pendant une garde (la tête a les siens).
    pub(super) fn sous_garde(&self) -> bool {
        self.tour.garde.is_some()
    }

    /// Remet la montée à neuf (début de la phase 2).
    pub(super) fn remettre_la_tour(&mut self) {
        self.tour.gagnees = 0;
        self.tour.garde = None;
        self.tour.lachee = None;
        self.tour.horloge_mer = 0;
        self.tour.recul = 0;
        self.tour.balayages.clear();
        self.tour.arrachages.clear();
        self.pv = self.pv_de_la_fuite();
    }

    /// Pépin est mort en phase 2 et réapparaît dans `occupant` (sa lanterne) : ce
    /// qui est au-dessus de lui repart à neuf, ce qui est dessous reste gagné ; la
    /// mer revient 3 tuiles sous ses pieds (au début de la phase 2 si aucune
    /// rencontre n'est gagnée dessous).
    pub fn reprendre_a(&mut self, occupant: &Aabb) {
        if !self.en_fuite() || !matches!(self.phase(), PhasePieuvre::Furie | PhasePieuvre::Montee) {
            self.reprendre();
            return;
        }
        let gagnees = self.tour.gagnees;
        let arrachages = std::mem::take(&mut self.tour.arrachages);
        self.reprendre();
        let dessous = |tete: &Aabb| tete.bas() > occupant.bas();
        let gardees = self.tour.rencontres[..gagnees]
            .iter()
            .take_while(|r| dessous(&r.tete))
            .count();
        if gardees == 0 {
            return;
        }
        self.tour.gagnees = gardees;
        self.pv = self.pv_de_la_fuite().saturating_sub(gardees as u8).max(1);
        // Les plateformes arrachées sous Pépin restent parties (horloge 0 : partie).
        self.tour.arrachages = arrachages
            .iter()
            .enumerate()
            .map(|(k, a)| {
                let haut = self.tour.arrachables[k]
                    .iter()
                    .map(|c| c.1)
                    .min()
                    .unwrap_or(0);
                a.filter(|_| haut as f32 * TAILLE_TUILE > occupant.bas())
                    .map(|_| u16::MAX)
            })
            .collect();
        let cible = occupant.bas() + 3.0 * TAILLE_TUILE;
        self.tour.horloge_mer = (0..u16::MAX)
            .find(|&t| self.surface_mer_a(t) <= cible)
            .unwrap_or(0);
    }
}
