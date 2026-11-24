//! Les réactions aux entrées, UNE fois par image affichée (carte 10.2b) : menus,
//! pause, son, raccourcis de mise au point, réponse au mode facile, perte ou retour
//! du focus. Les appuis eux-mêmes (clavier, toucher, souris) sont lus par
//! `crate::entrees`, `crate::navigation` et `crate::tactile` ; ici on décide ce
//! qu'ils changent dans la partie.
//!
//! Même nom que `crate::entrees` mais pas le même rôle : là-bas on LIT les touches,
//! ici on y RÉAGIT. Dans ce fichier, `entrees::...` désigne `crate::entrees` (importé
//! par `use crate::entrees`), jamais ce sous-module.

use pepin_core::lettres;
use pepin_core::levels;

use crate::boutique::{self, EtatBoutique};
use crate::course::Force;
use crate::ecrans::histoire::{self, Chapitre};
use crate::ecrans::{self, boutique as ecran_boutique, carte, plan as ecran_plan, Ecran};
use crate::entrees::{self, Commandes, RaccourciNiveau};
use crate::facile::{self as mode_facile, Origine, Proposition, Reponse};
use crate::menu::Choix;
use crate::navigation::Navigation;
use crate::plan::{self, ActionPlan, EtatPlan};

use super::transitions::Suite;
use super::Application;

/// Une commande de jeu est-elle tenue (clavier ou boutons tactiles) ? Démo du titre.
fn commandes_actives(c: &Commandes) -> bool {
    c.gauche || c.droite || c.saut || c.pause
}

