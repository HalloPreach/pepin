//! Tests des NIVEAUX D'ESSAI : le robot de QA (`essais.rs`) finit chaque
//! niveau d'essai SANS MOURIR, et les parcours des mécaniques à horloge
//! (aiguille d'horloge, Grand Nuage) TIENT à une image près (rejeu avec une
//! ou deux images d'avance ou de retard : même fin, mêmes gouttes).
//!
//! Chaque test rejoue le parcours de référence (`parcours_essai_*` dans
//! `essais.rs`) dans la VRAIE simulation, image par image ; les vérifications
//! portent sur l'arrivée (drapeau), les morts (0) et les gouttes (3, sauf
//! les démonstrations qui ne les comptent pas).
//!
//! ## Ajouter un niveau d'essai
//!
//! Un test « le robot finit le niveau d'essai X sans mourir » (à la suite
//! des autres, même forme : jouer, compter morts/gouttes, vérifier l'état
//! final de la mécanique) + un test « le parcours tient à une image près »
//! (via `rejouer_decale`) si la mécanique a une horloge. Le test ignore
//! qui écrit les scripts/touches/trace du niveau va dans `scripts.rs`.
//!
//! Notions Rust illustrées ici : des asserts avec des messages qui disent
//! QUOI vérifier, des `matches!` sur les phases des mécaniques, et un helper
//! privé (`rejouer_decale`) partagé par les tests de tenue.

use pepin_core::entities::ennemi::{pas_du_cycle, PhaseChat, DORT_PAS, REVEIL_PAS};
use pepin_core::entities::nuage::PhaseNuage;
use pepin_core::entities::ventilateur::PhaseVentilateur;
use pepin_core::player::Entrees;

use crate::bot_qa::essais::*;
use crate::bot_qa::parcours::MARGE_CHAT_PAS;
use crate::monde::Monde;

/// Le robot finit aussi le niveau d'essai « vent » (un écran, carte 5.3) sans
/// mourir, à travers l'`Application` (touche F1 du build de QA). Le plané qui
/// traverse le vide est porté par le vent latéral du niveau ; la caméra reste
/// au sol (niveau d'un écran) et les trois gouttes sont ramassées.
#[test]
fn le_robot_finit_le_niveau_d_essai_vent() {
    let parcours = parcours_essai_vent();
    let partie = jouer_essai_vent(&parcours);
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.gouttes, 3,
        "les trois gouttes du niveau d'essai vent"
    );
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F1, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    assert!(essai.len() > 100, "{} images de jeu", essai.len());
    // Niveau d'un écran : la caméra reste au bas du niveau (y = -4 : le niveau de
    // 176 px est aligné au bas de l'écran de 180 px, le ciel remplit le reste).
    assert!(
        essai.iter().all(|i| i.camera.1 == -4),
        "la caméra d'un niveau plat reste au sol"
    );
    // Le robot a bel et bien traversé le vide : il arrive à droite de la zone de
    // vent (x > 400), où se trouve le drapeau.
    let dernier = essai.last().unwrap();
    assert!(
        dernier.trame.x > 400.0,
        "arrivée à x = {} (le drapeau est à droite du vide)",
        dernier.trame.x
    );
    println!(
        "essai vent : {} images de jeu, gouttes {}, fin à x = {:.1}",
        essai.len(),
        partie.gouttes,
        dernier.trame.x
    );
}

/// Le robot finit le niveau d'essai « grêle » (carte 5.4) sans mourir, à travers
/// l'`Application` (touche F2 du build de QA) : trois gouttes (dont celle de la
/// corniche, SOUS la grêle arrêtée), et il a bel et bien plané DANS la grêle.
#[test]
fn le_robot_finit_le_niveau_d_essai_grele() {
    let parcours = parcours_essai_grele();
    let partie = jouer_essai_grele(&parcours);
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.gouttes, 3,
        "les trois gouttes du niveau d'essai grêle"
    );
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F2, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    assert!(essai.len() > 100, "{} images de jeu", essai.len());
    assert!(
        essai.iter().all(|i| i.camera.1 == -4),
        "la caméra d'un niveau plat reste au sol"
    );
    let sous_la_grele = essai.iter().filter(|i| i.sous_la_grele).count();
    assert!(
        sous_la_grele > 20,
        "{sous_la_grele} images à planer dans la grêle"
    );
    let dernier = essai.last().unwrap();
    assert!(dernier.trame.x > 500.0, "arrivée à x = {}", dernier.trame.x);
    println!(
            "essai grêle : {} images de jeu, {sous_la_grele} sous la grêle, gouttes {}, fin à x = {:.1}",
            essai.len(),
            partie.gouttes,
            dernier.trame.x
        );
}

/// Rebonds d'auvent d'une partie d'essai : indices (dans les images de jeu) des
/// images où un auvent relance Pépin.
fn rebonds(essai: &[&ImageEssai]) -> Vec<usize> {
    (0..essai.len())
        .filter(|&k| essai[k].rebond_auvent)
        .collect()
}

/// Le robot (le plus rapide) finit le niveau d'essai « auvent » (carte 5.5) sans
/// mourir, à travers l'`Application` (touche F3 du build de QA) : il traverse le
/// vide par les auvents sans toucher le sol entre son premier et son dernier
/// rebond (en gardant la droite, il en saute certains : c'est permis).
#[test]
fn le_robot_finit_le_niveau_d_essai_auvent() {
    let partie = jouer_essai_auvent(&parcours_essai_auvent());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F3, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let r = rebonds(&essai);
    assert!(r.len() >= 3, "{} rebonds d'auvent", r.len());
    let (premier, dernier) = (r[0], *r.last().unwrap());
    assert!(
        essai[premier..=dernier].iter().all(|i| !i.au_sol),
        "Pépin a touché le sol entre deux auvents"
    );
    println!(
        "essai auvent (le plus rapide) : {} images de jeu, {} rebonds, gouttes {}",
        essai.len(),
        r.len(),
        partie.gouttes
    );
}

