//! Défis par niveau, médailles et bonus de fin de partie (carte 7.3) : la source
//! de revenus de la boutique. Rien ici ne touche au score ni à la physique : on
//! REGARDE la tentative (morts, parapluie, temps, gouttes) et on note le résultat
//! dans la sauvegarde.
//!
//! Chaque niveau (boss, bonus et épilogue compris) a 3 défis :
//! 1. « SANS MOURIR » : aucune mort pendant la tentative ;
//! 2. « SANS PLANER » (jamais le parapluie ouvert) là où le robot de faisabilité
//!    le prouve possible (champ `InfosNiveau::sans_planer`), sinon « LES 3 GOUTTES »
//!    (les trois gouttes d'or ramassées pendant la tentative) ;
//! 3. « CHRONO » : finir en `duree_cible_s` secondes au plus.
//!
//! Une TENTATIVE commence quand le niveau (re)démarre (nouveau niveau,
//! « recommencer », après « plus de vies ») et finit au drapeau. La MÉDAILLE est le
//! nombre de défis réussis dans une même tentative (1 bronze, 2 argent, 3 or), la
//! meilleure de toutes les tentatives.
//!
//! Revenus (constantes réglables en une ligne, `GAIN_*`). RÈGLE de la carte 7.9 :
//! CHAQUE RÉCOMPENSE N'EST CRÉDITÉE QU'UNE FOIS (pas de farm avec REJOUER) :
//! - une goutte d'or (clé du niveau + numéro de la goutte) : la première fois qu'elle
//!   est ramassée dans un niveau RÉUSSI ;
//! - un défi : la première fois qu'il est réussi ;
//! - une médaille : seulement la DIFFÉRENCE de valeur quand on l'améliore
//!   (`valeur_medaille` : aujourd'hui, seul l'or vaut `GAIN_OR`) ;
//! - un bonus de fin de partie (`bonus_fin`) : la première fois pour chacun.
//!
//! Le registre de ce qui a déjà été crédité EST la progression de la sauvegarde
//! (gouttes trouvées, défis réussis, meilleure médaille, `bonus_credites`). REJOUER
//! reste utile pour le meilleur temps, la médaille et les défis manquants.
//!
//! Notions Rust illustrées ici : `enum` avec donnée (`Chrono(u32)`), tableaux
//! `[T; 3]` et `map` sur un tableau, fonctions pures testables sans fenêtre.

use pepin_core::physics::HZ;
use pepin_core::zones::InfosNiveau;

use crate::sauvegarde::Sauvegarde;

/// Gouttes pour un défi réussi la première fois.
pub const GAIN_DEFI: u32 = 3;
/// Gouttes pour la première médaille d'or d'un niveau (en plus des défis).
pub const GAIN_OR: u32 = 5;

/// Valeur d'une médaille (0 à 3 défis réussis) : améliorer sa médaille rapporte la
/// différence de valeur, une seule fois. Seul l'or vaut quelque chose aujourd'hui.
pub fn valeur_medaille(medaille: u8) -> u32 {
    if medaille >= 3 {
        GAIN_OR
    } else {
        0
    }
}
/// Bonus de fin de partie : partie finie.
pub const GAIN_PARTIE_FINIE: u32 = 5;
/// Bonus de fin de partie : aucun « plus de vies » de toute la partie.
pub const GAIN_SANS_GAME_OVER: u32 = 5;
/// Bonus de fin de partie : toutes les gouttes du parcours (bonus non compris).
pub const GAIN_TOUTES_LES_GOUTTES: u32 = 10;
/// Course contre le robot (carte 8.7) : première victoire contre la force EXPERT sur
/// un niveau (une fois par niveau, voir `crediter_course_expert`).
pub const GAIN_COURSE_EXPERT: u32 = 5;

/// Identifiant (dans `bonus_credites`) de la prime « EXPERT battu » du niveau de clé
/// `cle`. Ne jamais le changer : la prime serait créditée une seconde fois.
pub fn bonus_course_expert(cle: &str) -> String {
    format!("course_expert_{cle}")
}

/// L'EXPERT vient d'être battu sur le niveau de clé `cle` : la prime la PREMIÈRE fois
/// seulement (créditée au porte-monnaie). Renvoie le gain (0 si déjà touchée).
/// N'enregistre pas.
pub fn crediter_course_expert(s: &mut Sauvegarde, cle: &str) -> u32 {
    if s.crediter_bonus(&bonus_course_expert(cle)) {
        s.ajouter_gouttes(GAIN_COURSE_EXPERT);
        GAIN_COURSE_EXPERT
    } else {
        0
    }
}

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

