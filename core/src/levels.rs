//! Module `levels` : la table ORDONNÉE des niveaux et les règles qui les enchaînent.
//!
//! Un petit fichier par niveau (chacun exporte `INFOS`) ; la table `NIVEAUX` les range
//! dans l'ordre du parcours. Rien d'autre dans le jeu ne suppose un nombre de niveaux :
//! tout se déduit de cette table (`NB_NIVEAUX` est sa longueur).
//!
//! Chaque niveau a un GENRE (`GenreNiveau`) :
//! - `Normal` : le parcours principal, numéroté « NIVEAU 3/9 » ;
//! - `Bonus` : joué seulement si TOUTES les gouttes d'or des niveaux normaux ont été
//!   ramassées pendant la partie en cours ; sinon on passe directement au suivant ;
//! - `Epilogue` : le dernier niveau ; l'écran de fin vient juste après.
//!
//! Les règles (`apres`, `bonus_debloque`, `verifier_ordre`) sont des fonctions PURES
//! qui reçoivent la table en paramètre (`&[InfosNiveau]`) : les tests les essaient sur
//! de fausses listes (avec bonus, avec épilogue, avec les deux, sans aucun) sans
//! toucher au vrai jeu.
//!
//! Les dessins ASCII sont des `&str` embarqués dans le binaire : aucun fichier
//! à charger, donc aucune erreur d'entrée/sortie possible au lancement.
//!
//! Notions Rust illustrées ici : tranche constante `&[T]` (sa longueur `len()` est
//! connue à la compilation et sert de taille de tableau ailleurs), `enum` sans
//! données, `Option` / `Result` avec une erreur typée, itérateur `find` sur un
//! intervalle.

// Niveaux d'essai : HORS de la table `NIVEAUX` (tests et build de QA seulement).
pub mod essai_auvent;
pub mod essai_chat;
pub mod essai_escargot;
pub mod essai_grele;
pub mod essai_horloge;
pub mod essai_montgolfiere;
pub mod essai_nuage;
pub mod essai_vent;
pub mod essai_ventilateur;
pub mod essai_vertical;
pub mod gouttieres;
pub mod nuages;
pub mod orage;
pub mod toits;
pub mod vents;

use crate::level::GOUTTES_PAR_NIVEAU;
use crate::zones::InfosNiveau;

/// Rôle d'un niveau dans le parcours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenreNiveau {
    /// Parcours principal.
    Normal,
    /// Récompense : seulement avec toutes les gouttes des niveaux normaux.
    Bonus,
    /// Dernier niveau, juste avant l'écran de fin.
    Epilogue,
}

/// Tous les niveaux, dans l'ordre du parcours. Règles d'ordre : `verifier_ordre`.
pub const NIVEAUX: &[InfosNiveau] = &[
    toits::INFOS,
    gouttieres::INFOS,
    orage::INFOS,
    vents::INFOS,
    nuages::INFOS,
];

/// Nombre de niveaux du jeu (bonus et épilogue compris).
pub const NB_NIVEAUX: usize = NIVEAUX.len();

/// Ce qui vient après un niveau terminé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Apres {
    /// Le niveau d'indice donné (dans la table).
    Niveau(usize),
    /// L'écran de fin.
    Fin,
}

/// Nombre de niveaux normaux de `niveaux`.
pub fn nb_normaux(niveaux: &[InfosNiveau]) -> usize {
    niveaux
        .iter()
        .filter(|n| n.genre == GenreNiveau::Normal)
        .count()
}

/// Gouttes d'or qu'il faut ramasser dans les niveaux normaux pour débloquer le bonus.
pub fn gouttes_pour_le_bonus(niveaux: &[InfosNiveau]) -> u32 {
    (nb_normaux(niveaux) * GOUTTES_PAR_NIVEAU) as u32
}

/// Le bonus est-il débloqué ? `gouttes_normales` : gouttes ramassées pendant la
/// partie dans les niveaux normaux.
pub fn bonus_debloque(niveaux: &[InfosNiveau], gouttes_normales: u32) -> bool {
    gouttes_normales >= gouttes_pour_le_bonus(niveaux)
}