/// Rebond par rebond (le parcours des captures) : les SIX auvents, chacun à son
/// tour, sans toucher le sol du premier au dernier ; trois gouttes (dont celles
/// en l'air, entre deux rebonds), aucune mort.
#[test]
fn le_robot_enchaine_les_six_auvents_de_l_essai() {
    let partie = jouer_essai_auvent(&parcours_essai_auvent_rebond_par_rebond());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.gouttes, 3,
        "les trois gouttes du niveau d'essai auvent"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let r = rebonds(&essai);
    // Les six auvents (colonnes 9, 13, 17, 22, 25, 29), un rebond chacun, dans
    // l'ordre : l'auvent relanceur est celui que la boîte de Pépin (10 px) touche.
    let colonnes: Vec<i32> = r
        .iter()
        .filter_map(|&k| {
            let x = essai[k].trame.x;
            [9, 13, 17, 22, 25, 29]
                .into_iter()
                .find(|&c| x < (c * 16 + 16) as f32 && ((c * 16) as f32) < x + 10.0)
        })
        .collect();
    assert_eq!(
        colonnes,
        vec![9, 13, 17, 22, 25, 29],
        "rebonds aux images {r:?}"
    );
    assert!(
        essai[r[0]..=r[5]].iter().all(|i| !i.au_sol),
        "Pépin a touché le sol entre deux auvents"
    );
    println!(
        "essai auvent (rebond par rebond) : {} images de jeu, rebonds aux images {r:?}",
        essai.len()
    );
}

/// Le robot finit le niveau d'essai « ventilateur » (carte 5.6) sans mourir, à
/// travers l'`Application` (touche F4 du build de QA) : trois gouttes, et il
/// monte réellement au balcon du drapeau, le seul chemin passe par le souffle
/// du ventilateur, car le balcon est trop haut pour un saut direct.
#[test]
fn le_robot_finit_le_niveau_d_essai_ventilateur() {
    let partie = jouer_essai_ventilateur(&parcours_essai_ventilateur());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.gouttes, 3,
        "les trois gouttes du niveau d'essai ventilateur"
    );
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F4, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    assert!(essai.len() > 100, "{} images de jeu", essai.len());
    // Le balcon du drapeau est en HAUT du niveau (y petit). Le robot y est bien
    // arrivé : sa trame finale est nettement plus haute que le sol.
    let dernier = essai.last().unwrap();
    assert!(
        dernier.trame.y < 100.0,
        "arrivée en haut du niveau (y = {:.1}) : le balcon a été atteint",
        dernier.trame.y
    );
    println!(
        "essai ventilateur : {} images de jeu, gouttes {}, fin à x = {:.1}, y = {:.1}",
        essai.len(),
        partie.gouttes,
        dernier.trame.x,
        dernier.trame.y
    );
}

/// Le robot finit le niveau d'essai « montgolfière » (carte 5.7) sans mourir, à
/// travers l'`Application` (touche F5 du build de QA), par le parcours le plus
/// RAPIDE : trois gouttes, l'arrivée tout en haut (étage du drapeau). Il saute de
/// la nacelle dès que l'étage suivant est à portée de saut, mais il a dû y monter :
/// la première l'a porté de plus d'un saut (56 px). Pendant toute montée où il est
/// posé, ses pieds sont PILE sur le dessus d'une nacelle (porté, sans glisser).
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_finit_le_niveau_d_essai_montgolfiere() {
    let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.gouttes, 3,
        "les trois gouttes de l'essai montgolfière"
    );
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F5, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let dernier = essai.last().unwrap();
    assert!(
        dernier.trame.y < 80.0,
        "arrivée à l'étage du drapeau (y = {:.1})",
        dernier.trame.y
    );
    let plus_haut = |k: usize| essai.iter().map(|i| i.nacelles[k]).fold(f32::MAX, f32::min);
    assert!(plus_haut(0) < 256.0 - 48.0, "nacelle 1 : {}", plus_haut(0));
    let portees: Vec<&&ImageEssai> = essai.iter().filter(|i| i.montee && i.au_sol).collect();
    assert!(portees.len() > 60, "{} images portées", portees.len());
    for i in &portees {
        let pieds = i.trame.y + pepin_core::physics::JOUEUR_H;
        assert!(
            i.nacelles.iter().any(|&n| (pieds - n).abs() < 0.01),
            "pieds en y = {pieds}, nacelles en y = {:?}",
            i.nacelles
        );
    }
    println!(
        "essai montgolfière (le plus rapide) : {} images de jeu, {} portées, plus haut : \
             {:.1} et {:.1}",
        essai.len(),
        portees.len(),
        plus_haut(0),
        plus_haut(1)
    );
}

/// Le parcours « jusqu'en haut » (celui des captures) : chaque montgolfière monte
/// JUSQU'EN HAUT puis s'arrête, la première de 8 tuiles (nacelle de y = 256 à
/// y = 128), la seconde sous le nuage (de y = 144 à y = 64), Pépin y est
/// toujours posé pendant la montée, et la première, quittée, redescend vers son
/// quai pendant que Pépin monte dans la seconde. Trois gouttes, aucune mort.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_monte_jusqu_en_haut_des_deux_montgolfieres() {
    let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere_jusqu_en_haut());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(partie.gouttes, 3);
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let dernier = essai.last().unwrap();
    let plus_haut = |k: usize| essai.iter().map(|i| i.nacelles[k]).fold(f32::MAX, f32::min);
    assert_eq!(plus_haut(0), 128.0, "la première monte de 8 tuiles");
    assert_eq!(plus_haut(1), 64.0, "la seconde s'arrête sous le nuage");
    let montees = essai.iter().filter(|i| i.montee).count();
    assert!(
        montees >= 260,
        "{montees} images de montée (128 + 80 px à 0,8 px : 260)"
    );
    // Une montée où Pépin n'est pas posé ne dure qu'une image : celle où il
    // saute de la nacelle (elle montait encore au début du pas). Dès l'image
    // suivante, plus aucune nacelle ne monte.
    for (k, i) in essai.iter().enumerate() {
        if i.montee && !i.au_sol {
            assert!(
                essai.get(k + 1).is_none_or(|s| !s.montee),
                "image {k} : Pépin en l'air pendant une montée"
            );
        }
    }
    assert!(
        dernier.nacelles[0] > 128.0 + 20.0,
        "quittée, la première redescend (nacelle en y = {:.1})",
        dernier.nacelles[0]
    );
    println!(
        "essai montgolfière (jusqu'en haut) : {} images de jeu, {montees} de montée, \
             nacelles à la fin en y = {:?}",
        essai.len(),
        dernier.nacelles
    );
}

