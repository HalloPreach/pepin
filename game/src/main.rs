//! Point d'entrée macroquad : boucle principale à pas fixe 60 Hz (le `PasFixe` du
//! cœur), rendu interpolé découplé, dans une cible 320x180 agrandie d'un
//! facteur entier.
//!
//! Commandes : flèches ou Q/D (ou A/D) pour courir ; Espace, Haut, W ou Z pour sauter
//! (maintenu en l'air : parapluie ouvert) ; touches 1 à 5 pour changer de niveau.

mod camera;
mod decor;
mod monde;
mod particules;
mod rendu;
mod scene;
mod sprites;
mod texte;

use macroquad::prelude::*;
use pepin_core::entities::contacts::Contact;
use pepin_core::palette::{CORAIL_CLAIR, OR, PLUIE, PLUIE_SOMBRE};
use pepin_core::player::{Direction, Entrees};
use pepin_core::state::EtatJeu;
use pepin_core::timestep::PasFixe;
use pepin_core::zones::NB_NIVEAUX;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use camera::Camera;
use monde::Monde;
use particules::Particules;
use rendu::Cible;
use sprites::Banque;

/// Configuration de la fenêtre (échelle x4 de la résolution logique).
fn conf() -> Conf {
    Conf {
        window_title: "PÉPIN — un jeu de plateformes".to_owned(),
        window_width: (LARGEUR_LOGIQUE * 4) as i32,
        window_height: (HAUTEUR_LOGIQUE * 4) as i32,
        ..Default::default()
    }
}

/// Lit le clavier : une `Entrees` pour le prochain pas de simulation.
fn lire_entrees() -> Entrees {
    let gauche = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) || is_key_down(KeyCode::Q);
    let droite = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    let saut = is_key_down(KeyCode::Space)
        || is_key_down(KeyCode::Up)
        || is_key_down(KeyCode::W)
        || is_key_down(KeyCode::Z);
    Entrees {
        direction: match (gauche, droite) {
            (true, false) => Direction::Gauche,
            (false, true) => Direction::Droite,
            _ => Direction::Immobile,
        },
        saut,
    }
}

/// Niveau demandé par les touches 1 à 5, s'il y en a un.
fn niveau_demande() -> Option<usize> {
    [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5]
        .iter()
        .position(|&k| is_key_pressed(k))
}

/// Charge le niveau `indice` en conservant (ou non) le score de la partie.
fn charger(indice: usize, garder: Option<EtatJeu>) -> (Monde, Camera) {
    let mut monde = Monde::new(indice);
    if let Some(mut etat) = garder {
        etat.nouveau_niveau(monde.niveau.depart);
        monde.etat = etat;
    }
    let camera = Camera::new(&monde.joueur, monde.niveau.largeur_px(), monde.niveau.hauteur_px());
    (monde, camera)
}

/// Effets visuels déclenchés par ce qui s'est passé pendant un pas.
fn effets(monde: &Monde, particules: &mut Particules) {
    let (x, y) = (monde.joueur.boite.centre().0, monde.joueur.boite.bas());
    if monde.evenements.atterrissage {
        particules.eclabousser(x, y, 7, PLUIE, 1.6);
    }
    if monde.evenements.saut {
        particules.eclabousser(x, y, 4, PLUIE_SOMBRE, 1.2);
    }
    // Un petit filet d'eau sous les pieds quand Pépin court.
    if monde.joueur.au_sol && monde.joueur.vitesse.x.abs() > 8.0 && monde.pas % 6 == 0 {
        particules.eclabousser(x, y, 1, PLUIE_SOMBRE, 0.8);
    }
    for c in &monde.contacts {
        match c {
            Contact::Goutte(_) => particules.eclabousser(x, y - 6.0, 10, OR, 2.0),
            Contact::Ecrase => particules.eclabousser(x, y, 10, PLUIE, 2.0),
            Contact::Rebond => particules.eclabousser(x, y, 6, CORAIL_CLAIR, 2.0),
            Contact::Mort(_) => {
                let (mx, my) = monde.pos_mort;
                particules.eclabousser(mx + 5.0, my + 7.0, 14, CORAIL_CLAIR, 2.4);
            }
            _ => {}
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    let banque = Banque::charger();
    let cible = Cible::new();
    let mut pas_fixe = PasFixe::new();
    let mut particules = Particules::new();
    let (mut monde, mut camera) = charger(0, None);

    loop {
        // --- Changement de niveau (touches 1-5, ou drapeau atteint) -----------
        let mut suite = niveau_demande().map(|i| (i, None));
        if monde.termine {
            let suivant = (monde.infos.numero as usize) % NB_NIVEAUX;
            let etat = if suivant == 0 { None } else { Some(monde.etat.clone()) };
            suite = Some((suivant, etat));
        }
        if let Some((indice, etat)) = suite {
            (monde, camera) = charger(indice, etat);
            particules = Particules::new();
            pas_fixe.reinitialiser();
        }

        // --- Simulation à pas fixe -------------------------------------------
        let entrees = lire_entrees();
        let (largeur, hauteur) = (monde.niveau.largeur_px(), monde.niveau.hauteur_px());
        for _ in 0..pas_fixe.avancer(get_frame_time() as f64) {
            monde.avancer(&entrees);
            if monde.contacts.iter().any(|c| matches!(c, Contact::Mort(_))) {
                camera.recentrer(&monde.joueur, largeur, hauteur);
            }
            camera.suivre(&monde.joueur, largeur, hauteur);
            effets(&monde, &mut particules);
            particules.actualiser();
        }

        // --- Rendu interpolé, dans la cible logique ---------------------------
        let alpha = pas_fixe.alpha();
        let (cx, cy) = camera.position_rendue(alpha);
        cible.commencer();
        scene::dessiner(&monde, &banque, &particules, alpha, cx, cy);
        cible.afficher();
        next_frame().await;
    }
}
