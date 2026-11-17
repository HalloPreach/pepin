//! Stalactite qui tombe (carte 9.3, lettre `V`) : une pointe de pierre
//! accrochée au plafond (la case au-dessus de `V` est solide). Quand Pépin
//! passe dessous (dans sa colonne, jusqu'au premier solide), elle TREMBLE
//! 0,75 s (`TREMBLE_PAS`), puis TOMBE. En tombant, elle est mortelle (une
//! pointe : `CauseMort::Pointe`) ; accrochée ou tremblante, elle ne fait rien.
//! Elle se brise au sol (le premier solide sous elle) et ne revient qu'après
//! une mort de Pépin (`entities::reprendre_apres_une_mort`).
//!
//! On passe donc dessous SANS S'ARRÊTER : la course traverse une colonne en
//! un quart de seconde, bien avant la chute.
//!
//! Notions Rust : un `enum` d'états à données (`Tremble(n)`, `Chute(vitesse)`,
//! `Brisee(n)`), une machine à états dans `actualiser`.

use super::Actualisable;
use crate::aabb::Aabb;
use crate::physics::{Environnement, DT, GRAVITE, VITESSE_MAX_CHUTE};

/// Tremblement avant la chute : 0,75 s.
pub const TREMBLE_PAS: u16 = 45;
/// Éclats au sol, après la chute (pour le dessin) : 1/3 s.
pub const ECLATS_PAS: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatStalactite {
    /// Au plafond, immobile.
    Accrochee,
    /// Pépin est passé dessous : `n` pas avant la chute.
    Tremble(u16),
    /// En chute, à la vitesse donnée (px/s).
    Chute(f32),
    /// Brisée au sol : `n` pas d'éclats à montrer, puis plus rien (0).
    Brisee(u16),
}

#[derive(Debug, Clone)]
pub struct Stalactite {
    /// Boîte DANGEREUSE de la pierre (10 x 14, au milieu de sa case).
    pub boite: Aabb,
    /// Colonne où Pépin la déclenche : sa case et tout le dessous jusqu'au
    /// premier solide. Son bas est le sol où elle se brise.
    pub colonne: Aabb,
    /// Haut de la boîte au plafond (pour la raccrocher).
    accroche: f32,
    pub etat: EtatStalactite,
}

impl Stalactite {
    /// Stalactite de la case `(x, y)` (coin haut-gauche), qui tombe jusqu'à `sol`.
    pub fn new(x: f32, y: f32, sol: f32) -> Self {
        Self {
            boite: Aabb::new(x + 3.0, y, 10.0, 14.0),
            colonne: Aabb::new(x, y, 16.0, sol - y),
            accroche: y,
            etat: EtatStalactite::Accrochee,
        }
    }

    /// Pépin (sa boîte) est-il passé dessous ? La déclenche si elle est accrochée.
    pub fn guetter(&mut self, joueur: &Aabb) {
        if self.etat == EtatStalactite::Accrochee && joueur.chevauche(&self.colonne) {
            self.etat = EtatStalactite::Tremble(TREMBLE_PAS);
        }
    }

    /// Mortelle en ce moment (en chute) ?
    pub fn tombe(&self) -> bool {
        matches!(self.etat, EtatStalactite::Chute(_))
    }

    /// Après une mort de Pépin : de nouveau accrochée au plafond.
    pub fn raccrocher(&mut self) {
        self.boite.y = self.accroche;
        self.etat = EtatStalactite::Accrochee;
    }
}

impl Actualisable for Stalactite {
    fn actualiser(&mut self, _env: &Environnement<'_>) {
        self.etat = match self.etat {
            EtatStalactite::Tremble(n) if n <= 1 => EtatStalactite::Chute(0.0),
            EtatStalactite::Tremble(n) => EtatStalactite::Tremble(n - 1),
            EtatStalactite::Chute(v) => {
                let v = (v + GRAVITE * DT).min(VITESSE_MAX_CHUTE);
                self.boite.y += v * DT;
                if self.boite.bas() >= self.colonne.bas() {
                    self.boite.y = self.colonne.bas() - self.boite.h;
                    EtatStalactite::Brisee(ECLATS_PAS)
                } else {
                    EtatStalactite::Chute(v)
                }
            }
            EtatStalactite::Brisee(n) => EtatStalactite::Brisee(n.saturating_sub(1)),
            EtatStalactite::Accrochee => EtatStalactite::Accrochee,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    /// La règle centrale : Pépin passe dessous, elle tremble 0,75 s, tombe
    /// (mortelle), puis se brise au sol.
    #[test]
    fn elle_tremble_trois_quarts_de_seconde_puis_tombe_et_se_brise() {
        let g = GrilleTuiles::vide(10, 10);
        let env = Environnement::simple(&g);
        let mut s = Stalactite::new(32.0, 16.0, 144.0);
        s.guetter(&Aabb::new(100.0, 130.0, 10.0, 14.0));
        assert_eq!(s.etat, EtatStalactite::Accrochee, "Pépin n'est pas dessous");
        s.guetter(&Aabb::new(36.0, 130.0, 10.0, 14.0));
        for _ in 0..TREMBLE_PAS - 1 {
            s.actualiser(&env);
        }
        assert!(!s.tombe(), "encore accrochée juste avant 0,75 s");
        s.actualiser(&env);
        assert!(s.tombe(), "elle tombe à 0,75 s");
        for _ in 0..60 {
            s.actualiser(&env);
        }
        assert!(matches!(s.etat, EtatStalactite::Brisee(_)), "{:?}", s.etat);
        assert_eq!(s.boite.bas(), 144.0, "brisée au sol");
    }
}