/// Le robot finit le niveau d'essai « escargot » (touche F6 du build de QA) sans
/// mourir : trois gouttes, l'arrivée en haut de la falaise 2 (y < 48). Il a
/// rebondi au moins deux fois sur un escargot (une par falaise) ; chaque rebond
/// relance Pépin vers le haut et aplatit la coquille de l'escargot touché.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_finit_le_niveau_d_essai_escargot() {
    let partie = jouer_essai_escargot(&parcours_essai_escargot());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(partie.gouttes, 3, "les trois gouttes de l'essai escargot");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F6, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let dernier = essai.last().unwrap();
    assert!(
        dernier.trame.y < 48.0,
        "arrivée en haut de la falaise 2 (y = {:.1})",
        dernier.trame.y
    );
    let rebonds: Vec<usize> = (0..essai.len())
        .filter(|&k| essai[k].rebond_escargot)
        .collect();
    assert!(rebonds.len() >= 2, "{} rebond(s)", rebonds.len());
    for &k in &rebonds {
        assert!(
            essai[k].escargots.iter().any(|e| e.2),
            "image {k} : rebond sans coquille aplatie"
        );
    }
    let contre = essai.iter().filter(|i| i.contre_escargot).count();
    println!(
        "essai escargot : {} images de jeu, rebonds aux images {rebonds:?}, {contre} \
             image(s) au sol contre un escargot",
        essai.len()
    );
}

/// Le robot finit le niveau d'essai « chat » (touche F7 du build de QA) sans
/// mourir : trois gouttes (dont la goutte cachée derrière le chat), l'arrivée.
/// Il a écrasé le chat une fois, PENDANT son sommeil ou son annonce (jamais
/// réveillé), et il n'est passé sous le grand mur (x > 432) qu'après.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_finit_le_niveau_d_essai_chat() {
    let partie = jouer_essai_chat(&parcours_essai_chat());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(partie.gouttes, 3, "les trois gouttes de l'essai chat");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F7, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let ecrasements: Vec<usize> = (0..essai.len()).filter(|&k| essai[k].ecrase_chat).collect();
    assert_eq!(ecrasements.len(), 1, "un seul écrasement : {ecrasements:?}");
    let k = ecrasements[0];
    // La phase au moment du contact : celle de l'image de l'écrasement (le
    // chat avance d'un pas AVANT les contacts, voir `Monde::avancer`), et un
    // chat écrasé ne change plus de phase. Endormi ou en train de se réveiller.
    let (phase, ecrase) = essai[k].chat.unwrap();
    assert!(
        ecrase && !matches!(phase, PhaseChat::Eveille(_)),
        "{phase:?}"
    );
    // … et jamais de justesse : au moins `MARGE_CHAT_PAS` après qu'il s'est
    // endormi, et avant son réveil.
    let p = pas_du_cycle(phase);
    assert!(
        p >= MARGE_CHAT_PAS && p + MARGE_CHAT_PAS < DORT_PAS + REVEIL_PAS,
        "écrasé de justesse : {phase:?}"
    );
    assert!(
        essai[..k].iter().all(|i| i.trame.x < 432.0),
        "le robot passe sous le mur avant d'avoir écrasé le chat"
    );
    assert!(essai[k..].iter().any(|i| i.trame.x > 432.0));
    // Il est passé par le bloc (dessus à y = 112) et s'est laissé tomber de là
    // dans la fente. Depuis la carte 6.3, le chat dort 1,7 s au lieu de 4 : le
    // robot arrive sur le bloc pendant son 2e sommeil et saute aussitôt ; il
    // l'écrase avant son éveil (pendant le sommeil ou l'annonce), sans jamais
    // l'avoir vu éveillé entre le bloc et l'écrasement (avant la carte 6.3, il
    // arrivait sur le bloc pendant l'éveil et devait l'y attendre).
    let sur_le_bloc = essai[..k]
        .iter()
        .rposition(|i| i.au_sol && (i.trame.y - 98.0).abs() < 1.0)
        .expect("le robot n'est pas passé par le bloc");
    assert!(
        essai[sur_le_bloc..k].iter().all(|i| matches!(
            i.chat,
            Some((PhaseChat::Dort(_) | PhaseChat::Reveil(_), false))
        )),
        "le chat s'est réveillé entre le bloc et l'écrasement"
    );
    // Le chat s'est-il réveillé pendant que le robot l'approchait ?
    let eveils = essai[..k]
        .windows(2)
        .filter(|w| {
            matches!(w[1].chat, Some((PhaseChat::Eveille(_), false)))
                && !matches!(w[0].chat, Some((PhaseChat::Eveille(_), _)))
        })
        .count();
    println!(
        "essai chat : {} images de jeu, chat écrasé à l'image {k} ({phase:?}), \
             {eveils} réveil(s) avant",
        essai.len()
    );
}

