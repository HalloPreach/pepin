//! La FUITE DE LA PIEUVRE AU SOMMET, qu'on voit enfin (carte 9.36c).
//!
//! Le problème (relevé par la 9.36b) : quand Pépin atteint le sommet de la tour,
//! la Pieuvre plonge au large et la mer redescend, mais tout cela se passe
//! plusieurs écrans plus bas ; la caméra restait sur Pépin, on ne voyait rien.
//!
//! La séquence de fin, en deux temps :
//!
//! ```text
//!   Pépin touche le sommet ─► DESCENTE (`DESCENTE_PAS`) ─► PLONGÉE (`PLONGEE_PAS`) ─► bilan
//!                             le monde est figé, la       le monde reprend, Pépin
//!                             caméra descend seule        ne bouge plus ; la caméra
//!                             (en douceur) vers la mer    suit la mer qui redescend
//! ```
//!
//! LA RÈGLE NE CHANGE PAS. Le monde ne fait AUCUN pas pendant la descente : le
//! chrono, les gouttes et la sauvegarde ne la voient pas. La plongée, elle, est
//! exactement celle d'avant la carte (les mêmes pas du monde, comptés de la même
//! façon) : le chrono enregistré est donc le même, au pas près, avec ou sans
//! la séquence. Seules les entrées de Pépin sont ignorées (il est figé).
//!
//! PASSER LA SÉQUENCE : un appui sur SAUT/Entrée (ou un clic), comme pour les
//! écrans de texte (carte 9.31 : ignoré pendant les `DELAI_PAS` premiers pas).
//! Les pas de la plongée qui restent sont alors joués tout de suite, d'un coup :
//! même fin, même chrono.
//!
//! Notions Rust : une petite structure `Copy` qui garde l'état de la séquence
//! (`Option<FuiteAuSommet>` dans l'`Application` : `None` le reste du temps) ;
//! des fonctions pures (`cadre_de_la_fuite`, `adoucir`) que les tests appellent
//! sans fenêtre.

use pepin_core::entities::pieuvre::{PhasePieuvre, Pieuvre, PLONGEE_PAS};
use pepin_core::player::Entrees;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::dessin_pieuvre::la_pieuvre;
use crate::ecrans::histoire;
use crate::monde::Monde;
use crate::navigation::Navigation;

use super::Application;

/// La descente de la caméra, de Pépin jusqu'à la mer : de 0,75 s au moins
/// (`DESCENTE_MIN_PAS`) à 1,5 s au plus (`DESCENTE_MAX_PAS`), selon la distance
/// (`duree_descente`). Avec la plongée (`PLONGEE_PAS`, 2 s), la séquence dure donc
/// de 2,75 s à 3,5 s.
pub const DESCENTE_MIN_PAS: u32 = 45;
pub const DESCENTE_MAX_PAS: u32 = 90;
/// Vitesse visée de la caméra au plus fort de la descente (px par pas) : une
/// descente plus longue prend plus de temps, jusqu'à `DESCENTE_MAX_PAS`.
pub const VITESSE_DESCENTE: f32 = 16.0;
/// Durée totale de la séquence au plus (pas) : la descente puis la plongée.
#[cfg(test)]
pub const SEQUENCE_MAX_PAS: u32 = DESCENTE_MAX_PAS + PLONGEE_PAS as u32;
/// Dans le cadre de la fuite, la surface de la mer est à cette part de la hauteur
/// de l'écran (99 px du haut) : la tête de la Pieuvre, son encre et le bas des
/// corniches noyées au-dessus, de l'eau en dessous.
const PART_MER: f32 = 0.55;
/// Dans le cadre de la fuite, le centre de la tête de la Pieuvre est à cette part
/// de la largeur de l'écran (à droite : la tour et ses corniches à gauche).
const PART_TETE: f32 = 0.70;

/// L'état de la séquence (`Application::fuite_au_sommet`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuiteAuSommet {
    /// Pas écoulés depuis le début de la séquence.
    pub pas: u32,
    /// Durée de la descente (pas), fixée au début (`duree_descente`).
    pub descente: u32,
    /// La caméra quand Pépin a touché le sommet (le départ de la descente).
    depart: (f32, f32),
    /// Un appui l'a passée : la plongée se joue d'un coup au prochain pas.
    passee: bool,
}

