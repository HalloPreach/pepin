//! `Application` : la machine à écrans du jeu (titre, aide, histoire, carte de niveau,
//! jeu, pause, « plus de vies », fin, carte des niveaux et bilan d'un niveau joué
//! seul) et tout ce qui les relie (transitions, bilan, rejouer, recommencer le niveau
//! quand Pépin n'a plus de vies).
//!
//! Deux sortes de parties (carte 7.4) : « JOUER » enchaîne tous les niveaux (records
//! de partie, bonus de fin) ; un niveau choisi sur la carte des niveaux se joue SEUL
//! (`niveau_seul`) : mêmes vies, défis, médailles et gouttes, mais au drapeau on voit
//! son bilan (`Ecran::BilanNiveau`) puis on revient à la carte, sans toucher aux
//! records de partie.
//!
//! Deux rythmes cohabitent :
//! - `reagir` est appelé UNE fois par image affichée : il lit les appuis
//!   ponctuels (menus, pause) qu'un pas de simulation pourrait manquer ;
//! - `pas` est appelé à chaque pas FIXE de 1/60 s : il fait vivre le monde.
//!
//! Rien ici n'appelle macroquad hors de `dessiner` : toute la logique (qui mène
//! où, que devient le score...) se teste donc sans fenêtre.
//!
//! CARTE DES RESPONSABILITÉS (carte 10.2b). Chaque sous-module ajoute son propre bloc
//! `impl Application` ; les champs restent déclarés ici, une seule fois.
//! - `mod.rs` (ce fichier) : l'état (`Application`), le chargement d'un niveau,
//!   les menus de chaque écran (`choix_pour`) et la boucle : `pas`, appelé 60 fois
//!   par seconde, qui choisit quoi faire selon l'écran.
//! - `entrees` : `reagir`, une fois par image (menus, pause, son, raccourcis, focus).
//! - `progression` : un pas du niveau en cours (fin de niveau, gouttes, défis, plus
//!   de vies, mode facile du Grand Nuage).
//! - `camera_effets` : la caméra et les effets visuels branchés sur ce pas.
//! - `transitions` : `ouvrir`, `lancer` et `appliquer_suite`, le seul chemin qui
//!   change d'écran ; la `Suite` à appliquer derrière les volets.
//! - `demo_titre` : la démo du robot sur l'écran titre.
//! - `rendu` : `dessiner`, le seul endroit qui appelle macroquad.
//! - `tests` : les tests de la machine à écrans.
//!
//! Notions Rust illustrées ici : machine à états avec `enum` (`Ecran`, `Suite`),
//! `Option::take` pour consommer une demande en attente, `Vec<Choix>` reconstruit
//! à chaque écran, méthodes qui ne se contredisent pas (un seul endroit change
//! d'écran).

use pepin_core::player::Entrees;
use pepin_core::state::EtatJeu;

use crate::boutique::EtatBoutique;
use crate::camera::Camera;
use crate::cosmetiques::{self, Trainees};
use crate::course::{Fantome, Force, ResultatCourse};
use crate::defis::{self, LigneBonus, ResultatNiveau, Tentative};
use crate::demo::{self, Demo};
use crate::ecrans::bilan_course::TempsCourse;
use crate::ecrans::bilan_niveau::Temps;
use crate::ecrans::fin::{self, Bilan};
#[cfg(test)]
use crate::ecrans::histoire;
use crate::ecrans::{carte, Ecran};
use crate::facile::EtatFacile;
use crate::menu::{Choix, Menu};
use crate::monde::Monde;
use crate::particules::Particules;
use crate::plan::{self, EtatPlan};
use crate::sauvegarde::Sauvegarde;
use crate::secousse::Secousse;
use crate::skins::Skin;
use crate::transition::Transition;

mod camera_effets;
mod demo_titre;
mod entrees;
mod progression;
mod rendu;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_facile;
mod transitions;

use transitions::Suite;