/// Le robot finit le niveau d'essai « horloge » (touche F8) sans mourir, avec
/// les trois gouttes, porté par les DEUX aiguilles : il monte sur chacune (au
/// moins une tuile de montée pendant qu'elle le porte) avant de sauter plus haut.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_finit_le_niveau_d_essai_horloge() {
    let partie = jouer_essai_horloge(&parcours_essai_horloge());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(partie.gouttes, 3, "les trois gouttes de l'essai horloge");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F8, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    for k in 0..2 {
        let portees: Vec<f32> = essai
            .iter()
            .filter(|i| i.sur_aiguille == Some(k))
            .map(|i| i.trame.y)
            .collect();
        assert!(!portees.is_empty(), "jamais porté par l'aiguille {k}");
        let montee = portees.iter().fold(f32::MIN, |a, &y| a.max(y))
            - portees.iter().fold(f32::MAX, |a, &y| a.min(y));
        println!(
            "essai horloge : aiguille {k}, {} images portées, {montee:.1} px de montée",
            portees.len()
        );
        assert!(montee >= 16.0, "aiguille {k} : {montee} px");
    }
    // Porté, Pépin a les pieds pile sur le dessus de son aiguille.
    for i in essai.iter().filter(|i| i.sur_aiguille.is_some()) {
        let k = i.sur_aiguille.unwrap();
        assert!((i.trame.y + 14.0 - i.aiguilles[k].1).abs() < 0.01);
    }
    println!("essai horloge : {} images de jeu", essai.len());
}

/// Le robot bat le Grand Nuage (arène, touche F10) sans mourir : trois coups sur
/// sa tête, chacun PENDANT un essoufflement, le ciel qui se déchire, puis l'écran
/// de fin ; les trois gouttes. Ses points de vie descendent de 3 à 0, un coup à la
/// fois, et il a traversé ses trois motifs d'éclairs.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_bat_le_grand_nuage() {
    let partie = jouer_essai_nuage(&parcours_essai_nuage());
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(partie.gouttes, 3, "les trois gouttes de l'arène");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F10, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let coups: Vec<usize> = (0..essai.len()).filter(|&k| essai[k].coup_nuage).collect();
    assert_eq!(coups.len(), 3, "trois coups : {coups:?}");
    for (n, &k) in coups.iter().enumerate() {
        // Juste avant le coup, le nuage était essoufflé, avec `3 - n` points.
        let (phase, _, pv) = essai[k - 1].nuage.unwrap();
        assert_eq!(
            (phase, pv),
            (PhaseNuage::Essouffle, 3 - n as u8),
            "coup {n}"
        );
        let (apres, _, pv) = essai[k].nuage.unwrap();
        assert_eq!(pv, 2 - n as u8);
        let attendu = if n == 2 {
            PhaseNuage::Dechirure
        } else {
            PhaseNuage::Touche
        };
        assert_eq!(apres, attendu);
    }
    // Les trois motifs ont été joués (une colère à 3, à 2, puis à 1 point).
    for pv in 1..=3 {
        assert!(
            essai
                .iter()
                .any(|i| matches!(i.nuage, Some((PhaseNuage::Colere, _, p)) if p == pv)),
            "aucune colère à {pv} point(s)"
        );
    }
    // Le ciel s'est déchiré jusqu'au bout (120 images après le dernier coup),
    // puis le nuage est vaincu et la partie passe à l'écran de fin.
    let dernier = coups[2];
    assert!(essai[dernier..]
        .iter()
        .all(|i| matches!(i.nuage, Some((PhaseNuage::Dechirure, _, 0)))));
    assert_eq!(essai.len() - dernier, 120, "déchirure : 2 s");
    assert!(partie
        .images
        .iter()
        .any(|i| matches!(i.nuage, Some((PhaseNuage::Vaincu, _, 0)))));
    println!(
        "essai nuage : {} images de jeu, coups aux images {coups:?}",
        essai.len()
    );
}

/// Le parcours de l'arène tient si le monde a une ou deux images d'avance ou de
/// retard sur les touches (comme celui de l'horloge) : nuage vaincu, 0 mort, 3
/// gouttes à chaque fois.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_parcours_du_grand_nuage_tient_a_une_image_pres() {
    let parcours = parcours_essai_nuage();
    for decalage in [-2, -1, 0, 1, 2] {
        let (fin, morts, gouttes) =
            rejouer_decale(crate::monde::INDICE_ESSAI_NUAGE, &parcours, decalage);
        assert!(
            fin && morts == 0 && gouttes == 3,
            "décalage {decalage} : fin {fin}, {morts} mort(s), {gouttes} goutte(s)"
        );
    }
}

/// Rejoue `parcours` dans le niveau `indice` avec les entrées décalées de
/// `decalage` images par rapport au monde (positif : Pépin agit plus tard ;
/// négatif : plus tôt), puis 60 images sans rien toucher (comme le navigateur à
/// la fin du script). Renvoie (drapeau atteint, morts, gouttes du niveau).
fn rejouer_decale(indice: usize, parcours: &[Entrees], decalage: i32) -> (bool, u32, usize) {
    let mut p = parcours.to_vec();
    if decalage > 0 {
        p.splice(
            0..0,
            std::iter::repeat_n(Entrees::default(), decalage as usize),
        );
    } else {
        // Le parcours commence par l'attente du départ : on n'y retire que du vide.
        assert!(p[..(-decalage) as usize]
            .iter()
            .all(|e| *e == Entrees::default()));
        p.drain(0..(-decalage) as usize);
    }
    p.extend(std::iter::repeat_n(Entrees::default(), 60));
    let mut m = Monde::new(indice);
    for e in &p {
        m.avancer(e);
        if m.termine {
            break;
        }
    }
    let gouttes = m.gouttes_niveau.iter().filter(|&&g| g).count();
    (m.termine, m.etat.morts, gouttes)
}

/// Le parcours de l'essai horloge tient si le monde a une ou deux images
/// d'avance ou de retard sur les touches (le rejeu dans le navigateur peut
/// démarrer le niveau d'essai une image plus tôt ou plus tard) : drapeau, 0
/// mort, 3 gouttes à chaque fois. C'est ce que garantit `tient_au_decalage`.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_parcours_de_l_horloge_tient_a_une_image_pres() {
    let parcours = parcours_essai_horloge();
    for decalage in [-2, -1, 0, 1, 2] {
        let (fin, morts, gouttes) =
            rejouer_decale(crate::monde::INDICE_ESSAI_HORLOGE, &parcours, decalage);
        assert!(
            fin && morts == 0 && gouttes == 3,
            "décalage {decalage} : drapeau {fin}, {morts} mort(s), {gouttes} goutte(s)"
        );
    }
}