impl FuiteAuSommet {
    /// La caméra descend encore (le monde est figé) ? (Tests.)
    #[cfg(test)]
    pub fn en_descente(&self) -> bool {
        self.pas < self.descente
    }
}

/// La durée de la descente (pas) pour une distance de `distance` px : au plus
/// fort (pente 1,5 de `adoucir`), la caméra va à `VITESSE_DESCENTE` px par pas,
/// sans sortir de `DESCENTE_MIN_PAS`..=`DESCENTE_MAX_PAS`. Pure.
pub fn duree_descente(distance: f32) -> u32 {
    let pas = (1.5 * distance.abs() / VITESSE_DESCENTE).ceil() as u32;
    pas.clamp(DESCENTE_MIN_PAS, DESCENTE_MAX_PAS)
}

/// Le coin haut-gauche de la caméra qui cadre la fuite : la tête de la Pieuvre au
/// loin (son centre en `x_tete`) et la mer (`mer`, sa surface), en pixels du
/// niveau (avant bornage au niveau). Pure.
pub fn cadre_de_la_fuite(x_tete: f32, mer: f32) -> (f32, f32) {
    (
        x_tete - LARGEUR_LOGIQUE as f32 * PART_TETE,
        mer - HAUTEUR_LOGIQUE as f32 * PART_MER,
    )
}

