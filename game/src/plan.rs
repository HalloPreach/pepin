//! Carte des niveaux (carte 7.4) : choisir un niveau pour le REJOUER et s'améliorer
//! (meilleur temps, gouttes, défis, médaille). Une carte VERTICALE qui monte des
//! toits (en bas) jusqu'au soleil (en haut) : les niveaux normaux sont les étapes
//! du chemin qui monte, le bonus est une branche secrète à gauche près du soleil,
//! l'épilogue redescend de l'autre côté (à droite).
//!
//! Ce module est PUR (aucun dessin) : il dit où est chaque étape (`Geometrie`),
//! si elle est jouable (`etat_etape`, d'après la sauvegarde v2 de la carte 7.3) et
//! ce que fait un appui (`EtatPlan::reagir`). L'écran est dans `ecrans/plan.rs`.
//!
//! Tout se déduit de la table `NIVEAUX` (genre de chaque niveau) : aucun numéro ni
//! nombre de niveaux en dur. Un niveau normal ajouté à la table devient une étape
//! de plus sur le chemin, la carte s'allonge toute seule.
//!
//! Notions Rust illustrées ici : `enum` d'états, itérateurs (`filter`, `position`,
//! `rev`), `f32::clamp`, trait pour les zones cliquables (`Zones`) que l'écran
//! fournit (la logique ne connaît pas la mise en page exacte).

use pepin_core::levels::{gouttes_pour_le_bonus, GenreNiveau};
use pepin_core::zones::InfosNiveau;

use crate::navigation::Navigation;
use crate::sauvegarde::Sauvegarde;

/// Ce qu'une étape montre et permet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatEtape {
    /// Pas encore atteinte : grisée, avec un cadenas ; pas jouable.
    Verrouillee,
    /// Le bonus tant qu'il n'est pas débloqué : « ? », nom caché ; pas jouable.
    Cachee,
    /// Jouable, jamais finie.
    Ouverte,
    /// Finie au moins une fois (elle a un meilleur temps).
    Finie,
}

impl EtatEtape {
    pub fn jouable(self) -> bool {
        matches!(self, EtatEtape::Ouverte | EtatEtape::Finie)
    }
}

/// Le niveau `i` a-t-il été fini au moins une fois ?
fn fini(s: &Sauvegarde, i: usize) -> bool {
    s.niveaux.get(i).is_some_and(|p| p.meilleur_temps.is_some())
}

/// Gouttes TROUVÉES (au cumul de la sauvegarde) dans les niveaux normaux.
pub fn gouttes_trouvees_normales(niveaux: &[InfosNiveau], s: &Sauvegarde) -> u32 {
    niveaux
        .iter()
        .zip(&s.niveaux)
        .filter(|(n, _)| n.genre == GenreNiveau::Normal)
        .map(|(_, p)| p.gouttes.iter().filter(|&&g| g).count() as u32)
        .sum()
}

/// État de l'étape `i` de `niveaux` d'après la sauvegarde :
/// - le premier niveau est toujours ouvert ;
/// - un niveau normal ou l'épilogue est ouvert s'il a été atteint
///   (`niveau_max_atteint`) ou si le niveau qui le précède sur le chemin (bonus
///   sauté) a été fini : l'épilogue s'ouvre donc une fois le dernier niveau normal
///   fini ;
/// - le bonus se révèle avec toutes les gouttes des niveaux normaux TROUVÉES (au
///   cumul de la sauvegarde, la règle de `levels::bonus_debloque`), ou s'il a déjà
///   été fini ; sinon il reste caché.
pub fn etat_etape(niveaux: &[InfosNiveau], s: &Sauvegarde, i: usize) -> EtatEtape {
    let Some(n) = niveaux.get(i) else {
        return EtatEtape::Verrouillee;
    };
    let ouverte = match n.genre {
        GenreNiveau::Bonus => {
            let toutes = gouttes_trouvees_normales(niveaux, s) >= gouttes_pour_le_bonus(niveaux);
            if !(toutes || fini(s, i)) {
                return EtatEtape::Cachee;
            }
            true
        }
        GenreNiveau::Normal | GenreNiveau::Epilogue => {
            let precedent = (0..i)
                .rev()
                .find(|&j| niveaux[j].genre != GenreNiveau::Bonus);
            i == 0 || i <= s.niveau_max_atteint || precedent.is_some_and(|j| fini(s, j))
        }
    };
    match (ouverte, fini(s, i)) {
        (_, true) => EtatEtape::Finie,
        (true, false) => EtatEtape::Ouverte,
        (false, false) => EtatEtape::Verrouillee,
    }
}