/// Le robot finit aussi le niveau d'essai vertical (45 rangées) sans mourir, à travers
/// l'`Application` (touche 0 du build de QA). Pendant tout le parcours, la caméra suit
/// Pépin en hauteur : elle part de sa borne basse, monte jusqu'en haut du niveau et
/// redescend ; Pépin est TOUJOURS entier à l'écran, sauf quand il est lancé au-dessus
/// du niveau, et alors la flèche de repère est là (et seulement alors).
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_finit_le_niveau_d_essai_vertical() {
    let parcours = parcours_essai_vertical();
    let partie = jouer_essai_vertical(&parcours);
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche 0, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    assert!(essai.len() > 300, "{} images de jeu", essai.len());
    let ys: Vec<i32> = essai.iter().map(|i| i.camera.1).collect();
    let (haut, bas) = (*ys.iter().min().unwrap(), *ys.iter().max().unwrap());
    assert_eq!((haut, bas), (0, 540), "la caméra parcourt toute la hauteur");
    assert_eq!(ys[0], 540, "départ en bas");
    let sommet = ys.iter().position(|&y| y == 0).unwrap();
    assert!(ys[sommet..].contains(&540), "puis redescend jusqu'en bas");
    let mut reperes = 0;
    for (k, i) in essai.iter().enumerate() {
        let (y, h) = (i.pepin_ecran_y, 14);
        assert!(y + h <= 180, "image {k} : Pépin sous l'écran (y = {y})");
        if i.camera.1 > 0 {
            // Caméra sous sa borne haute : Pépin est entier à l'écran, sans flèche.
            assert!(
                y >= 0 && !i.repere,
                "image {k} : Pépin hors de l'écran (y = {y}, caméra {:?})",
                i.camera
            );
        } else {
            // Caméra en haut du niveau : la flèche apparaît dès que Pépin est
            // entièrement au-dessus du cadre, jamais avant.
            assert_eq!(i.repere, y + h <= 0, "image {k} : y = {y}");
        }
        reperes += usize::from(i.repere);
    }
    assert!(reperes > 0, "le champignon lance Pépin au-dessus du niveau");
    println!(
        "essai vertical : {} images de jeu, {reperes} avec la flèche, gouttes {}",
        essai.len(),
        partie.gouttes
    );
}

/// Démonstration de la carte 5.6b : Pépin posé sur la casette pendant toute
/// l'annonce, porté par le souffle suivant jusqu'en haut de la colonne, puis
/// redescendu à l'arrêt, sans mourir.
#[test]
fn pepin_attend_sur_la_casette_puis_le_souffle_le_porte() {
    let parcours = parcours_essai_ventilateur_caisse().expect("un saut pose Pépin sur la casette");
    let partie = jouer_essai_ventilateur(&parcours);
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let phase = |i: &ImageEssai| i.ventilateur.map(|v| v.0);
    // L'annonce qui suit la montée sur la casette : Pépin y est posé.
    let fin_elan = parcours.len();
    let annonce: Vec<&&ImageEssai> = essai
        .iter()
        .skip(fin_elan)
        .take_while(|i| phase(i) != Some(PhaseVentilateur::Souffle))
        .filter(|i| phase(i) == Some(PhaseVentilateur::Annonce))
        .collect();
    assert!(!annonce.is_empty(), "une annonce après l'élan");
    assert!(annonce
        .iter()
        .all(|i| i.au_sol && (i.trame.y - 298.0).abs() < 0.5));
    // Le souffle suivant le porte haut (au moins 10 tuiles), puis, à l'arrêt, il
    // retombe sur la casette.
    let (k_haut, haut) = essai
        .iter()
        .enumerate()
        .skip(fin_elan)
        .map(|(k, i)| (k, i.trame.y))
        .fold((0, f32::MAX), |a, b| if b.1 < a.1 { b } else { a });
    assert!(haut < 298.0 - 160.0, "porté jusqu'à y = {haut}");
    assert!(
        essai[k_haut..]
            .iter()
            .any(|i| i.au_sol && (i.trame.y - 298.0).abs() < 0.5),
        "Pépin retombe sur la casette"
    );
    assert_eq!(partie.morts, 0);
}

/// Le robot finit le niveau d'essai « eau » (carte 9.2, touche F11 du build de QA)
/// sans mourir : le bassin ne se traverse qu'en BARQUE (y marcher noie, il est trop
/// large pour un saut planant depuis le sol), donc le parcours passe par l'eau
/// (planer pour flotter, laisser le courant porter, sauter hors de l'eau). Les
/// trois gouttes sont ramassées, aucune mort.
#[test]
fn le_robot_finit_le_niveau_d_essai_eau() {
    let parcours = parcours_essai_eau();
    let partie = jouer_essai_eau(&parcours);
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(partie.gouttes, 3, "les trois gouttes du niveau d'essai eau");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche F11, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    assert!(essai.len() > 100, "{} images de jeu", essai.len());
    // La barque flotte : les pieds de Pépin restent `ENFONCEMENT_BARQUE` (3 px)
    // sous la surface (144, au ras du sol), le haut de sa boîte (`trame.y`) est
    // donc à 144 - 14 + 3 = 133 : ni au sol (130), ni sur le quai (114), ni en
    // vol (il traverserait cette hauteur en une image ou deux). Il a bel et bien
    // traversé l'eau en barque. Depuis la carte 9.2b (BAS fait la barque), le
    // robot « ricoche » plus court : quelques images à bord suffisent (5 au
    // dernier passage, plus de 10 avant).
    let y_barque = 144.0 - pepin_core::physics::JOUEUR_H + pepin_core::physics::ENFONCEMENT_BARQUE;
    let en_barque = essai
        .iter()
        .filter(|i| !i.au_sol && (i.trame.y - y_barque).abs() < 0.01)
        .count();
    assert!(en_barque >= 3, "{en_barque} images à bord de la barque");
    let dernier = essai.last().unwrap();
    assert!(dernier.trame.x > 500.0, "arrivée à x = {}", dernier.trame.x);
    println!(
        "essai eau : {} images de jeu, {} en barque, gouttes {}, fin à x = {:.1}",
        essai.len(),
        en_barque,
        partie.gouttes,
        dernier.trame.x
    );
}