/// Charge le niveau `indice` en conservant (ou non) le score de la partie.
pub fn charger(indice: usize, garder: Option<EtatJeu>) -> (Monde, Camera) {
    charger_mode(indice, garder, false)
}

/// Comme `charger`, en nouvelle partie + si `ng_plus` (carte 8.8).
pub fn charger_mode(indice: usize, garder: Option<EtatJeu>, ng_plus: bool) -> (Monde, Camera) {
    let mut monde = Monde::charger(indice, ng_plus);
    // La partie joue le temps de mort (carte 9.24), pas les simulations.
    monde.temps_de_mort = true;
    if let Some(mut etat) = garder {
        // Un NOUVEAU niveau : nouveau départ, et sa vie des trois gouttes à prendre.
        etat.nouveau_niveau(monde.niveau.depart, monde.infos.genre);
        monde.etat = etat;
    }
    let camera = Camera::new(
        &monde.joueur,
        monde.niveau.largeur_px(),
        monde.niveau.hauteur_px(),
    );
    (monde, camera)
}

/// NOUVELLE PARTIE + (carte 8.8) ouverte aux joueurs ? Désactivée pour l'instant à la
/// demande d'AGA : la ligne disparaît du titre, même une fois le jeu fini. Le code, les
/// variantes et les records restent en place ; repasser à `true` suffit pour la rouvrir.
pub const NOUVELLE_PARTIE_PLUS_OUVERTE: bool = false;

