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

/// Écrit le script de la partie « vies » (carte 6.2) pour le navigateur, à travers
/// la vraie `Application` (titre, cartes, transitions) :
///
/// 1. niveau 1 : le parcours COMPLET du robot (ses trois gouttes : une vie de plus,
///    six vies) ;
/// 2. niveau 2 : un joueur têtu qui court tout droit (et saute quand il bute) jusqu'à
///    perdre ses six vies : écran « PLUS DE VIES », puis le niveau 2 recommence avec
///    cinq vies ;
/// 3. la suite : le parcours du robot (le niveau 2 depuis son début, puis les autres,
///    bonus sauté, épilogue), jusqu'à l'écran de fin.
///
/// Fichiers : `target/qa/script-62.txt` (un chiffre hexadécimal par image, comme
/// `script.txt`) et `target/qa/trace-62.csv` (écran, niveau, vies, morts, gouttes,
/// éclat du « +1 », position de Pépin), d'où viennent les numéros d'image des
/// captures `62-` de QA.md. Lancer avec : cargo test -p game --no-default-features
/// -- --ignored ecrire_script_vies
#[test]
#[ignore]
fn ecrire_script_vies() {
    use crate::app::Application;
    use crate::bot_qa::{nom_ecran, tous_les_parcours, tous_les_parcours_complets, Trame};
    use crate::ecrans::Ecran;
    use crate::entrees::Commandes;
    use crate::navigation::Navigation;
    use pepin_core::player::{Direction, Entrees};
    use pepin_core::vies::VIES_DEPART;

    let complets = tous_les_parcours_complets();
    let normaux = tous_les_parcours();
    let mut app = Application::new(true);
    let mut script = String::new();
    let mut trace = String::from("image,ecran,niveau,vies,morts,gouttes,gouttes_ici,eclat,x,y\n");
    // Le joueur têtu du niveau 2 : jusqu'au « PLUS DE VIES ».
    let mut plus_de_vies_vu = false;
    let (mut immobile, mut saut, mut dernier_x) = (0u32, 0u32, f32::NAN);
    let mut fin = false;
    for image in 0..60 * 60 * 20 {
        let mut trame = Trame {
            valider: image == 0,
            ..Trame::default()
        };
        if app.ecran == Ecran::Jeu && app.en_jeu() {
            let n = app.monde.infos.numero as usize;
            let robot = |p: &Vec<Vec<Entrees>>| {
                p.get(n - 1)
                    .and_then(|l| l.get(app.monde.pas as usize))
                    .copied()
            };
            trame.entrees = if n == 1 {
                robot(&complets).unwrap_or_default()
            } else if n == 2 && !plus_de_vies_vu {
                // Court à droite ; s'il ne bouge plus (mur), saute 20 images.
                let j = &app.monde.joueur;
                if (j.boite.x - dernier_x).abs() < 0.01 {
                    immobile += 1;
                } else {
                    immobile = 0;
                }
                dernier_x = j.boite.x;
                if immobile > 10 && saut == 0 {
                    saut = 20;
                }
                let sauter = saut > 0;
                saut = saut.saturating_sub(1);
                Entrees {
                    direction: Direction::Droite,
                    saut: sauter,
                }
            } else {
                robot(&normaux).unwrap_or_default()
            };
        }
        let nav = Navigation {
            valider: trame.valider,
            ..Navigation::default()
        };
        app.reagir(&nav, &Commandes::default(), None);
        app.pas(&trame.entrees);
        if matches!(app.ecran, Ecran::PlusDeVies(_)) {
            plus_de_vies_vu = true;
        }
        script.push(trame.code());
        let e = &app.monde.etat;
        trace.push_str(&format!(
            "{image},{},{},{},{},{},{},{},{:.1},{:.1}\n",
            nom_ecran(app.ecran),
            if app.ecran == Ecran::Jeu {
                app.monde.infos.numero
            } else {
                0
            },
            e.vies.restantes(),
            e.morts,
            e.gouttes,
            e.gouttes_ici(),
            app.eclat_vie(),
            app.monde.joueur.boite.x,
            app.monde.joueur.boite.y,
        ));
        if app.ecran == Ecran::Fin {
            // Quelques images de l'écran de fin, pour la capture.
            for k in 1..=240 {
                app.reagir(&Navigation::default(), &Commandes::default(), None);
                app.pas(&Entrees::default());
                script.push('0');
                let e = &app.monde.etat;
                trace.push_str(&format!(
                    "{},fin,0,{},{},{},0,0,0,0\n",
                    image + k,
                    e.vies.restantes(),
                    e.morts,
                    e.gouttes
                ));
            }
            fin = true;
            break;
        }
        assert!(
            image < 60 * 60 * 2 || plus_de_vies_vu,
            "pas de « PLUS DE VIES » au bout de deux minutes"
        );
    }
    assert!(fin, "l'écran de fin n'a pas été atteint");
    assert!(plus_de_vies_vu);
    let morts = app.monde.etat.morts;
    assert_eq!(
        morts,
        u32::from(VIES_DEPART) + 1,
        "six vies perdues au niveau 2, aucune ensuite"
    );
    std::fs::create_dir_all("../target/qa").unwrap();
    std::fs::write("../target/qa/script-62.txt", &script).unwrap();
    std::fs::write("../target/qa/trace-62.csv", &trace).unwrap();
    println!(
        "images : {} ; morts : {morts} ; vies à la fin : {} ; gouttes : {}",
        script.len(),
        app.monde.etat.vies.restantes(),
        app.monde.etat.gouttes
    );
}

