//! Défis par niveau, médailles et bonus de fin de partie (carte 7.3) : la source
//! de revenus de la boutique. Rien ici ne touche au score ni à la physique : on
//! REGARDE la tentative (morts, parapluie, temps, gouttes) et on note le résultat
//! dans la sauvegarde.
//!
//! Chaque niveau (boss, bonus et épilogue compris) a 3 défis :
//! 1. « SANS MOURIR » : aucune mort pendant la tentative ;
//! 2. « SANS PLANER » (jamais le parapluie ouvert) là où le robot de faisabilité
//!    le prouve possible (`levels::SANS_PLANER_POSSIBLE`), sinon « LES 3 GOUTTES »
//!    (les trois gouttes d'or ramassées pendant la tentative) ;
//! 3. « CHRONO » : finir en `duree_cible_s` secondes au plus.
//!
//! Une TENTATIVE commence quand le niveau (re)démarre (nouveau niveau,
//! « recommencer », après « plus de vies ») et finit au drapeau. La MÉDAILLE est le
//! nombre de défis réussis dans une même tentative (1 bronze, 2 argent, 3 or), la
//! meilleure de toutes les tentatives.
//!
//! Revenus (constantes réglables en une ligne, `GAIN_*`) : chaque défi réussi pour
//! la PREMIÈRE fois, puis la PREMIÈRE médaille d'or du niveau (pas de farm : rejouer
//! un défi déjà réussi ne rapporte plus rien) ; à chaque partie finie, le bonus de
//! fin (`bonus_fin`). Les gouttes ramassées, elles, créditent toujours le
//! porte-monnaie à chaque niveau réussi, comme à la carte 7.2.
//!
//! Notions Rust illustrées ici : `enum` avec donnée (`Chrono(u32)`), tableaux
//! `[T; 3]` et `map` sur un tableau, fonctions pures testables sans fenêtre.

use pepin_core::levels::{NIVEAUX, SANS_PLANER_POSSIBLE};
use pepin_core::physics::HZ;
use pepin_core::zones::InfosNiveau;

use crate::sauvegarde::Sauvegarde;

/// Gouttes pour un défi réussi la première fois.
pub const GAIN_DEFI: u32 = 3;
/// Gouttes pour la première médaille d'or d'un niveau (en plus des défis).
pub const GAIN_OR: u32 = 5;
/// Bonus de fin de partie : partie finie.
pub const GAIN_PARTIE_FINIE: u32 = 5;
/// Bonus de fin de partie : aucun « plus de vies » de toute la partie.
pub const GAIN_SANS_GAME_OVER: u32 = 5;
/// Bonus de fin de partie : toutes les gouttes du parcours (bonus non compris).
pub const GAIN_TOUTES_LES_GOUTTES: u32 = 10;

/// Un défi de niveau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Defi {
    SansMourir,
    SansPlaner,
    TroisGouttes,
    /// Finir en au plus ce nombre de secondes.
    Chrono(u32),
}

impl Defi {
    /// Libellé affiché, en capitales.
    pub fn libelle(self) -> String {
        match self {
            Defi::SansMourir => "SANS MOURIR".into(),
            Defi::SansPlaner => "SANS PLANER".into(),
            Defi::TroisGouttes => "LES 3 GOUTTES".into(),
            Defi::Chrono(s) => format!("CHRONO {}:{:02}", s / 60, s % 60),
        }
    }
}

/// Les 3 défis du niveau d'indice `indice` dans `NIVEAUX` (hors table, niveaux
/// d'essai compris : « LES 3 GOUTTES » au milieu).
pub fn defis_du_niveau(indice: usize, infos: &InfosNiveau) -> [Defi; 3] {
    let milieu = if SANS_PLANER_POSSIBLE.get(indice) == Some(&true)
        && NIVEAUX.get(indice).map(|n| n.nom) == Some(infos.nom)
    {
        Defi::SansPlaner
    } else {
        Defi::TroisGouttes
    };
    [
        Defi::SansMourir,
        milieu,
        Defi::Chrono(u32::from(infos.duree_cible_s)),
    ]
}

/// Ce qu'on observe pendant une tentative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tentative {
    pub morts: u32,
    /// Le parapluie a-t-il été ouvert au moins une fois ?
    pub a_plane: bool,
    /// Durée de jeu de la tentative, en pas de 1/60 s.
    pub pas: u32,
    /// Gouttes d'or ramassées pendant la tentative.
    pub gouttes: [bool; 3],
}

impl Tentative {
    /// Le défi est-il réussi par cette tentative (finie) ?
    pub fn reussit(&self, defi: Defi) -> bool {
        match defi {
            Defi::SansMourir => self.morts == 0,
            Defi::SansPlaner => !self.a_plane,
            Defi::TroisGouttes => self.gouttes.iter().all(|&g| g),
            Defi::Chrono(s) => self.pas <= s * HZ,
        }
    }
}

/// Résultat d'un niveau réussi, pour l'affichage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultatNiveau {
    pub nom: &'static str,
    pub defis: [Defi; 3],
    /// Défis réussis par CETTE tentative.
    pub reussis: [bool; 3],
    /// Médaille de cette tentative (0 à 3).
    pub medaille: u8,
    /// Toutes les gouttes créditées au porte-monnaie à cette fin de niveau
    /// (ramassées + défis nouveaux + or).
    pub gain: u32,
}