/// Écrit le parcours du robot dans le niveau d'essai « eau » pour les captures de la carte
/// 9.2 (`docs/capture-92.mjs`) : `target/qa/script-92-eau.txt`, un chiffre hexadécimal par
/// pas (2 gauche, 4 droite, 8 saut), comme les autres scripts du navigateur. Ignoré : on le
/// lance à la main (`--ignored ecrire_script_essai_eau_92`).
#[test]
#[ignore]
fn ecrire_script_essai_eau_92() {
    let parcours = parcours_essai_eau();
    let script: String = parcours
        .iter()
        .map(|e| crate::bot_qa::code_entrees(e, false).to_ascii_uppercase())
        .collect();
    std::fs::create_dir_all("../target/qa").unwrap();
    std::fs::write("../target/qa/script-92-eau.txt", &script).unwrap();
    println!("script-92-eau.txt : {} pas", script.len());
}

/// Le robot finit le niveau d'essai « ennemis d'eau » (carte 9.4, touche J du build
/// de QA) sans mourir : il passe le crabe, la grenouille, le bassin du poisson, le
/// tunnel du rat et la grue de la mouette.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_finit_le_niveau_d_essai_ennemis_eau() {
    let parcours = parcours_essai_ennemis_eau();
    let partie = jouer_essai_ennemis_eau(&parcours);
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche J, une seule fois"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    let dernier = essai.last().unwrap();
    assert!(dernier.trame.x > 900.0, "arrivée à x = {}", dernier.trame.x);
    println!(
        "essai ennemis d'eau : {} images de jeu, gouttes {}, fin à x = {:.1}",
        essai.len(),
        partie.gouttes,
        dernier.trame.x
    );
}

/// Le robot bat la Pieuvre du port dans son arène d'essai (carte 9.12a, touche L du
/// build de QA) sans mourir, après avoir pris les trois gouttes : l'arène n'a pas de
/// drapeau, la fin n'est atteinte que si elle est vaincue.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_bat_la_pieuvre_sans_mourir() {
    let parcours = parcours_essai_pieuvre();
    let partie = jouer_essai_pieuvre(&parcours);
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(partie.gouttes, 3);
    assert_eq!(
        partie.images.iter().filter(|i| i.touche_essai).count(),
        1,
        "la touche L, une seule fois"
    );
    println!(
        "essai pieuvre : {} images de jeu",
        partie.images.iter().filter(|i| i.en_essai).count()
    );
}

/// Écrit le parcours du robot dans l'arène de la Pieuvre pour les captures de la carte
/// 9.12a (`docs/capture-912.mjs`) : `target/qa/script-912-pieuvre.txt` (un chiffre
/// hexadécimal par pas, comme `ecrire_script_essai_ennemis_eau_94`), le décalage entre
/// l'appui sur L et la première image de jeu (`decalage-912.txt`), et la chronologie de
/// la pieuvre (pour choisir les pas). Ignoré : on le lance à la main.
#[test]
#[ignore]
fn ecrire_script_essai_pieuvre_912() {
    let parcours = parcours_essai_pieuvre();
    let script: String = parcours
        .iter()
        .map(|e| crate::bot_qa::code_entrees(e, false).to_ascii_uppercase())
        .collect();
    std::fs::create_dir_all("../target/qa").unwrap();
    std::fs::write("../target/qa/script-912-pieuvre.txt", &script).unwrap();
    let partie = jouer_essai_pieuvre(&parcours);
    let appui = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let debut = partie.images.iter().position(|i| i.en_essai).unwrap();
    std::fs::write("../target/qa/decalage-912.txt", (debut - appui).to_string()).unwrap();
    println!(
        "script : {} pas, décalage : {} images",
        script.len(),
        debut - appui
    );
    let mut m = Monde::charger(crate::monde::INDICE_ESSAI_PIEUVRE, false);
    let mut avant = None;
    for (k, e) in parcours.iter().enumerate() {
        m.avancer(e);
        let etat = m.entites.iter().find_map(|x| match x {
            pepin_core::entities::Entite::Pieuvre(p) => Some((p.phase(), p.pv())),
            _ => None,
        });
        if etat != avant {
            println!("pas {k} : Pépin x = {:.0} : {etat:?}", m.joueur.boite.x);
            avant = etat;
        }
    }
}

