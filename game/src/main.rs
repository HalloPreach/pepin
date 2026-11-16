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
//! (changer de niveau), K (skin suivant), G (100 gouttes en poche), F9 (plantage
//! volontaire) et F11 (force la musique suivante) : uniquement dans le build de mise au point
//! (`./build.sh --qa`, fonctionnalités `debug_niveaux` et `qa_plantage`).

mod app;
mod audio;
#[cfg(test)]
mod bot_qa;
mod boutique;
mod camera;
mod chrono;
mod contenu;
mod decor;
mod defis;
mod demo;
#[cfg(test)]
mod difficulte;
mod ecrans;
mod effets;
mod entrees;
mod focus;
mod grele;
mod hud;
mod menu;
mod monde;
mod mur_pluie;
mod musiques;
mod navigation;
mod orientation;
mod particules;
mod plan;
mod rendu;
mod sauvegarde;
mod scene;
mod secousse;
mod skins;
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
        window_title: "PÉPIN : un jeu de plateformes".to_owned(),
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
    // Musique imposée par-dessus celle de la zone (`None` : la musique de la zone).
    // Aujourd'hui seule la touche F11 du build de QA la règle.
    let mut forcage: Option<musiques::Musique> = None;

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
        // Build de QA seulement (`debug_niveaux`) : K passe au skin suivant (captures).
        if entrees::skin_suivant_demande() {
            app.skin = app.skin.suivant();
        }
        // Build de QA seulement (`debug_niveaux`) : G met 100 gouttes en poche (essai de la
        // boutique), enregistrées comme des gouttes gagnées.
        if entrees::gouttes_qa_demandees() {
            app.sauvegarde.ajouter_gouttes(100);
            app.sauvegarde.enregistrer();
        }
        // Build de QA seulement (`debug_niveaux`) : U avance la progression de la carte des
        // niveaux (carte 7.4), enregistrée.
        if entrees::progression_qa_demandee() {
            app.progression_qa();
        }
        // Build de QA seulement (`qa_plantage`) : F9 provoque une vraie panique, pour
        // vérifier le message de plantage de la page web. Toujours faux en production.
        if entrees::plantage_qa() {
            panic!("plantage volontaire (build de QA, touche F9)");
        }
        // Build de QA seulement (F11) : FORCE la musique suivante du catalogue, pour
        // l'écouter partout. (La future boutique vendra ce même forçage.)
        if musiques::musique_suivante_qa() {
            forcage = Some(audio.musique_active().suivante());
        }
        // Une musique par zone (carte 8.7) : celle de l'écran et du niveau chargé, sauf
        // forçage. `choisir_musique` n'agit que si elle CHANGE : deux niveaux de la même
        // zone gardent la boucle sans coupure.
        let voulue =
            forcage.unwrap_or_else(|| musiques::musique_de_l_ecran(app.ecran, app.monde.infos));
        audio.choisir_musique(voulue);
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