/// Un niveau RÉUSSI : note la tentative dans la sauvegarde (gouttes trouvées,
/// meilleur temps, défis, médaille), crédite le porte-monnaie (gouttes ramassées
/// et premières réussites) et renvoie ce qu'il faut afficher. N'enregistre pas.
pub fn noter_niveau(
    s: &mut Sauvegarde,
    indice: usize,
    infos: &InfosNiveau,
    t: &Tentative,
) -> ResultatNiveau {
    let defis = defis_du_niveau(indice, infos);
    let reussis = defis.map(|d| t.reussit(d));
    let medaille = reussis.iter().filter(|&&r| r).count() as u8;
    let ramassees = t.gouttes.iter().filter(|&&g| g).count() as u32;
    let mut gain = ramassees;
    s.noter_niveau_atteint(indice);
    if let Some(p) = s.niveaux.get_mut(indice) {
        for i in 0..3 {
            p.gouttes[i] |= t.gouttes[i];
            if reussis[i] && !p.defis[i] {
                p.defis[i] = true;
                gain += GAIN_DEFI;
            }
        }
        if medaille == 3 && p.medaille < 3 {
            gain += GAIN_OR;
        }
        p.medaille = p.medaille.max(medaille);
        p.meilleur_temps = Some(p.meilleur_temps.map_or(t.pas, |m| m.min(t.pas)));
    }
    s.ajouter_gouttes(gain);
    ResultatNiveau {
        nom: infos.nom,
        defis,
        reussis,
        medaille,
        gain,
    }
}

/// Une ligne du bonus de fin de partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LigneBonus {
    pub nom: &'static str,
    /// Ce que rapporte la ligne quand elle est gagnée.
    pub gouttes: u32,
    pub gagne: bool,
}

/// Le bonus de fin de partie, ligne par ligne (une ligne ratée reste affichée, en
/// gris : le joueur voit ce qu'il aurait pu gagner).
pub fn bonus_fin(sans_game_over: bool, toutes_les_gouttes: bool) -> [LigneBonus; 3] {
    let ligne = |nom, gouttes, gagne| LigneBonus {
        nom,
        gouttes,
        gagne,
    };
    [
        ligne("FIN", GAIN_PARTIE_FINIE, true),
        ligne("SANS GAME OVER", GAIN_SANS_GAME_OVER, sans_game_over),
        ligne("100% GOUTTES", GAIN_TOUTES_LES_GOUTTES, toutes_les_gouttes),
    ]
}

/// Gouttes gagnées par un bonus de fin.
pub fn total_bonus(bonus: &[LigneBonus]) -> u32 {
    bonus.iter().filter(|l| l.gagne).map(|l| l.gouttes).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Défis de chaque niveau, réussites, médaille, gains de la PREMIÈRE réussite
    /// seulement (pas de farm), meilleur temps et gouttes trouvées gardés.
    #[test]
    fn les_defis_rapportent_une_seule_fois_et_gardent_le_meilleur() {
        for (i, n) in NIVEAUX.iter().enumerate() {
            let d = defis_du_niveau(i, n);
            assert_eq!(d[0], Defi::SansMourir);
            assert_eq!(d[2], Defi::Chrono(u32::from(n.duree_cible_s)));
            let attendu = if SANS_PLANER_POSSIBLE[i] {
                Defi::SansPlaner
            } else {
                Defi::TroisGouttes
            };
            assert_eq!(d[1], attendu, "{}", n.nom);
        }
        // Niveau 2 (sans planer possible) : 55 s.
        let (i, n) = (1, &NIVEAUX[1]);
        let mut s = Sauvegarde::default();
        let lente = Tentative {
            morts: 1,
            a_plane: false,
            pas: 70 * HZ,
            gouttes: [true, false, false],
        };
        let r = noter_niveau(&mut s, i, n, &lente);
        assert_eq!((r.reussis, r.medaille), ([false, true, false], 1));
        assert_eq!(r.gain, 1 + GAIN_DEFI);
        // La même chose encore : seule la goutte ramassée rapporte.
        assert_eq!(noter_niveau(&mut s, i, n, &lente).gain, 1);
        let parfaite = Tentative {
            morts: 0,
            a_plane: false,
            pas: 55 * HZ,
            gouttes: [false, false, true],
        };
        let r = noter_niveau(&mut s, i, n, &parfaite);
        assert_eq!((r.reussis, r.medaille), ([true; 3], 3));
        assert_eq!(r.gain, 1 + 2 * GAIN_DEFI + GAIN_OR);
        assert_eq!(noter_niveau(&mut s, i, n, &parfaite).gain, 1, "or déjà eu");
        let p = s.niveaux[i];
        assert_eq!(p.gouttes, [true, false, true]);
        assert_eq!((p.defis, p.medaille), ([true; 3], 3));
        assert_eq!(p.meilleur_temps, Some(55 * HZ));
        assert_eq!(s.niveau_max_atteint, 1);
        assert_eq!(
            s.gouttes_en_poche,
            1 + GAIN_DEFI + 1 + 1 + 2 * GAIN_DEFI + GAIN_OR + 1
        );
        // Une médaille moindre ensuite ne fait pas descendre la meilleure.
        noter_niveau(&mut s, i, n, &lente);
        assert_eq!(s.niveaux[i].medaille, 3);
        // Bonus de fin : 5 + 5 + 10 au mieux, 5 au moins.
        assert_eq!(total_bonus(&bonus_fin(true, true)), 20);
        assert_eq!(total_bonus(&bonus_fin(false, false)), GAIN_PARTIE_FINIE);
        // Les bandes d'affichage tiennent dans l'écran, pour chaque niveau.
        for (i, n) in NIVEAUX.iter().enumerate() {
            let r = noter_niveau(&mut Sauvegarde::default(), i, n, &lente);
            let (defis, bonus) =
                crate::ecrans::resultats::largeurs_pire_cas(&r, &bonus_fin(true, true));
            let max = crate::ecrans::resultats::LARGEUR_BANDE_FIN;
            assert!(defis <= max && bonus <= max, "{} : {defis} {bonus}", n.nom);
        }
    }
}
