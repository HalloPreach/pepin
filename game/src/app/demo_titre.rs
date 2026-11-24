//! La démo du titre (carte 8.1) : après dix secondes sans entrée, le robot joue
//! un niveau, puis on revient au titre. Le robot et ses parcours : `crate::demo`.

use crate::cosmetiques::Trainees;
use crate::demo::{self, Demo, Rejeu};
use crate::ecrans::Ecran;
use crate::effets::effets;
use crate::particules::Particules;

use super::transitions::Suite;
use super::Application;

impl Application {
    /// Dix secondes sans entrée sur le titre : le robot joue le niveau suivant de la
    /// rotation (`demo::PARCOURS`). Rien à jouer (aucun niveau retrouvé) : on reste au
    /// titre, le compteur repart.
    pub(super) fn demarrer_demo(&mut self) {
        self.inactif = 0;
        let Some((rang, indice)) = demo::prochaine(self.rang_demo) else {
            return;
        };
        self.rang_demo = rang + 1;
        // Un monde tout neuf, à part de la sauvegarde : ni défis ni gouttes notés. Le
        // parcours de la démo est celui des niveaux normaux (pas de nouvelle partie +).
        self.ng_plus = false;
        self.recharger(indice, None);
        self.particules = Particules::new();
        self.trainees = Trainees::new();
        self.ouvrir(Ecran::Demo);
        self.demo = Some(Demo {
            rejeu: Rejeu::new(demo::PARCOURS[rang].plages),
            pas: 0,
        });
    }

    /// Un pas de démo : les entrées enregistrées du robot. Drapeau atteint, mort (un
    /// niveau changé depuis l'enregistrement), entrées épuisées ou `DUREE_MAX_PAS`
    /// écoulés : retour au titre derrière les volets.
    pub(super) fn pas_demo(&mut self) {
        let entrees = self.demo.as_mut().and_then(|d| {
            d.pas += 1;
            if d.pas > demo::DUREE_MAX_PAS {
                None
            } else {
                d.rejeu.next()
            }
        });
        let Some(entrees) = entrees else {
            self.lancer(Suite::Titre);
            return;
        };
        let (largeur, hauteur) = (
            self.monde.niveau.largeur_px(),
            self.monde.niveau.hauteur_px(),
        );
        self.monde.avancer(&entrees);
        self.trainees
            .actualiser(self.cosmetiques.trainee, &self.monde);
        self.camera.mer = crate::mer_montee::surface(&self.monde);
        self.camera.suivre(&self.monde.joueur, largeur, hauteur);
        effets(
            &self.monde,
            &mut self.particules,
            &mut self.secousse,
            self.camera.x,
            self.camera.y,
        );
        if self.monde.termine || self.monde.etat.morts > 0 {
            self.lancer(Suite::Titre);
        }
    }

    /// Une entrée pendant la démo : retour immédiat au titre, curseur en haut.
    pub(super) fn quitter_demo(&mut self) {
        self.particules = Particules::new();
        self.ouvrir(Ecran::Titre);
    }
}
