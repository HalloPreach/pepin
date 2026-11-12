//! Point d'entrée macroquad : boucle principale à pas fixe 60 Hz (le `PasFixe` du
//! cœur), rendu interpolé découplé, dans une cible 320x180 agrandie d'un
//! facteur entier.
//!
//! Commandes : flèches ou Q/D (ou A/D) pour courir ; Espace, Haut, W ou Z pour sauter
//! (maintenu en l'air : parapluie ouvert) ; touches 1 à 5 pour changer de niveau.

mod camera;
mod decor;
mod entrees;
mod monde;
mod particules;
mod rendu;
mod scene;
mod secousse;
mod sprites;
mod texte;
mod transition;

use macroquad::prelude::*;
use pepin_core::entities::contacts::Contact;
use pepin_core::palette::{BLANC_CASSE, CORAIL_CLAIR, OR, OR_PALE, PLUIE, PLUIE_SOMBRE};
use pepin_core::state::EtatJeu;
use pepin_core::timestep::PasFixe;
use pepin_core::zones::NB_NIVEAUX;
use pepin_core::{HAUTEUR_LOGIQUE, LARGEUR_LOGIQUE};

use camera::Camera;
use entrees::{Clavier, Sources};
use monde::Monde;
use particules::Particules;
use rendu::Cible;
use secousse::Secousse;
use sprites::Banque;
use transition::Transition;

/// Configuration de la fenêtre (échelle x4 de la résolution logique).
fn conf() -> Conf {
    Conf {
        window_title: "PÉPIN — un jeu de plateformes".to_owned(),
        window_width: (LARGEUR_LOGIQUE * 4) as i32,
        window_height: (HAUTEUR_LOGIQUE * 4) as i32,
        ..Default::default()
    }
}

/// Charge le niveau `indice` en conservant (ou non) le score de la partie.
fn charger(indice: usize, garder: Option<EtatJeu>) -> (Monde, Camera) {
    let mut monde = Monde::new(indice);
    if let Some(mut etat) = garder {
        etat.nouveau_niveau(monde.niveau.depart);
        monde.etat = etat;
    }
    let camera = Camera::new(
        &monde.joueur,
        monde.niveau.largeur_px(),
        monde.niveau.hauteur_px(),
    );
    (monde, camera)
}

