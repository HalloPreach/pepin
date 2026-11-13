//! Scripts et traces du navigateur, un par NIVEAU D'ESSAI : des tests
//! `ignore` (on n'écrit pas ces fichiers à chaque `cargo test`) qui
//! rejouent le parcours de référence du niveau (`essais.rs`) et ÉCRivent
//! dans `../target/qa/` :
//!
//! - `script-*.txt` : un chiffre hexadécimal par image (le code `Trame::code`
//!   de la touche à appuyer — `0` quand la touche est lâchée) ;
//! - `touches-*.txt` : l'image du PILOTE où appuyer sur la touche raccourci
//!   du niveau (0, F1, F2, … : le navigateur a un pas de retard sur la
//!   numérotation du pilote, une touche d'UNE image doit tomber une image
//!   plus tard que dans la trace) ;
//! - `trace-*.csv` : écran, position de Pépin, caméra rendue et état de la
//!   mécanique, image par image (les numéros d'image des captures de QA.md
//!   viennent d'ici).
//!
//! Un navigateur réel (build de QA servi en local) rejoue le script image
//! par image : c'est la preuve que le parcours du robot tient dans le
//! navigateur, pas seulement dans la simulation (voir QA.md, captures
//! `52-` à `511-`). Le test `ecrire_script_navigateur` (script de TOUS les
//! niveaux, `script.txt`/`trace.csv`) est dans `mod.rs`, à côté de
//! `jouer_tout`.
//!
//! Lancer : `cargo test -p game --no-default-features -- --ignored ecrire_script`.
//!
//! ## Ajouter un niveau d'essai
//!
//! Un test `ecrire_script_essai_X` à la suite des autres (même forme : jouer,
//! écrire les trois fichiers, `println!` récapitulatif), avec le préfixe
//! `*.X` des fichiers et le bon numéro d'image de la touche raccourci.
//!
//! Notions Rust illustrées ici : des `String` assemblés par `format!` et
//! `push_str` (pas de `Vec` temporaire par image), des `Option` (la position
//! de la touche est `position(...)`, vérifiée par `unwrap` : elle DOIT y
//! être) et des tests `ignore` (preuve sur demande, jamais au vert par
//! défaut).

use pepin_core::entities::ennemi::PhaseChat;
use pepin_core::entities::ventilateur::PhaseVentilateur;

use crate::bot_qa::essais::*;