/// Écrit le script des captures de la carte 6.3 (chats plus menaçants), au marché
/// couvert (niveau 3, touche 3 du build de QA) : « JOUER », la carte du niveau 1,
/// la touche 3 dès la première image de jeu, puis un joueur scripté face au
/// chat de la colonne 12 (le premier du toit de départ, centre en x = 200) :
/// 1. il attend au départ (x = 19, à 11 tuiles du chat) tout un cycle : le chat
///    s'annonce (« ! », couché, un œil ouvert) et se réveille SUR PLACE, Pépin est
///    hors de portée ;
/// 2. le chat rendormi, Pépin s'approche à moins de 5 tuiles (x ≥ 118) et
///    s'arrête : à l'annonce suivante, le chat se ramasse, prêt à bondir ;
/// 3. le bond part vers l'endroit où se tient Pépin : Pépin s'enfuit vers la
///    gauche (40 images) ; le chat se pose là où il était, puis revient à sa
///    place d'un second bond et se rendort ;
/// 4. une seconde d'attente, et le script s'arrête (pas de suite à jouer).
///
/// Fichiers : `target/qa/script-63.txt` (un chiffre hexadécimal par image),
/// `target/qa/touches-63.txt` (l'image du PILOTE où appuyer sur la touche 3, une
/// image après la trace) et `target/qa/trace-63.csv` (pas du monde, Pépin, le
/// chat : position, phase, pas restants, prêt, bond, en l'air), d'où viennent les
/// numéros d'image des captures `63-` de QA.md.
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_chats
#[test]
#[ignore]
fn ecrire_script_chats() {
    use crate::app::Application;
    use crate::bot_qa::{nom_ecran, Trame};
    use crate::entrees::{Commandes, RaccourciNiveau};
    use crate::navigation::Navigation;
    use pepin_core::entities::{Entite, TypeEnnemi};
    use pepin_core::player::{Direction, Entrees};

    /// Ce que fait le joueur scripté.
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Etape {
        /// Au départ, loin du chat, tout un cycle (jusqu'à son 2e sommeil).
        AttendreLoin,
        /// Vers la droite, jusqu'à x = 118 (à portée).
        Approcher,
        /// Immobile à portée, jusqu'au départ du bond.
        AttendreLeBond,
        /// Fuite vers la gauche pendant `n` images.
        Fuir(u32),
        /// Immobile, jusqu'au retour du chat (rendormi) puis `n` images.
        Regarder(u32),
        Fin,
    }

    let mut app = Application::new(true);
    // Le chat suivi : celui de la colonne 12 (toit de départ). Depuis la carte
    // 6.4a, le chat du mur (rangée 4) le précède dans l'ordre de lecture : on ne
    // prend donc plus « le premier chat du niveau ».
    let chat_suivi = |entites: &[Entite]| {
        entites.iter().find_map(|e| match e {
            Entite::Ennemi(c)
                if c.type_ennemi == TypeEnnemi::Chat && (c.place().0 / 16.0) as i32 == 12 =>
            {
                Some(c.clone())
            }
            _ => None,
        })
    };
    let mut script = String::new();
    let mut trace = String::from(
        "image,ecran,indice,pas,x,y,chat_x,chat_y,phase,reste,pret,bond,en_vol,morts,etape\n",
    );
    let mut touche: Option<usize> = None;
    let mut etape = Etape::AttendreLoin;
    let mut eveils = 0;
    let mut etait_eveille = false;
    for image in 0..60 * 60 {
        let mut trame = Trame {
            valider: image == 0,
            ..Trame::default()
        };
        let raccourci = if touche.is_none() && app.en_jeu() && app.monde.indice == 0 {
            touche = Some(image);
            Some(RaccourciNiveau::Aller(2))
        } else {
            None
        };
        let au_marche = app.en_jeu() && app.monde.indice == 2;
        let chat = chat_suivi(&app.monde.entites);
        if au_marche {
            let c = chat.as_ref().expect("un chat au marché couvert");
            let x = app.monde.joueur.boite.x;
            let eveille = c.est_eveille();
            if etait_eveille && !eveille {
                eveils += 1;
            }
            etait_eveille = eveille;
            let mut direction = Direction::Immobile;
            etape = match etape {
                Etape::AttendreLoin if eveils >= 1 => Etape::Approcher,
                Etape::Approcher if x >= 118.0 => Etape::AttendreLeBond,
                Etape::AttendreLeBond if c.bond().is_some() => Etape::Fuir(40),
                Etape::Fuir(0) => Etape::Regarder(60),
                Etape::Regarder(0) => Etape::Fin,
                autre => autre,
            };
            match etape {
                Etape::Approcher => direction = Direction::Droite,
                Etape::Fuir(n) => {
                    direction = Direction::Gauche;
                    etape = Etape::Fuir(n - 1);
                }
                Etape::Regarder(n) if !eveille => etape = Etape::Regarder(n - 1),
                _ => {}
            }
            trame.entrees = Entrees {
                direction,
                saut: false,
            };
        }
        if etape == Etape::Fin {
            break;
        }
        let nav = Navigation {
            valider: trame.valider,
            ..Navigation::default()
        };
        app.reagir(&nav, &Commandes::default(), raccourci);
        app.pas(&trame.entrees);
        script.push(trame.code());
        let chat = chat_suivi(&app.monde.entites);
        let (cx, cy, phase, reste, pret, bond, en_vol) = match &chat {
            Some(c) => {
                let (phase, reste) = match c.phase_chat() {
                    PhaseChat::Dort(n) => ("dort", n),
                    PhaseChat::Reveil(n) => ("reveil", n),
                    PhaseChat::Eveille(n) => ("eveille", n),
                };
                (
                    c.boite.x,
                    c.boite.y,
                    phase,
                    reste,
                    c.est_pret_a_bondir(),
                    c.bond().unwrap_or(0.0),
                    c.en_plein_bond(),
                )
            }
            None => (0.0, 0.0, "-", 0, false, 0.0, false),
        };
        trace.push_str(&format!(
            "{image},{},{},{},{:.1},{:.1},{cx:.1},{cy:.1},{phase},{reste},{},{bond:.0},{},{},{etape:?}\n",
            nom_ecran(app.ecran),
            app.monde.indice,
            app.monde.pas,
            app.monde.joueur.boite.x,
            app.monde.joueur.boite.y,
            u8::from(pret),
            u8::from(en_vol),
            app.monde.etat.morts,
        ));
    }
    assert_eq!(etape, Etape::Fin, "le scénario n'est pas allé au bout");
    assert_eq!(app.monde.etat.morts, 0, "Pépin doit échapper au bond");
    std::fs::create_dir_all("../target/qa").unwrap();
    std::fs::write("../target/qa/script-63.txt", &script).unwrap();
    let pilote = touche.expect("touche 3 appuyée") + 1;
    std::fs::write("../target/qa/touches-63.txt", format!("{pilote}:Digit3\n")).unwrap();
    std::fs::write("../target/qa/trace-63.csv", &trace).unwrap();
    println!(
        "images : {} ; touche 3 à l'image {} (pilote : {pilote})",
        script.len(),
        pilote - 1
    );
}

