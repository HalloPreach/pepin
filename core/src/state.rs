//! Module `state` : l'état global de la partie (score, checkpoint, phase du jeu).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `enum` pour une MACHINE À ÉTATS (`Phase`) : une partie est au titre, en jeu,
//!   en pause OU finie, jamais deux à la fois. Les transitions interdites sont
//!   refusées par un `Result` plutôt que de corrompre l'état ;
//! - `Result<T, E>` avec une erreur typée (`TransitionInvalide`) ;
//! - ownership : `EtatJeu` possède ses données ; `traiter` emprunte le joueur en
//!   `&mut` juste le temps de le replacer au dernier checkpoint.
//!
//! Choix de conception : le respawn est INSTANTANÉ et les essais illimités (pas de
//! « game over ») ; on compte simplement les morts.

use crate::entities::contacts::Contact;
use crate::player::Joueur;

/// Points gagnés pour une goutte d'or.
pub const POINTS_GOUTTE: u32 = 100;
/// Points gagnés pour un ennemi écrasé.
pub const POINTS_ECRASEMENT: u32 = 50;

/// Phase de la partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Titre,
    EnJeu,
    Pause,
    Fin,
}

/// Ordre qui peut faire changer la phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordre {
    /// Titre -> EnJeu.
    Demarrer,
    /// EnJeu <-> Pause.
    BasculerPause,
    /// EnJeu -> Fin (la sortie du dernier niveau est atteinte).
    Terminer,
    /// Fin -> Titre (nouvelle partie).
    Recommencer,
}

/// Erreur : l'ordre n'a pas de sens dans la phase actuelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionInvalide {
    pub depuis: Phase,
    pub ordre: Ordre,
}

/// L'état d'une partie.
#[derive(Debug, Clone)]
pub struct EtatJeu {
    phase: Phase,
    /// Score en points.
    pub score: u32,
    /// Nombre de gouttes d'or ramassées depuis le début de la partie.
    pub gouttes: u32,
    /// Nombre de morts depuis le début de la partie.
    pub morts: u32,
    /// Point de réapparition `(x, y)` (coin haut-gauche du joueur).
    checkpoint: (f32, f32),
    /// Point de départ du niveau courant, pour `Recommencer`.
    depart: (f32, f32),
}

