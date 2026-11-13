//! `Application` : la machine à écrans du jeu (titre, aide, carte de niveau, jeu,
//! pause, fin) et tout ce qui les relie (transitions, bilan, rejouer).
//!
//! Deux rythmes cohabitent :
//! - `reagir` est appelé UNE fois par image affichée : il lit les appuis
//!   ponctuels (menus, pause) qu'un pas de simulation pourrait manquer ;
//! - `pas` est appelé à chaque pas FIXE de 1/60 s : il fait vivre le monde.
//!
//! Rien ici n'appelle macroquad hors de `dessiner` : toute la logique (qui mène
//! où, que devient le score...) se teste donc sans fenêtre.
//!
//! Notions Rust illustrées ici : machine à états avec `enum` (`Ecran`, `Suite`),
//! `Option::take` pour consommer une demande en attente, `Vec<Choix>` reconstruit
//! à chaque écran, méthodes qui ne se contredisent pas (un seul endroit change
//! d'écran).

use pepin_core::levels::{self, Apres, NB_NIVEAUX, NIVEAUX};
use pepin_core::player::Entrees;
use pepin_core::state::EtatJeu;

use crate::camera::Camera;
use crate::contenu;
use crate::ecrans::fin::{self, Bilan};
use crate::ecrans::{self, aide, carte, pause, titre, Disposition, Ecran};
use crate::effets::effets;
use crate::entrees::{self, Commandes, RaccourciNiveau};
use crate::hud;
use crate::menu::{Choix, Menu};
use crate::monde::Monde;
use crate::navigation::Navigation;
use crate::particules::Particules;
use crate::scene;
use crate::secousse::Secousse;
use crate::sprites::Banque;
use crate::transition::Transition;
use pepin_core::entities::contacts::Contact;

/// Ce qui doit se produire quand l'écran est entièrement couvert par la transition.
#[derive(Debug, Clone)]
enum Suite {
    /// Charger le niveau `indice`. `etat` : partie gardée ; `None` = nouvelle partie.
    Niveau {
        indice: usize,
        etat: Option<EtatJeu>,
    },
    /// Recharger le niveau courant tel qu'il était à son début.
    Recommencer,
    Titre,
    Fin,
}

