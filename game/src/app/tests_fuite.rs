//! Tests de la fuite de la Pieuvre au sommet (carte 9.36c) : la caméra descend
//! en douceur vers la mer, la séquence finit toujours sur le bilan, un appui la
//! passe (après le délai des écrans de texte), et le chrono enregistré est le
//! même, au pas près, avec ou sans la séquence.
//!
//! Le niveau de la Pieuvre est lancé SEUL depuis la carte des niveaux ; la phase 1
//! est sautée (comme le fait le robot : `sortir_la_tete` puis `toucher`), la furie
//! passe, et Pépin est posé au sommet au tout début de la montée : la mer est
//! alors tout en bas de la tour, le cas où la caméra descend le plus loin.

use pepin_core::entities::pieuvre::{PhasePieuvre, Pieuvre, FURIE_PAS};
use pepin_core::entities::Entite;
use pepin_core::player::{Entrees, Joueur};
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use crate::bot_qa::pilote_pieuvre::indice_pieuvre;
use crate::ecrans::histoire;
use crate::entrees::Commandes;
use crate::navigation::Navigation;

use super::fuite_au_sommet::SEQUENCE_MAX_PAS;
use super::*;

/// La caméra ne bouge jamais de plus de ce nombre de pixels d'une image à la
/// suivante pendant la séquence (descente comprise). Le pire cas est celui de ces
/// tests : la mer tout en bas de la tour, 1 100 px sous Pépin, descendus en
/// `DESCENTE_MAX_PAS` (1,5 s) : 18,3 px au plus fort. En jeu, la mer suit Pépin
/// de plus près : la caméra va moins vite (`VITESSE_DESCENTE`, 16 px).
const SAUT_MAX_PX: f32 = 20.0;

fn valider() -> Navigation {
    Navigation {
        valider: true,
        ..Navigation::default()
    }
}

fn pieuvre_mut(app: &mut Application) -> &mut Pieuvre {
    app.monde
        .entites
        .iter_mut()
        .find_map(|e| match e {
            Entite::Pieuvre(p) => Some(&mut **p),
            _ => None,
        })
        .expect("le niveau a sa pieuvre")
}

/// Le niveau de la Pieuvre joué seul, la phase 1 sautée, la furie passée ; Pépin
/// est posé dans le sommet, la caméra sur lui : le PROCHAIN pas, elle s'enfuit.
fn au_sommet(facile: bool) -> Application {
    let mut app = Application::new(true);
    app.lancer(Suite::NiveauSeul(indice_pieuvre()));
    for _ in 0..600 {
        if app.ecran == Ecran::Jeu && !app.transition.est_active() {
            break;
        }
        if app.texte_en_attente() {
            app.reagir(&valider(), &Commandes::default(), None);
        }
        app.pas(&Entrees::default());
    }
    assert_eq!(app.ecran, Ecran::Jeu);
    if facile {
        app.monde.passer_nuage_en_facile();
    }
    let p = pieuvre_mut(&mut app);
    for _ in 0..p.pv_max() {
        p.sortir_la_tete(0);
        p.toucher();
    }
    assert_eq!(p.phase(), PhasePieuvre::Furie);
    let sommet = p.sommet.expect("la tour a son sommet");
    // La furie (sans danger hors de l'eau) : Pépin attend à sa place.
    for _ in 0..=FURIE_PAS {
        app.pas(&Entrees::default());
    }
    assert_eq!(pieuvre_mut(&mut app).phase(), PhasePieuvre::Montee);
    let mut j = Joueur::new(sommet.x + 8.0, 0.0);
    j.boite.y = sommet.bas() - j.boite.h;
    app.monde.joueur = j;
    let (l, h) = (app.monde.niveau.largeur_px(), app.monde.niveau.hauteur_px());
    app.camera.recentrer(&app.monde.joueur, l, h);
    assert!(app.fuite_en_cours().is_none());
    app
}

/// Joue jusqu'au bilan sans appui ; renvoie les positions de la caméra à chaque
/// pas de la séquence, et le nombre de pas de jeu (chrono) qu'elle a comptés.
fn jusqu_au_bilan(app: &mut Application) -> (Vec<(f32, f32)>, u32) {
    let chrono = app.chrono();
    let mut cameras = vec![(app.camera.x, app.camera.y)];
    for _ in 0..SEQUENCE_MAX_PAS + 60 {
        app.reagir(&Navigation::default(), &Commandes::default(), None);
        app.pas(&Entrees::default());
        if app.ecran != Ecran::Jeu {
            break;
        }
        cameras.push((app.camera.x, app.camera.y));
    }
    (cameras, app.chrono() - chrono)
}

