//! Les transitions entre écrans (carte 10.2b). Un seul chemin change d'écran :
//! `ouvrir` (tout de suite) ou `lancer` (derrière les volets de `crate::transition`).
//! Quand les volets couvrent l'écran, `appliquer_suite` exécute la `Suite` demandée :
//! charger un niveau, recommencer, « plus de vies », histoire, carte, fin, titre.
//!
//! Pour changer le dessin des volets : `crate::transition`. Pour changer où mène
//! une fin de niveau ou un menu : ici (et `progression.rs`, `entrees.rs`).
//!
//! Notion Rust : `Option::take` consomme la demande en attente (`self.suite`), ce
//! qui garantit qu'une `Suite` ne s'applique qu'une fois.

use pepin_core::levels::{self, GenreNiveau};
use pepin_core::state::EtatJeu;

use crate::cosmetiques::Trainees;
use crate::course::{self, Fantome, Force};
use crate::defis::{self, Tentative};
use crate::ecrans::fin::Bilan;
use crate::ecrans::histoire::{self, Chapitre};
use crate::ecrans::{carte, Ecran};
use crate::facile as mode_facile;
use crate::menu::{Choix, Menu};
use crate::particules::Particules;
use crate::plan::{self, EtatPlan};
use crate::sauvegarde::Sauvegarde;

use super::{charger_mode, choix_pour, Application, NOUVELLE_PARTIE_PLUS_OUVERTE};

/// Ce qui doit se produire quand l'écran est entièrement couvert par la transition.
#[derive(Debug, Clone)]
pub(super) enum Suite {
    /// Charger le niveau `indice`. `etat` : partie gardée ; `None` = nouvelle partie.
    Niveau {
        indice: usize,
        etat: Option<EtatJeu>,
    },
    /// Recharger le niveau courant tel qu'il était à son début.
    Recommencer,
    /// Plus de vies (carte 6.2) : recharger le niveau courant depuis son début, avec
    /// `VIES_DEPART` vies (`EtatJeu::apres_plus_de_vies`).
    PlusDeVies,
    /// Une image de l'histoire revue depuis le titre (la suivante, carte 8.4).
    Histoire(Chapitre),
    /// La carte de présentation du niveau déjà chargé, après une image de l'histoire.
    Carte,
    Titre,
    Fin,
    /// L'écran de fin, son bilan déjà fait : après l'image de la fin secrète (carte 8.6).
    EcranFin,
    /// Jouer SEUL le niveau `indice`, choisi sur la carte des niveaux (carte 7.4).
    NiveauSeul(usize),
    /// Courir SEUL le niveau `indice` contre le fantôme de force `force` (carte 8.7) :
    /// un niveau joué seul, avec le fantôme en plus.
    Course {
        indice: usize,
        force: Force,
    },
    /// Revenir à la carte des niveaux (fin ou abandon d'un niveau joué seul).
    Plan,
}

impl Application {
    /// Affiche un écran à menu (ou sans) : reconstruit ses lignes, remet le curseur en haut.
    pub(super) fn ouvrir(&mut self, ecran: Ecran) {
        self.ecran = ecran;
        self.anim = 0;
        self.choix = choix_pour(ecran);
        // AGA : « quand on a fini le monde 1, le niveau d'après [doit être] le premier
        // du monde 2, même si on n'a pas ouvert la carte ». À la fin d'un monde dont le
        // suivant est ouvert (carte 11.2 : tout monde, pas seulement le Monde 1) :
        // CONTINUER en premier (sélectionné), qui enchaîne sur le chapitre suivant. Un
        // bilan de monde 0 (celui par défaut, jamais celui d'une vraie partie) : rien.
        if ecran == Ecran::Fin
            && self.bilan.monde >= 1
            && plan::monde_jouable(
                niveaux::niveaux(),
                &self.sauvegarde,
                self.bilan.monde.saturating_add(1),
            )
        {
            self.choix.insert(0, Choix::Continuer);
        }
        // HISTOIRE n'apparaît au titre qu'une fois le jeu fini (carte 8.11) : avant,
        // ses images se découvrent en jouant. Pas de ligne grisée ni de cadenas.
        // La NOUVELLE PARTIE + aussi (carte 8.8).
        if self.sauvegarde.parties_finies == 0 {
            self.choix
                .retain(|&c| c != Choix::Histoire && c != Choix::NouvellePartiePlus);
        }
        // Pas de ligne MONDE 2 (carte 9.1b, AGA : « c'est juste la suite du jeu ») :
        // JOUER reprend là où on en est (`monde_de_jouer`).
        // NOUVELLE PARTIE + désactivée pour l'instant (AGA), sauf pour le robot de QA.
        if !NOUVELLE_PARTIE_PLUS_OUVERTE && !self.ng_plus_qa {
            self.choix.retain(|&c| c != Choix::NouvellePartiePlus);
        }
        // Course (carte 8.7) : seulement les forces qui ont une partie pour ce niveau.
        let indice = self.plan.selection;
        self.choix.retain(|&c| match c {
            Choix::Force(f) => course::partie(indice, f).is_some(),
            _ => true,
        });
        // Pause d'un niveau joué seul : on quitte vers la carte des niveaux.
        if self.niveau_seul {
            for c in self
                .choix
                .iter_mut()
                .filter(|c| **c == Choix::QuitterAuTitre)
            {
                *c = Choix::Carte;
            }
        }
        self.menu = Menu::new(self.choix.len());
        self.inactif = 0;
        if ecran != Ecran::Demo {
            self.demo = None;
        }
        // Pause, titre, aide ou fin : une pause différée n'a plus lieu d'être.
        self.pause_differee = false;
    }