/// Effets visuels déclenchés par ce qui s'est passé pendant un pas.
/// `cam_x` : bord gauche de la zone visible, pour n'animer que le vent qu'on voit.
fn effets(monde: &Monde, particules: &mut Particules, secousse: &mut Secousse, cam_x: f32) {
    let (x, y) = (monde.joueur.boite.centre().0, monde.joueur.boite.bas());
    let sens = if monde.joueur.regarde_droite {
        1.0
    } else {
        -1.0
    };
    if monde.evenements.atterrissage {
        particules.eclabousser(x, y, 7, PLUIE, 1.6);
        // Poussière des deux côtés des pieds.
        particules.poussiere(x - 3.0, y - 1.0, 3, BLANC_CASSE, -1.0);
        particules.poussiere(x + 3.0, y - 1.0, 3, BLANC_CASSE, 1.0);
    }
    if monde.evenements.saut {
        particules.eclabousser(x, y, 4, PLUIE_SOMBRE, 1.2);
        particules.poussiere(x, y - 1.0, 3, BLANC_CASSE, -sens);
    }
    // En courant : un filet d'eau et un peu de poussière derrière les pieds.
    if monde.joueur.au_sol && monde.joueur.vitesse.x.abs() > 8.0 {
        if monde.pas.is_multiple_of(6) {
            particules.eclabousser(x, y, 1, PLUIE_SOMBRE, 0.8);
        }
        if monde.pas.is_multiple_of(5) {
            particules.poussiere(x - sens * 4.0, y - 1.0, 1, BLANC_CASSE, -sens);
        }
    }
    // Le vent des courants ascendants devient visible (brins qui montent).
    if monde.pas.is_multiple_of(3) {
        for (k, c) in monde.niveau.courants.iter().enumerate() {
            let z = c.zone;
            if z.x + z.w < cam_x || z.x > cam_x + LARGEUR_LOGIQUE as f32 {
                continue;
            }
            let decal = (monde.pas as usize * 7 + k * 13) as f32 % z.w.max(1.0);
            particules.souffle(z.x + decal, z.bas() - 2.0, BLANC_CASSE);
        }
    }
    for c in &monde.contacts {
        match c {
            Contact::Goutte(_) => {
                particules.eclabousser(x, y - 6.0, 10, OR, 2.0);
                particules.etincelles(x, y - 8.0, 8, OR_PALE);
            }
            Contact::Ecrase => particules.eclabousser(x, y, 10, PLUIE, 2.0),
            Contact::Rebond => particules.eclabousser(x, y, 6, CORAIL_CLAIR, 2.0),
            Contact::Lanterne(..) => particules.etincelles(x, y - 8.0, 10, OR),
            Contact::Mort(_) => {
                let (mx, my) = monde.pos_mort;
                particules.eclabousser(mx + 5.0, my + 7.0, 14, CORAIL_CLAIR, 2.4);
                secousse.declencher(2, 14);
            }
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    let banque = Banque::charger();
    let cible = Cible::new();
    let mut pas_fixe = PasFixe::new();
    let mut particules = Particules::new();
    let mut secousse = Secousse::new(true);
    let mut transition = Transition::new();
    let mut sources = Sources::new();
    sources.ajouter(Box::new(Clavier));
    let (mut monde, mut camera) = charger(0, None);
    // Niveau (et partie gardée) à charger quand l'écran sera entièrement couvert.
    let mut suite: Option<(usize, Option<EtatJeu>)> = None;

    loop {
        let commandes = sources.lire();
        let entrees = commandes.vers_entrees();

        // --- Demande de changement de niveau (touches 1-5, ou drapeau atteint) ---
        if !transition.est_active() {
            if let Some(i) = entrees::raccourci_niveau() {
                suite = Some((i, None));
                transition.demarrer();
            } else if monde.termine {
                let suivant = (monde.infos.numero as usize) % NB_NIVEAUX;
                // Après le dernier niveau on repart de zéro ; sinon le score est gardé.
                let etat = (suivant != 0).then(|| monde.etat.clone());
                suite = Some((suivant, etat));
                transition.demarrer();
            }
        }

        // --- Simulation à pas fixe ---------------------------------------------
        for _ in 0..pas_fixe.avancer(get_frame_time() as f64) {
            secousse.actualiser();
            if transition.est_active() {
                // Le monde est figé pendant les volets ; les particules finissent leur vie.
                if transition.actualiser() {
                    if let Some((indice, etat)) = suite.take() {
                        (monde, camera) = charger(indice, etat);
                        particules = Particules::new();
                    }
                }
                particules.actualiser();
                continue;
            }
            let (largeur, hauteur) = (monde.niveau.largeur_px(), monde.niveau.hauteur_px());
            monde.avancer(&entrees);
            if monde.contacts.iter().any(|c| matches!(c, Contact::Mort(_))) {
                camera.recentrer(&monde.joueur, largeur, hauteur);
            }
            camera.suivre(&monde.joueur, largeur, hauteur);
            effets(&monde, &mut particules, &mut secousse, camera.x);
            particules.actualiser();
        }

        // --- Rendu interpolé, dans la cible logique -----------------------------
        let alpha = pas_fixe.alpha();
        let (cx, cy) = camera.position_rendue(alpha);
        let (sx, sy) = secousse.decalage();
        cible.commencer();
        scene::dessiner(&monde, &banque, &particules, alpha, cx + sx, cy + sy);
        transition.dessiner();
        cible.afficher();
        next_frame().await;
    }
}
