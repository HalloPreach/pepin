//! Niveau d'eau cyclique, la « marée » (carte 9.3, lettre `=`) : dans une zone
//! (les cases `=` de la grille, assemblées en rectangle), la surface de l'eau
//! monte et descend selon une période fixe. En bas, la zone est sèche (ou
//! l'eau est au niveau de l'étendue qui est dessous) ; en haut, elle est pleine.
//!
//! Le cycle (`PERIODE_MAREE` pas) : basse mer (`BASSE_MER_PAS`), montée
//! (`MONTEE_PAS`), pleine mer (`PLEINE_MER_PAS`), descente (`DESCENTE_PAS`). Le
//! mouvement est ANNONCÉ : pendant la dernière seconde de chaque palier
//! (`ANNONCE_MAREE_PAS`), le repère au mur clignote de plus en plus vite
//! (comme l'éclair et le ventilateur), avec une flèche dans le sens du
//! prochain mouvement.
//!
//! Ce que fait l'eau qui monte : rien de nouveau. Elle est une étendue d'eau
//! de plus (`Eau`, via `entities::eaux_des_entites`), dont la surface bouge :
//! la barque la suit (`Joueur::actualiser`, pas 3bis : elle reste posée sur la
//! surface sous son centre), et un Pépin à pied dont les pieds passent sous la
//! surface est noyé (pas 8), parapluie fermé, comme partout ailleurs.
//!
//! Règles de pose : le bas de la zone repose sur un solide ou sur de l'eau,
//! et la rangée au-dessus de la zone reste libre (la barque en pleine mer
//! dépasse la surface de 11 px).
//!
//! Notions Rust : un `match` sur des intervalles (`a..b`) pour découper le
//! temps du cycle ; une fonction PURE du temps (`surface_a`), testable seule.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Eau, Environnement};

/// Basse mer : 3 s.
pub const BASSE_MER_PAS: u32 = 180;
/// Montée : 2 s.
pub const MONTEE_PAS: u32 = 120;
/// Pleine mer : 3 s.
pub const PLEINE_MER_PAS: u32 = 180;
/// Descente : 2 s.
pub const DESCENTE_PAS: u32 = 120;
/// Un cycle complet : 10 s.
pub const PERIODE_MAREE: u32 = BASSE_MER_PAS + MONTEE_PAS + PLEINE_MER_PAS + DESCENTE_PAS;
/// Annonce du mouvement : la dernière seconde de chaque palier.
pub const ANNONCE_MAREE_PAS: u32 = 60;

#[derive(Debug, Clone)]
pub struct Maree {
    /// La zone où monte l'eau : sa surface va du bas (basse mer) au haut
    /// (pleine mer).
    pub zone: Aabb,
    /// Pas écoulés dans le cycle (0 = début de la basse mer).
    pas: u32,
}

/// Où en est la marée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseMaree {
    BasseMer,
    Montee,
    PleineMer,
    Descente,
}

impl Maree {
    pub fn new(zone: Aabb) -> Self {
        Self { zone, pas: 0 }
    }

    /// Pas écoulés dans le cycle.
    pub fn pas(&self) -> u32 {
        self.pas
    }

    /// Phase et pas écoulés dans cette phase, au pas `t` du cycle.
    fn phase_a(t: u32) -> (PhaseMaree, u32) {
        let t = t % PERIODE_MAREE;
        let montee = BASSE_MER_PAS;
        let haut = montee + MONTEE_PAS;
        let descente = haut + PLEINE_MER_PAS;
        match t {
            _ if t < montee => (PhaseMaree::BasseMer, t),
            _ if t < haut => (PhaseMaree::Montee, t - montee),
            _ if t < descente => (PhaseMaree::PleineMer, t - haut),
            _ => (PhaseMaree::Descente, t - descente),
        }
    }

    /// Phase du moment.
    pub fn phase(&self) -> PhaseMaree {
        Self::phase_a(self.pas).0
    }

    /// Ordonnée (px) de la surface au pas `t` du cycle (pure). Montée et
    /// descente à vitesse constante.
    pub fn surface_a(&self, t: u32) -> f32 {
        let (bas, haut) = (self.zone.bas(), self.zone.y);
        let (phase, ecoule) = Self::phase_a(t);
        match phase {
            PhaseMaree::BasseMer => bas,
            PhaseMaree::Montee => bas + (haut - bas) * ecoule as f32 / MONTEE_PAS as f32,
            PhaseMaree::PleineMer => haut,
            PhaseMaree::Descente => haut + (bas - haut) * ecoule as f32 / DESCENTE_PAS as f32,
        }
    }

