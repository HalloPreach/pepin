//! Module `levels` : la table ORDONNÉE des niveaux et les règles qui les enchaînent.
//!
//! Un petit fichier par niveau (chacun exporte `INFOS`) ; la table `NIVEAUX` les range
//! dans l'ordre du parcours. Rien d'autre dans le jeu ne suppose un nombre de niveaux :
//! tout se déduit de cette table (`NB_NIVEAUX` est sa longueur).
//!
//! Chaque niveau a un GENRE (`GenreNiveau`) :
//! - `Normal` : le parcours principal, numéroté « NIVEAU 3/10 » ;
//! - `Bonus` : joué seulement si TOUTES les gouttes d'or des niveaux normaux ont été
//!   ramassées pendant la partie en cours ; sinon on passe directement au suivant ;
//! - `Epilogue` : le dernier niveau ; l'écran de fin vient juste après.
//!
//! Les règles (`apres`, `bonus_debloque`, `verifier_ordre`) sont des fonctions PURES
//! qui reçoivent la table en paramètre (`&[InfosNiveau]`) : les tests les essaient sur
//! de fausses listes (avec bonus, avec épilogue, avec les deux, sans aucun) sans
//! toucher au vrai jeu.
//!
//! Chaque niveau appartient à un MONDE (carte 9.1, `InfosNiveau::monde`) : les mondes
//! sont des blocs contigus de la table, dans l'ordre, et les règles valent PAR MONDE
//! (`niveaux_du_monde` donne la tranche d'un monde).
//!
//! Les dessins ASCII sont des `&str` embarqués dans le binaire : aucun fichier
//! à charger, donc aucune erreur d'entrée/sortie possible au lancement.
//!
//! Notions Rust illustrées ici : tranche constante `&[T]` (sa longueur `len()` est
//! connue à la compilation et sert de taille de tableau ailleurs), `enum` sans
//! données, `Option` / `Result` avec une erreur typée, itérateur `find` sur un
//! intervalle.

// Niveaux d'essai : HORS de la table `NIVEAUX` (tests et build de QA seulement).
pub mod arc;
pub mod clocher;
pub mod cyclone;
pub mod descente;
pub mod essai_auvent;
pub mod essai_chat;
pub mod essai_eau;
pub mod essai_ennemis_eau;
pub mod essai_escargot;
pub mod essai_grele;
pub mod essai_horloge;
pub mod essai_montgolfiere;
pub mod essai_nuage;
pub mod essai_vent;
pub mod essai_ventilateur;
pub mod essai_vertical;
pub mod gouttieres;
pub mod marche;
pub mod mer;
pub mod nuages;
pub mod orage;
pub mod poursuite;
pub mod toits;
pub mod variantes;
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
    marche::INFOS,
    orage::INFOS,
    clocher::INFOS,
    cyclone::INFOS,
    vents::INFOS,
    mer::INFOS,
    nuages::INFOS,
    poursuite::INFOS,
    arc::INFOS,
    descente::INFOS,
    // Build de QA seulement (carte 9.1) : le faux Monde 2, à la FIN de la table.
    #[cfg(feature = "monde2_qa")]
    TOITS_MONDE2,
];

/// Faux Monde 2 du build de QA (carte 9.1) : une COPIE du niveau des toits rangée en
/// Monde 2 (son « NIVEAU 1 »), sous sa propre clé et son propre nom. Elle n'entre dans
/// `NIVEAUX` qu'avec la fonctionnalité `monde2_qa` (activée par le `debug_niveaux` du
/// jeu) : la production n'en a aucune trace. Les vrais niveaux du Monde 2 (cartes 9.6
/// à 9.13) la remplaceront. La syntaxe `..toits::INFOS` copie tous les autres champs.
pub const TOITS_MONDE2: InfosNiveau = InfosNiveau {
    monde: 2,
    numero: 1,
    cle: "toits_monde2",
    nom: "Les toits du Monde 2",
    ..toits::INFOS
};

/// Nombre de niveaux du jeu (bonus et épilogue compris).
pub const NB_NIVEAUX: usize = NIVEAUX.len();