/// Ennemis ajoutés à un niveau par une carte : type, colonne et rangée de la
/// lettre dans la grille (c'est ainsi qu'on les retrouve dans le monde : la case
/// de leur point de départ, `Ennemi::perchoir`).
type Ajouts = &'static [(pepin_core::entities::TypeEnnemi, i32, i32)];

/// Écrit les scripts des captures de la carte 6.4a (plus d'ennemis aux niveaux 1
/// à 5) : pour chaque niveau N de 1 à 5, « JOUER », la carte du niveau 1, la
/// touche N dès la première image de jeu (build de QA ; aucune au niveau 1), puis
/// le parcours du robot dans le niveau N (`trouver_parcours`, le même que celui de
/// `script.txt`), joué jusqu'au drapeau.
///
/// Un script par niveau plutôt que la partie entière : sur une longue partie, le
/// rejeu du navigateur peut glisser d'un pas et rater un écrasement réglé à
/// l'image près (voir QA.md, carte 6.4a). Fichiers, dans `target/qa/` :
/// `script-64-N.txt`, `touches-64-N.txt` (image du PILOTE où appuyer sur la
/// touche N, une image après la trace) et `trace-64-N.csv` : Pépin, la caméra
/// rendue, et pour chaque ennemi AJOUTÉ par la carte (repéré par sa case de
/// départ) sa position, son état, la phase du chat et s'il est à l'écran ; les
/// numéros d'image des captures `64-` de QA.md en viennent.
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_ennemis
#[test]
#[ignore]
fn ecrire_script_ennemis() {
    use pepin_core::entities::TypeEnnemi;
    ecrire_scripts_des_ajouts(
        "64",
        &[
            (
                0,
                &[(TypeEnnemi::Flaque, 76, 7), (TypeEnnemi::Flaque, 118, 7)],
            ),
            (
                1,
                &[(TypeEnnemi::Corbeau, 50, 5), (TypeEnnemi::Corbeau, 106, 5)],
            ),
            (2, &[(TypeEnnemi::Chat, 68, 4)]),
            (3, &[(TypeEnnemi::Corbeau, 36, 5)]),
            (
                4,
                &[(TypeEnnemi::Corbeau, 26, 18), (TypeEnnemi::Chat, 28, 36)],
            ),
        ],
    );
}