    /// Ordonnée (px) de la surface en ce moment.
    pub fn surface(&self) -> f32 {
        self.surface_a(self.pas)
    }

    /// L'eau de la zone en ce moment, de la surface au bas de la zone ; `None`
    /// à basse mer (la zone est sèche).
    pub fn eau(&self) -> Option<Eau> {
        let s = self.surface();
        (s < self.zone.bas()).then(|| Eau {
            zone: Aabb::new(self.zone.x, s, self.zone.w, self.zone.bas() - s),
            courant: 0,
        })
    }

    /// L'eau de la zone en PLEINE MER (la zone entière) : le robot de
    /// faisabilité s'en sert pour son état « marée haute ».
    pub fn eau_pleine(&self) -> Eau {
        Eau {
            zone: self.zone,
            courant: 0,
        }
    }

    /// Annonce du prochain mouvement : `Some((monte, progression))` pendant la
    /// dernière seconde d'un palier (`monte` : l'eau va monter ; `progression`
    /// de 0 à 1), `None` sinon.
    pub fn annonce(&self) -> Option<(bool, f32)> {
        let (phase, ecoule) = Self::phase_a(self.pas);
        let (duree, monte) = match phase {
            PhaseMaree::BasseMer => (BASSE_MER_PAS, true),
            PhaseMaree::PleineMer => (PLEINE_MER_PAS, false),
            _ => return None,
        };
        let debut = duree - ANNONCE_MAREE_PAS;
        (ecoule >= debut).then(|| (monte, (ecoule - debut) as f32 / ANNONCE_MAREE_PAS as f32))
    }
}

impl Actualisable for Maree {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.pas = (self.pas + 1) % PERIODE_MAREE;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::entities::contacts::{resoudre_contacts, CauseMort, Contact};
    use crate::entities::{eaux_des_entites, Entite};
    use crate::physics::{ENFONCEMENT_BARQUE, JOUEUR_H};
    use crate::player::{Entrees, Joueur};

    /// La règle centrale : la barque suit la surface qui monte, et un Pépin à
    /// pied au fond de la zone est noyé par la montée.
    #[test]
    fn la_barque_suit_la_maree_et_la_montee_noie_le_pieton() {
        // Sol plein à la rangée 8 (y = 128) ; zone de marée : rangées 5 à 7
        // (y de 80 à 128), colonnes 0 à 9.
        let mut lignes = vec!["..........".to_string(); 8];
        lignes.push("##########".to_string());
        let lignes: Vec<&str> = lignes.iter().map(String::as_str).collect();
        let g = GrilleTuiles::depuis_texte(&lignes);
        let mut ents = vec![Entite::Maree(Maree::new(Aabb::new(0.0, 80.0, 160.0, 48.0)))];
        // Le piéton, au fond ; la barque, mise à l'eau dès que la montée commence
        // (l'eau la porte ensuite jusqu'en haut).
        let mut pieton = Joueur::new(20.0, 128.0 - JOUEUR_H);
        let mut barque: Option<Joueur> = None;
        let mut noye = false;
        for _ in 0..(BASSE_MER_PAS + MONTEE_PAS + 10) {
            for e in &mut ents {
                e.actualiser(&Environnement::simple(&g));
            }
            let eaux = eaux_des_entites(&[], &ents);
            let env = Environnement {
                eaux: &eaux,
                ..Environnement::simple(&g)
            };
            pieton.actualiser(&Entrees::default(), &env);
            noye |= resoudre_contacts(&mut pieton, &mut [])
                .contains(&Contact::Mort(CauseMort::Noyade));
            if let (None, Some(eau)) = (&barque, eaux.first()) {
                let mut b = Joueur::new(100.0, eau.surface() - JOUEUR_H + ENFONCEMENT_BARQUE);
                b.entrer_en_barque();
                barque = Some(b);
            }
            if let Some(b) = &mut barque {
                b.actualiser(&Entrees::default(), &env);
            }
        }
        assert!(noye, "le piéton est noyé par la montée");
        let barque = barque.expect("l'eau est montée");
        assert!(barque.en_barque());
        assert_eq!(barque.boite.y, 80.0 - JOUEUR_H + ENFONCEMENT_BARQUE);
    }
}
