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
//! (changer de niveau), K (skin suivant), T, Y (traînée, motif de parapluie
//! suivants), G (100 gouttes en poche), V (défilé des 7 fonds du Monde 2, carte 9.5),
//! F9 (plantage
//! volontaire) et F11 (force la musique suivante) : uniquement dans le build de mise au point
//! (`./build.sh --qa`, fonctionnalités `debug_niveaux` et `qa_plantage`).

mod app;
#[cfg(test)]
mod assets;
mod audio;
#[cfg(test)]
mod bot_qa;
mod boutique;
mod camera;
mod chrono;
mod contenu;
mod cosmetiques;
mod course;
mod decor;
mod decor_cascade;
mod decor_egouts;
mod decor_grotte;
mod decor_port;
mod decor_riviere;
mod decor_rue;
mod defis;
mod demo;
mod dessin_courant;
mod dessin_eau;
mod dessin_montee;
mod dessin_mort;
mod dessin_pieuvre;
mod dessin_port;
mod dessin_tour;
// La mesure de difficulté décrit la courbe du MONDE 1 (tableaux de
// `NB_NIVEAUX_MONDE_1` cases, carte 9.6) : elle ne dépend pas du Monde 2.
#[cfg(test)]
mod difficulte;
mod eau;
mod ecrans;
mod effets;
mod entrees;
// Build de QA (carte 10.1b) : niveau RON venu du navigateur, erreurs de niveau à l'écran.
#[cfg(any(test, feature = "debug_niveaux"))]
mod essai_externe;
mod facile;
// Un faux Monde 3 (carte 11.2) : un monde de plus par des données, en test seulement.
#[cfg(test)]
mod faux_monde_3;
mod focus;
mod fond_egouts;
mod fonds;
mod grele;
mod hud;
// La marée (carte 9.32) : la sauvegarde d'un joueur qui avait battu la Pieuvre.
#[cfg(test)]
mod maree_sauvegarde;
mod menu;
mod mer_montee;
mod mobilier_rue;
mod monde;
mod mort;
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
mod tuiles_theme;

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
        window_title: "PÉPIN".to_owned(),
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
    // Les sons sont des fichiers (carte 10.4e), téléchargés en arrière-plan APRÈS la
    // première image (`audio.avancer`, en fin de boucle) : l'écran titre n'attend
    // rien. Désactivés tant que le joueur ne les demande pas (et un navigateur
    // n'autorise l'audio qu'après une première interaction).
    let mut audio = Audio::new();
    // Site de test (build de QA) : toute la carte des niveaux ouverte, Monde 2 compris
    // (`plan::ouvrir_tout_qa`), AVANT de construire l'application et son titre.
    #[cfg(feature = "debug_niveaux")]
    plan::ouvrir_tout_qa();
    // Build de QA (carte 10.1b) : un fichier de niveau invalide s'affiche à l'écran
    // au lieu de faire planter le jeu, et `?essai=1` lance le niveau posé dans le
    // navigateur. Le jeu publié n'en a pas besoin : ses fichiers sont vérifiés
    // avant le build (`./build.sh`).
    #[cfg(feature = "debug_niveaux")]
    let lancement = essai_externe::au_lancement();
    #[cfg(feature = "debug_niveaux")]
    if let essai_externe::Lancement::Erreur { message, rapport } = &lancement {
        // Carte 10.3b : l'éditeur qui a ouvert le jeu reçoit la case fautive.
        essai_externe::signaler(rapport);
        essai_externe::afficher_erreur(&cible, message).await;
    }
    // Carte 10.3b : Échap ou la fin du niveau d'essai ramène à l'éditeur.
    #[cfg(feature = "debug_niveaux")]
    let mut suivi_essai = essai_externe::SuiviEssai::new(&lancement);
    let mut app = Application::new(true);
    #[cfg(feature = "debug_niveaux")]
    if let essai_externe::Lancement::Essai = lancement {
        app.lancer_essai_externe();
    }
    // Musique imposée par-dessus celle de la zone (`None` : la musique de la zone).
    // Aujourd'hui seule la touche F11 du build de QA la règle.
    let mut forcage: Option<musiques::Musique> = None;
    // Build de QA seulement (carte 9.5) : la touche V défile les 7 nouveaux fonds
    // du Monde 2, plein écran (captures `docs/qa/95-*.png`). `planche_pas` anime
    // la pluie et les motifs du fond affiché, indépendamment de la simulation.
    #[cfg(feature = "debug_niveaux")]
    let mut fonds_qa: Option<usize> = None;
    #[cfg(feature = "debug_niveaux")]
    let mut fonds_v_appuyee = false;
    #[cfg(feature = "debug_niveaux")]
    let mut planche_pas: u32 = 0;

    loop {
        // Le bouton tactile BAS (carte 9.2b) : en jeu, dans un niveau qui a de l'eau.
        tactile.montrer_bas(matches!(app.ecran, Ecran::Jeu) && app.monde.a_de_l_eau());
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
        // Build de QA seulement : T, Y passent à la traînée, au motif suivants (« rien »
        // compris), sans les enregistrer (captures de la carte 7.6).
        let [trainee, motif] = entrees::cosmetiques_suivants_demandes();
        let c = &mut app.cosmetiques;
        if trainee {
            c.trainee = cosmetiques::suivant(&cosmetiques::Trainee::TOUTES, c.trainee);
        }
        if motif {
            c.motif = cosmetiques::suivant(&cosmetiques::Motif::TOUS, c.motif);
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
        // l'écouter partout (la boutique vend ce forçage pour le jeu, carte 7.6).
        if musiques::musique_suivante_qa() {
            forcage = Some(audio.musique_active().suivante());
        }
        // Une musique par zone (carte 8.7) : celle de l'écran et du niveau chargé, sauf
        // musique choisie dans la boutique (en jeu, carte 7.6) ou forçage.
        // `choisir_musique` n'agit que si elle CHANGE : deux niveaux de la même zone
        // gardent la boucle sans coupure.
        let voulue = forcage.unwrap_or_else(|| {
            musiques::musique_jouee(app.ecran, app.monde.infos, app.cosmetiques.musique)
        });
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
        #[cfg(feature = "debug_niveaux")]
        suivi_essai.apres_image(app.ecran);

        // --- Rendu interpolé, dans la cible logique -----------------------------
        cible.commencer();
        if a_tourner {
            orientation::dessiner(get_time());
        } else {
            // Build de QA seulement (carte 9.5) : la touche V (appuyée, un appui
            // = un fond) remplace le jeu par le défilé plein écran des 7 nouveaux
            // fonds du Monde 2 ; sans V, le jeu se dessine comme d'habitude.
            #[cfg(feature = "debug_niveaux")]
            {
                let v_appuyee = entrees::fonds_qa_demande();
                if v_appuyee && !fonds_v_appuyee {
                    fonds_qa = Some(match fonds_qa {
                        None => 0,
                        Some(k) => (k + 1) % 7,
                    });
                }
                fonds_v_appuyee = v_appuyee;
                if let Some(k) = fonds_qa {
                    planche_pas = planche_pas.wrapping_add(1);
                    decor::planche_fonds_qa(k, planche_pas);
                } else {
                    banque.choisir_motif(app.cosmetiques.motif);
                    app.dessiner(&banque, pas_fixe.alpha());
                }
            }
            #[cfg(not(feature = "debug_niveaux"))]
            {
                banque.choisir_motif(app.cosmetiques.motif);
                app.dessiner(&banque, pas_fixe.alpha());
            }
        }
        cible.afficher();
        if !a_tourner && matches!(app.ecran, Ecran::Jeu) {
            tactile.dessiner();
        }
        next_frame().await;
        audio.avancer();
    }
}