/// Progression de mise au point (touche U du build de QA, carte 7.4) : premier appui,
/// les sept premiers niveaux finis (médailles et gouttes variées, bonus encore
/// caché) ; appui suivant, tous les niveaux finis avec toutes leurs gouttes (bonus
/// révélé). Seulement la progression : le porte-monnaie et les records ne bougent pas.
pub fn progression_qa(niveaux: &[InfosNiveau], s: &mut Sauvegarde) {
    let tout = s.niveaux.iter().take(7).all(|p| p.meilleur_temps.is_some());
    // On ne fait qu'AJOUTER à la progression déjà là (rien de vrai n'est effacé).
    for (i, p) in s.niveaux.iter_mut().enumerate().take(niveaux.len()) {
        let (gouttes, temps, medaille) = if tout {
            ([true; 3], 45 * 60 + i as u32 * 97, 1 + (i % 3) as u8)
        } else if i < 7 {
            (
                [i % 3 != 2, true, i % 2 == 0],
                30 * 60 + i as u32 * 431,
                (i % 4) as u8,
            )
        } else {
            continue;
        };
        for (k, (g, d)) in p.gouttes.iter_mut().zip(&mut p.defis).enumerate() {
            *g |= gouttes[k];
            *d |= medaille as usize > k;
        }
        p.meilleur_temps = Some(p.meilleur_temps.map_or(temps, |t| t.min(temps)));
        p.medaille = p.medaille.max(medaille);
    }
    let atteint = if tout { niveaux.len() - 1 } else { 7 };
    s.niveau_max_atteint = s.niveau_max_atteint.max(atteint);
}

// --- Géométrie de la carte (pixels du « monde » de la carte) ---------------------

/// Écart vertical entre deux niveaux normaux.
pub const ECART: f32 = 44.0;
/// Ordonnée du soleil (son centre) en haut de la carte.
pub const Y_SOLEIL: f32 = 26.0;
/// Le niveau normal le plus haut, sous le soleil.
const Y_SOMMET: f32 = 70.0;
/// Hauteur des toits sous la première étape.
const SOUS_PREMIERE: f32 = 26.0;
/// Hauteur des toits dessinés en bas de la carte.
pub const H_TOITS: f32 = 32.0;
/// Colonnes du chemin qui serpente (centre des étapes), puis la branche secrète du
/// bonus (à gauche) et la descente (à droite).
const COLONNES: [f32; 4] = [60.0, 92.0, 124.0, 92.0];
pub const X_BONUS: f32 = 22.0;
pub const X_DESCENTE: f32 = 154.0;

/// Où sont les étapes de la carte.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometrie {
    /// Centre de chaque étape, dans l'ordre de la table.
    pub etapes: Vec<(f32, f32)>,
    /// Hauteur totale de la carte.
    pub hauteur: f32,
}

impl Geometrie {
    pub fn new(niveaux: &[InfosNiveau]) -> Self {
        let nb = niveaux
            .iter()
            .filter(|n| n.genre == GenreNiveau::Normal)
            .count()
            .max(1);
        let y_premier = Y_SOMMET + (nb - 1) as f32 * ECART;
        let hauteur = y_premier + SOUS_PREMIERE + H_TOITS;
        let (mut k, mut b, mut e) = (0usize, 0usize, 0usize);
        let etapes = niveaux
            .iter()
            .map(|n| match n.genre {
                GenreNiveau::Normal => {
                    let p = (COLONNES[k % COLONNES.len()], y_premier - k as f32 * ECART);
                    k += 1;
                    p
                }
                // La branche secrète part du sommet vers la gauche, un peu plus haut.
                GenreNiveau::Bonus => {
                    b += 1;
                    (X_BONUS, Y_SOMMET - 22.0 + (b - 1) as f32 * ECART)
                }
                // La descente : de l'autre côté du soleil, deux étages plus bas.
                GenreNiveau::Epilogue => {
                    e += 1;
                    (X_DESCENTE, Y_SOMMET + (e + 1) as f32 * ECART)
                }
            })
            .collect();
        Self { etapes, hauteur }
    }
}

// --- Réactions aux appuis ---------------------------------------------------------

/// Hauteur visible de la carte à l'écran (entre le bandeau du haut et celui du bas).
pub const H_VUE: f32 = 152.0;
/// Défilement d'un cran de molette.
pub const PAS_MOLETTE: f32 = 24.0;

