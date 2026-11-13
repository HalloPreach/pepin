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
/// monte réellement au balcon du drapeau — le seul chemin passe par le souffle
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
/// JUSQU'EN HAUT puis s'arrête — la première de 8 tuiles (nacelle de y = 256 à
/// y = 128), la seconde sous le nuage (de y = 144 à y = 64) —, Pépin y est
/// toujours posé pendant la montée, et la première, quittée, redescend vers son
/// quai pendant que Pépin monte dans la seconde. Trois gouttes, aucune mort.
#[test]
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
    // Il est arrivé sur le bloc (dessus à y = 112) pendant l'éveil du chat, et
    // l'y a attendu : posé là-haut, chat réveillé.
    assert!(
        essai[..k].iter().any(|i| {
            i.au_sol
                && (i.trame.y - 98.0).abs() < 1.0
                && matches!(i.chat, Some((PhaseChat::Eveille(_), false)))
        }),
        "le robot n'a pas attendu le chat sur le bloc"
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