/// Ce qui suit le niveau `indice` de `niveaux` :
/// - après un épilogue (ou un indice hors de la table) : l'écran de fin ;
/// - sinon le niveau suivant de la table, en sautant le bonus s'il n'est pas débloqué ;
/// - plus aucun niveau : l'écran de fin.
///
/// `gouttes_normales` ne compte que si le niveau suivant est un bonus. Le jeu passe les
/// gouttes de toute la partie : grâce à l'ordre imposé par `verifier_ordre` (tous les
/// niveaux normaux d'abord), au moment de choisir le bonus ce sont exactement celles des
/// niveaux normaux.
pub fn apres(niveaux: &[InfosNiveau], indice: usize, gouttes_normales: u32) -> Apres {
    if niveaux
        .get(indice)
        .is_none_or(|n| n.genre == GenreNiveau::Epilogue)
    {
        return Apres::Fin;
    }
    let bonus = bonus_debloque(niveaux, gouttes_normales);
    (indice + 1..niveaux.len())
        .find(|&j| bonus || niveaux[j].genre != GenreNiveau::Bonus)
        .map_or(Apres::Fin, Apres::Niveau)
}

/// Pourquoi une table de niveaux est mal rangée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErreurOrdre {
    /// Aucun niveau normal : il n'y aurait rien à jouer avant le bonus.
    AucunNiveauNormal,
    /// Le numéro affiché n'est pas la place dans la table (1 pour le premier).
    NumeroFaux { indice: usize, numero: u8 },
    /// Un niveau normal après le bonus ou l'épilogue.
    NormalApresLaFin { indice: usize },
    /// Plus d'un bonus.
    BonusMultiple,
    /// Un épilogue qui n'est pas le dernier niveau (deux épilogues compris).
    EpilogueAvantLaFin { indice: usize },
}

