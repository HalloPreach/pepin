//! Point d'entrée macroquad : boucle principale à pas fixe 60 Hz (le `PasFixe` du
//! cœur), rendu interpolé découplé, dans une cible 320x180 agrandie d'un
//! facteur entier. Tout le reste (écrans, niveaux, pause...) vit dans `app`.
//!
//! Commandes : flèches ou Q/D (ou A/D) pour courir ; Espace, Haut, W ou Z pour sauter
//! (maintenu en l'air : parapluie ouvert) ; Échap ou P pour la pause ; M active/coupe
//! le son (coupé par défaut). La pause s'ouvre aussi toute seule si la fenêtre perd le
//! focus en pleine partie (onglet masqué, autre application : voir `focus.rs`).
//! Les touches sont lues par POSITION physique : les listes
//! de `entrees.rs` couvrent QWERTY et AZERTY (sur AZERTY, M est à la position du
//! point-virgule). Dans les menus : flèches + Entrée, ou clic / toucher. Sur écran
//! tactile : boutons virtuels. Touches 1 à 9, Page suivante / N et Page précédente / B
//! (changer de niveau) et F9 (plantage volontaire) : uniquement dans le build de mise
//! au point (`./build.sh --qa`, fonctionnalités `debug_niveaux` et `qa_plantage`).

mod app;
mod audio;
#[cfg(test)]
mod bot_qa;
mod camera;
mod chrono;
mod contenu;
mod decor;
#[cfg(test)]
mod difficulte;
mod ecrans;
mod effets;
mod entrees;
mod focus;
mod hud;
mod menu;
mod monde;
mod navigation;
mod orientation;
mod particules;
mod rendu;
mod scene;
mod secousse;
mod sprites;
mod tactile;
mod texte;
mod transition;

use macroquad::prelude::*;
use pepin_core::sons::Effet;
use pepin_core::timestep::PasFixe;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use app::Application;
use audio::Audio;
use ecrans::Ecran;
use entrees::{Clavier, Sources};
use focus::Focus;
use navigation::LecteurNavigation;
use rendu::Cible;
use sprites::Banque;
use tactile::Tactile;

/// Configuration de la fenêtre (échelle x4 de la résolution logique).
///
/// `high_dpi` : le canevas a la résolution PHYSIQUE de l'écran (et non celle en pixels
/// CSS). Sans cela, le facteur entier d'agrandissement était calculé en pixels CSS : sur
/// un téléphone en paysage (844x340 CSS, DPR 3) le jeu restait en x1, un tiers de
/// l'écran ; et à 125 % / 150 % d'échelle les pixels étaient irréguliers. Le rendu
/// (`rendu.rs`), la souris (`navigation.rs`) et le tactile (`tactile.rs`) convertissent
/// déjà entre pixels physiques et logiques avec `screen_dpi_scale()`.
fn conf() -> Conf {
    Conf {
        window_title: "PÉPIN — un jeu de plateformes".to_owned(),
        window_width: (LARGEUR_LOGIQUE * 4) as i32,
        window_height: (HAUTEUR_LOGIQUE * 4) as i32,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let banque = Banque::charger();
    let cible = Cible::new();
    let mut pas_fixe = PasFixe::new();
    let mut sources = Sources::new();
    sources.ajouter(Box::new(Clavier));
    // Le tactile est cloné : une copie lit les touchers, l'autre dessine les boutons.
    let tactile = Tactile::new();
    sources.ajouter(Box::new(tactile.clone()));
    let mut navigation = LecteurNavigation::new();
    // Perte de focus (onglet masqué, autre application...) : voir `focus.rs`.
    let mut focus = Focus::new();
    // Sons fabriqués en mémoire. Désactivés tant que le joueur ne les demande pas
    // (et un navigateur n'autorise l'audio qu'après une première interaction).
    let mut audio = Audio::charger().await;
    let mut app = Application::new(true);

    loop {
        let commandes = sources.lire();
        let nav = navigation.lire();
        // Focus perdu en pleine partie : la pause s'ouvre AVANT tout pas de simulation.
        // Sur le web, un onglet masqué ne dessine plus : on lit l'événement au retour, et
        // le long « temps d'image » du retour est plafonné par le `PasFixe` (0,25 s, 5 pas
        // au plus, qui tombent alors dans l'écran de pause : rien ne bouge). On remet
        // aussi l'accumulateur à zéro pour repartir d'un temps propre.
        let changements = focus.lire();
        if !changements.is_empty() {
            for present in changements {
                app.focus(present);
            }
            pas_fixe.reinitialiser();
        }
        // Téléphone tactile en portrait : message « Tourne ton téléphone » et jeu en pause.
        let a_tourner =
            orientation::doit_afficher(screen_width(), screen_height(), tactile.est_actif());
        if !a_tourner {
            app.reagir(&nav, &commandes, entrees::raccourci_niveau());
        }
        // Build de QA seulement (`qa_plantage`) : F9 provoque une vraie panique, pour
        // vérifier le message de plantage de la page web. Toujours faux en production.
        if entrees::plantage_qa() {
            panic!("plantage volontaire (build de QA, touche F9)");
        }
        // Le réglage son vient des menus ou de la touche M ; l'audio ne fait que l'appliquer.
        // La musique et la pluie se taisent pendant la pause et reprennent à sa sortie.
        let bascule_son = app.son != audio.est_actif();
        audio.regler(app.son, app.boucles_sonores());
        if bascule_son {
            audio.jouer(Effet::Menu);
        }
        let entrees = commandes.vers_entrees();

        // --- Simulation à pas fixe ---------------------------------------------
        // Le temps s'écoule quand même (sinon un retard s'accumulerait), mais en
        // portrait aucun pas n'est simulé : ni le monde, ni le chrono n'avancent.
        let nb_pas = pas_fixe.avancer(get_frame_time() as f64);
        for _ in 0..if a_tourner { 0 } else { nb_pas } {
            if app.pas(&entrees) {
                audio.apres_pas(&app.monde, app.camera.x);
            }
        }

        // --- Rendu interpolé, dans la cible logique -----------------------------
        cible.commencer();
        if a_tourner {
            orientation::dessiner(get_time());
        } else {
            app.dessiner(&banque, pas_fixe.alpha());
        }
        cible.afficher();
        if !a_tourner && matches!(app.ecran, Ecran::Jeu) {
            tactile.dessiner();
        }
        next_frame().await;
    }
}