/// Les 3 défis du niveau `infos` (milieu : son champ `sans_planer`, rangé par clé
/// depuis la carte 8.13 ; niveaux d'essai : « LES 3 GOUTTES »).
pub fn defis_du_niveau(infos: &InfosNiveau) -> [Defi; 3] {
    let milieu = if infos.sans_planer {
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
    /// Gouttes VRAIMENT créditées au porte-monnaie à cette fin de niveau (gouttes
    /// nouvelles + défis nouveaux + amélioration de médaille) : +0 est possible.
    pub gain: u32,
    /// Gouttes d'or ramassées cette fois mais déjà comptées avant (affichées « déjà
    /// comptées », sans rien rapporter).
    pub deja_comptees: u32,
    /// Boss vaincu en MODE FACILE (carte 8.14, `facile::noter_victoire_facile`) :
    /// ni défi ni médaille, mention « FACILE » au bilan.
    pub facile: bool,
}

/// Un niveau RÉUSSI : note la tentative dans la sauvegarde, par la CLÉ du niveau
/// (gouttes trouvées, meilleur temps, défis, médaille), crédite le porte-monnaie de
/// ce qui ne l'avait jamais été et renvoie ce qu'il faut afficher. N'enregistre pas.
/// Niveau hors de la table (essai du build de QA) : rien n'est noté ni crédité.
pub fn noter_niveau(s: &mut Sauvegarde, infos: &InfosNiveau, t: &Tentative) -> ResultatNiveau {
    let defis = defis_du_niveau(infos);
    let reussis = defis.map(|d| t.reussit(d));
    let medaille = reussis.iter().filter(|&&r| r).count() as u8;
    let (mut gain, mut deja_comptees) = (0, 0);
    if let Some(p) = s.progres_mut(infos.cle) {
        p.atteint = true;
        for (i, &reussi) in reussis.iter().enumerate() {
            // Une goutte : créditée la première fois qu'elle est trouvée.
            if t.gouttes[i] {
                if p.gouttes[i] {
                    deja_comptees += 1;
                } else {
                    p.gouttes[i] = true;
                    gain += 1;
                }
            }
            // Un défi : crédité la première fois qu'il est réussi.
            if reussi && !p.defis[i] {
                p.defis[i] = true;
                gain += GAIN_DEFI;
            }
        }
        // Une médaille : la différence de valeur, seulement si elle s'améliore.
        gain += valeur_medaille(medaille).saturating_sub(valeur_medaille(p.medaille));
        p.medaille = p.medaille.max(medaille);
        p.meilleur_temps = Some(p.meilleur_temps.map_or(t.pas, |m| m.min(t.pas)));
        // Carte 8.14 : le boss vaincu, ses défaites d'affilée repartent de zéro.
        p.defaites = 0;
    }
    s.ajouter_gouttes(gain);
    ResultatNiveau {
        nom: infos.nom,
        defis,
        reussis,
        medaille,
        gain,
        deja_comptees,
        facile: false,
    }
}

/// Identifiants des bonus de fin dans la sauvegarde (`bonus_credites`). Ne jamais
/// les changer : un bonus renommé serait crédité une seconde fois.
pub const BONUS_FIN: &str = "fin";
pub const BONUS_SANS_GAME_OVER: &str = "sans_game_over";
pub const BONUS_TOUTES_LES_GOUTTES: &str = "toutes_les_gouttes";

/// Une ligne du bonus de fin de partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LigneBonus {
    /// Identifiant dans la sauvegarde (`BONUS_*`).
    pub id: &'static str,
    pub nom: &'static str,
    /// Ce que rapporte la ligne la première fois qu'elle est gagnée.
    pub gouttes: u32,
    pub gagne: bool,
    /// Gagnée, mais déjà créditée lors d'une partie précédente : ne rapporte rien.
    pub deja: bool,
}

/// Le bonus de fin de partie, ligne par ligne (une ligne ratée reste affichée, en
/// gris : le joueur voit ce qu'il aurait pu gagner). Rien n'est encore crédité :
/// voir `crediter_bonus_fin`.
pub fn bonus_fin(sans_game_over: bool, toutes_les_gouttes: bool) -> [LigneBonus; 3] {
    let ligne = |id, nom, gouttes, gagne| LigneBonus {
        id,
        nom,
        gouttes,
        gagne,
        deja: false,
    };
    [
        ligne(BONUS_FIN, "FIN", GAIN_PARTIE_FINIE, true),
        ligne(
            BONUS_SANS_GAME_OVER,
            "SANS GAME OVER",
            GAIN_SANS_GAME_OVER,
            sans_game_over,
        ),
        ligne(
            BONUS_TOUTES_LES_GOUTTES,
            "100% GOUTTES",
            GAIN_TOUTES_LES_GOUTTES,
            toutes_les_gouttes,
        ),
    ]
}