#[test]
fn la_camera_descend_en_douceur_vers_la_mer_puis_le_bilan() {
    for facile in [false, true] {
        let mut app = au_sommet(facile);
        let depart = (app.camera.x, app.camera.y);
        let pepin = app.monde.joueur.boite;
        // Le pas du sommet : elle s'enfuit, la séquence commence.
        app.pas(&Entrees::default());
        let f = app
            .fuite_en_cours()
            .expect("la séquence commence au sommet");
        assert_eq!(f.pas, 0);
        assert!(f.en_descente());
        let pepin = (pepin.x, app.monde.joueur.boite.y);
        let pas_monde = app.monde.pas;
        // La descente : le monde ne bouge pas (ni Pépin, ni ses pas).
        for _ in 0..f.descente {
            app.pas(&Entrees {
                saut: true,
                ..Entrees::default()
            });
        }
        assert_eq!(
            app.monde.pas, pas_monde,
            "le monde est figé pendant la descente"
        );
        assert_eq!((app.monde.joueur.boite.x, app.monde.joueur.boite.y), pepin);
        // Arrivée : la tête de la Pieuvre et la mer sont à l'écran.
        let p = pieuvre_mut(&mut app);
        let (tete, mer) = (p.au_loin().unwrap(), p.surface_mer().unwrap());
        let (cx, cy) = (app.camera.x, app.camera.y);
        assert!(tete.x >= cx && tete.droite() <= cx + LARGEUR_LOGIQUE as f32);
        assert!(mer > cy + 32.0 && mer < cy + HAUTEUR_LOGIQUE as f32 - 32.0);
        let descente = cy - depart.1;
        println!("facile {facile} : la caméra est descendue de {descente:.0} px");
        assert!(
            descente > HAUTEUR_LOGIQUE as f32,
            "la mer était hors de l'écran"
        );
        // Puis la plongée, jusqu'au bilan.
        jusqu_au_bilan(&mut app);
        assert_eq!(
            app.ecran,
            Ecran::BilanNiveau,
            "la séquence finit sur le bilan"
        );
        assert!(app.fuite_en_cours().is_none());
        // Aucun saut de caméra, sur TOUTE la séquence (descente comprise).
        let mut app2 = au_sommet(facile);
        let (toutes, _) = jusqu_au_bilan(&mut app2);
        assert_eq!(app2.ecran, Ecran::BilanNiveau);
        let pire = toutes
            .windows(2)
            .map(|w| (w[1].0 - w[0].0).abs().max((w[1].1 - w[0].1).abs()))
            .fold(0.0, f32::max);
        println!(
            "facile {facile} : {} pas de séquence, pire saut {pire:.2} px",
            toutes.len()
        );
        assert!(pire <= SAUT_MAX_PX, "saut de {pire} px");
        // Toute la séquence : 2 à 3,5 s (le pas du sommet compris).
        let duree = toutes.len();
        assert!(
            (120..=SEQUENCE_MAX_PAS as usize + 2).contains(&duree),
            "{duree} pas"
        );
    }
}

#[test]
fn un_appui_passe_la_fuite_apres_le_delai() {
    let mut app = au_sommet(false);
    app.pas(&Entrees::default());
    assert!(app.fuite_en_cours().is_some());
    // Avant le délai des écrans de texte : ignoré.
    for _ in 0..histoire::DELAI_PAS {
        app.reagir(&valider(), &Commandes::default(), None);
        app.pas(&Entrees::default());
        assert_eq!(app.ecran, Ecran::Jeu);
    }
    assert!(
        app.fuite_en_cours().is_some(),
        "un appui trop tôt ne la passe pas"
    );
    // Après : passée, le bilan au pas suivant.
    app.reagir(&valider(), &Commandes::default(), None);
    app.pas(&Entrees::default());
    assert_eq!(app.ecran, Ecran::BilanNiveau);
    assert!(app.fuite_en_cours().is_none());
}

#[test]
fn le_chrono_enregistre_est_le_meme_avec_ou_sans_sequence() {
    // SANS séquence : le monde seul, comme avant la carte, jusqu'à la fin.
    let app = au_sommet(false);
    let mut monde = app.monde.clone();
    let mut sans = 0;
    while !monde.termine {
        monde.avancer(&Entrees::default());
        sans += 1;
        assert!(sans < 1000);
    }
    // AVEC la séquence, attendue jusqu'au bout.
    let mut app = au_sommet(false);
    let (_, avec) = jusqu_au_bilan(&mut app);
    let temps_avec = app.temps_niveau.pas;
    // AVEC la séquence, passée dès que possible.
    let mut app = au_sommet(false);
    let chrono = app.chrono();
    for _ in 0..=histoire::DELAI_PAS {
        app.pas(&Entrees::default());
    }
    app.reagir(&valider(), &Commandes::default(), None);
    app.pas(&Entrees::default());
    assert_eq!(app.ecran, Ecran::BilanNiveau);
    let passee = app.chrono() - chrono;
    println!("chrono depuis le sommet : sans {sans}, avec {avec}, passée {passee}");
    assert_eq!(avec, sans);
    assert_eq!(passee, sans);
    assert_eq!(
        app.temps_niveau.pas, temps_avec,
        "le temps enregistré du niveau"
    );
}
