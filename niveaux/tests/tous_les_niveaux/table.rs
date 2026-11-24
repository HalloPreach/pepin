//! La table des niveaux (autrefois dans `core/src/zones.rs` et `levels.rs`) :
//! numérotation, thèmes, chargement, ordre du parcours et des mondes.

use crate::commun::*;
use pepin_core::level::GOUTTES_PAR_NIVEAU;
use pepin_core::levels::{
    apres, compter_monde, debut_monde, gouttes_du_bonus, gouttes_du_parcours,
    gouttes_pour_le_bonus, niveaux_du_monde, verifier_ordre, Apres,
};
use GenreNiveau::{Bonus, Epilogue, Normal};

#[test]
fn les_niveaux_sont_numerotes_dans_l_ordre() {
    // La numérotation est PAR MONDE (carte 9.1) : le premier niveau de chaque
    // monde est le « 1 ». Les mondes sont des blocs contigus, dans l'ordre.
    let mut par_monde: Vec<(u8, u32)> = Vec::new();
    for n in niveaux() {
        if !par_monde.iter().any(|(m, _)| *m == n.monde) {
            par_monde.push((n.monde, 0));
        }
        let p = par_monde.iter_mut().find(|(m, _)| *m == n.monde).unwrap();
        p.1 += 1;
        assert_eq!(n.numero, p.1 as u8, "{}", n.nom);
        assert!(
            !n.nom.is_empty()
                && !n.intro_texte.is_empty()
                && !n.intro_astuce.is_empty()
                && !n.mecanique.is_empty()
        );
    }
    let mut dernier = 0u8;
    for n in niveaux() {
        assert!(n.monde == dernier || n.monde == dernier + 1, "{}", n.nom);
        dernier = n.monde;
    }
    assert_eq!(niveaux().first().unwrap().nom, "Les toits");
    assert!(niveaux().get(NB_NIVEAUX).is_none());
}

#[test]
fn tous_les_themes_dans_l_ordre() {
    for (i, t) in Theme::TOUS.into_iter().enumerate() {
        assert_eq!(t.indice(), i, "{t:?}");
    }
    // Chaque thème utilisé par un niveau est dans la liste (le `match` de
    // `luminosite` oblige déjà à traiter chaque variante).
    for n in niveaux() {
        assert!(Theme::TOUS.contains(&n.theme), "{}", n.nom);
    }
}

#[test]
fn tous_les_niveaux_se_chargent() {
    for n in niveaux().iter().chain([
        par_cle("essai_vertical"),
        par_cle("essai_vent"),
        par_cle("essai_grele"),
        par_cle("essai_auvent"),
        par_cle("essai_ventilateur"),
        par_cle("essai_montgolfiere"),
        par_cle("essai_escargot"),
        par_cle("essai_chat"),
        par_cle("essai_horloge"),
        par_cle("essai_nuage"),
        par_cle("essai_eau"),
        par_cle("essai_eau2"),
        par_cle("essai_ennemis_eau"),
        par_cle("essai_pieuvre"),
        par_cle("essai_balancement"),
    ]) {
        let niv = n
            .charger()
            .unwrap_or_else(|e| panic!("niveau {} : {e}", n.numero));
        // Au moins un écran de haut (11 rangées, 176 px) : un niveau plus haut que
        // l'écran (carte 5.2) fait défiler la caméra en hauteur.
        assert!(
            niv.hauteur() >= 11,
            "niveau {} : 11 lignes au moins",
            n.numero
        );
    }
    // Le niveau d'essai vertical fait quatre écrans de haut et n'est pas dans la table.
    let essai = par_cle("essai_vertical").charger().unwrap();
    assert_eq!(essai.hauteur(), 45);
    assert!(niveaux()
        .iter()
        .all(|n| n.nom != par_cle("essai_vertical").nom));
}