/// Même chose pour la carte 6.4b (plus d'ennemis aux niveaux 7, 8, 9 et 11) :
/// fichiers `script-64b-N.txt`, `touches-64b-N.txt` et `trace-64b-N.csv`. Les
/// touches 1 à 9 du build de QA ne mènent qu'aux neuf premiers niveaux : pour la
/// descente (le 11e), touche 9 puis deux fois N (niveau suivant, bonus compris).
/// La trace donne aussi la phase de vol des pigeons (plongée, remontée…).
/// Lancer avec : cargo test -p game --no-default-features -- --ignored ecrire_script_ennemis_64b
#[test]
#[ignore]
fn ecrire_script_ennemis_64b() {
    use pepin_core::entities::TypeEnnemi;
    ecrire_scripts_des_ajouts(
        "64b",
        &[
            (
                6,
                &[
                    (TypeEnnemi::Pigeon, 38, 3),
                    (TypeEnnemi::Corbeau, 76, 3),
                    (TypeEnnemi::Corbeau, 147, 3),
                ],
            ),
            (
                7,
                &[
                    (TypeEnnemi::Pigeon, 65, 3),
                    (TypeEnnemi::Corbeau, 85, 4),
                    (TypeEnnemi::Corbeau, 97, 7),
                    (TypeEnnemi::Corbeau, 162, 6),
                ],
            ),
            (
                8,
                &[
                    (TypeEnnemi::Flaque, 37, 5),
                    (TypeEnnemi::Corbeau, 63, 3),
                    (TypeEnnemi::Corbeau, 120, 2),
                    (TypeEnnemi::Flaque, 124, 3),
                ],
            ),
            (10, &[(TypeEnnemi::Corbeau, 10, 12)]),
        ],
    );
}