/// Écrit le parcours du robot dans le niveau d'essai « ennemis d'eau » pour les captures
/// de la carte 9.4 (`docs/capture-94.mjs`) : `target/qa/script-94-ennemis.txt`, un chiffre
/// hexadécimal par pas (2 gauche, 4 droite, 8 saut). Ignoré : on le lance à la main
/// (`--ignored ecrire_script_essai_ennemis_eau_94`).
#[test]
#[ignore]
fn ecrire_script_essai_ennemis_eau_94() {
    let parcours = parcours_essai_ennemis_eau();
    let script: String = parcours
        .iter()
        .map(|e| crate::bot_qa::code_entrees(e, false).to_ascii_uppercase())
        .collect();
    std::fs::create_dir_all("../target/qa").unwrap();
    std::fs::write("../target/qa/script-94-ennemis.txt", &script).unwrap();
    println!("script-94-ennemis.txt : {} pas", script.len());
    // Images entre l'appui sur J et la première image de jeu du niveau d'essai (la
    // carte du niveau) : le pilote du navigateur appuie sur J au début du script et
    // rejoue ce nombre d'images immobiles avant le parcours, pour que le monde (le
    // poisson, le crabe, qui vivent sans Pépin) soit exactement celui du robot.
    let partie = jouer_essai_ennemis_eau(&parcours);
    let appui = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let debut = partie.images.iter().position(|i| i.en_essai).unwrap();
    std::fs::write("../target/qa/decalage-94.txt", (debut - appui).to_string()).unwrap();
    println!("décalage : {} images", debut - appui);
    // Chronologie des phases des ennemis le long du parcours (pour choisir les pas des
    // captures) : une ligne à chaque changement de phase.
    let mut m = Monde::charger(crate::monde::INDICE_ESSAI_ENNEMIS_EAU, false);
    let mut avant: Vec<String> = Vec::new();
    for (k, e) in parcours.iter().enumerate() {
        m.avancer(e);
        let phases: Vec<String> = m
            .entites
            .iter()
            .filter_map(|x| match x {
                pepin_core::entities::Entite::Ennemi(c) => Some(format!(
                    "{:?}:{:?}/{:?}/{:?}@{:.0}",
                    c.type_ennemi,
                    c.etat,
                    c.phase_eau(),
                    c.phase(),
                    c.boite.x
                )),
                _ => None,
            })
            .collect();
        for (i, p) in phases.iter().enumerate() {
            let court = |s: &str| s.split('@').next().unwrap().to_string();
            if avant.get(i).map(|a| court(a)) != Some(court(p)) {
                println!("pas {k} : Pépin x = {:.0} : {p}", m.joueur.boite.x);
            }
        }
        avant = phases;
    }
}

/// Scénarios des captures de la carte 9.4 que le parcours du robot ne montre pas :
/// Pépin qui REBONDIT sur le crabe (ses pinces claquent), et le rat qui le voit, se
/// dresse puis FONCE (le robot, lui, l'écrase sans lui laisser le temps). Cherchés par
/// force brute sur la vraie simulation, écrits dans `target/qa/script-94-crabe.txt` et
/// `script-94-rat.txt` (même format que le parcours), avec les pas à capturer.
/// Ignoré : on le lance à la main (`--ignored ecrire_scenarios_94`).
#[test]
#[ignore]
fn ecrire_scenarios_94() {
    use pepin_core::entities::contacts::Contact;
    use pepin_core::entities::ennemi::PhaseEau;
    use pepin_core::entities::{Entite, TypeEnnemi};
    use pepin_core::player::Direction;
    let i = crate::monde::INDICE_ESSAI_ENNEMIS_EAU;
    let encoder = |p: &[Entrees]| -> String {
        p.iter()
            .map(|e| crate::bot_qa::code_entrees(e, false).to_ascii_uppercase())
            .collect()
    };
    let phase = |m: &Monde, t: TypeEnnemi| {
        m.entites.iter().find_map(|e| match e {
            Entite::Ennemi(c) if c.type_ennemi == t && c.est_visible() => Some(c.phase_eau()),
            _ => None,
        })
    };
    let droite = |saut| Entrees {
        direction: Direction::Droite,
        saut,
        bas: false,
    };
    std::fs::create_dir_all("../target/qa").unwrap();
    // 1. Le crabe : attendre, courir à droite, sauter ; un rebond sur la carapace.
    let mut trouve = false;
    'crabe: for attente in (0..120).step_by(10) {
        for saut_a in (0..40).step_by(2) {
            for duree in [4, 8, 12, 16] {
                let mut p = vec![Entrees::default(); 40 + attente];
                for k in 0..120 {
                    p.push(droite(k >= saut_a && k < saut_a + duree));
                }
                let mut m = Monde::charger(i, false);
                for (k, e) in p.iter().enumerate() {
                    m.avancer(e);
                    if m.derniere_mort.is_some() {
                        break;
                    }
                    if matches!(phase(&m, TypeEnnemi::Crabe), Some(PhaseEau::Attaque(_)))
                        && m.contacts.contains(&Contact::Rebond)
                    {
                        let script = encoder(&p[..k + 1 + 30]);
                        std::fs::write("../target/qa/script-94-crabe.txt", &script).unwrap();
                        println!("crabe : rebond au pas {k}, capture au pas {}", k + 4);
                        trouve = true;
                        break 'crabe;
                    }
                }
            }
        }
    }
    assert!(trouve, "aucun rebond sur le crabe");
    // 2. Le rat : le parcours du robot jusqu'avant le tunnel, attendre que le rat
    // s'éloigne, puis courir à droite et s'arrêter dans le tunnel ; le rat se dresse
    // puis fonce.
    let parcours = parcours_essai_ennemis_eau();
    let mut trouve = false;
    'rat: for attente in (0..400).step_by(20) {
        for course in (10..60).step_by(4) {
            let mut p = parcours[..380].to_vec();
            p.extend(std::iter::repeat_n(Entrees::default(), attente));
            p.extend((0..course).map(|_| droite(false)));
            p.extend(std::iter::repeat_n(Entrees::default(), 150));
            let mut m = Monde::charger(i, false);
            let (mut annonce, mut fonce) = (None, None);
            for (k, e) in p.iter().enumerate() {
                m.avancer(e);
                if m.derniere_mort.is_some() {
                    break;
                }
                match phase(&m, TypeEnnemi::Rat) {
                    Some(PhaseEau::Annonce(12)) => {
                        // Pépin assez loin : la course se voit avant de l'atteindre.
                        let rat = m.entites.iter().find_map(|e| match e {
                            Entite::Ennemi(c) if c.type_ennemi == TypeEnnemi::Rat => Some(c.boite),
                            _ => None,
                        });
                        if rat.is_some_and(|r| (r.x - m.joueur.boite.x).abs() > 56.0) {
                            annonce = Some(k);
                        }
                    }
                    Some(PhaseEau::Attaque(6)) => fonce = Some(k),
                    _ => {}
                }
                if fonce.is_some() {
                    break;
                }
            }
            if let (Some(a), Some(f)) = (annonce, fonce) {
                std::fs::write("../target/qa/script-94-rat.txt", encoder(&p[..f + 60])).unwrap();
                println!("rat : annonce au pas {a}, fonce au pas {f}");
                trouve = true;
                break 'rat;
            }
        }
    }
    assert!(trouve, "le rat ne fonce jamais");
}