impl EtatJeu {
    /// Nouvelle partie, sur l'écran titre, avec le point de départ du premier niveau.
    pub fn new(depart: (f32, f32)) -> Self {
        Self {
            phase: Phase::Titre,
            score: 0,
            gouttes: 0,
            morts: 0,
            checkpoint: depart,
            depart,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn checkpoint(&self) -> (f32, f32) {
        self.checkpoint
    }

    /// Les entités doivent-elles avancer ? Seulement en jeu.
    pub fn en_jeu(&self) -> bool {
        self.phase == Phase::EnJeu
    }

    /// Applique un ordre. `Ok(nouvelle_phase)` ou `Err` si la transition est interdite
    /// (l'état reste alors inchangé).
    pub fn appliquer(&mut self, ordre: Ordre) -> Result<Phase, TransitionInvalide> {
        let suivante = match (self.phase, ordre) {
            (Phase::Titre, Ordre::Demarrer) => Phase::EnJeu,
            (Phase::EnJeu, Ordre::BasculerPause) => Phase::Pause,
            (Phase::Pause, Ordre::BasculerPause) => Phase::EnJeu,
            (Phase::EnJeu, Ordre::Terminer) => Phase::Fin,
            (Phase::Fin, Ordre::Recommencer) => Phase::Titre,
            (depuis, ordre) => return Err(TransitionInvalide { depuis, ordre }),
        };
        if ordre == Ordre::Recommencer {
            *self = EtatJeu::new(self.depart);
        }
        self.phase = suivante;
        Ok(suivante)
    }

    /// Change de niveau : nouveau départ, nouveau checkpoint. Le score est conservé.
    pub fn nouveau_niveau(&mut self, depart: (f32, f32)) {
        self.depart = depart;
        self.checkpoint = depart;
    }

    /// Digère les contacts d'un pas de simulation. Sans effet hors de la phase `EnJeu`.
    ///
    /// En cas de mort, le joueur est replacé IMMÉDIATEMENT au dernier checkpoint
    /// (vitesse et minuteries remises à zéro par `Joueur::placer`).
    pub fn traiter(&mut self, joueur: &mut Joueur, contacts: &[Contact]) {
        if self.phase != Phase::EnJeu {
            return;
        }
        for contact in contacts {
            match *contact {
                Contact::Goutte(_) => {
                    self.gouttes += 1;
                    self.score += POINTS_GOUTTE;
                }
                Contact::Lanterne(x, y) => self.checkpoint = (x, y),
                Contact::Ecrase => self.score += POINTS_ECRASEMENT,
                Contact::Rebond => {}
                Contact::Mort(_) => {
                    self.morts += 1;
                    joueur.placer(self.checkpoint.0, self.checkpoint.1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::contacts::CauseMort;

    fn partie() -> (EtatJeu, Joueur) {
        let mut s = EtatJeu::new((16.0, 32.0));
        s.appliquer(Ordre::Demarrer).unwrap();
        (s, Joueur::new(16.0, 32.0))
    }

    #[test]
    fn on_commence_au_titre() {
        let s = EtatJeu::new((0.0, 0.0));
        assert_eq!(s.phase(), Phase::Titre);
        assert!(!s.en_jeu());
    }

    #[test]
    fn cycle_complet_des_phases() {
        let mut s = EtatJeu::new((0.0, 0.0));
        assert_eq!(s.appliquer(Ordre::Demarrer), Ok(Phase::EnJeu));
        assert_eq!(s.appliquer(Ordre::BasculerPause), Ok(Phase::Pause));
        assert_eq!(s.appliquer(Ordre::BasculerPause), Ok(Phase::EnJeu));
        assert_eq!(s.appliquer(Ordre::Terminer), Ok(Phase::Fin));
        assert_eq!(s.appliquer(Ordre::Recommencer), Ok(Phase::Titre));
    }

    #[test]
    fn transitions_interdites_refusees_sans_effet() {
        let mut s = EtatJeu::new((0.0, 0.0));
        assert_eq!(
            s.appliquer(Ordre::BasculerPause),
            Err(TransitionInvalide {
                depuis: Phase::Titre,
                ordre: Ordre::BasculerPause
            })
        );
        assert!(s.appliquer(Ordre::Terminer).is_err());
        assert_eq!(s.phase(), Phase::Titre);
        s.appliquer(Ordre::Demarrer).unwrap();
        assert!(s.appliquer(Ordre::Demarrer).is_err());
        assert!(s.appliquer(Ordre::Recommencer).is_err());
        assert_eq!(s.phase(), Phase::EnJeu);
    }

    #[test]
    fn recommencer_remet_les_compteurs_a_zero() {
        let (mut s, mut j) = partie();
        s.traiter(
            &mut j,
            &[Contact::Goutte(0), Contact::Mort(CauseMort::Vide)],
        );
        s.appliquer(Ordre::Terminer).unwrap();
        s.appliquer(Ordre::Recommencer).unwrap();
        assert_eq!((s.score, s.gouttes, s.morts), (0, 0, 0));
        assert_eq!(s.checkpoint(), (16.0, 32.0));
    }

    #[test]
    fn score_gouttes_et_ecrasements() {
        let (mut s, mut j) = partie();
        s.traiter(
            &mut j,
            &[Contact::Goutte(0), Contact::Ecrase, Contact::Rebond],
        );
        assert_eq!(s.score, POINTS_GOUTTE + POINTS_ECRASEMENT);
        assert_eq!(s.gouttes, 1);
        assert_eq!(s.morts, 0);
    }

    #[test]
    fn mort_et_respawn_au_depart_puis_au_checkpoint() {
        let (mut s, mut j) = partie();
        j.placer(300.0, 50.0);
        j.vitesse.y = 400.0;
        s.traiter(&mut j, &[Contact::Mort(CauseMort::Pointe)]);
        assert_eq!((j.boite.x, j.boite.y), (16.0, 32.0), "retour au départ");
        assert_eq!(j.vitesse.y, 0.0);
        assert_eq!(s.morts, 1);

        s.traiter(&mut j, &[Contact::Lanterne(200.0, 64.0)]);
        assert_eq!(s.checkpoint(), (200.0, 64.0));
        j.placer(500.0, 10.0);
        s.traiter(&mut j, &[Contact::Mort(CauseMort::Vide)]);
        assert_eq!(
            (j.boite.x, j.boite.y),
            (200.0, 64.0),
            "retour à la lanterne"
        );
        assert_eq!(s.morts, 2);
    }

    #[test]
    fn rien_ne_compte_hors_de_la_phase_en_jeu() {
        let (mut s, mut j) = partie();
        s.appliquer(Ordre::BasculerPause).unwrap();
        j.placer(300.0, 50.0);
        s.traiter(
            &mut j,
            &[Contact::Goutte(0), Contact::Mort(CauseMort::Vide)],
        );
        assert_eq!((s.score, s.morts), (0, 0));
        assert_eq!(j.boite.x, 300.0, "pas de respawn en pause");
    }

    #[test]
    fn nouveau_niveau_conserve_le_score() {
        let (mut s, mut j) = partie();
        s.traiter(&mut j, &[Contact::Goutte(0)]);
        s.nouveau_niveau((48.0, 80.0));
        assert_eq!(s.score, POINTS_GOUTTE);
        assert_eq!(s.checkpoint(), (48.0, 80.0));
        j.placer(900.0, 0.0);
        s.traiter(&mut j, &[Contact::Mort(CauseMort::Ennemi)]);
        assert_eq!((j.boite.x, j.boite.y), (48.0, 80.0));
    }
}