    /// Ouvre la carte des niveaux, l'étape `selection` choisie.
    pub(super) fn ouvrir_plan(&mut self, selection: usize) {
        self.niveau_seul = false;
        self.fantome = None;
        self.resultat_niveau = None;
        self.plan = EtatPlan::new(niveaux::niveaux(), selection);
        self.ouvrir(Ecran::Plan);
    }

    /// Revient au titre, curseur sur la ligne `choix` (retour de l'aide, de la boutique).
    pub(super) fn retour_titre_sur(&mut self, choix: Choix) {
        self.ouvrir(Ecran::Titre);
        if let Some(i) = self.choix.iter().position(|&c| c == choix) {
            self.menu.choisir(i);
        }
    }

    /// Prépare un changement d'écran derrière la transition en volets.
    pub(super) fn lancer(&mut self, suite: Suite) {
        self.suite = Some(suite);
        self.transition.demarrer();
    }

    /// L'écran est couvert : on change de décor sans que le joueur le voie.
    pub(super) fn appliquer_suite(&mut self) {
        let Some(suite) = self.suite.take() else {
            return;
        };
        self.particules = Particules::new();
        self.trainees = Trainees::new();
        match suite {
            Suite::Niveau { indice, etat } => {
                self.niveau_seul = false;
                self.fantome = None;
                if let (Some(n), true) = (niveaux::niveaux().get(indice), etat.is_some()) {
                    // Atteint pendant une partie : ouvert sur la carte des niveaux.
                    self.sauvegarde.noter_niveau_atteint(n.cle);
                    self.sauvegarde.enregistrer();
                }
                let partie_neuve = etat.is_none();
                if partie_neuve {
                    self.pas_partie = 0;
                    self.gouttes_bonus = None;
                    self.game_over_partie = false;
                    self.resultat_niveau = None;
                    self.facile.dans_la_partie = false;
                }
                self.recharger(indice, etat);
                // AGA : « quand on quitte et qu'on revient sur jouer on revient bien au bon
                // niveau mais on perd l'avancement des gouttes d'or ». Une partie reprise
                // en cours de monde (JOUER reprend où l'on en est) compte d'emblée les
                // gouttes déjà trouvées dans les niveaux d'avant de ce monde (sauvegarde) :
                // le compteur, le bonus et l'écran de fin restent justes.
                if partie_neuve {
                    let deja = gouttes_deja_trouvees(&self.sauvegarde, indice);
                    self.monde.etat.gouttes = deja;
                    self.debut_niveau.gouttes = deja;
                }
                // Une image de l'histoire avant certains niveaux d'une partie lancée par
                // JOUER (carte 8.4) ; chaque niveau n'est chargé ainsi qu'une fois par
                // partie (« recommencer » et « plus de vies » passent par `relancer`),
                // donc chaque image n'est vue qu'une fois.
                match histoire::chapitre_avant(niveaux::niveaux(), indice)
                    .filter(|_| self.partie_racontee)
                {
                    Some(chapitre) => {
                        self.histoire_depuis_titre = false;
                        self.ouvrir(Ecran::Histoire(chapitre));
                    }
                    None => {
                        self.anim = 0;
                        self.ecran = Ecran::Carte(carte::DUREE_PAS);
                    }
                }
            }
            Suite::Carte => {
                self.anim = 0;
                self.ecran = Ecran::Carte(carte::DUREE_PAS);
            }
            Suite::Histoire(chapitre) => self.ouvrir(Ecran::Histoire(chapitre)),
            Suite::Recommencer => {
                // Le niveau repart comme au début, mais les morts restent comptées et
                // les vies perdues restent perdues.
                let etat = self.monde.etat.recommencer_niveau(&self.debut_niveau);
                self.relancer(etat);
                self.ecran = Ecran::Jeu;
            }
            Suite::PlusDeVies => {
                // Pareil, avec `VIES_DEPART` vies : les gouttes de CE niveau sont à
                // reprendre (celles d'avant restent), le chrono continue, les morts
                // restent comptées, le Grand Nuage repart avec tous ses points de vie.
                let etat = self.monde.etat.apres_plus_de_vies(&self.debut_niveau);
                self.relancer(etat);
                self.ecran = Ecran::Jeu;
            }
            Suite::NiveauSeul(indice) => self.niveau_seul(indice, None),
            Suite::Course { indice, force } => self.niveau_seul(indice, Some(force)),
            Suite::Plan => self.ouvrir_plan(self.monde.indice),
            Suite::Titre => {
                self.niveau_seul = false;
                self.fantome = None;
                self.ouvrir(Ecran::Titre)
            }
            Suite::Fin => {
                // Le monde de la partie (carte 9.1) : celui de son dernier niveau.
                let monde = self.monde.infos.monde;
                let e = &self.monde.etat;
                self.bilan = Bilan {
                    pas: self.pas_partie,
                    // Les gouttes du parcours : celles du bonus sont à part.
                    gouttes: e.gouttes.saturating_sub(self.gouttes_bonus.unwrap_or(0)),
                    morts: e.morts,
                    vies: e.vies.restantes(),
                    score: e.score,
                    gouttes_bonus: self.gouttes_bonus,
                    epilogue: self.monde.infos.genre == GenreNiveau::Epilogue,
                    // Les lettres cachées sont celles du Monde 1 (carte 8.6).
                    lettres: (monde == 1).then(|| self.sauvegarde.lettres()),
                    monde,
                };
                // Un boss vaincu en facile (carte 8.14) : partie finie, mais pas de record ;
                // une nouvelle partie + (carte 8.8) a ses records à part.
                // Une partie d'un monde suivant (cartes 9.1 et 11.2) a ses records à
                // part : `partie.mondeN`.
                self.nouveau_record = if monde > 1 {
                    self.sauvegarde
                        .noter_partie_monde(monde, e.score, self.pas_partie)
                } else if self.ng_plus {
                    self.sauvegarde
                        .ng_plus
                        .noter_partie(e.score, self.pas_partie)
                } else {
                    mode_facile::noter_partie(
                        &mut self.sauvegarde,
                        e.score,
                        self.pas_partie,
                        self.facile.dans_la_partie,
                    )
                };
                // Bonus de fin de partie (carte 7.3) : chaque ligne n'est créditée que
                // la première fois qu'elle est gagnée (carte 7.9).
                let parcours = levels::gouttes_du_parcours(levels::niveaux_du_monde(
                    niveaux::niveaux(),
                    monde,
                ));
                let toutes = self.bilan.gouttes >= parcours;
                self.bonus_fin = defis::bonus_fin_du_monde(monde, !self.game_over_partie, toutes);
                defis::crediter_bonus_fin(&mut self.sauvegarde, &mut self.bonus_fin);
                self.sauvegarde.enregistrer();
                // Les cinq lettres réunies (carte 8.6) : la 4e image, puis la fin secrète.
                if self.bilan.fin_secrete() {
                    self.histoire_depuis_titre = false;
                    self.ouvrir(Ecran::Histoire(Chapitre::Secret));
                } else {
                    self.ouvrir(Ecran::Fin);
                }
            }
            Suite::EcranFin => self.ouvrir(Ecran::Fin),
        }
    }