/// Charge le niveau `indice` en conservant (ou non) le score de la partie.
pub fn charger(indice: usize, garder: Option<EtatJeu>) -> (Monde, Camera) {
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

/// Les lignes du menu d'un écran, dans l'ordre. `son_disponible` : le jeu sait-il
/// faire du son ? (sinon la ligne SON n'est pas proposée)
pub fn choix_pour(ecran: Ecran, son_disponible: bool) -> Vec<Choix> {
    let mut v = match ecran {
        Ecran::Titre => vec![Choix::Jouer, Choix::Aide],
        Ecran::Pause => vec![Choix::Reprendre, Choix::Recommencer],
        Ecran::Fin => vec![Choix::Rejouer, Choix::RetourTitre],
        Ecran::Aide | Ecran::Carte(_) | Ecran::Jeu => Vec::new(),
    };
    if son_disponible && matches!(ecran, Ecran::Titre | Ecran::Pause) {
        v.push(Choix::Son);
    }
    if ecran == Ecran::Pause {
        v.push(Choix::QuitterAuTitre);
    }
    v
}

pub struct Application {
    pub ecran: Ecran,
    pub monde: Monde,
    pub camera: Camera,
    particules: Particules,
    secousse: Secousse,
    transition: Transition,
    /// Le son est-il demandé ? (coupé au départ ; la boucle principale le transmet à l'audio)
    pub son: bool,
    son_disponible: bool,
    choix: Vec<Choix>,
    menu: Menu,
    suite: Option<Suite>,
    /// Pas écoulés depuis l'ouverture de l'écran (animations et clignotements).
    anim: u32,
    /// Durée de jeu de la partie en cours (pas), sans les menus ni les transitions.
    pas_partie: u32,
    /// Photo de la partie au début du niveau, pour « recommencer le niveau ».
    debut_niveau: EtatJeu,
    bilan: Bilan,
    /// Le focus a été perdu juste avant que le jeu ne (re)démarre (carte de présentation,
    /// volets) : la pause s'ouvrira au premier pas de jeu, sauf si le focus revient avant.
    pause_differee: bool,
}

impl Application {
    /// `son_disponible` : afficher l'option SON et réagir à la touche M.
    pub fn new(son_disponible: bool) -> Self {
        let (monde, camera) = charger(0, None);
        let debut_niveau = monde.etat.clone();
        let mut app = Self {
            ecran: Ecran::Titre,
            monde,
            camera,
            particules: Particules::new(),
            secousse: Secousse::new(true),
            transition: Transition::new(),
            son: false,
            son_disponible,
            choix: Vec::new(),
            menu: Menu::new(1),
            suite: None,
            anim: 0,
            pas_partie: 0,
            debut_niveau,
            bilan: Bilan::default(),
            pause_differee: false,
        };
        app.ouvrir(Ecran::Titre);
        app
    }

    /// Affiche un écran à menu (ou sans) : reconstruit ses lignes, remet le curseur en haut.
    fn ouvrir(&mut self, ecran: Ecran) {
        self.ecran = ecran;
        self.anim = 0;
        self.choix = choix_pour(ecran, self.son_disponible);
        self.menu = Menu::new(self.choix.len());
        // Pause, titre, aide ou fin : une pause différée n'a plus lieu d'être.
        self.pause_differee = false;
    }

    /// Prépare un changement d'écran derrière la transition en volets.
    fn lancer(&mut self, suite: Suite) {
        self.suite = Some(suite);
        self.transition.demarrer();
    }

    /// Chrono de la partie en cours (pas de jeu), pour le robot de QA.
    #[cfg(test)]
    pub fn chrono(&self) -> u32 {
        self.pas_partie
    }

    /// Le monde de jeu est-il en train d'avancer (donc à interpoler) ?
    pub fn en_jeu(&self) -> bool {
        self.ecran == Ecran::Jeu && !self.transition.est_active()
    }

    /// La fenêtre perd (`present = false`) ou retrouve (`true`) le focus : onglet masqué,
    /// autre application, téléphone verrouillé (voir `focus.rs`).
    ///
    /// - En pleine partie : l'écran de pause s'ouvre (le monde et le chrono s'arrêtent,
    ///   la musique et la pluie aussi) ; seul le joueur en sort.
    /// - Déjà en pause : rien (la pause ne s'ouvre jamais deux fois, le curseur reste).
    /// - Titre, aide, fin : rien. Carte de présentation ou volets qui mènent au jeu :
    ///   la pause est DIFFÉRÉE au premier pas de jeu, et annulée si le focus revient avant
    ///   (sinon, fenêtre visible mais sans focus, la carte finirait et Pépin partirait seul).
    pub fn focus(&mut self, present: bool) {
        if present {
            self.pause_differee = false;
        } else if self.en_jeu() {
            self.ouvrir(Ecran::Pause);
        } else if self.partie_imminente() {
            self.pause_differee = true;
        }
    }

    /// Le jeu va-t-il démarrer tout seul (carte de présentation, volets vers un niveau) ?
    fn partie_imminente(&self) -> bool {
        match self.suite {
            // Volets qui se ferment : tout dépend de ce qui vient derrière.
            Some(Suite::Niveau { .. }) | Some(Suite::Recommencer) => true,
            Some(Suite::Titre) | Some(Suite::Fin) => false,
            // Rien en attente : carte de présentation, ou volets qui s'ouvrent sur le
            // niveau recommencé.
            None => {
                matches!(self.ecran, Ecran::Carte(_))
                    || (self.ecran == Ecran::Jeu && self.transition.est_active())
            }
        }
    }

    /// La musique et la pluie doivent-elles tourner ? Non pendant la pause : elles
    /// reprennent quand le joueur en sort (la boucle principale le transmet à l'audio).
    pub fn boucles_sonores(&self) -> bool {
        self.ecran != Ecran::Pause
    }

    // --- Réactions aux appuis (une fois par image) -------------------------------

    /// `raccourci` : raccourci de niveau du build de mise au point (touches 1 à 9, niveau
    /// suivant ou précédent), à charger tout de suite comme une nouvelle partie.
    pub fn reagir(
        &mut self,
        nav: &Navigation,
        commandes: &Commandes,
        raccourci: Option<RaccourciNiveau>,
    ) {
        if nav.son && self.son_disponible {
            self.son = !self.son;
        }
        if self.transition.est_active() {
            return;
        }
        match self.ecran {
            Ecran::Titre | Ecran::Pause | Ecran::Fin => self.reagir_menu(nav, commandes),
            Ecran::Aide => {
                if nav.retour || nav.valider || nav.clic.is_some() {
                    self.ouvrir(Ecran::Titre);
                    if let Some(i) = self.choix.iter().position(|&c| c == Choix::Aide) {
                        self.menu.choisir(i);
                    }
                }
            }
            Ecran::Carte(_) => {
                if nav.valider || nav.clic.is_some() {
                    self.ecran = Ecran::Jeu;
                }
            }
            Ecran::Jeu => {
                if commandes.pause {
                    self.ouvrir(Ecran::Pause);
                } else if let Some(indice) = raccourci
                    .and_then(|r| entrees::cible_raccourci(r, self.monde.indice, NB_NIVEAUX))
                {
                    self.lancer(Suite::Niveau { indice, etat: None });
                }
            }
        }
    }

    fn reagir_menu(&mut self, nav: &Navigation, commandes: &Commandes) {
        if self.ecran == Ecran::Pause && (commandes.pause || nav.retour) {
            self.valider_choix(Choix::Reprendre);
            return;
        }
        if nav.haut {
            self.menu.monter();
        }
        if nav.bas {
            self.menu.descendre();
        }
        let mut valider = nav.valider;
        if let Some(d) = ecrans::disposition(self.ecran) {
            let nb = self.choix.len();
            if let Some((x, y)) = nav.survol {
                if let Some(i) = d.ligne_sous(nb, x, y) {
                    self.menu.choisir(i);
                }
            }
            if let Some((x, y)) = nav.clic {
                if let Some(i) = d.ligne_sous(nb, x, y) {
                    self.menu.choisir(i);
                    valider = true;
                }
            }
        }
        if valider {
            if let Some(&c) = self.choix.get(self.menu.selection()) {
                self.valider_choix(c);
            }
        }
    }

    fn valider_choix(&mut self, choix: Choix) {
        match choix {
            Choix::Jouer | Choix::Rejouer => self.lancer(Suite::Niveau {
                indice: 0,
                etat: None,
            }),
            Choix::Aide => self.ouvrir(Ecran::Aide),
            Choix::Son => self.son = !self.son,
            Choix::Reprendre => self.ecran = Ecran::Jeu,
            Choix::Recommencer => self.lancer(Suite::Recommencer),
            Choix::QuitterAuTitre | Choix::RetourTitre => self.lancer(Suite::Titre),
        }
    }

    // --- Pas de simulation (60 fois par seconde) ----------------------------------

    /// Un pas de 1/60 s. Renvoie `true` si le monde de jeu a avancé (pour l'audio).
    pub fn pas(&mut self, entrees: &Entrees) -> bool {
        self.secousse.actualiser();
        self.anim = self.anim.wrapping_add(1);
        let mut monde_avance = false;
        if self.transition.est_active() {
            // Le monde est figé pendant les volets ; les particules finissent leur vie.
            if self.transition.actualiser() {
                self.appliquer_suite();
            }
        } else {
            match self.ecran {
                Ecran::Carte(n) => {
                    self.ecran = if n <= 1 {
                        Ecran::Jeu
                    } else {
                        Ecran::Carte(n - 1)
                    };
                }
                Ecran::Jeu => {
                    if std::mem::take(&mut self.pause_differee) {
                        // Focus perdu pendant la carte : on s'arrête avant le premier pas.
                        self.ouvrir(Ecran::Pause);
                    } else {
                        self.pas_jeu(entrees);
                        monde_avance = true;
                    }
                }
                Ecran::Fin => fin::confettis(&mut self.particules, self.anim),
                Ecran::Titre | Ecran::Aide | Ecran::Pause => {}
            }
        }
        self.particules.actualiser();
        monde_avance
    }

    fn pas_jeu(&mut self, entrees: &Entrees) {
        let (largeur, hauteur) = (
            self.monde.niveau.largeur_px(),
            self.monde.niveau.hauteur_px(),
        );
        self.monde.avancer(entrees);
        if self
            .monde
            .contacts
            .iter()
            .any(|c| matches!(c, Contact::Mort(_)))
        {
            self.camera.recentrer(&self.monde.joueur, largeur, hauteur);
        }
        self.camera.suivre(&self.monde.joueur, largeur, hauteur);
        effets(
            &self.monde,
            &mut self.particules,
            &mut self.secousse,
            self.camera.x,
            self.camera.y,
        );
        self.pas_partie += 1;
        if self.monde.termine {
            // Niveau suivant (le score est gardé), en sautant le bonus s'il manque des
            // gouttes ; après l'épilogue ou le dernier niveau : l'écran de fin.
            self.lancer(
                match levels::apres(NIVEAUX, self.monde.indice, self.monde.etat.gouttes) {
                    Apres::Niveau(indice) => Suite::Niveau {
                        indice,
                        etat: Some(self.monde.etat.clone()),
                    },
                    Apres::Fin => Suite::Fin,
                },
            );
        }
    }

    /// L'écran est couvert : on change de décor sans que le joueur le voie.
    fn appliquer_suite(&mut self) {
        let Some(suite) = self.suite.take() else {
            return;
        };
        self.particules = Particules::new();
        match suite {
            Suite::Niveau { indice, etat } => {
                if etat.is_none() {
                    self.pas_partie = 0;
                }
                self.recharger(indice, etat);
                self.anim = 0;
                self.ecran = Ecran::Carte(carte::DUREE_PAS);
            }
            Suite::Recommencer => {
                let indice = self.monde.indice;
                // Le niveau repart comme au début, mais les morts restent comptées.
                let mut etat = self.debut_niveau.clone();
                etat.morts = self.monde.etat.morts;
                self.recharger(indice, Some(etat));
                self.ecran = Ecran::Jeu;
            }
            Suite::Titre => self.ouvrir(Ecran::Titre),
            Suite::Fin => {
                let e = &self.monde.etat;
                self.bilan = Bilan {
                    pas: self.pas_partie,
                    gouttes: e.gouttes,
                    morts: e.morts,
                    score: e.score,
                };
                self.ouvrir(Ecran::Fin);
            }
        }
    }

    fn recharger(&mut self, indice: usize, etat: Option<EtatJeu>) {
        let (monde, camera) = charger(indice, etat);
        self.debut_niveau = monde.etat.clone();
        self.monde = monde;
        self.camera = camera;
    }

    // --- Dessin --------------------------------------------------------------------

    /// Dessine l'écran courant dans la cible logique. `alpha` : interpolation entre deux pas.
    pub fn dessiner(&self, banque: &Banque, alpha: f32) {
        // Monde figé (pause, transition, carte) : on montre le dernier pas tel quel,
        // sinon l'interpolation ferait trembler les sprites sur place.
        let alpha = if self.en_jeu() { alpha } else { 1.0 };
        let libelles: Vec<&str> = self
            .choix
            .iter()
            .map(|&c| contenu::libelle(c, self.son))
            .collect();
        let dispo = ecrans::disposition(self.ecran).unwrap_or(Disposition { y0: 0.0, pas: 20.0 });
        match self.ecran {
            Ecran::Titre => titre::dessiner(banque, self.anim, &libelles, &self.menu, &dispo),
            Ecran::Aide => aide::dessiner(),
            Ecran::Fin => fin::dessiner(
                &self.bilan,
                self.anim,
                &self.particules,
                &libelles,
                &self.menu,
                &dispo,
            ),
            Ecran::Jeu | Ecran::Carte(_) | Ecran::Pause => {
                // Pépin et la caméra sont arrondis ensemble (écart arrondi une seule fois) : pas de
                // tremblement d'un pixel en courant (ni en tombant, dans un niveau haut).
                let pepin = self.monde.joueur_interpole(alpha);
                let (cx, cy) = self.camera.position_rendue(alpha, pepin);
                let (sx, sy) = self.secousse.decalage();
                scene::dessiner(
                    &self.monde,
                    banque,
                    &self.particules,
                    alpha,
                    cx + sx,
                    cy + sy,
                );
                // Sous la trame (carte, pause) le HUD serait illisible : on ne le montre qu'en jeu.
                if self.ecran == Ecran::Jeu {
                    hud::dessiner(banque, self.monde.etat.gouttes, self.pas_partie);
                }
                match self.ecran {
                    Ecran::Carte(_) => {
                        carte::dessiner(self.monde.indice, self.monde.infos, self.anim)
                    }
                    Ecran::Pause => pause::dessiner(self.anim, &libelles, &self.menu, &dispo),
                    _ => {}
                }
            }
        }
        self.transition.dessiner();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::state::POINTS_GOUTTE;

    fn rien() -> Navigation {
        Navigation::default()
    }

    fn valider() -> Navigation {
        Navigation {
            valider: true,
            ..Navigation::default()
        }
    }

    fn pause() -> Commandes {
        Commandes {
            pause: true,
            ..Commandes::default()
        }
    }

    /// Fait avancer `n` pas sans entrée.
    fn avancer(app: &mut Application, n: u32) {
        for _ in 0..n {
            app.pas(&Entrees::default());
        }
    }

    /// Avance jusqu'à ce que l'écran satisfasse `ok` (borné pour ne jamais boucler).
    fn jusqu_a(app: &mut Application, ok: impl Fn(Ecran) -> bool) {
        for _ in 0..600 {
            if ok(app.ecran) && !app.transition.est_active() {
                return;
            }
            app.pas(&Entrees::default());
        }
        panic!("écran attendu jamais atteint (écran : {:?})", app.ecran);
    }

    /// Lance une partie depuis le titre et attend la fin de la carte du niveau 1.
    fn en_jeu() -> Application {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Jeu);
        app
    }

    #[test]
    fn les_menus_ont_les_bonnes_lignes() {
        assert_eq!(
            choix_pour(Ecran::Titre, true),
            vec![Choix::Jouer, Choix::Aide, Choix::Son]
        );
        assert_eq!(
            choix_pour(Ecran::Titre, false),
            vec![Choix::Jouer, Choix::Aide]
        );
        assert_eq!(
            choix_pour(Ecran::Pause, true),
            vec![
                Choix::Reprendre,
                Choix::Recommencer,
                Choix::Son,
                Choix::QuitterAuTitre
            ]
        );
        assert_eq!(
            choix_pour(Ecran::Pause, false),
            vec![Choix::Reprendre, Choix::Recommencer, Choix::QuitterAuTitre]
        );
        assert_eq!(
            choix_pour(Ecran::Fin, true),
            vec![Choix::Rejouer, Choix::RetourTitre]
        );
        assert!(choix_pour(Ecran::Jeu, true).is_empty());
    }

    #[test]
    fn on_demarre_au_titre_son_coupe() {
        let app = Application::new(true);
        assert_eq!(app.ecran, Ecran::Titre);
        assert!(!app.son);
    }

    #[test]
    fn jouer_passe_par_la_transition_puis_la_carte_puis_le_jeu() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(
            app.ecran,
            Ecran::Titre,
            "pas de changement avant les volets"
        );
        assert!(app.transition.est_active());
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        // La carte est passée toute seule au bout de DUREE_PAS.
        avancer(&mut app, carte::DUREE_PAS as u32 + 2);
        assert_eq!(app.ecran, Ecran::Jeu);
        assert_eq!(app.monde.infos.numero, 1);
    }

    #[test]
    fn la_carte_se_passe_au_clavier_ou_au_toucher() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        let toucher = Navigation {
            clic: Some((10.0, 10.0)),
            ..Navigation::default()
        };
        app.reagir(&toucher, &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Jeu);
    }

    #[test]
    fn le_monde_est_fige_pendant_la_carte() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        let avant = app.monde.pas;
        avancer(&mut app, 30);
        assert_eq!(app.monde.pas, avant);
    }

    #[test]
    fn la_pause_va_et_vient() {
        let mut app = en_jeu();
        app.reagir(&rien(), &pause(), None);
        assert_eq!(app.ecran, Ecran::Pause);
        let pas = app.monde.pas;
        avancer(&mut app, 20);
        assert_eq!(app.monde.pas, pas, "le monde ne bouge pas en pause");
        app.reagir(&rien(), &pause(), None);
        assert_eq!(app.ecran, Ecran::Jeu);
    }

    #[test]
    fn echap_ferme_la_pause_aussi() {
        let mut app = en_jeu();
        app.reagir(&rien(), &pause(), None);
        let retour = Navigation {
            retour: true,
            ..Navigation::default()
        };
        app.reagir(&retour, &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Jeu);
    }

    #[test]
    fn quitter_au_titre_depuis_la_pause() {
        let mut app = en_jeu();
        app.reagir(&rien(), &pause(), None);
        let bas = Navigation {
            bas: true,
            ..Navigation::default()
        };
        // Reprendre, Recommencer, Son, Quitter : on descend en bas en bouclant par le haut.
        let haut = Navigation {
            haut: true,
            ..Navigation::default()
        };
        app.reagir(&haut, &Commandes::default(), None);
        assert_eq!(app.choix[app.menu.selection()], Choix::QuitterAuTitre);
        app.reagir(&bas, &Commandes::default(), None);
        app.reagir(&haut, &Commandes::default(), None);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| e == Ecran::Titre);
        assert_eq!(app.ecran, Ecran::Titre);
    }

    #[test]
    fn la_touche_m_bascule_le_son_seulement_s_il_est_disponible() {
        let m = Navigation {
            son: true,
            ..Navigation::default()
        };
        let mut avec = Application::new(true);
        avec.reagir(&m, &Commandes::default(), None);
        assert!(avec.son);
        avec.reagir(&m, &Commandes::default(), None);
        assert!(!avec.son);
        let mut sans = Application::new(false);
        sans.reagir(&m, &Commandes::default(), None);
        assert!(!sans.son);
    }

    #[test]
    fn la_ligne_son_du_menu_bascule_sans_quitter_le_menu() {
        let mut app = Application::new(true);
        let bas = Navigation {
            bas: true,
            ..Navigation::default()
        };
        app.reagir(&bas, &Commandes::default(), None);
        app.reagir(&bas, &Commandes::default(), None);
        app.reagir(&valider(), &Commandes::default(), None);
        assert!(app.son);
        assert_eq!(app.ecran, Ecran::Titre);
        assert!(!app.transition.est_active());
    }

    #[test]
    fn un_clic_sur_une_ligne_la_valide() {
        let mut app = Application::new(true);
        let d = ecrans::disposition(Ecran::Titre).unwrap();
        let (x, y, _, h) = d.rect_ligne(1);
        let clic = Navigation {
            clic: Some((x + 10.0, y + h / 2.0)),
            ..Navigation::default()
        };
        app.reagir(&clic, &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Aide);
        // Un clic n'importe où ramène au titre, curseur sur « comment jouer ».
        app.reagir(&clic, &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Titre);
        assert_eq!(app.choix[app.menu.selection()], Choix::Aide);
    }

    #[test]
    fn un_clic_dans_le_vide_ne_fait_rien() {
        let mut app = Application::new(true);
        let clic = Navigation {
            clic: Some((2.0, 2.0)),
            ..Navigation::default()
        };
        app.reagir(&clic, &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Titre);
        assert!(!app.transition.est_active());
    }

    #[test]
    fn le_survol_deplace_le_curseur_sans_valider() {
        let mut app = Application::new(true);
        let d = ecrans::disposition(Ecran::Titre).unwrap();
        let (x, y, _, h) = d.rect_ligne(1);
        let survol = Navigation {
            survol: Some((x + 10.0, y + h / 2.0)),
            ..Navigation::default()
        };
        app.reagir(&survol, &Commandes::default(), None);
        assert_eq!(app.menu.selection(), 1);
        assert_eq!(app.ecran, Ecran::Titre);
    }

    #[test]
    fn recommencer_le_niveau_garde_les_morts_mais_rend_les_gouttes() {
        let mut app = en_jeu();
        app.monde.etat.gouttes = 2;
        app.monde.etat.score = 2 * POINTS_GOUTTE;
        app.monde.etat.morts = 3;
        app.reagir(&rien(), &pause(), None);
        app.valider_choix(Choix::Recommencer);
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        assert_eq!(
            app.monde.etat.gouttes, 0,
            "les gouttes du niveau sont à reprendre"
        );
        assert_eq!(app.monde.etat.morts, 3, "les morts restent comptées");
        assert_eq!(app.monde.pas, 0, "niveau tout neuf");
    }

    #[test]
    fn finir_un_niveau_garde_le_score_et_ouvre_le_suivant() {
        let mut app = en_jeu();
        app.monde.etat.gouttes = 4;
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.numero, 2);
        assert_eq!(app.monde.etat.gouttes, 4);
    }

    #[test]
    fn finir_le_dernier_niveau_ouvre_l_ecran_de_fin_avec_le_bilan() {
        let mut app = en_jeu();
        let (monde, camera) = charger(NB_NIVEAUX - 1, None);
        app.monde = monde;
        app.camera = camera;
        app.monde.etat.gouttes = 9;
        app.monde.etat.morts = 2;
        app.monde.etat.score = 900;
        app.pas_partie = 3600;
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| e == Ecran::Fin);
        assert_eq!(
            app.bilan,
            Bilan {
                pas: 3601,
                gouttes: 9,
                morts: 2,
                score: 900
            }
        );
        assert_eq!(app.choix, vec![Choix::Rejouer, Choix::RetourTitre]);
    }

    #[test]
    fn rejouer_remet_la_partie_a_zero() {
        let mut app = en_jeu();
        app.monde.etat.gouttes = 5;
        app.pas_partie = 1234;
        app.monde.termine = false;
        app.ecran = Ecran::Fin;
        app.choix = choix_pour(Ecran::Fin, true);
        app.menu = Menu::new(2);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.numero, 1);
        assert_eq!(app.monde.etat.gouttes, 0);
        assert_eq!(app.pas_partie, 0);
    }

    #[test]
    fn le_raccourci_de_mise_au_point_charge_le_niveau_demande() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Aller(3)),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.numero, 4);
    }

    /// Touche 0 du build de QA : le niveau d'essai vertical (hors de la table), avec sa
    /// caméra qui suit en hauteur ; le finir mène à l'écran de fin (rien après lui).
    #[test]
    fn le_raccourci_d_essai_charge_le_niveau_vertical() {
        let mut app = en_jeu();
        app.reagir(&rien(), &Commandes::default(), Some(RaccourciNiveau::Essai));
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI);
        assert_eq!(app.monde.infos.nom, "Essai vertical");
        assert_eq!(app.monde.niveau.hauteur(), 45);
        // Départ en bas du niveau : caméra sur sa borne basse (720 - 180).
        assert_eq!(app.camera.y, 540.0);
        app.monde.termine = true;
        app.reagir(&valider(), &Commandes::default(), None);
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| e == Ecran::Fin);
    }

    #[test]
    fn les_raccourcis_suivant_et_precedent_parcourent_la_table() {
        let mut app = en_jeu();
        // Au premier niveau, « précédent » ne fait rien (aucune transition).
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Precedent),
        );
        assert!(!app.transition.est_active());
        // « Suivant » jusqu'au dernier niveau, en passant par tous.
        for attendu in 1..NB_NIVEAUX {
            app.reagir(
                &rien(),
                &Commandes::default(),
                Some(RaccourciNiveau::Suivant),
            );
            jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
            app.reagir(&valider(), &Commandes::default(), None);
            assert_eq!(app.monde.indice, attendu);
            assert_eq!(app.ecran, Ecran::Jeu);
        }
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Suivant),
        );
        assert!(!app.transition.est_active(), "rien après le dernier niveau");
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Precedent),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, NB_NIVEAUX - 2);
    }

    /// Chaque niveau terminé mène là où le dit `levels::apres` (avec les gouttes de la
    /// partie) : le niveau suivant, ou l'écran de fin.
    #[test]
    fn la_suite_de_chaque_niveau_suit_la_table() {
        for indice in 0..NB_NIVEAUX {
            for gouttes in [0, levels::gouttes_pour_le_bonus(NIVEAUX)] {
                let mut app = en_jeu();
                let (monde, camera) = charger(indice, None);
                app.monde = monde;
                app.camera = camera;
                app.monde.etat.gouttes = gouttes;
                app.monde.termine = true;
                avancer(&mut app, 1);
                match levels::apres(NIVEAUX, indice, gouttes) {
                    Apres::Niveau(j) => {
                        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
                        assert_eq!(app.monde.indice, j);
                        assert_eq!(app.monde.etat.gouttes, gouttes, "gouttes gardées");
                    }
                    Apres::Fin => jusqu_a(&mut app, |e| e == Ecran::Fin),
                }
            }
        }
    }

    #[test]
    fn rien_ne_reagit_pendant_une_transition() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        assert!(app.transition.est_active());
        // Deuxième appui pendant les volets : ignoré (pas de nouvelle demande).
        app.reagir(
            &valider(),
            &Commandes::default(),
            Some(RaccourciNiveau::Aller(2)),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.numero, 1);
    }

    #[test]
    fn le_chrono_ne_compte_que_le_jeu() {
        let mut app = en_jeu();
        avancer(&mut app, 100);
        assert_eq!(app.pas_partie, 100);
        app.reagir(&rien(), &pause(), None);
        avancer(&mut app, 100);
        assert_eq!(app.pas_partie, 100, "rien ne compte en pause");
    }

    // --- Perte de focus (onglet masqué, autre application, téléphone verrouillé) ---

    #[test]
    fn la_perte_de_focus_en_partie_ouvre_la_pause_et_fige_le_temps() {
        let mut app = en_jeu();
        avancer(&mut app, 30);
        let (pas_monde, chrono) = (app.monde.pas, app.pas_partie);
        app.focus(false);
        assert_eq!(app.ecran, Ecran::Pause);
        assert_eq!(app.choix[app.menu.selection()], Choix::Reprendre);
        // Retour sur l'onglet : la pause RESTE (seul le joueur en sort).
        app.focus(true);
        assert_eq!(app.ecran, Ecran::Pause);
        avancer(&mut app, 600);
        assert_eq!(app.monde.pas, pas_monde, "le monde ne bouge pas");
        assert_eq!(app.pas_partie, chrono, "le chrono ne compte pas l'absence");
        // Reprise : le temps repart exactement d'où il était, sans rattrapage.
        app.reagir(&rien(), &pause(), None);
        assert_eq!(app.ecran, Ecran::Jeu);
        avancer(&mut app, 1);
        assert_eq!(app.monde.pas, pas_monde + 1);
        assert_eq!(app.pas_partie, chrono + 1);
    }

    #[test]
    fn la_pause_ne_s_ouvre_pas_deux_fois() {
        let mut app = en_jeu();
        app.focus(false);
        let bas = Navigation {
            bas: true,
            ..Navigation::default()
        };
        app.reagir(&bas, &Commandes::default(), None);
        let curseur = app.menu.selection();
        assert_eq!(app.choix[curseur], Choix::Recommencer);
        // Nouvelle perte (masqué puis fenêtre quittée...) : l'écran n'est pas rouvert.
        app.focus(false);
        app.focus(false);
        assert_eq!(app.ecran, Ecran::Pause);
        assert_eq!(
            app.menu.selection(),
            curseur,
            "le curseur n'est pas remis en haut"
        );
        // Pause manuelle puis perte de focus : toujours la même pause.
        app.reagir(&rien(), &pause(), None);
        assert_eq!(app.ecran, Ecran::Jeu);
        app.reagir(&rien(), &pause(), None);
        app.focus(false);
        assert_eq!(app.ecran, Ecran::Pause);
    }

    #[test]
    fn la_perte_de_focus_hors_partie_n_ouvre_rien() {
        // Titre
        let mut app = Application::new(true);
        app.focus(false);
        app.focus(true);
        assert_eq!(app.ecran, Ecran::Titre);
        // Aide
        app.valider_choix(Choix::Aide);
        app.focus(false);
        assert_eq!(app.ecran, Ecran::Aide);
        // Fin
        let mut app = en_jeu();
        app.ecran = Ecran::Fin;
        app.focus(false);
        assert_eq!(app.ecran, Ecran::Fin);
        // Volets vers le titre (« quitter ») : rien, ni maintenant ni après.
        let mut app = en_jeu();
        app.reagir(&rien(), &pause(), None);
        app.valider_choix(Choix::QuitterAuTitre);
        app.focus(false);
        jusqu_a(&mut app, |e| e == Ecran::Titre);
        assert!(!app.pause_differee);
    }

    #[test]
    fn focus_perdu_pendant_la_carte_la_pause_attend_le_jeu() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        app.focus(false);
        assert!(
            matches!(app.ecran, Ecran::Carte(_)),
            "pas de pause sur la carte"
        );
        // La carte se termine seule (fenêtre visible mais sans focus) : la pause s'ouvre
        // au premier pas de jeu, AVANT que le monde ne bouge.
        avancer(&mut app, carte::DUREE_PAS as u32 + 2);
        assert_eq!(app.ecran, Ecran::Pause);
        assert_eq!(app.monde.pas, 0);
        assert_eq!(app.pas_partie, 0);
        // Une seule fois : après la reprise, le jeu avance normalement.
        app.reagir(&rien(), &pause(), None);
        avancer(&mut app, 10);
        assert_eq!(app.ecran, Ecran::Jeu);
        assert_eq!(app.monde.pas, 10);
    }

    #[test]
    fn focus_revenu_pendant_la_carte_aucune_pause() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        // Perte pendant les volets, retour pendant la carte (onglet masqué puis revenu).
        app.focus(false);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        app.focus(true);
        avancer(&mut app, carte::DUREE_PAS as u32 + 12);
        assert_eq!(app.ecran, Ecran::Jeu);
        assert!(app.monde.pas > 0);
    }

    #[test]
    fn focus_perdu_pendant_les_volets_de_recommencer() {
        // Volets qui se ferment (la suite « recommencer » est en attente).
        let mut app = en_jeu();
        app.reagir(&rien(), &pause(), None);
        app.valider_choix(Choix::Recommencer);
        app.focus(false);
        // Les volets se referment sur la pause, s'ouvrent sur le niveau tout neuf, puis la
        // pause différée s'ouvre au premier pas de jeu.
        jusqu_a(&mut app, |e| e == Ecran::Pause);
        avancer(&mut app, 30);
        assert_eq!(
            app.ecran,
            Ecran::Pause,
            "le niveau recommencé attend le joueur"
        );
        assert_eq!(app.monde.pas, 0);
        // Volets qui s'OUVRENT sur le niveau recommencé (plus rien en attente).
        app.valider_choix(Choix::Recommencer);
        for _ in 0..200 {
            if app.ecran == Ecran::Jeu {
                break;
            }
            avancer(&mut app, 1);
        }
        assert!(app.ecran == Ecran::Jeu && app.transition.est_active());
        app.focus(false);
        jusqu_a(&mut app, |e| e == Ecran::Pause);
        assert_eq!(app.monde.pas, 0);
    }

    #[test]
    fn focus_perdu_en_allant_vers_la_fin_aucune_pause_plus_tard() {
        let mut app = en_jeu();
        let (monde, camera) = charger(NB_NIVEAUX - 1, None);
        app.monde = monde;
        app.camera = camera;
        app.monde.termine = true;
        avancer(&mut app, 1);
        // Volets vers l'écran de fin : perte de focus, la fenêtre reste sans focus.
        app.focus(false);
        jusqu_a(&mut app, |e| e == Ecran::Fin);
        // « Rejouer » (au clic, sans que le focus soit revenu) : le jeu démarre normalement.
        app.valider_choix(Choix::Rejouer);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        app.reagir(&valider(), &Commandes::default(), None);
        avancer(&mut app, 5);
        assert_eq!(app.ecran, Ecran::Jeu);
        assert_eq!(app.monde.pas, 5);
    }

    #[test]
    fn la_musique_et_la_pluie_s_arretent_pendant_la_pause() {
        let app = Application::new(true);
        assert!(app.boucles_sonores(), "titre");
        let mut app = en_jeu();
        assert!(app.boucles_sonores(), "jeu");
        app.focus(false);
        assert!(!app.boucles_sonores(), "pause automatique");
        app.focus(true);
        assert!(
            !app.boucles_sonores(),
            "retour sur l'onglet : toujours en pause"
        );
        app.reagir(&rien(), &pause(), None);
        assert!(app.boucles_sonores(), "le joueur sort de la pause");
        app.reagir(&rien(), &pause(), None);
        assert!(!app.boucles_sonores(), "pause manuelle");
    }
}