// Les CLÉS de la table (carte 7.9) sont vérifiées À LA COMPILATION : chacune est un
// identifiant (minuscules, chiffres, `_`) et aucune n'est en double. Une clé fautive
// ou répétée empêche le jeu de compiler, avant même les tests.
const _: () = assert!(cles_valides(NIVEAUX), "clé de niveau invalide ou en double");

/// Indice dans `niveaux` du niveau de clé `cle` (`None` : inconnue).
pub fn indice_de_cle(niveaux: &[InfosNiveau], cle: &str) -> Option<usize> {
    niveaux.iter().position(|n| n.cle == cle)
}

/// Les clés sont-elles des identifiants, toutes différentes ? Fonction `const`
/// (boucles `while` et octets, `==` sur `&str` n'étant pas permis en `const`).
const fn cles_valides(niveaux: &[InfosNiveau]) -> bool {
    let mut i = 0;
    while i < niveaux.len() {
        let a = niveaux[i].cle.as_bytes();
        if a.is_empty() || a.len() > 32 {
            return false;
        }
        let mut k = 0;
        while k < a.len() {
            if !(a[k].is_ascii_lowercase() || a[k].is_ascii_digit() || a[k] == b'_') {
                return false;
            }
            k += 1;
        }
        let mut j = i + 1;
        while j < niveaux.len() {
            let b = niveaux[j].cle.as_bytes();
            if a.len() == b.len() {
                let mut k = 0;
                while k < a.len() && a[k] == b[k] {
                    k += 1;
                }
                if k == a.len() {
                    return false;
                }
            }
            j += 1;
        }
        i += 1;
    }
    true
}

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

/// Gouttes d'or du PARCOURS : celles de tous les niveaux sauf le bonus (les niveaux
/// normaux et l'épilogue, joués dans toute partie). C'est le total du compteur du
/// jeu et de l'écran de fin ; le bonus, joué ou non, est compté à part
/// (`gouttes_du_bonus`).
pub fn gouttes_du_parcours(niveaux: &[InfosNiveau]) -> u32 {
    let parcours = niveaux
        .iter()
        .filter(|n| n.genre != GenreNiveau::Bonus)
        .count();
    (parcours * GOUTTES_PAR_NIVEAU) as u32
}

/// Gouttes d'or du bonus (0 si la table n'a pas de bonus).
pub fn gouttes_du_bonus(niveaux: &[InfosNiveau]) -> u32 {
    let bonus = niveaux
        .iter()
        .filter(|n| n.genre == GenreNiveau::Bonus)
        .count();
    (bonus * GOUTTES_PAR_NIVEAU) as u32
}

/// Le bonus est-il débloqué ? `gouttes_normales` : gouttes ramassées pendant la
/// partie dans les niveaux normaux.
pub fn bonus_debloque(niveaux: &[InfosNiveau], gouttes_normales: u32) -> bool {
    gouttes_normales >= gouttes_pour_le_bonus(niveaux)
}

/// Ce qui suit le niveau `indice` de `niveaux`, DANS SON MONDE (carte 9.1) :
/// - après un épilogue (ou un indice hors de la table) : l'écran de fin ;
/// - sinon le niveau suivant du monde, en sautant le bonus s'il n'est pas débloqué ;
/// - plus aucun niveau dans le monde : l'écran de fin (une partie ne passe jamais
///   d'un monde à l'autre).
///
/// `gouttes_normales` ne compte que si le niveau suivant est un bonus. Le jeu passe les
/// gouttes de toute la partie : grâce à l'ordre imposé par `verifier_ordre` (tous les
/// niveaux normaux du monde d'abord), au moment de choisir le bonus ce sont exactement
/// celles des niveaux normaux du monde. Avec un seul monde, la tranche du monde est la
/// table entière : rien ne change pour le Monde 1.
pub fn apres(niveaux: &[InfosNiveau], indice: usize, gouttes_normales: u32) -> Apres {
    let Some(n) = niveaux.get(indice) else {
        return Apres::Fin;
    };
    if n.genre == GenreNiveau::Epilogue {
        return Apres::Fin;
    }
    let debut = debut_monde(niveaux, n.monde).unwrap_or(indice);
    let monde = niveaux_du_monde(niveaux, n.monde);
    let bonus = bonus_debloque(monde, gouttes_normales);
    (indice + 1..debut + monde.len())
        .find(|&j| bonus || niveaux[j].genre != GenreNiveau::Bonus)
        .map_or(Apres::Fin, Apres::Niveau)
}