    /// Gouttes d'or ramassées dans le niveau en cours.
    pub(super) fn gouttes_du_niveau(&self) -> u32 {
        self.monde.gouttes_niveau.iter().filter(|&&g| g).count() as u32
    }

    /// Une partie d'un seul niveau, comme une nouvelle partie (5 vies, chrono à zéro) ;
    /// à son drapeau, le bilan puis la carte des niveaux. `course` : la force du
    /// fantôme d'une course contre le robot (carte 8.7), sinon `None`.
    fn niveau_seul(&mut self, indice: usize, course: Option<Force>) {
        self.pas_partie = 0;
        self.gouttes_bonus = None;
        self.game_over_partie = false;
        self.resultat_niveau = None;
        self.resultat_course = None;
        self.facile.dans_la_partie = false;
        self.ng_plus = false;
        self.fantome = course.map(|force| Fantome::new(indice, force));
        self.recharger(indice, None);
        self.niveau_seul = true;
        self.anim = 0;
        self.ecran = Ecran::Carte(carte::DUREE_PAS);
    }

    pub(super) fn recharger(&mut self, indice: usize, etat: Option<EtatJeu>) {
        let (monde, camera) = charger_mode(indice, etat, self.ng_plus);
        self.debut_niveau = monde.etat.clone();
        self.monde = monde;
        self.camera = camera;
        self.eclat_vie = 0;
        // Un niveau chargé : en facile seulement si on vient de le choisir (carte 8.14).
        // C'est une nouvelle VISITE (carte 9.34) : ses défaites repartent de zéro, et
        // un NON d'avant ne tient plus.
        self.facile.actif = std::mem::take(&mut self.facile.au_prochain);
        self.facile.refusee = false;
        self.facile.defaites_visite = 0;
        self.facile.proposition = None;
        if self.facile.actif {
            self.monde.passer_nuage_en_facile();
        }
        self.nouvelle_tentative();
    }

