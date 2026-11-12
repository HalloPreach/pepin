//! Point d'entrée macroquad : boucle principale à pas fixe 60 Hz (le `PasFixe` du
//! cœur), rendu interpolé découplé, dans une cible 320x180 agrandie d'un
//! facteur entier. Tout le reste (écrans, niveaux, pause...) vit dans `app`.
//!
//! Commandes : flèches ou Q/D (ou A/D) pour courir ; Espace, Haut, W ou Z pour sauter
//! (maintenu en l'air : parapluie ouvert) ; Échap ou P pour la pause ; M active/coupe
//! le son (coupé par défaut) ; touches 1 à 5 pour changer de niveau. Dans les menus :
//! flèches + Entrée, ou clic / toucher. Sur écran tactile : boutons virtuels.

mod app;
mod audio;
#[cfg(test)]
mod bot_qa;
mod camera;
mod chrono;
mod contenu;
mod decor;
mod ecrans;
mod effets;
mod entrees;
mod hud;
mod menu;
mod monde;
mod navigation;
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
use navigation::LecteurNavigation;
use rendu::Cible;
use sprites::Banque;
use tactile::Tactile;

/// Configuration de la fenêtre (échelle x4 de la résolution logique).
fn conf() -> Conf {
    Conf {
        window_title: "PÉPIN — un jeu de plateformes".to_owned(),
        window_width: (LARGEUR_LOGIQUE * 4) as i32,
        window_height: (HAUTEUR_LOGIQUE * 4) as i32,
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
    // Sons fabriqués en mémoire. Désactivés tant que le joueur ne les demande pas
    // (et un navigateur n'autorise l'audio qu'après une première interaction).
    let mut audio = Audio::charger().await;
    let mut app = Application::new(true);

    loop {
        let commandes = sources.lire();
        let nav = navigation.lire();
        app.reagir(&nav, &commandes, entrees::raccourci_niveau());
        // Le réglage son vient des menus ou de la touche M ; l'audio ne fait que l'appliquer.
        if app.son != audio.est_actif() {
            audio.regler(app.son);
            audio.jouer(Effet::Menu);
        }
        let entrees = commandes.vers_entrees();

        // --- Simulation à pas fixe ---------------------------------------------
        for _ in 0..pas_fixe.avancer(get_frame_time() as f64) {
            if app.pas(&entrees) {
                audio.apres_pas(&app.monde, app.camera.x);
            }
        }

        // --- Rendu interpolé, dans la cible logique -----------------------------
        cible.commencer();
        app.dessiner(&banque, pas_fixe.alpha());
        cible.afficher();
        if matches!(app.ecran, Ecran::Jeu) {
            tactile.dessiner();
        }
        next_frame().await;
    }
}