/// Pourquoi une table de niveaux est mal rangée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErreurOrdre {
    /// Aucun niveau normal : il n'y aurait rien à jouer avant le bonus.
    AucunNiveauNormal,
    /// Le numéro affiché n'est pas la place DANS SON MONDE (1 pour le premier).
    NumeroFaux { indice: usize, numero: u8 },
    /// Un monde qui n'apparaît pas dans l'ordre (les mondes sont des blocs
    /// contigus, numérotés 1, 2, ... sans saut, et le premier est le Monde 1).
    MondeDesordonne { indice: usize, monde: u8 },
    /// Un niveau normal après le bonus ou l'épilogue (de SON monde).
    NormalApresLaFin { indice: usize },
    /// Plus d'un bonus (dans le même monde).
    BonusMultiple,
    /// Un épilogue qui n'est pas le dernier niveau de la table (deux épilogues
    /// compris : chaque monde a le sien, le Monde 1 étant toujours le premier).
    EpilogueAvantLaFin { indice: usize },
}

/// Vérifie l'ordre d'une table : les mondes apparaissent dans l'ordre (1, 2, ...),
/// chacun en un seul bloc ; dans chaque monde, numéros 1, 2, 3... ; au moins un
/// niveau normal ; TOUS les niveaux normaux d'abord, puis au plus un bonus, puis
/// au plus un épilogue (le tout dernier de la table). Les règles s'appliquent PAR
/// MONDE (carte 9.1) : le bonus et l'épilogue du Monde 2 n'entrent pas dans le
/// Monde 1, et réciproquement.
pub fn verifier_ordre(niveaux: &[InfosNiveau]) -> Result<(), ErreurOrdre> {
    if nb_normaux(niveaux) == 0 {
        return Err(ErreurOrdre::AucunNiveauNormal);
    }
    let mut extras = 0;
    let mut bonus = 0;
    let mut monde: u8 = 0;
    let mut position_dans_le_monde = 0u32;
    for (indice, n) in niveaux.iter().enumerate() {
        // Les mondes : blocs contigus, dans l'ordre, à partir de 1.
        if n.monde < monde || n.monde > monde + 1 {
            return Err(ErreurOrdre::MondeDesordonne {
                indice,
                monde: n.monde,
            });
        }
        if n.monde != monde {
            monde = n.monde;
            position_dans_le_monde = 0;
            extras = 0;
            bonus = 0;
        }
        position_dans_le_monde += 1;
        if n.numero as u32 != position_dans_le_monde {
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
            // L'épilogue est le DERNIER niveau de son MONDE (carte 9.1) : le niveau
            // suivant, s'il existe, ouvre le monde suivant (un normal). L'épilogue
            // du dernier monde est donc le dernier niveau de la table.
            GenreNiveau::Epilogue => {
                if let Some(suivant) = niveaux.get(indice + 1) {
                    if suivant.monde == n.monde {
                        return Err(ErreurOrdre::EpilogueAvantLaFin { indice });
                    }
                }
            }
        }
        if n.genre != GenreNiveau::Normal {
            extras += 1;
        }
    }
    Ok(())
}

/// Premier indice de `niveaux` appartenant au monde `monde` (`None` : ce monde
/// n'existe pas). Les mondes sont des blocs contigus de la table, dans l'ordre
/// (règle de `verifier_ordre`), donc les indices du Monde 1 ne bougent jamais quand
/// on ajoute le Monde 2 à la fin (carte 9.1).
pub fn debut_monde(niveaux: &[InfosNiveau], monde: u8) -> Option<usize> {
    niveaux.iter().position(|n| n.monde == monde)
}