impl Application {
    /// La fenêtre perd (`present = false`) ou retrouve (`true`) le focus : onglet masqué,
    /// autre application, téléphone verrouillé (voir `focus.rs`).
    ///
    /// - En pleine partie : l'écran de pause s'ouvre (le monde et le chrono s'arrêtent,
    ///   la musique et la pluie aussi) ; seul le joueur en sort.
    /// - Déjà en pause : rien (la pause ne s'ouvre jamais deux fois, le curseur reste).
    /// - Titre, aide, histoire, fin : rien (une image de l'histoire attend un appui). Carte de présentation, écran « plus de vies » ou volets
    ///   qui mènent au jeu : la pause est DIFFÉRÉE au premier pas de jeu, et annulée si le
    ///   focus revient avant (sinon, fenêtre visible mais sans focus, la carte finirait et
    ///   Pépin partirait seul).
    pub fn focus(&mut self, present: bool) {
        if present {
            self.pause_differee = false;
        } else if self.en_jeu() {
            self.ouvrir(Ecran::Pause);
        } else if self.partie_imminente() {
            self.pause_differee = true;
        }
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
        let entree = nav.une_entree() || commandes_actives(commandes);
        // Démo du titre (carte 8.1) : toute entrée ramène au titre, sans rien d'autre
        // (ni bascule du son, ni ligne de menu validée).
        if self.ecran == Ecran::Demo {
            if entree && !self.transition.est_active() {
                self.quitter_demo();
            }
            return;
        }
        if entree {
            self.inactif = 0;
        }
        if nav.son && self.son_disponible {
            self.son = !self.son;
        }
        if self.transition.est_active() {
            return;
        }
        // Bannière du mode facile ouverte (carte 8.14) : elle seule répond.
        if let Some(mut p) = self.facile.proposition {
            let reponse = p.reagir(nav, ecrans::facile::bouton_sous);
            self.facile.proposition = Some(p);
            self.repondre_facile(p.origine, reponse);
            return;
        }
        match self.ecran {
            Ecran::Titre | Ecran::Pause | Ecran::Fin => self.reagir_menu(nav, commandes),
            // Traitée plus haut.
            Ecran::Demo => {}
            Ecran::BilanNiveau | Ecran::BilanCourse => {
                if nav.retour {
                    self.valider_choix(Choix::Carte);
                } else {
                    self.reagir_menu(nav, commandes);
                }
            }
            // Échap : retour à la carte des niveaux, telle qu'on l'a laissée.
            Ecran::ChoixForce => {
                if nav.retour {
                    self.ouvrir(Ecran::Plan);
                } else {
                    self.reagir_menu(nav, commandes);
                }
            }
            Ecran::Plan => {
                let zones = ecran_plan::MiseEnPage;
                match self
                    .plan
                    .reagir(nav, niveaux::niveaux(), &self.sauvegarde, &zones)
                {
                    ActionPlan::Rien => {}
                    ActionPlan::Quitter => self.retour_titre_sur(Choix::Carte),
                    // Boss à proposer en facile (carte 8.14) : la bannière d'abord.
                    ActionPlan::Jouer(indice)
                        if mode_facile::proposer(&plan::progres(
                            niveaux::niveaux(),
                            &self.sauvegarde,
                            indice,
                        )) =>
                    {
                        self.facile.proposition = Some(Proposition::new(Origine::Plan(indice)));
                    }
                    ActionPlan::Jouer(indice) => self.lancer(Suite::NiveauSeul(indice)),
                    // `plan.selection` est déjà l'étape à courir.
                    ActionPlan::Course(_) => self.ouvrir(Ecran::ChoixForce),
                }
            }
            Ecran::Aide => {
                if nav.retour || nav.valider || nav.clic.is_some() {
                    self.retour_titre_sur(Choix::Aide);
                }
            }
            Ecran::Boutique => {
                let zones = ecran_boutique::MiseEnPage;
                match self.boutique.reagir(nav, &mut self.sauvegarde, &zones) {
                    boutique::Action::Rien => {}
                    boutique::Action::Quitter => self.retour_titre_sur(Choix::Boutique),
                    boutique::Action::Enregistrer => {
                        self.sauvegarde.enregistrer();
                        self.appliquer_skin();
                    }
                }
            }
            Ecran::Histoire(chapitre) => {
                if nav.retour && self.histoire_depuis_titre {
                    // Revue depuis le titre : Échap y revient tout de suite.
                    self.retour_titre_sur(Choix::Histoire);
                } else if histoire::prete(self.anim)
                    && (nav.valider || nav.retour || nav.clic.is_some())
                {
                    self.apres_histoire(chapitre);
                }
            }
            // Carte 9.31 : la carte du niveau et « PLUS DE VIES » ne se passent plus
            // seules ; elles attendent un appui, ignoré pendant le court délai de
            // l'histoire (un saut tenu ne les fait pas disparaître sans être lues).
            Ecran::Carte(_) => {
                if histoire::prete(self.anim) && (nav.valider || nav.clic.is_some()) {
                    // Le jeu commence au prochain pas (`pas`).
                    self.ecran = Ecran::Carte(carte::PASSEE);
                }
            }
            Ecran::PlusDeVies(_) => {
                if commandes.pause {
                    // Pause demandée : elle s'ouvrira sur le niveau recommencé, avant
                    // son premier pas (l'écran, lui, est déjà à l'arrêt).
                    self.pause_differee = true;
                } else if histoire::prete(self.anim) && (nav.valider || nav.clic.is_some()) {
                    self.lancer(Suite::PlusDeVies);
                }
            }
            Ecran::Jeu => {
                if commandes.pause {
                    self.ouvrir(Ecran::Pause);
                } else if let Some(indice) = raccourci.and_then(|r| {
                    entrees::cible_raccourci(r, self.monde.indice, niveaux::NB_NIVEAUX)
                }) {
                    // Raccourci du build de QA : une partie sans histoire.
                    self.partie_racontee = false;
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
        // Clic ou toucher sur l'icône du son (titre, pause ; carte 7.8b) : même bascule
        // que la touche M, et rien d'autre (aucune ligne de menu n'est là).
        if let Some((x, y)) = nav.clic {
            if self.son_disponible
                && matches!(self.ecran, Ecran::Titre | Ecran::Pause)
                && ecrans::sur_icone_son(x, y)
            {
                self.son = !self.son;
                return;
            }
        }
        if nav.haut {
            self.menu.monter();
        }
        if nav.bas {
            self.menu.descendre();
        }
        let mut valider = nav.valider;
        let nb = self.choix.len();
        if let Some(d) = ecrans::disposition_menu(self.ecran, nb) {
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

    /// Une image de l'histoire est passée : la suivante (ou le titre) si elle a été
    /// ouverte depuis le titre, sinon la carte du niveau qui l'attend ; après la fin
    /// secrète (carte 8.6), l'écran de fin. Depuis le titre, la fin secrète suit la
    /// 3e image une fois les cinq lettres réunies.
    fn apres_histoire(&mut self, chapitre: Chapitre) {
        let secrete = lettres::manquantes(&self.sauvegarde.lettres()) == 0;
        let suivant = chapitre
            .suivant()
            .or_else(|| (chapitre == Chapitre::Soleil && secrete).then_some(Chapitre::Secret));
        if chapitre == Chapitre::Secret && !self.histoire_depuis_titre {
            self.lancer(Suite::EcranFin);
        } else if !self.histoire_depuis_titre {
            self.lancer(Suite::Carte);
        } else if let Some(suivant) = suivant {
            self.lancer(Suite::Histoire(suivant));
        } else {
            self.retour_titre_sur(Choix::Histoire);
        }
    }

    pub(super) fn valider_choix(&mut self, choix: Choix) {
        // CONTINUER (écran de fin du Monde 1) : exactement JOUER, qui part dans le
        // Monde 2 dès que sa porte est ouverte, là où l'on en est (niveau 1 la
        // première fois).
        let choix = if choix == Choix::Continuer {
            Choix::Jouer
        } else {
            choix
        };
        match choix {
            // REJOUER garde le mode de la partie finie ; JOUER : une partie normale ;
            // NOUVELLE PARTIE + : la même, variantes et pluie forte (carte 8.8), sans
            // les images de l'histoire (déjà vues).
            // JOUER reprend LÀ OÙ ON EN EST (carte 9.1b, `monde_de_jouer`) : le Monde 1
            // tant que sa porte est fermée, puis le Monde 2 (sans variante ni image de
            // l'histoire du Monde 1) ; REJOUER garde le monde de la partie finie.
            Choix::Jouer | Choix::Rejouer | Choix::NouvellePartiePlus => {
                let monde = match choix {
                    Choix::Jouer => plan::selon_la_vraie_progression(|| self.monde_de_jouer()),
                    Choix::Rejouer => self.monde.infos.monde,
                    _ => 1,
                };

                match choix {
                    Choix::Jouer => self.ng_plus = false,
                    Choix::NouvellePartiePlus => self.ng_plus = true,
                    _ => {}
                }
                // AGA (2026-11-19) : « quand je fais jouer ça me mets au niveau 1 du
                // monde 2 alors que sur la carte j'ai le niveau 3 du monde 2 de fini »,
                // puis « je veux qu'il reprenne ou j'en suis quoi qu'il arrive ».
                // JOUER reprend donc TOUJOURS au niveau le plus avancé déjà ouvert (celui
                // que la carte montre comme « où l'on en est »), dans les deux mondes.
                // REJOUER et NOUVELLE PARTIE + repartent du début du monde.
                let debut = levels::debut_monde(niveaux::niveaux(), monde).unwrap_or(0);
                let indice = if choix == Choix::Jouer {
                    plan::selon_la_vraie_progression(|| {
                        plan::EtatPlan::ou_on_en_est_du_monde(
                            niveaux::niveaux(),
                            &self.sauvegarde,
                            monde,
                        )
                    })
                } else {
                    debut
                };
                // Les images de l'histoire racontent le départ du Monde 1 : seulement
                // quand JOUER commence vraiment au tout premier niveau.
                self.partie_racontee = choix == Choix::Jouer && monde == 1 && indice == debut;
                self.lancer(Suite::Niveau { indice, etat: None });
            }
            Choix::Histoire => {
                self.histoire_depuis_titre = true;
                self.ouvrir(Ecran::Histoire(Chapitre::Depart));
            }
            Choix::Boutique => {
                self.boutique = EtatBoutique::new();
                self.ouvrir(Ecran::Boutique);
            }
            Choix::Aide => self.ouvrir(Ecran::Aide),
            // Depuis le titre : la carte, sur le niveau où l'on en est. Depuis la pause
            // ou le bilan d'un niveau joué seul : retour à la carte, derrière les volets.
            Choix::Carte if self.ecran == Ecran::Titre => {
                self.ouvrir_plan(EtatPlan::ou_on_en_est(niveaux::niveaux(), &self.sauvegarde))
            }
            // Choix de la force (carte 8.7) : la carte telle qu'on l'a laissée.
            Choix::Carte if self.ecran == Ecran::ChoixForce => self.ouvrir(Ecran::Plan),
            Choix::Carte => self.lancer(Suite::Plan),
            Choix::RejouerNiveau => self.lancer(Suite::NiveauSeul(self.monde.indice)),
            Choix::Force(force) => self.lancer(Suite::Course {
                indice: self.plan.selection,
                force,
            }),
            Choix::RejouerCourse => self.lancer(Suite::Course {
                indice: self.monde.indice,
                force: self.resultat_course.map_or(Force::Debutant, |r| r.force),
            }),
            Choix::Reprendre => self.ecran = Ecran::Jeu,
            Choix::Recommencer => self.lancer(Suite::Recommencer),
            // Déjà ramené à JOUER en tête de fonction.
            Choix::Continuer => {}
            Choix::QuitterAuTitre | Choix::RetourTitre => self.lancer(Suite::Titre),
        }
    }

    /// La réponse à la bannière. Après une mort : OUI recommence le niveau en facile,
    /// NON reprend le combat tel quel. Sur la carte des niveaux : OUI ou NON lance le
    /// niveau (en facile ou non), Échap revient à la carte.
    fn repondre_facile(&mut self, origine: Origine, reponse: Reponse) {
        if reponse == Reponse::Rien {
            return;
        }
        self.facile.proposition = None;
        match (origine, reponse) {
            (Origine::Mort, Reponse::Oui) => {
                self.facile.actif = true;
                self.lancer(Suite::Recommencer);
            }
            (Origine::Mort, _) => self.facile.refusee = true,
            (Origine::Plan(indice), Reponse::Oui | Reponse::Non) => {
                self.facile.au_prochain = reponse == Reponse::Oui;
                self.lancer(Suite::NiveauSeul(indice));
            }
            (Origine::Plan(_), _) => {}
        }
    }
}