/// Le travail commun des deux tests précédents : pour chaque `(indice, ajouts)`,
/// le script du parcours du robot dans le niveau d'indice `indice`, ses touches de
/// raccourci et sa trace, dans `target/qa/` avec le préfixe `prefixe`.
fn ecrire_scripts_des_ajouts(prefixe: &str, niveaux: &[(usize, Ajouts)]) {
    use crate::app::Application;
    use crate::bot_qa::{nom_ecran, trouver_parcours, Trame};
    use crate::entrees::{Commandes, RaccourciNiveau};
    use crate::navigation::Navigation;
    use pepin_core::aabb::TAILLE_TUILE;
    use pepin_core::entities::{Entite, TypeEnnemi};

    /// Plus haut indice qu'une touche chiffre atteint (touche 9 : indice 8).
    const DERNIER_CHIFFRE: usize = 8;

    std::fs::create_dir_all("../target/qa").unwrap();
    for &(cible, ajouts) in niveaux {
        let parcours = trouver_parcours(cible, false)
            .or_else(|| trouver_parcours(cible, true))
            .expect("parcours du robot");
        let mut app = Application::new(true);
        let mut script = String::new();
        let mut trace = String::from("image,ecran,indice,pas,x,y,cam_x,cam_y,morts");
        for (k, (t, c, r)) in ajouts.iter().enumerate() {
            trace.push_str(&format!(
                ",e{k}_{t:?}_{c}_{r}_x,e{k}_y,e{k}_etat,e{k}_phase,e{k}_ecran"
            ));
        }
        trace.push('\n');
        // Touches de raccourci appuyées : (image de la trace, code de la touche).
        let mut touches: Vec<(usize, String)> = Vec::new();
        let mut commence = false;
        for image in 0..60 * 60 {
            let mut trame = Trame {
                valider: image == 0,
                ..Trame::default()
            };
            // En jeu, mais pas encore dans le niveau voulu : une touche. Chiffre
            // d'abord (le niveau voulu, ou le 9e s'il est plus loin), puis « niveau
            // suivant » (N) autant qu'il faut, une fois chaque niveau commencé.
            let raccourci = if !commence && app.en_jeu() && app.monde.indice != cible {
                if touches.is_empty() {
                    let i = cible.min(DERNIER_CHIFFRE);
                    touches.push((image, format!("Digit{}", i + 1)));
                    Some(RaccourciNiveau::Aller(i))
                } else {
                    touches.push((image, "KeyN".to_string()));
                    Some(RaccourciNiveau::Suivant)
                }
            } else {
                None
            };
            let dans_la_cible = app.en_jeu() && app.monde.indice == cible && raccourci.is_none();
            if dans_la_cible {
                commence = true;
                if let Some(e) = parcours.get(app.monde.pas as usize) {
                    trame.entrees = *e;
                }
            } else if commence {
                break; // le drapeau est atteint : la transition commence
            }
            let nav = Navigation {
                valider: trame.valider,
                ..Navigation::default()
            };
            app.reagir(&nav, &Commandes::default(), raccourci);
            app.pas(&trame.entrees);
            script.push(trame.code());
            let j = app.monde.joueur.boite;
            let cam = app.camera.position_rendue(1.0, (j.x, j.y));
            trace.push_str(&format!(
                "{image},{},{},{},{:.1},{:.1},{},{},{}",
                nom_ecran(app.ecran),
                app.monde.indice,
                app.monde.pas,
                j.x,
                j.y,
                cam.0,
                cam.1,
                app.monde.etat.morts
            ));
            for &(t, c, r) in ajouts.iter() {
                let ennemi = app.monde.entites.iter().find_map(|e| match e {
                    Entite::Ennemi(en)
                        if app.monde.indice == cible
                            && en.type_ennemi == t
                            && (en.perchoir().0 / TAILLE_TUILE) as i32 == c
                            && (en.perchoir().1 / TAILLE_TUILE) as i32 == r =>
                    {
                        Some(en)
                    }
                    _ => None,
                });
                match ennemi {
                    Some(en) => {
                        let b = en.boite;
                        let a_l_ecran = b.droite() > cam.0 as f32
                            && b.x < cam.0 as f32 + 320.0
                            && b.bas() > cam.1 as f32
                            && b.y < cam.1 as f32 + 180.0;
                        let phase = match t {
                            TypeEnnemi::Chat => format!("{:?}", en.phase_chat()),
                            TypeEnnemi::Pigeon => format!("{:?}", en.phase()),
                            _ => "-".to_string(),
                        };
                        trace.push_str(&format!(
                            ",{:.1},{:.1},{:?},{phase},{}",
                            b.x,
                            b.y,
                            en.etat,
                            u8::from(a_l_ecran)
                        ));
                    }
                    None => trace.push_str(",-,-,-,-,0"),
                }
            }
            trace.push('\n');
        }
        assert!(commence, "niveau {} jamais atteint", cible + 1);
        assert_eq!(app.monde.etat.morts, 0, "le robot ne meurt pas");
        let n = cible + 1;
        std::fs::write(format!("../target/qa/script-{prefixe}-{n}.txt"), &script).unwrap();
        let touches: String = touches
            .iter()
            .map(|(t, code)| format!("{}:{code}\n", t + 1))
            .collect();
        std::fs::write(format!("../target/qa/touches-{prefixe}-{n}.txt"), &touches).unwrap();
        std::fs::write(format!("../target/qa/trace-{prefixe}-{n}.csv"), &trace).unwrap();
        println!(
            "niveau {n} : {} images ; touches : {}",
            script.len(),
            touches.trim().replace('\n', ", ")
        );
    }
}