/// Écrit le script du niveau d'essai vertical pour le navigateur (build de QA) :
/// `target/qa/script-52.txt` (un chiffre hexadécimal par image, comme `script.txt`),
/// `target/qa/touches-52.txt` (image du pilote où appuyer sur la touche 0) et
/// `target/qa/trace-52.csv` (position de Pépin et caméra rendue, image par image).
/// Dans le navigateur, le jeu a un pas de retard sur la numérotation du pilote (déjà
/// noté à la carte 3.1b, voir QA.md) : sans effet sur des touches tenues plusieurs
/// images, mais une touche d'UNE image doit tomber une image plus tard que dans la
/// trace, sinon elle arrive pendant la dernière image de la carte du niveau 1 et le jeu
/// l'ignore (vérifié : à l'image 228, rien ; à 229, la carte « NIVEAU D'ESSAI »).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai
#[test]
#[ignore]
fn ecrire_script_essai_vertical() {
    let partie = jouer_essai_vertical(&parcours_essai_vertical());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-52.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-52.txt", format!("{pilote}:Digit0\n")).unwrap();
    let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,pepin_ecran_y,repere\n");
    for (k, i) in partie.images.iter().enumerate() {
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            i.pepin_ecran_y,
            u8::from(i.repere)
        ));
    }
    std::fs::write("../target/qa/trace-52.csv", trace).unwrap();
    println!(
        "images : {} ; touche 0 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script du niveau d'essai « vent » pour le navigateur (build de QA) :
/// `target/qa/script-53.txt`, `target/qa/touches-53.txt` (image du pilote où
/// appuyer sur F1) et `target/qa/trace-53.csv` (position de Pépin et caméra
/// rendue, image par image). Comme la carte 5.2, la touche du pilote tombe une
/// image plus tard que dans la trace (le navigateur a un pas de retard).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_vent
#[test]
#[ignore]
fn ecrire_script_essai_vent() {
    let partie = jouer_essai_vent(&parcours_essai_vent());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-53.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-53.txt", format!("{pilote}:F1\n")).unwrap();
    let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y\n");
    for (k, i) in partie.images.iter().enumerate() {
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1
        ));
    }
    std::fs::write("../target/qa/trace-53.csv", trace).unwrap();
    println!(
        "images : {} ; F1 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script du niveau d'essai « grêle » pour le navigateur (build de QA) :
/// `target/qa/script-54.txt`, `target/qa/touches-54.txt` (image du pilote où
/// appuyer sur F2, une image après la trace : le navigateur a un pas de retard) et
/// `target/qa/trace-54.csv` (position de Pépin, caméra, plané dans la grêle).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_grele
#[test]
#[ignore]
fn ecrire_script_essai_grele() {
    let partie = jouer_essai_grele(&parcours_essai_grele());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-54.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-54.txt", format!("{pilote}:F2\n")).unwrap();
    let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,sous_la_grele\n");
    for (k, i) in partie.images.iter().enumerate() {
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.sous_la_grele)
        ));
    }
    std::fs::write("../target/qa/trace-54.csv", trace).unwrap();
    println!(
        "images : {} ; F2 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script du niveau d'essai « auvent » (parcours rebond par rebond : les
/// six auvents à la suite) pour le navigateur :
/// `target/qa/script-55.txt`, `target/qa/touches-55.txt` (image du pilote où
/// appuyer sur F3, une image après la trace : le navigateur a un pas de retard) et
/// `target/qa/trace-55.csv` (position de Pépin, caméra, rebonds d'auvent, sol).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_auvent
#[test]
#[ignore]
fn ecrire_script_essai_auvent() {
    let partie = jouer_essai_auvent(&parcours_essai_auvent_rebond_par_rebond());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-55.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-55.txt", format!("{pilote}:F3\n")).unwrap();
    let mut trace = String::from("image,ecran,essai,x,y,cam_x,cam_y,rebond_auvent,au_sol\n");
    for (k, i) in partie.images.iter().enumerate() {
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.rebond_auvent),
            u8::from(i.au_sol)
        ));
    }
    std::fs::write("../target/qa/trace-55.csv", trace).unwrap();
    println!(
        "images : {} ; F3 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script du niveau d'essai « ventilateur » (parcours du robot) pour le
/// navigateur : `target/qa/script-56b.txt`, `target/qa/touches-56b.txt` (image du
/// pilote où appuyer sur F4, une image après la trace : le navigateur a un pas de
/// retard) et `target/qa/trace-56b.csv` (position de Pépin, caméra, sol, phase du
/// ventilateur et pas de souffle restants) : de quoi choisir les images à capturer
/// pendant le souffle, juste après l'arrêt et pendant l'annonce (carte 5.6b).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_ventilateur
#[test]
#[ignore]
fn ecrire_script_essai_ventilateur() {
    let partie = jouer_essai_ventilateur(&parcours_essai_ventilateur());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-56b.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-56b.txt", format!("{pilote}:F4\n")).unwrap();
    let mut trace =
        String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,phase,souffle_restant\n");
    for (k, i) in partie.images.iter().enumerate() {
        let (phase, restant) = match i.ventilateur {
            Some((PhaseVentilateur::Arret, _)) => ("arret", 0),
            Some((PhaseVentilateur::Annonce, _)) => ("annonce", 0),
            Some((PhaseVentilateur::Souffle, r)) => ("souffle", r),
            None => ("-", 0),
        };
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{phase},{restant}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.au_sol)
        ));
    }
    std::fs::write("../target/qa/trace-56b.csv", trace).unwrap();
    println!(
        "images : {} ; F4 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script de la démonstration « Pépin sur la casette » (carte 5.6b) :
/// `target/qa/script-56b-caisse.txt` (1 200 images : la montée sur la casette
/// puis deux cycles du ventilateur) et `target/qa/trace-56b-caisse.csv` (même
/// format que `trace-56b.csv`). Touche F4 : la même image que
/// `touches-56b.txt` (les images avant le niveau d'essai sont les mêmes).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_ventilateur_caisse
#[test]
#[ignore]
fn ecrire_script_essai_ventilateur_caisse() {
    let parcours = parcours_essai_ventilateur_caisse().expect("un saut pose Pépin sur la casette");
    let partie = jouer_essai_ventilateur(&parcours);
    let images = &partie.images[..partie.images.len().min(1200)];
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-56b-caisse.txt", &script).unwrap();
    let mut trace =
        String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,phase,souffle_restant\n");
    for (k, i) in images.iter().enumerate() {
        let (phase, restant) = match i.ventilateur {
            Some((PhaseVentilateur::Arret, _)) => ("arret", 0),
            Some((PhaseVentilateur::Annonce, _)) => ("annonce", 0),
            Some((PhaseVentilateur::Souffle, r)) => ("souffle", r),
            None => ("-", 0),
        };
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{phase},{restant}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.au_sol)
        ));
    }
    std::fs::write("../target/qa/trace-56b-caisse.csv", trace).unwrap();
    let touche = images.iter().position(|i| i.touche_essai).unwrap();
    println!(
        "images : {} ; élan : {} pas ; F4 à l'image {touche} (pilote : {})",
        images.len(),
        parcours.len(),
        touche + 1
    );
}