/// La courbe de la descente, de 0 à 1 (`t` de 0 à 1) : lente au départ, rapide
/// au milieu, lente à l'arrivée (« smoothstep »), donc sans à-coup. Sa pente la
/// plus forte vaut 1,5 (au milieu). Pure.
pub fn adoucir(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// La Pieuvre de la phase 2 qui vient de commencer sa fuite au sommet (plongée
/// en cours, mer et tête au loin connues), ou `None`.
fn pieuvre_qui_fuit(monde: &Monde) -> Option<&Pieuvre> {
    la_pieuvre(monde)
        .filter(|p| p.en_fuite() && p.phase() == PhasePieuvre::Plongee && p.sommet.is_some())
}

/// Le cadre de la fuite pour le monde tel qu'il est (`None` sans fuite en cours).
fn cadre_du_monde(monde: &Monde) -> Option<(f32, f32)> {
    let p = pieuvre_qui_fuit(monde)?;
    let tete = p.au_loin()?;
    let mer = p.surface_mer()?;
    Some(cadre_de_la_fuite(tete.x + tete.w / 2.0, mer))
}

impl Application {
    /// Un pas de l'écran de jeu (`Application::pas`). Hors de la séquence : le pas
    /// du niveau habituel (`pas_jeu`), qui la commence si Pépin vient de toucher le
    /// sommet. Renvoie `true` si le monde a avancé (pour l'audio).
    pub(super) fn pas_en_jeu(&mut self, entrees: &Entrees) -> bool {
        let Some(mut f) = self.fuite_au_sommet else {
            self.pas_jeu(entrees);
            self.commencer_la_fuite_si_besoin();
            return true;
        };
        // Passée : les pas de la plongée qui restent, d'un coup (même chrono).
        if f.passee {
            for _ in 0..=PLONGEE_PAS {
                if self.monde.termine || self.fuite_au_sommet.is_none() {
                    break;
                }
                self.pas_jeu(&Entrees::default());
            }
            self.fuite_au_sommet = None;
            return true;
        }
        f.pas += 1;
        self.fuite_au_sommet = Some(f);
        if f.pas <= f.descente {
            // La descente : le monde ne bouge pas, seule la caméra.
            self.monde.figer_interpolation();
            self.mener_la_camera();
            return false;
        }
        // La plongée : le monde reprend, Pépin figé (aucune entrée).
        self.pas_jeu(&Entrees::default());
        if self.monde.termine {
            self.fuite_au_sommet = None;
        }
        true
    }

    /// Juste après un pas du niveau : Pépin vient-il de toucher le sommet (la
    /// Pieuvre commence sa plongée) ? Alors la séquence commence.
    fn commencer_la_fuite_si_besoin(&mut self) {
        let Some((x, y)) = cadre_du_monde(&self.monde).filter(|_| !self.monde.termine) else {
            return;
        };
        let depart = (self.camera.x, self.camera.y);
        let distance = (x - depart.0).abs().max((y - depart.1).abs());
        self.fuite_au_sommet = Some(FuiteAuSommet {
            pas: 0,
            descente: duree_descente(distance),
            depart,
            passee: false,
        });
    }

    /// La caméra pendant la séquence : pendant la descente, de sa place au sommet
    /// jusqu'au cadre de la fuite (`adoucir`) ; pendant la plongée, elle suit la mer
    /// qui redescend. Appelée à chaque pas de la séquence (`camera_effets` pour les
    /// pas de la plongée).
    pub(super) fn mener_la_camera(&mut self) {
        let (Some(f), Some((x, y))) = (self.fuite_au_sommet, cadre_du_monde(&self.monde)) else {
            // Elle a fini de plonger (le dernier pas) : la caméra reste où elle est.
            self.camera.figer();
            return;
        };
        let (largeur, hauteur) = (
            self.monde.niveau.largeur_px(),
            self.monde.niveau.hauteur_px(),
        );
        let (x, y) = if f.pas < f.descente {
            let k = adoucir(f.pas as f32 / f.descente as f32);
            (
                f.depart.0 + (x - f.depart.0) * k,
                f.depart.1 + (y - f.depart.1) * k,
            )
        } else {
            (x, y)
        };
        self.camera.placer(x, y, largeur, hauteur);
    }

    /// Un appui (SAUT/Entrée, clic) pendant la séquence, passé le délai des écrans
    /// de texte (`histoire::prete`) : elle sera passée au prochain pas. Renvoie
    /// `true` si l'appui a servi à cela.
    pub(super) fn passer_la_fuite(&mut self, nav: &Navigation) -> bool {
        let Some(f) = self.fuite_au_sommet.as_mut() else {
            return false;
        };
        if !histoire::prete(f.pas) || !(nav.valider || nav.clic.is_some()) {
            return false;
        }
        f.passee = true;
        true
    }

    /// La séquence en cours (robots, captures et tests) : ses pas, et la durée de
    /// la descente.
    #[cfg(test)]
    pub fn fuite_en_cours(&self) -> Option<FuiteAuSommet> {
        self.fuite_au_sommet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_sequence_dure_de_2_a_3_5_secondes() {
        for d in [0.0, 100.0, 500.0, 1100.0, 5000.0] {
            let s = (duree_descente(d) + u32::from(PLONGEE_PAS)) as f32 / 60.0;
            assert!((2.0..=3.5).contains(&s), "{d} px : {s} s");
        }
        assert_eq!(SEQUENCE_MAX_PAS, 210);
        const { assert!(histoire::DELAI_PAS < DESCENTE_MIN_PAS) };
        // Une descente qui tient dans la durée la plus longue va au plus à
        // `VITESSE_DESCENTE` px par pas.
        for d in [10.0, 300.0, 960.0] {
            assert!(1.5 * d / duree_descente(d) as f32 <= VITESSE_DESCENTE);
        }
    }

    #[test]
    fn la_courbe_part_et_arrive_sans_a_coup() {
        assert_eq!(adoucir(0.0), 0.0);
        assert_eq!(adoucir(1.0), 1.0);
        assert!((adoucir(0.5) - 0.5).abs() < 1e-6);
        // Croissante, et sa pente ne dépasse jamais 1,5.
        let mut avant = 0.0;
        for i in 1..=1000 {
            let v = adoucir(i as f32 / 1000.0);
            assert!(v >= avant);
            assert!((v - avant) * 1000.0 <= 1.5 + 1e-3);
            avant = v;
        }
    }

    #[test]
    fn le_cadre_montre_la_tete_et_la_mer() {
        let (x, y) = cadre_de_la_fuite(500.0, 1000.0);
        let (l, h) = (LARGEUR_LOGIQUE as f32, HAUTEUR_LOGIQUE as f32);
        assert!(
            x < 500.0 - 24.0 && 500.0 + 24.0 < x + l,
            "la tête à l'écran"
        );
        assert!(
            y < 1000.0 - 40.0 && 1000.0 + 40.0 < y + h,
            "la mer à l'écran"
        );
    }
}
