//! Tests propres au niveau « essai_balancement » (carte 11.1, prototype du Monde 3 :
//! se balancer au parapluie). La grille et ses commentaires de conception sont dans
//! son fichier `.ron` (fait à l'éditeur).

use crate::commun::*;
use crate::faisabilite::verifier_infos;
use std::time::Instant;

/// Le niveau de ce module, lu dans son fichier `.ron`.
fn infos() -> &'static InfosNiveau {
    par_cle("essai_balancement")
}

/// Les huit crochets sont là où le commentaire le dit (le point d'accroche au
/// milieu de la case `y`).
#[test]
fn huit_crochets_a_leur_place() {
    let n = infos().charger().unwrap();
    let cases: Vec<(i32, i32)> = n
        .crochets
        .iter()
        .map(|c| ((c.x / 16.0) as i32, (c.y / 16.0) as i32))
        .collect();
    assert_eq!(
        cases,
        vec![
            (71, 3),
            (81, 3),
            (67, 5),
            (10, 7),
            (23, 7),
            (41, 7),
            (47, 7),
            (63, 7)
        ]
    );
}

/// Le robot de faisabilité finit le niveau (arrivée, 3 gouttes, lanternes), en se
/// balançant : son temps et son nombre d'états sont écrits (carte 11.1).
#[test]
#[ignore = "lot : recherche complète du robot de faisabilité"]
fn le_robot_de_faisabilite_finit_le_niveau() {
    let debut = Instant::now();
    let r = verifier_infos(infos());
    println!(
        "essai_balancement : robot de faisabilité en {:.1} s, {} états, course rapide {:.1} s",
        debut.elapsed().as_secs_f32(),
        r.etats,
        r.pas_arrivee.unwrap_or(0) as f32 / 60.0
    );
}

/// Sans crochet (les `y` remplacés par du vide), l'arrivée est hors d'atteinte :
/// le niveau exige le balancement.
#[test]
#[ignore = "lot : le niveau exige sa mécanique (recherche complète sans elle)"]
fn sans_crochet_l_arrivee_est_hors_d_atteinte() {
    let sans = Niveau::depuis_ascii(&infos().ascii.replace('y', ".")).unwrap();
    let r = analyser(&sans, sans.depart, Options::arrivee_seule());
    assert!(!r.arrivee, "arrivée atteinte sans crochet");
}