/// Les lignes du menu d'un écran, dans l'ordre. Le son n'a plus de ligne depuis la
/// carte 7.8b : c'est une icône en bas à gauche du titre et de la pause (clic ou M).
/// Le titre perd HISTOIRE tant qu'aucune partie n'est finie (`Application::ouvrir`).
pub fn choix_pour(ecran: Ecran) -> Vec<Choix> {
    match ecran {
        Ecran::Titre => vec![
            Choix::Jouer,
            Choix::NouvellePartiePlus,
            Choix::Carte,
            Choix::Boutique,
            Choix::Histoire,
            Choix::Aide,
        ],
        Ecran::Pause => vec![Choix::Reprendre, Choix::Recommencer, Choix::QuitterAuTitre],
        Ecran::Fin => vec![Choix::Rejouer, Choix::RetourTitre],
        Ecran::BilanNiveau => vec![Choix::RejouerNiveau, Choix::Carte],
        Ecran::BilanCourse => vec![Choix::RejouerCourse, Choix::Carte],
        // Les forces sans partie embarquée sont retirées à l'ouverture (`ouvrir`).
        Ecran::ChoixForce => vec![
            Choix::Force(Force::Debutant),
            Choix::Force(Force::Habitue),
            Choix::Force(Force::Expert),
            Choix::Carte,
        ],
        Ecran::Aide
        | Ecran::Boutique
        | Ecran::Histoire(_)
        | Ecran::Carte(_)
        | Ecran::Jeu
        | Ecran::PlusDeVies(_)
        | Ecran::Plan
        | Ecran::Demo => Vec::new(),
    }
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
    /// Gouttes ramassées dans le niveau bonus, une fois celui-ci fini (`None` : pas
    /// joué dans cette partie). Le compteur du jeu et l'écran de fin les comptent à
    /// part (carte 5.17).
    gouttes_bonus: Option<u32>,
    /// Le focus a été perdu juste avant que le jeu ne (re)démarre (carte de présentation,
    /// volets) : la pause s'ouvrira au premier pas de jeu, sauf si le focus revient avant.
    pause_differee: bool,
    /// Pas restants du petit signe d'une vie gagnée dans le HUD (0 : aucun).
    eclat_vie: u16,
    /// Porte-monnaie de gouttes et records, gardés d'une partie à l'autre (carte 7.2).
    pub sauvegarde: Sauvegarde,
    /// La partie qui vient de finir a battu le record de score (écran de fin).
    nouveau_record: bool,
    /// Tenue de Pépin (carte 7.1) : celle choisie dans la boutique et gardée par la
    /// sauvegarde (carte 7.5), Pépin par défaut. La touche K du build de QA la change
    /// aussi, sans l'enregistrer.
    pub skin: Skin,
    /// Traînée, motif de parapluie et musique choisis dans la boutique (carte 7.6),
    /// gardés par la sauvegarde. Les touches T et Y du build de QA les changent aussi,
    /// sans les enregistrer.
    pub cosmetiques: cosmetiques::Choix,
    /// Les brins de la traînée derrière Pépin (rendu seulement).
    trainees: Trainees,
    /// L'écran boutique : onglet, ligne choisie, dialogue (carte 7.5).
    boutique: EtatBoutique,
    /// Tentative en cours du niveau (carte 7.3) : ce que les défis regardent. Remise à
    /// zéro à chaque (re)chargement du niveau.
    tentative: Tentative,
    /// Morts de la partie au début de la tentative (les morts d'une tentative sont
    /// la différence).
    morts_debut_tentative: u32,
    /// Défis du dernier niveau réussi : montrés sous la carte du niveau suivant, puis
    /// sur l'écran de fin (carte 7.3).
    resultat_niveau: Option<ResultatNiveau>,
    /// Un « plus de vies » a eu lieu dans la partie (bonus de fin « sans game over »).
    game_over_partie: bool,
    /// Bonus de la partie qui vient de finir (écran de fin).
    bonus_fin: [LigneBonus; 3],
    /// La carte des niveaux : étape choisie et défilement (carte 7.4).
    plan: EtatPlan,
    /// La partie en cours est un niveau joué SEUL depuis la carte (carte 7.4).
    niveau_seul: bool,
    /// Bilan d'un niveau joué seul : son temps et le meilleur temps d'avant.
    temps_niveau: Temps,
    /// La partie en cours raconte-t-elle l'histoire (carte 8.4) ? Oui si elle a été
    /// lancée par JOUER ; non par REJOUER (l'histoire vient d'être vue jusqu'au bout)
    /// ni par un raccourci de niveau du build de QA.
    partie_racontee: bool,
    /// L'image de l'histoire affichée a été ouverte depuis le titre (ligne HISTOIRE) :
    /// elle mène à la suivante puis au titre, et non à un niveau.
    histoire_depuis_titre: bool,
    /// Pas passés sur le titre sans aucune entrée : à `demo::ATTENTE_PAS`, la démo
    /// démarre (carte 8.1).
    inactif: u32,
    /// La démo en cours (écran `Demo`) : le rejeu du parcours embarqué.
    demo: Option<Demo>,
    /// Rang, dans `demo::PARCOURS`, de la prochaine démo (elles tournent).
    rang_demo: usize,
    /// Mode facile du Grand Nuage (carte 8.14) : niveau en facile, bannière ouverte.
    facile: EtatFacile,
    /// Course contre le robot (carte 8.7) : le fantôme de la course en cours (`None` :
    /// pas de course ; la partie est alors un niveau seul ordinaire ou une partie).
    fantome: Option<Fantome>,
    /// Bilan de la dernière course : issue, meilleur résultat d'avant, prime.
    resultat_course: Option<ResultatCourse>,
    temps_course: TempsCourse,
    /// La partie en cours est une NOUVELLE PARTIE + (carte 8.8) : chaque niveau est
    /// chargé avec sa variante ; ses records sont à part (`Sauvegarde::ng_plus`).
    pub ng_plus: bool,
    /// Robot de QA (carte 8.8) : rouvre la NOUVELLE PARTIE + même quand
    /// `NOUVELLE_PARTIE_PLUS_OUVERTE` est faux, pour continuer à la vérifier.
    ng_plus_qa: bool,
}