/// Relance le robot de faisabilité, parapluie interdit, sur chaque niveau et
/// compare au champ `InfosNiveau::sans_planer` (lent : ignoré, voir la doc du
/// champ).
#[test]
#[ignore]
fn sans_planer_possible_d_apres_le_robot() {
    let sans_planer = Options {
        planer: false,
        ..Options::default()
    };
    let mut ecarts = Vec::new();
    for n in niveaux() {
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
    assert_eq!(verifier_ordre(niveaux()), Ok(()));
    assert_eq!(NB_NIVEAUX, niveaux().len());
}

/// Le Monde 2 (carte 9.6, qui remplace le faux Monde 2 de la carte 9.1) : à la
/// FIN de la table, après tout le Monde 1, dont les indices ne bougent pas ; son
/// premier niveau est le caniveau ; aucune partie ne passe d'un monde à l'autre.
#[test]
fn le_monde_2_suit_le_monde_1() {
    let cles = |l: &[InfosNiveau]| l.iter().map(|n| n.cle).collect::<Vec<_>>();
    assert_eq!(
        cles(niveaux_du_monde(niveaux(), 1)),
        cles(niveaux_monde_1())
    );
    assert_eq!(NB_NIVEAUX_MONDE_1, 12);
    assert_eq!(niveaux_monde_1().last().unwrap().cle, "descente");
    assert_eq!(debut_monde(niveaux(), 2), Some(NB_NIVEAUX_MONDE_1));
    assert_eq!(niveaux()[NB_NIVEAUX_MONDE_1].cle, "caniveau");
    assert_eq!(
        NB_NIVEAUX_MONDE_1 + compter_monde(niveaux(), 2),
        NB_NIVEAUX,
        "deux mondes seulement"
    );
    // L'épilogue du Monde 1 finit la partie ; le dernier niveau du Monde 2 aussi.
    assert_eq!(apres(niveaux(), NB_NIVEAUX_MONDE_1 - 1, 36), Apres::Fin);
    assert_eq!(apres(niveaux(), NB_NIVEAUX - 1, 99), Apres::Fin);
}

/// La vraie table depuis la carte 8.2 : dix niveaux normaux (le 10e, « Le mur de
/// pluie », est le sommet), le bonus (« L'arc-en-ciel », 11e) et l'épilogue
/// (« La descente », 12e). Les DEUX chemins : avec les 30 gouttes des dix niveaux
/// normaux, le bonus est joué après le 10e, puis l'épilogue ; avec une goutte de
/// moins (ou aucune), on passe du 10e directement à l'épilogue. Après l'épilogue :
/// la fin. Avant le 10e, les gouttes ne changent rien.
#[test]
fn la_vraie_table_joue_le_bonus_avec_toutes_les_gouttes_et_finit_par_l_epilogue() {
    // Le Monde 1 : le début de la table (carte 9.6 : le Monde 2 vient après).
    let monde_1 = niveaux_monde_1();
    let normaux = NB_NIVEAUX_MONDE_1 - 2;
    assert_eq!(normaux, 10);
    assert_eq!(NB_NIVEAUX_MONDE_1, 12);
    assert!(monde_1[..normaux].iter().all(|n| n.genre == Normal));
    assert_eq!(monde_1[normaux].genre, Bonus);
    assert_eq!(monde_1[normaux].nom, par_cle("arc").nom);
    assert_eq!(monde_1[normaux + 1].genre, Epilogue);
    assert_eq!(monde_1[normaux + 1].nom, par_cle("descente").nom);
    assert_eq!(gouttes_pour_le_bonus(monde_1), 30);
    // Toutes les gouttes : 1 à 10, le bonus, l'épilogue, puis la fin.
    assert_eq!(parcours(monde_1, 30), (0..12).collect::<Vec<_>>());
    // Une goutte manque : de 10 à l'épilogue, sans le bonus.
    let sans_bonus: Vec<usize> = (0..10).chain([11]).collect();
    assert_eq!(parcours(monde_1, 29), sans_bonus);
    assert_eq!(parcours(monde_1, 0), sans_bonus);
    assert_eq!(apres(monde_1, 9, 30), Apres::Niveau(10));
    assert_eq!(apres(monde_1, 9, 29), Apres::Niveau(11));
    // Après le bonus : l'épilogue, quelles que soient les gouttes (33 avec les
    // siennes) ; après l'épilogue : la fin.
    assert_eq!(apres(monde_1, 10, 33), Apres::Niveau(11));
    assert_eq!(apres(monde_1, 11, 36), Apres::Fin);
    assert_eq!(apres(monde_1, 11, 0), Apres::Fin);
    // Avant le 10e : toujours le niveau suivant.
    for gouttes in [0, 29, 30] {
        for i in 0..normaux - 1 {
            assert_eq!(apres(monde_1, i, gouttes), Apres::Niveau(i + 1));
        }
    }
}

/// Le compteur du jeu sur la vraie table : le Monde 1 compte 10 normaux et
/// l'épilogue (33), le bonus (3) ; en tout, ses 36 gouttes. Le jeu entier : la
/// somme de chaque monde, soit 3 gouttes par niveau (autrefois la constante
/// `GOUTTES_TOTAL` de `core`, retirée avec la table).
#[test]
fn les_gouttes_du_parcours_et_celles_du_bonus_de_la_vraie_table() {
    assert_eq!(
        (
            gouttes_du_parcours(niveaux_monde_1()),
            gouttes_du_bonus(niveaux_monde_1())
        ),
        (33, 3)
    );
    let total: u32 = [1, 2]
        .map(|m| niveaux_du_monde(niveaux(), m))
        .iter()
        .map(|t| gouttes_du_parcours(t) + gouttes_du_bonus(t))
        .sum();
    assert_eq!(total as usize, NB_NIVEAUX * GOUTTES_PAR_NIVEAU);
}