/// Vérifie l'ordre d'une table : numéros 1, 2, 3... ; au moins un niveau normal ;
/// TOUS les niveaux normaux d'abord, puis au plus un bonus, puis au plus un épilogue
/// (le tout dernier).
pub fn verifier_ordre(niveaux: &[InfosNiveau]) -> Result<(), ErreurOrdre> {
    if nb_normaux(niveaux) == 0 {
        return Err(ErreurOrdre::AucunNiveauNormal);
    }
    let mut extras = 0;
    let mut bonus = 0;
    for (indice, n) in niveaux.iter().enumerate() {
        if n.numero as usize != indice + 1 {
            return Err(ErreurOrdre::NumeroFaux {
                indice,
                numero: n.numero,
            });
        }
        match n.genre {
            GenreNiveau::Normal if extras > 0 => {
                return Err(ErreurOrdre::NormalApresLaFin { indice })
            }
            GenreNiveau::Normal => {}
            GenreNiveau::Bonus if bonus > 0 => return Err(ErreurOrdre::BonusMultiple),
            GenreNiveau::Bonus => bonus += 1,
            GenreNiveau::Epilogue if indice + 1 != niveaux.len() => {
                return Err(ErreurOrdre::EpilogueAvantLaFin { indice })
            }
            GenreNiveau::Epilogue => {}
        }
        if n.genre != GenreNiveau::Normal {
            extras += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zones::Theme;
    use GenreNiveau::{Bonus, Epilogue, Normal};

    /// Un faux niveau : seuls le numéro et le genre comptent pour les règles.
    const fn faux(numero: u8, genre: GenreNiveau) -> InfosNiveau {
        InfosNiveau {
            numero,
            nom: "faux",
            theme: Theme::Toits,
            genre,
            intro: "",
            mecanique: "",
            duree_cible_s: 0,
            ascii: "",
            aiguilles: &[],
        }
    }

    /// Une fausse table rangée dans l'ordre donné.
    fn table(genres: &[GenreNiveau]) -> Vec<InfosNiveau> {
        genres
            .iter()
            .enumerate()
            .map(|(i, &g)| faux(i as u8 + 1, g))
            .collect()
    }

    /// Le parcours complet d'une partie : indices joués, du premier à la fin.
    fn parcours(niveaux: &[InfosNiveau], gouttes_normales: u32) -> Vec<usize> {
        let mut joues = vec![0];
        while let Apres::Niveau(j) = apres(niveaux, *joues.last().unwrap(), gouttes_normales) {
            joues.push(j);
        }
        joues
    }

    #[test]
    fn la_vraie_table_est_bien_rangee() {
        assert_eq!(verifier_ordre(NIVEAUX), Ok(()));
        assert_eq!(NB_NIVEAUX, NIVEAUX.len());
    }

    /// Avec les cinq niveaux actuels (tous normaux), l'enchaînement est exactement
    /// celui d'avant la carte 5.1 : 1, 2, 3, 4, 5 puis la fin, quelles que soient
    /// les gouttes ramassées.
    #[test]
    fn les_cinq_niveaux_actuels_s_enchainent_comme_avant() {
        assert!(NIVEAUX.iter().all(|n| n.genre == Normal));
        for gouttes in 0..=gouttes_pour_le_bonus(NIVEAUX) {
            for i in 0..NB_NIVEAUX {
                let avant = if i + 1 >= NB_NIVEAUX {
                    Apres::Fin
                } else {
                    Apres::Niveau(i + 1)
                };
                assert_eq!(apres(NIVEAUX, i, gouttes), avant, "niveau {i}, {gouttes}");
            }
        }
    }

    /// L'ordre décidé pour l'étape 5 : 9 niveaux normaux, le bonus (10), l'épilogue (11).
    #[test]
    fn onze_niveaux_avec_bonus_et_epilogue() {
        let mut genres = vec![Normal; 9];
        genres.extend([Bonus, Epilogue]);
        let t = table(&genres);
        assert_eq!(verifier_ordre(&t), Ok(()));
        assert_eq!(gouttes_pour_le_bonus(&t), 27);
        // Toutes les gouttes : 1 à 9, le bonus, l'épilogue, puis la fin.
        assert_eq!(parcours(&t, 27), (0..11).collect::<Vec<_>>());
        // Une goutte manque : de 9 directement à l'épilogue.
        let sans_bonus: Vec<usize> = (0..9).chain([10]).collect();
        assert_eq!(parcours(&t, 26), sans_bonus);
        assert_eq!(parcours(&t, 0), sans_bonus);
        // Après le bonus vient toujours l'épilogue ; après l'épilogue, la fin.
        assert_eq!(apres(&t, 9, 0), Apres::Niveau(10));
        assert_eq!(apres(&t, 10, 27), Apres::Fin);
    }

    #[test]
    fn sans_bonus_l_epilogue_suit_le_dernier_niveau_normal() {
        let t = table(&[Normal, Normal, Epilogue]);
        assert_eq!(verifier_ordre(&t), Ok(()));
        assert_eq!(parcours(&t, 0), [0, 1, 2]);
        assert_eq!(parcours(&t, 6), [0, 1, 2]);
    }

    #[test]
    fn sans_epilogue_la_fin_suit_le_bonus_ou_le_dernier_niveau() {
        let t = table(&[Normal, Normal, Bonus]);
        assert_eq!(verifier_ordre(&t), Ok(()));
        assert_eq!(parcours(&t, 6), [0, 1, 2], "bonus débloqué");
        assert_eq!(parcours(&t, 5), [0, 1], "bonus manqué : la fin");
    }

    #[test]
    fn le_bonus_demande_toutes_les_gouttes_des_niveaux_normaux() {
        let t = table(&[Normal, Normal, Normal, Bonus, Epilogue]);
        assert!(!bonus_debloque(&t, 8));
        assert!(bonus_debloque(&t, 9));
        // Seuls les 3 niveaux normaux comptent (ni le bonus ni l'épilogue).
        assert_eq!(gouttes_pour_le_bonus(&t), 9);
    }

    #[test]
    fn hors_de_la_table_c_est_la_fin() {
        let t = table(&[Normal, Normal]);
        assert_eq!(apres(&t, 2, 0), Apres::Fin);
        assert_eq!(apres(&t, 99, 0), Apres::Fin);
        assert_eq!(apres(&[], 0, 0), Apres::Fin);
    }

    #[test]
    fn les_tables_mal_rangees_sont_refusees() {
        assert_eq!(verifier_ordre(&[]), Err(ErreurOrdre::AucunNiveauNormal));
        assert_eq!(
            verifier_ordre(&table(&[Bonus, Epilogue])),
            Err(ErreurOrdre::AucunNiveauNormal)
        );
        assert_eq!(
            verifier_ordre(&table(&[Normal, Bonus, Normal])),
            Err(ErreurOrdre::NormalApresLaFin { indice: 2 })
        );
        assert_eq!(
            verifier_ordre(&table(&[Normal, Bonus, Bonus])),
            Err(ErreurOrdre::BonusMultiple)
        );
        assert_eq!(
            verifier_ordre(&table(&[Normal, Epilogue, Bonus])),
            Err(ErreurOrdre::EpilogueAvantLaFin { indice: 1 })
        );
        assert_eq!(
            verifier_ordre(&table(&[Normal, Epilogue, Epilogue])),
            Err(ErreurOrdre::EpilogueAvantLaFin { indice: 1 })
        );
        let mut t = table(&[Normal, Normal]);
        t[1].numero = 5;
        assert_eq!(
            verifier_ordre(&t),
            Err(ErreurOrdre::NumeroFaux {
                indice: 1,
                numero: 5
            })
        );
    }
}