impl Application {
    /// `son_disponible` : afficher l'icône du son (titre, pause) et réagir à M et au clic.
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
            gouttes_bonus: None,
            pause_differee: false,
            eclat_vie: 0,
            sauvegarde: Sauvegarde::charger(),
            nouveau_record: false,
            skin: Skin::default(),
            cosmetiques: cosmetiques::Choix::default(),
            trainees: Trainees::new(),
            boutique: EtatBoutique::new(),
            tentative: Tentative::default(),
            morts_debut_tentative: 0,
            resultat_niveau: None,
            game_over_partie: false,
            bonus_fin: defis::bonus_fin(false, false),
            plan: EtatPlan::new(niveaux::niveaux(), 0),
            niveau_seul: false,
            temps_niveau: Temps::default(),
            partie_racontee: false,
            histoire_depuis_titre: false,
            inactif: 0,
            demo: None,
            rang_demo: 0,
            facile: EtatFacile::default(),
            fantome: None,
            resultat_course: None,
            temps_course: TempsCourse::default(),
            ng_plus: false,
            ng_plus_qa: false,
        };
        app.appliquer_skin();
        app.ouvrir(Ecran::Titre);
        app
    }

    /// Touche U du build de QA (carte 7.4) : progression avancée, enregistrée ; la
    /// carte des niveaux ouverte se met à jour.
    pub fn progression_qa(&mut self) {
        plan::progression_qa(niveaux::niveaux(), &mut self.sauvegarde);
        self.sauvegarde.enregistrer();
    }

    /// Build de QA (carte 10.1b, `?essai=1`) : lance tout de suite le niveau posé
    /// dans le navigateur (`monde::INDICE_ESSAI_EXTERNE`), comme un raccourci de
    /// niveau : une partie sans histoire.
    #[cfg(feature = "debug_niveaux")]
    pub fn lancer_essai_externe(&mut self) {
        self.partie_racontee = false;
        self.lancer(Suite::Niveau {
            indice: crate::monde::INDICE_ESSAI_EXTERNE,
            etat: None,
        });
    }

    /// Le skin et les cosmétiques choisis dans la sauvegarde deviennent ceux du rendu
    /// (inconnu : Pépin, rien).
    fn appliquer_skin(&mut self) {
        self.skin = Skin::depuis_id(&self.sauvegarde.skin_choisi).unwrap_or_default();
        self.cosmetiques = cosmetiques::Choix::depuis(&self.sauvegarde);
    }

    /// Le monde que lance JOUER au titre (carte 9.1b, AGA : « c'est juste la suite du
    /// jeu ») : le dernier monde jouable (`plan::dernier_monde_jouable`, carte 11.2),
    /// donc le Monde 2 dès que sa porte est ouverte (« poursuite » fini), le Monde 1
    /// sinon, exactement comme avant. Aucune ligne de menu de plus.
    fn monde_de_jouer(&self) -> u8 {
        plan::dernier_monde_jouable(niveaux::niveaux(), &self.sauvegarde)
    }

    /// Robot de QA (carte 8.8) : une première partie est finie, le titre est rouvert
    /// avec la ligne NOUVELLE PARTIE+ (juste sous JOUER).
    #[cfg(test)]
    pub fn debloquer_ng_plus_qa(&mut self) {
        self.sauvegarde.parties_finies = 1;
        self.ng_plus_qa = true;
        self.ouvrir(Ecran::Titre);
    }

    /// Robot de QA (carte 9.6) : la sauvegarde `texte` remplace celle du jeu et le
    /// titre est rouvert (JOUER y lance le Monde 2 si « poursuite » y est fini).
    #[cfg(test)]
    pub fn charger_sauvegarde_qa(&mut self, texte: &str) {
        self.sauvegarde = Sauvegarde::depuis_texte(texte);
        self.ouvrir(Ecran::Titre);
    }

    /// Chrono de la partie en cours (pas de jeu), pour le robot de QA.
    #[cfg(test)]
    pub fn chrono(&self) -> u32 {
        self.pas_partie
    }

    /// Pas restants du petit signe d'une vie gagnée (robot de QA : trace des captures).
    #[cfg(test)]
    pub fn eclat_vie(&self) -> u16 {
        self.eclat_vie
    }

    /// Un écran de texte (image de l'histoire, carte du niveau, « PLUS DE VIES ») attend
    /// qu'on le passe et accepte l'appui (robot de QA : il appuie alors sur Entrée,
    /// comme un joueur, pour ne pas y rester bloqué ; aucun ne se passe seul).
    #[cfg(test)]
    pub fn texte_en_attente(&self) -> bool {
        matches!(
            self.ecran,
            Ecran::Histoire(_) | Ecran::Carte(_) | Ecran::PlusDeVies(_)
        ) && !self.transition.est_active()
            && histoire::prete(self.anim)
    }

    /// Le monde de jeu est-il en train d'avancer (donc à interpoler) ?
    pub fn en_jeu(&self) -> bool {
        self.ecran == Ecran::Jeu
            && !self.transition.est_active()
            && self.facile.proposition.is_none()
    }

    /// Le jeu va-t-il démarrer tout seul (carte de présentation, volets vers un niveau) ?
    fn partie_imminente(&self) -> bool {
        match self.suite {
            // Volets qui se ferment : tout dépend de ce qui vient derrière.
            Some(Suite::Niveau { .. })
            | Some(Suite::NiveauSeul(_))
            | Some(Suite::Course { .. })
            | Some(Suite::Recommencer)
            | Some(Suite::PlusDeVies)
            | Some(Suite::Carte) => true,
            Some(Suite::Titre)
            | Some(Suite::Fin)
            | Some(Suite::EcranFin)
            | Some(Suite::Plan)
            | Some(Suite::Histoire(_)) => false,
            // Rien en attente : carte de présentation, écran « plus de vies » (un appui
            // y mène au niveau recommencé, carte 9.31), ou volets qui s'ouvrent sur le niveau
            // recommencé.
            None => {
                matches!(self.ecran, Ecran::Carte(_) | Ecran::PlusDeVies(_))
                    || (self.ecran == Ecran::Jeu && self.transition.est_active())
            }
        }
    }

    /// La musique et la pluie doivent-elles tourner ? Non pendant la pause : elles
    /// reprennent quand le joueur en sort (la boucle principale le transmet à l'audio).
    pub fn boucles_sonores(&self) -> bool {
        self.ecran != Ecran::Pause
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
                // La carte vient d'être passée (`reagir`) : le jeu commence, son
                // premier pas au prochain (le monde ne bouge pas pendant celui-ci).
                Ecran::Carte(carte::PASSEE) => {
                    // Les défis du niveau précédent (bande sous la carte) ont été vus.
                    self.resultat_niveau = None;
                    self.ecran = Ecran::Jeu;
                }
                // Bannière du mode facile (carte 8.14) : le monde attend la réponse.
                Ecran::Jeu if self.facile.proposition.is_some() => {
                    if let Some(p) = self.facile.proposition.as_mut() {
                        p.attendre();
                    }
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
                Ecran::Titre => {
                    self.inactif += 1;
                    if self.inactif >= demo::ATTENTE_PAS {
                        self.demarrer_demo();
                    }
                }
                // Pas de « monde avancé » : la démo ne fait aucun bruitage.
                Ecran::Demo => self.pas_demo(),
                Ecran::Aide
                | Ecran::Boutique
                | Ecran::Pause
                | Ecran::Plan
                | Ecran::BilanNiveau
                | Ecran::ChoixForce
                | Ecran::BilanCourse
                // Carte 9.31 : ces écrans de texte attendent un appui (`reagir`).
                | Ecran::Carte(_)
                | Ecran::PlusDeVies(_)
                | Ecran::Histoire(_) => {}
            }
        }
        self.particules.actualiser();
        monde_avance
    }
}