/// Les niveaux du monde `monde`, dans l'ordre de la table (une table vide si ce
/// monde n'y figure pas). C'est sur CETTE tranche que s'appliquent les règles
/// par monde : `apres`, `gouttes_pour_le_bonus`, `gouttes_du_parcours`...
pub fn niveaux_du_monde(niveaux: &[InfosNiveau], monde: u8) -> &[InfosNiveau] {
    let Some(debut) = debut_monde(niveaux, monde) else {
        return &[];
    };
    let fin = niveaux[debut..]
        .iter()
        .position(|n| n.monde != monde)
        .map_or(niveaux.len(), |p| debut + p);
    &niveaux[debut..fin]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zones::Theme;
    use GenreNiveau::{Bonus, Epilogue, Normal};

    /// Un faux niveau : seuls le numéro et le genre comptent pour les règles.
    const fn faux_monde(numero: u8, genre: GenreNiveau, monde: u8) -> InfosNiveau {
        InfosNiveau {
            monde,
            numero,
            cle: "faux",
            sans_planer: false,
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

    /// Un faux niveau du Monde 1.
    const fn faux(numero: u8, genre: GenreNiveau) -> InfosNiveau {
        faux_monde(numero, genre, 1)
    }

    /// Relance le robot de faisabilité, parapluie interdit, sur chaque niveau et
    /// compare au champ `InfosNiveau::sans_planer` (lent : ignoré, voir la doc du
    /// champ).
    #[test]
    #[ignore]
    fn sans_planer_possible_d_apres_le_robot() {
        use crate::faisabilite::{analyser, Options};
        let sans_planer = Options {
            planer: false,
            ..Options::default()
        };
        let mut ecarts = Vec::new();
        for n in NIVEAUX {
            let niveau = n.charger().expect("niveau valide");
            let r = analyser(&niveau, niveau.depart, sans_planer);
            println!("{} {} : sans planer {}", n.numero, n.cle, r.arrivee);
            if r.arrivee != n.sans_planer {
                ecarts.push(n.cle);
            }
        }
        assert!(
            ecarts.is_empty(),
            "à recopier dans `sans_planer` : {ecarts:?}"
        );
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

    /// Le faux Monde 2 du build de QA (carte 9.1) : la table du Monde 1 suivie de la
    /// copie des toits reste bien rangée, les indices du Monde 1 ne bougent pas, et
    /// aucune partie ne passe d'un monde à l'autre.
    #[test]
    fn le_faux_monde_2_de_qa_suit_le_monde_1() {
        let mut t: Vec<InfosNiveau> = NIVEAUX.iter().filter(|n| n.monde == 1).copied().collect();
        let fin_monde_1 = t.len();
        t.push(TOITS_MONDE2);
        assert_eq!(verifier_ordre(&t), Ok(()));
        assert_eq!(debut_monde(&t, 2), Some(fin_monde_1));
        let cles = |l: &[InfosNiveau]| l.iter().map(|n| n.cle).collect::<Vec<_>>();
        assert_eq!(cles(niveaux_du_monde(&t, 1)), cles(&NIVEAUX[..fin_monde_1]));
        assert_eq!(apres(&t, fin_monde_1 - 1, 36), Apres::Fin);
        assert_eq!(apres(&t, fin_monde_1, 3), Apres::Fin);
        assert_eq!(TOITS_MONDE2.ascii, toits::INFOS.ascii);
        assert!(cles_valides(&t));
    }

    /// La vraie table depuis la carte 8.2 : dix niveaux normaux (le 10e, « Le mur de
    /// pluie », est le sommet), le bonus (« L'arc-en-ciel », 11e) et l'épilogue
    /// (« La descente », 12e). Les DEUX chemins : avec les 30 gouttes des dix niveaux
    /// normaux, le bonus est joué après le 10e, puis l'épilogue ; avec une goutte de
    /// moins (ou aucune), on passe du 10e directement à l'épilogue. Après l'épilogue :
    /// la fin. Avant le 10e, les gouttes ne changent rien.
    #[test]
    fn la_vraie_table_joue_le_bonus_avec_toutes_les_gouttes_et_finit_par_l_epilogue() {
        let normaux = NB_NIVEAUX - 2;
        assert_eq!(normaux, 10);
        assert_eq!(NB_NIVEAUX, 12);
        assert!(NIVEAUX[..normaux].iter().all(|n| n.genre == Normal));
        assert_eq!(NIVEAUX[normaux].genre, Bonus);
        assert_eq!(NIVEAUX[normaux].nom, arc::INFOS.nom);
        assert_eq!(NIVEAUX[normaux + 1].genre, Epilogue);
        assert_eq!(NIVEAUX[normaux + 1].nom, descente::INFOS.nom);
        assert_eq!(gouttes_pour_le_bonus(NIVEAUX), 30);
        // Toutes les gouttes : 1 à 10, le bonus, l'épilogue, puis la fin.
        assert_eq!(parcours(NIVEAUX, 30), (0..12).collect::<Vec<_>>());
        // Une goutte manque : de 10 à l'épilogue, sans le bonus.
        let sans_bonus: Vec<usize> = (0..10).chain([11]).collect();
        assert_eq!(parcours(NIVEAUX, 29), sans_bonus);
        assert_eq!(parcours(NIVEAUX, 0), sans_bonus);
        assert_eq!(apres(NIVEAUX, 9, 30), Apres::Niveau(10));
        assert_eq!(apres(NIVEAUX, 9, 29), Apres::Niveau(11));
        // Après le bonus : l'épilogue, quelles que soient les gouttes (33 avec les
        // siennes) ; après l'épilogue : la fin.
        assert_eq!(apres(NIVEAUX, 10, 33), Apres::Niveau(11));
        assert_eq!(apres(NIVEAUX, 11, 36), Apres::Fin);
        assert_eq!(apres(NIVEAUX, 11, 0), Apres::Fin);
        // Avant le 10e : toujours le niveau suivant.
        for gouttes in [0, 29, 30] {
            for i in 0..normaux - 1 {
                assert_eq!(apres(NIVEAUX, i, gouttes), Apres::Niveau(i + 1));
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

    /// Le compteur du jeu : le parcours (normaux et épilogue), le bonus à part.
    #[test]
    fn les_gouttes_du_parcours_et_celles_du_bonus() {
        let t = table(&[Normal, Normal, Normal, Bonus, Epilogue]);
        assert_eq!((gouttes_du_parcours(&t), gouttes_du_bonus(&t)), (12, 3));
        let t = table(&[Normal, Normal]);
        assert_eq!((gouttes_du_parcours(&t), gouttes_du_bonus(&t)), (6, 0));
        // La vraie table : 10 normaux et l'épilogue (33), le bonus (3) ; en tout, les
        // 36 gouttes du jeu.
        assert_eq!(
            (gouttes_du_parcours(NIVEAUX), gouttes_du_bonus(NIVEAUX)),
            (33, 3)
        );
        assert_eq!(
            (gouttes_du_parcours(NIVEAUX) + gouttes_du_bonus(NIVEAUX)) as usize,
            crate::zones::GOUTTES_TOTAL
        );
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

    /// Deux mondes (carte 9.1) : numérotation PAR MONDE ; le bonus et l'épilogue
    /// d'un monde ne touchent pas l'autre (les règles tournent sur la tranche du
    /// monde) ; les mondes sont des blocs contigus dans l'ordre, le premier étant
    /// le Monde 1.
    #[test]
    fn deux_mondes_les_regles_s_appliquent_par_monde() {
        // Monde 1 : 2 normaux, bonus, épilogue. Monde 2 : pareil, à la fin.
        let t: Vec<InfosNiveau> = [
            faux_monde(1, Normal, 1),
            faux_monde(2, Normal, 1),
            faux_monde(3, Bonus, 1),
            faux_monde(4, Epilogue, 1),
            faux_monde(1, Normal, 2),
            faux_monde(2, Normal, 2),
            faux_monde(3, Bonus, 2),
            faux_monde(4, Epilogue, 2),
        ]
        .to_vec();
        assert_eq!(verifier_ordre(&t), Ok(()));
        // Les tranches par monde (les indices du Monde 1 ne bougent pas).
        assert_eq!(debut_monde(&t, 1), Some(0));
        assert_eq!(debut_monde(&t, 2), Some(4));
        assert_eq!(niveaux_du_monde(&t, 1).len(), 4);
        assert_eq!(niveaux_du_monde(&t, 2).len(), 4);
        assert!(niveaux_du_monde(&t, 3).is_empty());
        // L'épilogue du Monde 1 FINIT la partie : le Monde 2 ne s'enchaîne PAS
        // par `apres` (c'est la carte des niveaux et le déblocage qui y mènent).
        assert_eq!(apres(&t, 3, 12), Apres::Fin);
        assert_eq!(apres(&t, 7, 12), Apres::Fin);
        // `apres` sur la table ENTIÈRE reste dans le monde : avec les 6 gouttes du
        // Monde 2, son bonus est joué ; avec 5, on passe à son épilogue.
        assert_eq!(apres(&t, 4, 0), Apres::Niveau(5));
        assert_eq!(apres(&t, 5, 6), Apres::Niveau(6));
        assert_eq!(apres(&t, 5, 5), Apres::Niveau(7));
        assert_eq!(apres(&t, 1, 6), Apres::Niveau(2));
        // Un monde SANS épilogue : son dernier niveau finit la partie, le monde
        // suivant ne s'enchaîne pas.
        let sans_epilogue: Vec<InfosNiveau> = [
            faux_monde(1, Normal, 1),
            faux_monde(2, Normal, 1),
            faux_monde(1, Normal, 2),
        ]
        .to_vec();
        assert_eq!(verifier_ordre(&sans_epilogue), Ok(()));
        assert_eq!(apres(&sans_epilogue, 0, 0), Apres::Niveau(1));
        assert_eq!(apres(&sans_epilogue, 1, 6), Apres::Fin);
        assert_eq!(apres(&sans_epilogue, 2, 3), Apres::Fin);
        // Chaque monde compte SES gouttes : avec les 6 du monde, le bonus du
        // monde est joué ; une de moins, on passe direct à l'épilogue. Les 12 de
        // la partie entière ne valent pas les 6 du monde (règle par monde).
        let w1 = niveaux_du_monde(&t, 1);
        let w2 = niveaux_du_monde(&t, 2);
        assert_eq!(parcours(w1, 6), [0, 1, 2, 3]);
        assert_eq!(parcours(w2, 6), [0, 1, 2, 3]);
        assert_eq!(parcours(w1, 5), [0, 1, 3]);
        assert_eq!(parcours(w2, 5), [0, 1, 3]);
        // Un monde qui commence sans le Monde 1, ou qui s'interrompt : refusé.
        let mal: Vec<InfosNiveau> = [faux_monde(1, Normal, 2), faux_monde(2, Normal, 1)].to_vec();
        assert_eq!(
            verifier_ordre(&mal),
            Err(ErreurOrdre::MondeDesordonne {
                indice: 0,
                monde: 2
            })
        );
        let mal: Vec<InfosNiveau> = [
            faux_monde(1, Normal, 1),
            faux_monde(1, Normal, 2),
            faux_monde(2, Normal, 1),
        ]
        .to_vec();
        assert_eq!(
            verifier_ordre(&mal),
            Err(ErreurOrdre::MondeDesordonne {
                indice: 2,
                monde: 1
            })
        );
        // La numérotation est par monde : le 1er du Monde 2 est le « 1 », pas le 5.
        let mal: Vec<InfosNiveau> = [faux_monde(1, Normal, 1), faux_monde(5, Normal, 2)].to_vec();
        assert_eq!(
            verifier_ordre(&mal),
            Err(ErreurOrdre::NumeroFaux {
                indice: 1,
                numero: 5
            })
        );
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