/// Crédite le bonus de fin : chaque ligne gagnée pour la PREMIÈRE fois rapporte (et
/// est notée dans la sauvegarde) ; une ligne déjà créditée est marquée `deja`.
/// Renvoie le gain crédité. N'enregistre pas.
pub fn crediter_bonus_fin(s: &mut Sauvegarde, bonus: &mut [LigneBonus]) -> u32 {
    for l in bonus.iter_mut().filter(|l| l.gagne) {
        l.deja = !s.crediter_bonus(l.id);
    }
    let gain = total_bonus(bonus);
    s.ajouter_gouttes(gain);
    gain
}

/// Gouttes VRAIMENT rapportées par un bonus de fin (lignes gagnées, pas déjà
/// créditées).
pub fn total_bonus(bonus: &[LigneBonus]) -> u32 {
    bonus
        .iter()
        .filter(|l| l.gagne && !l.deja)
        .map(|l| l.gouttes)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::levels::NIVEAUX;

    /// Défis de chaque niveau, réussites, médaille, gains de la PREMIÈRE réussite
    /// seulement (pas de farm), meilleur temps et gouttes trouvées gardés.
    #[test]
    fn les_defis_rapportent_une_seule_fois_et_gardent_le_meilleur() {
        for n in NIVEAUX {
            let d = defis_du_niveau(n);
            assert_eq!(d[0], Defi::SansMourir);
            assert_eq!(d[2], Defi::Chrono(u32::from(n.duree_cible_s)));
            let attendu = if n.sans_planer {
                Defi::SansPlaner
            } else {
                Defi::TroisGouttes
            };
            assert_eq!(d[1], attendu, "{}", n.nom);
        }
        // Niveau 2 (sans planer possible) : 55 s.
        let n = &NIVEAUX[1];
        let mut s = Sauvegarde::default();
        let lente = Tentative {
            morts: 1,
            a_plane: false,
            pas: 70 * HZ,
            gouttes: [true, false, false],
        };
        let r = noter_niveau(&mut s, n, &lente);
        assert_eq!((r.reussis, r.medaille), ([false, true, false], 1));
        assert_eq!((r.gain, r.deja_comptees), (1 + GAIN_DEFI, 0));
        // La même chose encore (carte 7.9) : rien, la goutte est déjà comptée.
        let r = noter_niveau(&mut s, n, &lente);
        assert_eq!((r.gain, r.deja_comptees), (0, 1));
        let parfaite = Tentative {
            morts: 0,
            a_plane: false,
            pas: 55 * HZ,
            gouttes: [false, false, true],
        };
        let r = noter_niveau(&mut s, n, &parfaite);
        assert_eq!((r.reussis, r.medaille), ([true; 3], 3));
        // Une goutte nouvelle, deux défis nouveaux, bronze → or : la différence.
        assert_eq!(r.gain, 1 + 2 * GAIN_DEFI + GAIN_OR - valeur_medaille(1));
        assert_eq!(noter_niveau(&mut s, n, &parfaite).gain, 0, "or déjà eu");
        let p = s.progres(n.cle);
        assert_eq!(p.gouttes, [true, false, true]);
        assert_eq!((p.defis, p.medaille, p.atteint), ([true; 3], 3, true));
        assert_eq!(p.meilleur_temps, Some(55 * HZ));
        assert_eq!(
            s.gouttes_en_poche,
            1 + GAIN_DEFI + 1 + 2 * GAIN_DEFI + GAIN_OR
        );
        // Une médaille moindre ensuite ne fait pas descendre la meilleure.
        noter_niveau(&mut s, n, &lente);
        assert_eq!(s.progres(n.cle).medaille, 3);
        // Bonus de fin : 5 + 5 + 10 au mieux, 5 au moins ; chacun une seule fois.
        let mut b = bonus_fin(false, false);
        assert_eq!(crediter_bonus_fin(&mut s, &mut b), GAIN_PARTIE_FINIE);
        let mut b = bonus_fin(true, true);
        assert_eq!(crediter_bonus_fin(&mut s, &mut b), 15, "FIN déjà crédité");
        assert!(b[0].deja && !b[1].deja && !b[2].deja);
        assert_eq!(crediter_bonus_fin(&mut s, &mut bonus_fin(true, true)), 0);
        assert_eq!(total_bonus(&bonus_fin(true, true)), 20);
        // Les bandes d'affichage tiennent dans l'écran, pour chaque niveau.
        for n in NIVEAUX {
            let r = noter_niveau(&mut Sauvegarde::default(), n, &lente);
            let (defis, bonus) =
                crate::ecrans::resultats::largeurs_pire_cas(&r, &bonus_fin(true, true));
            let max = crate::ecrans::resultats::LARGEUR_BANDE_FIN;
            assert!(defis <= max && bonus <= max, "{} : {defis} {bonus}", n.nom);
        }
    }
}