    /// Le niveau (re)commence : une nouvelle tentative pour les défis. Dans une course
    /// (carte 8.7), le fantôme repart aussi de zéro (recommencer, plus de vies) : la
    /// course reste à armes égales.
    fn nouvelle_tentative(&mut self) {
        self.tentative = Tentative::default();
        // Un niveau (re)chargé n'est plus dans la fuite de la Pieuvre (carte 9.36c).
        self.fuite_au_sommet = None;
        self.morts_debut_tentative = self.monde.etat.morts;
        if let Some(f) = &mut self.fantome {
            *f = Fantome::new(self.monde.indice, f.force);
        }
    }

    /// Recharge le niveau EN COURS tout neuf (entités, Grand Nuage, gouttes) avec
    /// l'état `etat` déjà prêt (« recommencer », « plus de vies ») : ce n'est pas un
    /// nouveau niveau, la vie de ses trois gouttes, si elle a été donnée, le reste.
    /// La photo du début du niveau (`debut_niveau`) ne change pas.
    fn relancer(&mut self, etat: EtatJeu) {
        let (mut monde, camera) = charger_mode(self.monde.indice, None, self.ng_plus);
        monde.etat = etat;
        // « Recommencer » et « plus de vies » gardent le mode facile (carte 8.14).
        if self.facile.actif {
            monde.passer_nuage_en_facile();
        }
        self.monde = monde;
        self.camera = camera;
        self.eclat_vie = 0;
        self.nouvelle_tentative();
    }
}

/// Gouttes d'or déjà trouvées (sauvegarde) dans les niveaux NORMAUX du même monde
/// AVANT le niveau `indice` : le point de départ du compteur d'une partie reprise en
/// cours de monde. Le bonus et l'épilogue ne comptent pas (le bonus a son compteur à
/// part). Vaut 0 au premier niveau d'un monde, comme une partie neuve.
fn gouttes_deja_trouvees(s: &Sauvegarde, indice: usize) -> u32 {
    let Some(n) = niveaux::niveaux().get(indice) else {
        return 0;
    };
    niveaux::niveaux()[..indice]
        .iter()
        .filter(|m| m.monde == n.monde && m.genre == GenreNiveau::Normal)
        .map(|m| s.progres(m.cle).gouttes.iter().filter(|&&g| g).count() as u32)
        .sum()
}