/// Écrit le script du niveau d'essai « montgolfière » (parcours jusqu'en haut :
/// les deux montées complètes) pour le navigateur :
/// `target/qa/script-57.txt`, `target/qa/touches-57.txt` (image du pilote où
/// appuyer sur F5, une image après la trace : le navigateur a un pas de retard) et
/// `target/qa/trace-57.csv` (position de Pépin, caméra, sol, nacelles, montée).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_montgolfiere
#[test]
#[ignore]
fn ecrire_script_essai_montgolfiere() {
    let partie = jouer_essai_montgolfiere(&parcours_essai_montgolfiere_jusqu_en_haut());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-57.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-57.txt", format!("{pilote}:F5\n")).unwrap();
    let mut trace =
        String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,nacelle1,nacelle2,montee\n");
    for (k, i) in partie.images.iter().enumerate() {
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{:.1},{:.1},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.au_sol),
            i.nacelles[0],
            i.nacelles[1],
            u8::from(i.montee)
        ));
    }
    std::fs::write("../target/qa/trace-57.csv", trace).unwrap();
    println!(
        "images : {} ; F5 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script du niveau d'essai « escargot » pour les captures de la carte
/// 5.8 : `target/qa/script-58.txt`, `target/qa/touches-58.txt` (l'image où
/// appuyer sur F6, une image après la trace : le navigateur a un pas de retard) et
/// `target/qa/trace-58.csv` (position de Pépin, caméra, sol, escargots, rebond,
/// contact de côté).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_escargot
#[test]
#[ignore]
fn ecrire_script_essai_escargot() {
    let partie = jouer_essai_escargot(&parcours_essai_escargot());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-58.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-58.txt", format!("{pilote}:F6\n")).unwrap();
    let mut trace = String::from(
        "image,ecran,essai,x,y,cam_x,cam_y,au_sol,escargot1_x,escargot1_aplati,\
             escargot2_x,escargot2_aplati,rebond,contre\n",
    );
    for (k, i) in partie.images.iter().enumerate() {
        let [e1, e2] = i.escargots;
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{:.1},{},{:.1},{},{},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.au_sol),
            e1.0,
            u8::from(e1.2),
            e2.0,
            u8::from(e2.2),
            u8::from(i.rebond_escargot),
            u8::from(i.contre_escargot)
        ));
    }
    std::fs::write("../target/qa/trace-58.csv", trace).unwrap();
    println!(
        "images : {} ; F6 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script du niveau d'essai « chat » pour les captures de la carte 5.9 :
/// `target/qa/script-59.txt`, `target/qa/touches-59.txt` (l'image où appuyer
/// sur F7, une image après la trace : le navigateur a un pas de retard) et
/// `target/qa/trace-59.csv` (position de Pépin, caméra, sol, phase du chat,
/// pas restants dans la phase, chat écrasé, écrasement).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_chat
#[test]
#[ignore]
fn ecrire_script_essai_chat() {
    let partie = jouer_essai_chat(&parcours_essai_chat());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-59.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-59.txt", format!("{pilote}:F7\n")).unwrap();
    let mut trace = String::from(
        "image,ecran,essai,x,y,cam_x,cam_y,au_sol,gouttes,chat_phase,chat_reste,\
             chat_ecrase,ecrasement\n",
    );
    for (k, i) in partie.images.iter().enumerate() {
        let (phase, reste, ecrase) = match i.chat {
            Some((PhaseChat::Dort(n), e)) => ("dort", n, e),
            Some((PhaseChat::Reveil(n), e)) => ("reveil", n, e),
            Some((PhaseChat::Eveille(n), e)) => ("eveille", n, e),
            None => ("-", 0, false),
        };
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{},{phase},{reste},{},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.au_sol),
            i.trame.gouttes,
            u8::from(ecrase),
            u8::from(i.ecrase_chat)
        ));
    }
    std::fs::write("../target/qa/trace-59.csv", trace).unwrap();
    println!(
        "images : {} ; F7 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Écrit le script du niveau d'essai « horloge » pour le navigateur :
/// `target/qa/script-510.txt` (un chiffre hexadécimal par image),
/// `target/qa/touches-510.txt` (F8 à l'image où le robot l'appuie) et
/// `target/qa/trace-510.csv` (position de Pépin, des deux aiguilles, et
/// l'aiguille qui le porte), pour choisir les images à photographier.
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_essai_horloge
#[test]
#[ignore]
fn ecrire_script_essai_horloge() {
    let partie = jouer_essai_horloge(&parcours_essai_horloge());
    assert!(partie.fin_atteinte && partie.morts == 0);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-510.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-510.txt", format!("{pilote}:F8\n")).unwrap();
    let mut trace = String::from(
        "image,ecran,essai,x,y,cam_x,cam_y,au_sol,gouttes,a0_x,a0_y,a1_x,a1_y,porte\n",
    );
    for (k, i) in partie.images.iter().enumerate() {
        let porte = i.sur_aiguille.map_or(-1, |a| a as i32);
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{},{:.1},{:.1},{:.1},{:.1},{porte}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.au_sol),
            i.trame.gouttes,
            i.aiguilles[0].0,
            i.aiguilles[0].1,
            i.aiguilles[1].0,
            i.aiguilles[1].1,
        ));
    }
    std::fs::write("../target/qa/trace-510.csv", trace).unwrap();
    println!(
        "images : {} ; F8 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}

/// Arène du Grand Nuage (carte 5.11) : écrit `target/qa/script-511.txt` (touches
/// du robot, une par image), `target/qa/touches-511.txt` (F10 à l'image du
/// pilote) et `target/qa/trace-511.csv` (position de Pépin, phase, temps et
/// points de vie du nuage, coups), pour rejouer la partie dans un navigateur et
/// viser les captures. Lancer avec : cargo test -p game --no-default-features --
/// --ignored ecrire_script_essai_nuage
#[test]
#[ignore]
fn ecrire_script_essai_nuage() {
    let partie = jouer_essai_nuage(&parcours_essai_nuage());
    assert!(partie.fin_atteinte && partie.morts == 0 && partie.gouttes == 3);
    std::fs::create_dir_all("../target/qa").unwrap();
    let script: String = partie.images.iter().map(|i| i.trame.code()).collect();
    std::fs::write("../target/qa/script-511.txt", &script).unwrap();
    let touche = partie.images.iter().position(|i| i.touche_essai).unwrap();
    let pilote = touche + 1;
    std::fs::write("../target/qa/touches-511.txt", format!("{pilote}:F10\n")).unwrap();
    let mut trace =
        String::from("image,ecran,essai,x,y,cam_x,cam_y,au_sol,gouttes,phase,ecoule,pv,coup\n");
    for (k, i) in partie.images.iter().enumerate() {
        let (phase, ecoule, pv) = i.nuage.map_or(("-".to_string(), 0, 0), |(p, e, v)| {
            (format!("{p:?}"), e, v)
        });
        trace.push_str(&format!(
            "{k},{},{},{:.1},{:.1},{},{},{},{},{phase},{ecoule},{pv},{}\n",
            i.trame.ecran,
            u8::from(i.en_essai),
            i.trame.x,
            i.trame.y,
            i.camera.0,
            i.camera.1,
            u8::from(i.au_sol),
            i.trame.gouttes,
            u8::from(i.coup_nuage),
        ));
    }
    std::fs::write("../target/qa/trace-511.csv", trace).unwrap();
    println!(
        "images : {} ; F10 à l'image {touche} (pilote : {pilote}) ; gouttes : {}",
        partie.images.len(),
        partie.gouttes
    );
}
