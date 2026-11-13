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
    /// (vitesse et minuteries remises à zéro par `Joueur::placer`) et protégé des
    /// dangers pendant `PROTECTION_REAPPARITION_PAS` pas (`Joueur::proteger`) : un
    /// ennemi qui passe sur la lanterne ne peut plus le tuer en boucle.
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
                Contact::Auvent => {}
                Contact::Mort(_) => {
                    self.morts += 1;
                    joueur.placer(self.checkpoint.0, self.checkpoint.1);
                    joueur.proteger();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;
    use crate::entities::contacts::{resoudre_contacts, verifier_vide, CauseMort};
    use crate::entities::{Actualisable, Ennemi, Entite, Lanterne, TypeEnnemi};
    use crate::physics::Environnement;
    use crate::player::Entrees;

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

    /// Seule une réapparition protège : allumer une lanterne, écraser un ennemi,
    /// rebondir ne donnent rien ; une mort, si. Et le vide tue même protégé.
    #[test]
    fn la_protection_ne_vient_que_d_une_reapparition() {
        let (mut s, mut j) = partie();
        s.traiter(
            &mut j,
            &[
                Contact::Lanterne(200.0, 64.0),
                Contact::Ecrase,
                Contact::Rebond,
                Contact::Goutte(1),
            ],
        );
        assert!(!j.est_protege());
        s.traiter(&mut j, &[Contact::Mort(CauseMort::Ennemi)]);
        assert!(j.est_protege(), "réapparu : protégé");
        // Protégé, il tombe quand même dans le vide : la mort compte.
        j.placer(500.0, 9999.0);
        j.proteger();
        let vide = verifier_vide(&j, 160.0).expect("sous le monde");
        s.traiter(&mut j, &[vide]);
        assert_eq!(s.morts, 2, "le vide reste mortel pendant la protection");
        assert_eq!((j.boite.x, j.boite.y), (200.0, 64.0));
        // En pause : ni réapparition ni protection.
        let (mut s, mut j) = partie();
        s.appliquer(Ordre::BasculerPause).unwrap();
        s.traiter(&mut j, &[Contact::Mort(CauseMort::Pointe)]);
        assert!(!j.est_protege());
    }

    /// Le défaut trouvé par la mesure 3.1 (niveau 2, lanterne de la colonne 104),
    /// rejoué sur un petit toit : Pépin attend sur sa lanterne, une flaque arrive
    /// et le touche. Il réapparaît SUR la lanterne, donc DANS la flaque, qui met
    /// encore près d'une seconde à passer. Sans protection : une mort à chaque pas
    /// de ce passage (jusqu'à 57 d'affilée dans la mesure). Avec : une seule.
    #[test]
    fn une_flaque_qui_passe_sur_la_lanterne_ne_tue_qu_une_fois() {
        // Toit de 20 tuiles entre deux murs, sol en y = 176 (pas d'un pas de jeu
        // complet : mêmes étapes, dans le même ordre, que `Monde::avancer`).
        let mut lignes = vec!["#..................#"; 12];
        lignes[11] = "####################";
        let g = GrilleTuiles::depuis_texte(&lignes);
        let env = Environnement::simple(&g);
        let lanterne = Lanterne::new(100.0, 160.0);
        let point = lanterne.point_de_reapparition();
        let mut entites = vec![
            Entite::Lanterne(lanterne),
            // Flaque posée au sol à droite, elle part vers la gauche (24 px/s).
            Entite::Ennemi(Ennemi::new(TypeEnnemi::Flaque, 200.0, 168.0)),
        ];
        let mut s = EtatJeu::new((40.0, 162.0));
        s.appliquer(Ordre::Demarrer).unwrap();
        let mut j = Joueur::new(point.0, point.1);

        let mut morts_aux_pas = Vec::new();
        let mut dans_la_flaque_protege = 0;
        for pas in 0..400 {
            for e in entites.iter_mut() {
                e.actualiser(&env);
            }
            j.actualiser(&Entrees::default(), &env);
            let mut contacts = resoudre_contacts(&mut j, &mut entites);
            if !contacts.iter().any(|c| matches!(c, Contact::Mort(_))) {
                contacts.extend(verifier_vide(&j, 192.0));
            }
            let avant = s.morts;
            s.traiter(&mut j, &contacts);
            if s.morts > avant {
                morts_aux_pas.push(pas);
            }
            if let Entite::Ennemi(f) = &entites[1] {
                if j.est_protege() && f.boite.chevauche(&j.boite) {
                    dans_la_flaque_protege += 1;
                }
            }
        }
        assert_eq!(s.checkpoint(), point, "la lanterne a été allumée");
        assert_eq!(morts_aux_pas.len(), 1, "une seule mort : {morts_aux_pas:?}");
        assert!(
            dans_la_flaque_protege > 45,
            "la flaque est bien passée sur Pépin réapparu ({dans_la_flaque_protege} pas)"
        );
        assert!(
            matches!(&entites[1], Entite::Ennemi(f) if f.boite.droite() < j.boite.x),
            "la flaque est passée, Pépin est toujours là"
        );
        assert_eq!((j.boite.x, j.boite.bas()), (point.0, 176.0));
    }
}