/// Les zones cliquables, fournies par l'écran (coordonnées logiques de l'écran).
pub trait Zones {
    /// L'étape sous le point (avec le défilement `defil`).
    fn etape_sous(&self, g: &Geometrie, defil: f32, x: f32, y: f32) -> Option<usize>;
    /// Le bouton JOUER contient-il le point ?
    fn sur_jouer(&self, x: f32, y: f32) -> bool;
    /// Le bouton RETOUR contient-il le point ?
    fn sur_retour(&self, x: f32, y: f32) -> bool;
}

/// Ce que l'application doit faire après un appui.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionPlan {
    Rien,
    /// Retour au titre.
    Quitter,
    /// Jouer le niveau d'indice donné (partie d'un seul niveau).
    Jouer(usize),
}

/// L'écran de la carte : étape choisie et défilement.
#[derive(Debug, Clone, PartialEq)]
pub struct EtatPlan {
    pub geometrie: Geometrie,
    /// Indice (dans la table) de l'étape choisie.
    pub selection: usize,
    /// Ordonnée de la carte en haut de la vue.
    pub defil: f32,
}

impl EtatPlan {
    /// Ouvre la carte sur `selection` (bornée), la vue centrée dessus.
    pub fn new(niveaux: &[InfosNiveau], selection: usize) -> Self {
        let mut e = Self {
            geometrie: Geometrie::new(niveaux),
            selection: selection.min(niveaux.len().saturating_sub(1)),
            defil: 0.0,
        };
        e.centrer();
        e
    }

    /// Le niveau où l'on en est : le plus loin des niveaux jouables du chemin (le
    /// bonus n'y compte pas).
    pub fn ou_on_en_est(niveaux: &[InfosNiveau], s: &Sauvegarde) -> usize {
        (0..niveaux.len())
            .rev()
            .find(|&i| {
                niveaux[i].genre != GenreNiveau::Bonus && etat_etape(niveaux, s, i).jouable()
            })
            .unwrap_or(0)
    }

    fn defil_max(&self) -> f32 {
        (self.geometrie.hauteur - H_VUE).max(0.0)
    }

    fn borner(&mut self) {
        self.defil = self.defil.clamp(0.0, self.defil_max()).round();
    }

    /// Centre la vue sur l'étape choisie.
    pub fn centrer(&mut self) {
        if let Some(&(_, y)) = self.geometrie.etapes.get(self.selection) {
            self.defil = y - H_VUE / 2.0;
        }
        self.borner();
    }

    /// Un appui (une image) : haut / bas (ou droite / gauche) parcourent les étapes
    /// dans l'ordre du chemin ; Entrée ou JOUER lance l'étape choisie si elle est
    /// jouable ; toucher une étape la choisit ; molette et glisser font défiler ;
    /// Échap ou RETOUR revient au titre.
    pub fn reagir(
        &mut self,
        nav: &Navigation,
        niveaux: &[InfosNiveau],
        s: &Sauvegarde,
        z: &impl Zones,
    ) -> ActionPlan {
        if nav.retour {
            return ActionPlan::Quitter;
        }
        let nb = niveaux.len();
        if nb == 0 {
            return ActionPlan::Rien;
        }
        let jouer = |sel: usize| {
            if etat_etape(niveaux, s, sel).jouable() {
                ActionPlan::Jouer(sel)
            } else {
                ActionPlan::Rien
            }
        };
        if (nav.haut || nav.droite) && self.selection + 1 < nb {
            self.selection += 1;
            self.centrer();
        }
        if (nav.bas || nav.gauche) && self.selection > 0 {
            self.selection -= 1;
            self.centrer();
        }
        if nav.molette != 0.0 {
            self.defil -= nav.molette * PAS_MOLETTE;
            self.borner();
        }
        if nav.glisse != 0.0 {
            self.defil -= nav.glisse;
            self.borner();
        }
        if let Some((x, y)) = nav.clic {
            if z.sur_retour(x, y) {
                return ActionPlan::Quitter;
            }
            if z.sur_jouer(x, y) {
                return jouer(self.selection);
            }
            if let Some(i) = z.etape_sous(&self.geometrie, self.defil, x, y) {
                // Toucher choisit seulement (un glisser commence aussi par un appui) :
                // on lance avec JOUER ou Entrée.
                self.selection = i;
            }
        }
        if nav.valider {
            return jouer(self.selection);
        }
        ActionPlan::Rien
    }
}