/// Le robot finit le niveau d'essai « eau 2 » (carte 9.3, touche F12 du build de
/// QA) sans mourir, les trois gouttes prises : il passe sous la stalactite, saute
/// de nénuphar en nénuphar, monte dans les bulles, attend la marée en barque et
/// descend la chute d'eau parapluie ouvert.
#[test]
#[ignore = "lot : robot de QA : recherche de parcours puis partie rejouée"]
fn le_robot_finit_le_niveau_d_essai_eau2() {
    let parcours = parcours_essai_eau2();
    let partie = jouer_essai_eau2(&parcours);
    assert!(partie.fin_atteinte, "l'écran de fin n'a pas été atteint");
    assert_eq!(partie.morts, 0, "le robot ne doit jamais mourir");
    assert_eq!(
        partie.gouttes, 3,
        "les trois gouttes du niveau d'essai eau 2"
    );
    let essai: Vec<&ImageEssai> = partie.images.iter().filter(|i| i.en_essai).collect();
    println!("essai eau 2 : {} images de jeu", essai.len());
}

/// Écrit le parcours du robot dans le niveau d'essai « eau 2 » pour les captures
/// de la carte 9.3 (`docs/capture-93.mjs`) : `target/qa/script-93-eau2.txt`, un
/// chiffre hexadécimal par pas (2 gauche, 4 droite, 8 saut). Ignoré : on le
/// lance à la main (`--ignored ecrire_script_essai_eau2_93`).
#[test]
#[ignore]
fn ecrire_script_essai_eau2_93() {
    let parcours = parcours_essai_eau2();
    let script: String = parcours
        .iter()
        .map(|e| crate::bot_qa::code_entrees(e, false).to_ascii_uppercase())
        .collect();
    std::fs::create_dir_all("../target/qa").unwrap();
    std::fs::write("../target/qa/script-93-eau2.txt", &script).unwrap();
    // Images du navigateur entre l'appui sur F12 (compris) et le premier pas du
    // niveau d'essai : le pilote les laisse passer avant de rejouer le script,
    // pour que la marée soit à la même heure que dans ce test.
    let images = jouer_essai_eau2(&parcours).images;
    let appui = images.iter().position(|i| i.touche_essai).unwrap();
    let jeu = images.iter().position(|i| i.en_essai).unwrap();
    println!(
        "script-93-eau2.txt : {} pas ; {} images de F12 au jeu",
        script.len(),
        jeu - appui
    );
    // Les moments du parcours à capturer (pas du script) : la première image de
    // chaque mécanique en action, et l'heure de la marée.
    use pepin_core::entities::stalactite::EtatStalactite;
    use pepin_core::entities::Entite;
    let mut m = Monde::charger(crate::monde::INDICE_ESSAI_EAU2, false);
    let mut vus: Vec<&str> = Vec::new();
    for (pas, e) in parcours.iter().enumerate() {
        m.avancer(e);
        let j = &m.joueur;
        let mut moment = |nom: &'static str, vrai: bool| {
            if vrai && !vus.contains(&nom) {
                vus.push(nom);
                let maree = m.entites.iter().find_map(|e| match e {
                    Entite::Maree(r) => Some((r.phase(), r.pas())),
                    _ => None,
                });
                println!(
                    "{nom} : pas {pas}, x {:.0}, y {:.0}, marée {maree:?}",
                    j.boite.x, j.boite.y
                );
            }
        };
        let stal = m.entites.iter().find_map(|e| match e {
            Entite::Stalactite(s) => Some(s.etat),
            _ => None,
        });
        moment("tremble", matches!(stal, Some(EtatStalactite::Tremble(_))));
        moment(
            "chute_stalactite",
            matches!(stal, Some(EtatStalactite::Chute(_))),
        );
        let sur_feuille = m.entites.iter().any(|e| {
            matches!(e, Entite::Nenuphar(n) if j.au_sol && n.enfoncement() > 0.2
                && (j.boite.bas() - n.boite.y).abs() < 0.5)
        });
        moment("nenuphar", sur_feuille);
        moment(
            "bulles",
            (330.0..355.0).contains(&j.boite.x) && j.boite.y < 120.0,
        );
        moment(
            "barque_maree",
            j.en_barque() && (496.0..656.0).contains(&j.boite.x),
        );
        moment(
            "chute_d_eau",
            (778.0..812.0).contains(&j.boite.x) && j.boite.y > 120.0,
        );
        moment("arrivee", m.termine);
    }
    // Le nénuphar (le robot n'en a pas besoin : il traverse le bassin en barque) :
    // le début du parcours, puis la recherche du robot jusqu'à une feuille, et
    // Pépin y reste immobile : elle le met à l'eau. `script-93-nenuphar.txt`.
    use crate::bot_qa::parcours::{chercher, plans};
    let debut = 150;
    let mut m = Monde::charger(crate::monde::INDICE_ESSAI_EAU2, false);
    for e in &parcours[..debut] {
        m.avancer(e);
    }
    let sur_feuille = |m: &Monde| {
        m.joueur.au_sol
            && m.entites.iter().any(|e| {
                matches!(e, Entite::Nenuphar(n) if (m.joueur.boite.bas() - n.boite.y).abs() < 0.5)
            })
    };
    let etapes = chercher(&m, &plans(false), &sur_feuille, 50_000).expect("une feuille");
    let mut feuille: Vec<Entrees> = parcours[..debut].to_vec();
    for e in etapes {
        feuille.extend(e.entrees);
    }
    let pose = feuille.len();
    feuille.extend(vec![Entrees::default(); 120]);
    let texte: String = feuille
        .iter()
        .map(|e| crate::bot_qa::code_entrees(e, false).to_ascii_uppercase())
        .collect();
    std::fs::write("../target/qa/script-93-nenuphar.txt", &texte).unwrap();
    println!(
        "script-93-nenuphar.txt : {} pas, posé sur la feuille au pas {pose}",
        texte.len()
    );
}
