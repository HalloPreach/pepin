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
//! Notions Rust illustrées ici : machine à états avec `enum` (`Ecran`, `Suite`),
//! `Option::take` pour consommer une demande en attente, `Vec<Choix>` reconstruit
//! à chaque écran, méthodes qui ne se contredisent pas (un seul endroit change
//! d'écran).

use pepin_core::lettres;
use pepin_core::levels::{self, Apres, GenreNiveau};
use pepin_core::physics::HZ;
use pepin_core::player::Entrees;
use pepin_core::state::EtatJeu;

use crate::boutique::{self, EtatBoutique};
use crate::camera::Camera;
use crate::contenu;
use crate::cosmetiques::{self, Trainees};
use crate::course::{self, Fantome, Force, ResultatCourse};
use crate::defis::{self, LigneBonus, ResultatNiveau, Tentative};
use crate::demo::{self, Demo, Rejeu};
use crate::ecrans::bilan_course::{self, TempsCourse};
use crate::ecrans::bilan_niveau::{self, Temps};
use crate::ecrans::fin::{self, Bilan};
use crate::ecrans::histoire::{self, Chapitre};
use crate::ecrans::{
    self, aide, boutique as ecran_boutique, carte, course_force, pause, plan as ecran_plan,
    plus_de_vies, resultats, titre, Disposition, Ecran,
};
use crate::effets::effets;
use crate::entrees::{self, Commandes, RaccourciNiveau};
use crate::facile::{self as mode_facile, EtatFacile, Origine, Proposition, Reponse};
use crate::hud;
use crate::menu::{Choix, Menu};
use crate::monde::Monde;
use crate::mort;
use crate::navigation::Navigation;
use crate::particules::Particules;
use crate::plan::{self, ActionPlan, EtatPlan};
use crate::sauvegarde::Sauvegarde;
use crate::scene;
use crate::secousse::Secousse;
use crate::skins::Skin;
use crate::sprites::Banque;
use crate::transition::Transition;

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

/// Une commande de jeu est-elle tenue (clavier ou boutons tactiles) ? Démo du titre.
fn commandes_actives(c: &Commandes) -> bool {
    c.gauche || c.droite || c.saut || c.pause
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

    /// Affiche un écran à menu (ou sans) : reconstruit ses lignes, remet le curseur en haut.
    fn ouvrir(&mut self, ecran: Ecran) {
        self.ecran = ecran;
        self.anim = 0;
        self.choix = choix_pour(ecran);
        // AGA : « quand on a fini le monde 1, le niveau d'après [doit être] le premier
        // du monde 2, même si on n'a pas ouvert la carte ». À la fin du Monde 1, Monde 2
        // ouvert : CONTINUER en premier (sélectionné), qui enchaîne sur le chapitre 2.
        if ecran == Ecran::Fin
            && self.bilan.monde == 1
            && plan::monde_jouable(niveaux::niveaux(), &self.sauvegarde, 2)
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
    fn ouvrir_plan(&mut self, selection: usize) {
        self.niveau_seul = false;
        self.fantome = None;
        self.resultat_niveau = None;
        self.plan = EtatPlan::new(niveaux::niveaux(), selection);
        self.ouvrir(Ecran::Plan);
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
    /// jeu ») : le Monde 2 dès que sa porte est ouverte (« poursuite » fini, règle
    /// `plan::monde_jouable`), le Monde 1 sinon, exactement comme avant. Tout fini,
    /// JOUER relance donc le Monde 2. Aucune ligne de menu de plus.
    fn monde_de_jouer(&self) -> u8 {
        if plan::monde_jouable(niveaux::niveaux(), &self.sauvegarde, 2) {
            2
        } else {
            1
        }
    }

    /// Revient au titre, curseur sur la ligne `choix` (retour de l'aide, de la boutique).
    fn retour_titre_sur(&mut self, choix: Choix) {
        self.ouvrir(Ecran::Titre);
        if let Some(i) = self.choix.iter().position(|&c| c == choix) {
            self.menu.choisir(i);
        }
    }

    /// Prépare un changement d'écran derrière la transition en volets.
    fn lancer(&mut self, suite: Suite) {
        self.suite = Some(suite);
        self.transition.demarrer();
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

    fn valider_choix(&mut self, choix: Choix) {
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

    fn pas_jeu(&mut self, entrees: &Entrees) {
        let (largeur, hauteur) = (
            self.monde.niveau.largeur_px(),
            self.monde.niveau.hauteur_px(),
        );
        let vies_avant = self.monde.etat.vies.restantes();
        self.monde.avancer(entrees);
        self.trainees
            .actualiser(self.cosmetiques.trainee, &self.monde);
        // Course (carte 8.7) : le fantôme fait le même pas, dans son propre monde.
        if let Some(f) = &mut self.fantome {
            f.avancer();
        }
        self.eclat_vie = self.eclat_vie.saturating_sub(1);
        if self.monde.etat.vies.restantes() > vies_avant {
            self.eclat_vie = hud::DUREE_ECLAT_VIE;
        }
        // Temps de mort (carte 9.24) : la caméra reste sur l'endroit de la mort (elle
        // ne suit pas une chute dans le vide), puis saute à la lanterne quand le rideau
        // est fermé.
        // Carte 9.13e : la mer de la montée, que la caméra garde dans le cadre.
        self.camera.mer = crate::mer_montee::surface(&self.monde);
        if self.monde.reapparition > mort::RIDEAU_FERME {
            self.camera.figer();
        } else if self.monde.reapparition == mort::RIDEAU_FERME {
            self.camera.recentrer(&self.monde.joueur, largeur, hauteur);
        } else {
            self.camera.suivre(&self.monde.joueur, largeur, hauteur);
        }
        if let Some(mur) = self.monde.mur.filter(|m| !m.vaincu()) {
            self.camera.defiler_avec_le_mur(mur.front(), largeur);
        }
        effets(
            &self.monde,
            &mut self.particules,
            &mut self.secousse,
            self.camera.x,
            self.camera.y,
        );
        self.pas_partie += 1;
        // Défis (carte 7.3) : on regarde seulement, rien ne change dans le jeu.
        self.tentative.pas += 1;
        self.tentative.a_plane |= self.monde.joueur.plane;
        self.tentative.morts = self.monde.etat.morts - self.morts_debut_tentative;
        self.compter_defaite_au_boss();
        // La dernière vie perdue : l'écran « plus de vies », puis le niveau recommence
        // (avant tout, même si ce pas-là finissait aussi le niveau). Après l'animation
        // de la mort (carte 9.24), le rideau fermé.
        if self.monde.etat.vies.plus_de_vies() && self.monde.reapparition <= mort::RIDEAU_FERME {
            self.game_over_partie = true;
            self.ouvrir(Ecran::PlusDeVies(plus_de_vies::DUREE_PAS));
            return;
        }
        if self.monde.termine {
            // Niveau réussi (bonus compris) : ses gouttes NOUVELLES vont au
            // porte-monnaie, tout de suite enregistré (une goutte déjà comptée ne
            // rapporte plus, carte 7.9). Celles d'un niveau raté ne comptent pas : elles
            // sont remises à zéro quand le niveau recommence. Les défis (carte 7.3) sont
            // notés au passage, et leurs premières réussites créditées.
            self.tentative.gouttes = self.monde.gouttes_niveau;
            // La lettre cachée (carte 8.6) d'un niveau réussi : gardée pour toujours.
            if self.monde.lettre_prise {
                self.sauvegarde.noter_lettre(self.monde.infos.cle);
            }
            let ancien = self.sauvegarde.progres(self.monde.infos.cle).meilleur_temps;
            // Boss vaincu en mode facile (carte 8.14) : fini, gouttes créditées, mais
            // ni défi ni médaille ni meilleur temps.
            let r = if self.facile.actif && self.monde.pv_nuage().is_some() {
                self.facile.dans_la_partie = true;
                mode_facile::noter_victoire_facile(
                    &mut self.sauvegarde,
                    self.monde.infos,
                    &self.tentative,
                )
            } else {
                defis::noter_niveau(&mut self.sauvegarde, self.monde.infos, &self.tentative)
            };
            self.resultat_niveau = Some(r);
            // Nouvelle partie + (carte 8.8) : gouttes et défis comptent comme d'habitude,
            // mais le temps du niveau va dans ses records à elle.
            if self.ng_plus {
                let cle = self.monde.infos.cle;
                if let Some(p) = self.sauvegarde.progres_mut(cle) {
                    p.meilleur_temps = ancien;
                    self.sauvegarde.ng_plus.noter_temps(cle, self.tentative.pas);
                }
            }
            self.sauvegarde.enregistrer();
            // Un niveau joué seul (carte 7.4) : son bilan, par-dessus le niveau figé.
            if self.niveau_seul {
                // Une course (carte 8.7) : son bilan à elle (gagné ou perdu, l'écart).
                if let Some(f) = &self.fantome {
                    let robot = f.temps();
                    self.resultat_course = Some(course::noter_course(
                        &mut self.sauvegarde,
                        self.monde.infos.cle,
                        f.force,
                        self.tentative.pas,
                        robot,
                    ));
                    self.sauvegarde.enregistrer();
                    self.temps_course = TempsCourse {
                        joueur: self.tentative.pas,
                        robot,
                    };
                    self.ouvrir(Ecran::BilanCourse);
                    return;
                }
                self.temps_niveau = Temps {
                    pas: self.tentative.pas,
                    ancien,
                };
                self.ouvrir(Ecran::BilanNiveau);
                return;
            }
            // Le bonus fini : ses gouttes sont mises à part (compteur, écran de fin).
            if self.monde.infos.genre == GenreNiveau::Bonus {
                self.gouttes_bonus = Some(self.gouttes_du_niveau());
            }
            // Niveau suivant (le score est gardé), en sautant le bonus s'il manque des
            // gouttes ; après l'épilogue ou le dernier niveau : l'écran de fin.
            self.lancer(
                match levels::apres(
                    niveaux::niveaux(),
                    self.monde.indice,
                    self.monde.etat.gouttes,
                ) {
                    Apres::Niveau(indice) => Suite::Niveau {
                        indice,
                        etat: Some(self.monde.etat.clone()),
                    },
                    Apres::Fin => Suite::Fin,
                },
            );
        }
    }

    // --- Démo du titre (carte 8.1) -------------------------------------------------

    /// Dix secondes sans entrée sur le titre : le robot joue le niveau suivant de la
    /// rotation (`demo::PARCOURS`). Rien à jouer (aucun niveau retrouvé) : on reste au
    /// titre, le compteur repart.
    fn demarrer_demo(&mut self) {
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
    fn pas_demo(&mut self) {
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
    fn quitter_demo(&mut self) {
        self.particules = Particules::new();
        self.ouvrir(Ecran::Titre);
    }

    /// L'écran est couvert : on change de décor sans que le joueur le voie.
    fn appliquer_suite(&mut self) {
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
                // Une partie du Monde 2 (carte 9.1) a ses records à part : `partie.monde2`.
                self.nouveau_record = if monde > 1 {
                    self.sauvegarde
                        .partie_monde2
                        .noter_partie(e.score, self.pas_partie)
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
    fn gouttes_du_niveau(&self) -> u32 {
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

    fn recharger(&mut self, indice: usize, etat: Option<EtatJeu>) {
        let (monde, camera) = charger_mode(indice, etat, self.ng_plus);
        self.debut_niveau = monde.etat.clone();
        self.monde = monde;
        self.camera = camera;
        self.eclat_vie = 0;
        // Un niveau chargé : en facile seulement si on vient de le choisir (carte 8.14).
        self.facile.actif = std::mem::take(&mut self.facile.au_prochain);
        self.facile.refusee = false;
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

    // --- Mode facile du Grand Nuage (carte 8.14) ------------------------------------

    /// Une mort au boss en mode normal : une défaite de plus (enregistrée) ; à partir
    /// de `DEFAITES_AVANT_FACILE`, la bannière « TROP DUR ? » s'ouvre (pas pendant
    /// « plus de vies », ni si NON a déjà été répondu dans ce niveau). La démo du
    /// titre ne passe pas par ici (`pas_demo`), et une course ne compte pas.
    fn compter_defaite_au_boss(&mut self) {
        // Une fois l'animation de la mort jouée, le rideau fermé (carte 9.24) : la
        // bannière ne la cache pas.
        let mort = self.monde.reapparition == mort::RIDEAU_FERME;
        // Une course contre le robot (carte 8.7) ne compte pas : on y rejoue pour le chrono.
        if !mort || self.facile.actif || self.fantome.is_some() || self.monde.pv_nuage().is_none() {
            return;
        }
        let n = mode_facile::noter_defaite(&mut self.sauvegarde, self.monde.infos.cle);
        self.sauvegarde.enregistrer();
        if n >= mode_facile::DEFAITES_AVANT_FACILE
            && !self.facile.refusee
            && !self.monde.etat.vies.plus_de_vies()
        {
            self.facile.proposition = Some(Proposition::new(Origine::Mort));
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

    // --- Dessin --------------------------------------------------------------------

    /// Dessine l'écran courant dans la cible logique. `alpha` : interpolation entre deux pas.
    pub fn dessiner(&self, banque: &Banque, alpha: f32) {
        // Monde figé (pause, transition, carte) : on montre le dernier pas tel quel,
        // sinon l'interpolation ferait trembler les sprites sur place.
        let anime = matches!(self.ecran, Ecran::Jeu | Ecran::Demo) && !self.transition.est_active();
        let alpha = if anime { alpha } else { 1.0 };
        let libelles: Vec<&str> = self.choix.iter().map(|&c| contenu::libelle(c)).collect();
        let dispo = ecrans::disposition_menu(self.ecran, self.choix.len())
            .unwrap_or(Disposition { y0: 0.0, pas: 20.0 });
        match self.ecran {
            Ecran::Titre => {
                titre::dessiner(banque, self.skin, self.anim, &libelles, &self.menu, &dispo);
                titre::dessiner_poche(banque, &self.sauvegarde);
                if self.son_disponible {
                    ecrans::dessiner_icone_son(self.son);
                }
            }
            Ecran::Aide => aide::dessiner(),
            Ecran::Histoire(chapitre) => histoire::dessiner(banque, self.skin, chapitre, self.anim),
            Ecran::Boutique => {
                ecran_boutique::dessiner(banque, self.anim, &self.sauvegarde, &self.boutique)
            }
            Ecran::Fin => fin::dessiner(
                &self.bilan,
                self.anim,
                &self.particules,
                &libelles,
                &self.menu,
                &dispo,
                self.nouveau_record,
            ),
            Ecran::Plan => {
                ecran_plan::dessiner(banque, self.skin, self.anim, &self.sauvegarde, &self.plan)
            }
            // Course (carte 8.7) : le choix de la force, par-dessus la carte des niveaux.
            Ecran::ChoixForce => {
                ecran_plan::dessiner(banque, self.skin, self.anim, &self.sauvegarde, &self.plan);
                course_force::dessiner(
                    &niveaux::niveaux()[self.plan.selection],
                    &self.sauvegarde,
                    &self.choix,
                    &libelles,
                    &self.menu,
                    &dispo,
                    self.anim,
                );
            }
            Ecran::Jeu
            | Ecran::Carte(_)
            | Ecran::Pause
            | Ecran::PlusDeVies(_)
            | Ecran::BilanNiveau
            | Ecran::BilanCourse
            | Ecran::Demo => {
                // Pépin et la caméra sont arrondis ensemble (écart arrondi une seule fois) : pas de
                // tremblement d'un pixel en courant (ni en tombant, dans un niveau haut).
                let pepin = self.monde.joueur_interpole(alpha);
                let (cx, cy) = self.camera.position_rendue(alpha, pepin);
                let (sx, sy) = self.secousse.decalage();
                scene::dessiner(
                    &self.monde,
                    banque,
                    &self.particules,
                    &self.trainees,
                    self.skin,
                    alpha,
                    cx + sx,
                    cy + sy,
                );
                // Course (carte 8.7) : le fantôme par-dessus, tant qu'il court.
                if let Some(f) = self.fantome.as_ref().filter(|f| f.visible()) {
                    scene::dessiner_fantome(
                        &f.monde,
                        banque,
                        alpha,
                        (cx + sx, cy + sy),
                        f.force.libelle(),
                    );
                }
                // Sous la trame (carte, pause) le HUD serait illisible : on ne le montre qu'en jeu.
                if self.ecran == Ecran::Jeu {
                    // Niveau joué seul (carte 7.4) : les gouttes de CE niveau, sur 3.
                    let compteur = hud::compteur(
                        self.monde.infos.monde,
                        self.niveau_seul || self.monde.infos.genre == GenreNiveau::Bonus,
                        self.monde.etat.gouttes,
                        self.gouttes_bonus,
                        self.gouttes_du_niveau(),
                    );
                    let vies = hud::vies(self.monde.etat.vies.restantes(), self.eclat_vie);
                    // Niveau joué seul (carte 7.4) : le chrono de la tentative et le temps
                    // cible du défi CHRONO.
                    let (pas, cible) = if self.niveau_seul {
                        let c = u32::from(self.monde.infos.duree_cible_s) * HZ;
                        (self.tentative.pas, Some(c))
                    } else {
                        (self.pas_partie, None)
                    };
                    hud::dessiner(
                        banque,
                        self.skin,
                        compteur,
                        &vies,
                        pas,
                        cible,
                        self.monde.points_nuage(),
                        self.monde.ng_plus,
                        self.monde.boss_pieuvre(),
                    );
                    // Course (carte 8.7) : l'état du fantôme sous le chrono.
                    if let Some(f) = &self.fantome {
                        hud::dessiner_ligne_course(&course::ligne_hud(f));
                    }
                }
                match self.ecran {
                    Ecran::Carte(_) => carte::dessiner(
                        self.monde.indice,
                        self.monde.infos,
                        self.anim,
                        &self.sauvegarde.lettres(),
                    ),
                    Ecran::Pause => {
                        pause::dessiner(self.anim, &libelles, &self.menu, &dispo);
                        // En jeu, pas d'icône : le coin est aux commandes tactiles et au HUD.
                        if self.son_disponible {
                            ecrans::dessiner_icone_son(self.son);
                        }
                    }
                    Ecran::PlusDeVies(_) => plus_de_vies::dessiner(banque, self.anim),
                    Ecran::Demo => demo::dessiner_bandeau(self.anim),
                    Ecran::BilanCourse => {
                        if let (Some(c), Some(r)) = (&self.resultat_course, &self.resultat_niveau) {
                            bilan_course::dessiner(
                                &self.temps_course,
                                c,
                                r,
                                self.anim,
                                &libelles,
                                &self.menu,
                                &dispo,
                            );
                        }
                    }
                    Ecran::BilanNiveau => {
                        if let Some(r) = &self.resultat_niveau {
                            bilan_niveau::dessiner(
                                &self.temps_niveau,
                                r,
                                self.anim,
                                &libelles,
                                &self.menu,
                                &dispo,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        // Défis et bonus (carte 7.3), sous la carte du niveau suivant ou l'écran de fin.
        match (self.ecran, &self.resultat_niveau) {
            (Ecran::Carte(_), Some(r)) => resultats::dessiner_bande_niveau(r, self.anim),
            (Ecran::Fin, r) => {
                resultats::dessiner_bande_fin(r.as_ref(), &self.bonus_fin, self.anim)
            }
            _ => {}
        }
        // Bannière du mode facile (carte 8.14), par-dessus le jeu ou la carte des niveaux.
        if let Some(p) = &self.facile.proposition {
            ecrans::facile::dessiner(p);
        }
        self.transition.dessiner();
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

#[cfg(test)]
mod tests {
    use super::*;
    use pepin_core::entities::contacts::Contact;
    use pepin_core::entities::Entite;
    use pepin_core::state::POINTS_GOUTTE;
    use pepin_core::vies::{VIES_DEPART, VIES_MAX};

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
    /// Une image de l'histoire (carte 8.4) qui n'est pas l'écran attendu est passée
    /// d'un appui sur Entrée dès qu'elle l'accepte, comme le ferait le joueur. De même
    /// pour la carte du niveau et « PLUS DE VIES », qui ne se passent plus seuls (carte
    /// 9.31) ; quand c'est l'écran attendu, on attend qu'il accepte l'appui, pour que
    /// le test puisse le passer aussitôt.
    fn jusqu_a(app: &mut Application, ok: impl Fn(Ecran) -> bool) {
        for _ in 0..600 {
            let texte = matches!(app.ecran, Ecran::Carte(_) | Ecran::PlusDeVies(_));
            if ok(app.ecran)
                && !app.transition.est_active()
                && (!texte || histoire::prete(app.anim))
            {
                return;
            }
            if app.texte_en_attente() {
                app.reagir(&valider(), &Commandes::default(), None);
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
        passer_la_carte(&mut app);
        app
    }

    /// Le joueur passe la carte (prête) d'un appui ; le jeu commence au pas suivant,
    /// sans que le monde ait bougé (carte 9.31).
    fn passer_la_carte(app: &mut Application) {
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Carte(carte::PASSEE));
        let pas = app.monde.pas;
        avancer(app, 1);
        assert_eq!(app.ecran, Ecran::Jeu);
        assert_eq!(app.monde.pas, pas, "aucun pas de jeu pendant celui-ci");
    }

    /// Carte 9.1b (AGA : « c'est juste la suite du jeu ») : jamais de ligne MONDE 2 au
    /// titre ; JOUER reprend là où on en est : le Monde 1 tant que `poursuite` n'est
    /// pas fini, le Monde 2 ensuite (aussi quand tout est fini).
    #[test]
    fn jouer_reprend_la_ou_on_en_est() {
        let mut app = Application::new(true);
        app.sauvegarde =
            Sauvegarde::depuis_texte("version=3\nparties_finies=1\nniveau.poursuite=medaille:1");
        app.ouvrir(Ecran::Titre);
        let mut attendu = vec![
            Choix::Jouer,
            Choix::NouvellePartiePlus,
            Choix::Carte,
            Choix::Boutique,
            Choix::Histoire,
            Choix::Aide,
        ];
        // NOUVELLE PARTIE + désactivée pour l'instant (AGA) : absente du titre.
        if !NOUVELLE_PARTIE_PLUS_OUVERTE {
            attendu.retain(|&c| c != Choix::NouvellePartiePlus);
        }
        assert_eq!(app.choix, attendu);
        // Porte ouverte : JOUER (1re ligne) lance la 1re étape du Monde 2.
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.monde, 2);
        assert_eq!(
            Some(app.monde.indice),
            levels::debut_monde(niveaux::niveaux(), 2)
        );
        // AGA : trois niveaux du Monde 2 finis, JOUER reprend au 4e (pas au 1er).
        let mut app = Application::new(true);
        app.sauvegarde = Sauvegarde::depuis_texte(
            "version=3\nparties_finies=1\nniveau.poursuite=medaille:1\nniveau.caniveau=medaille:1\nniveau.egouts=medaille:1\nniveau.cascade=medaille:1",
        );
        app.ouvrir(Ecran::Titre);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.monde, 2);
        assert_eq!(
            app.monde.indice,
            levels::debut_monde(niveaux::niveaux(), 2).unwrap() + 3
        );
        // AGA : les gouttes déjà trouvées dans les niveaux d'avant restent comptées.
        let mut app = Application::new(true);
        app.sauvegarde = Sauvegarde::depuis_texte(
            "version=3\nparties_finies=1\nniveau.poursuite=medaille:1\nniveau.caniveau=medaille:1\nniveau.egouts=medaille:1\nniveau.cascade=medaille:1",
        );
        let d = levels::debut_monde(niveaux::niveaux(), 2).unwrap();
        if let Some(p) = app.sauvegarde.progres_mut(niveaux::niveaux()[d].cle) {
            p.gouttes = [true, true, false];
        }
        if let Some(p) = app.sauvegarde.progres_mut(niveaux::niveaux()[d + 2].cle) {
            p.gouttes = [true, false, false];
        }
        app.ouvrir(Ecran::Titre);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, d + 3);
        assert_eq!(app.monde.etat.gouttes, 3);
        // Porte fermée (`poursuite` pas fini) : JOUER lance le Monde 1, comme avant.
        app.sauvegarde = Sauvegarde::depuis_texte("version=3\nparties_finies=1");
        assert_eq!(app.monde_de_jouer(), 1);
        // AGA : « qu'il reprenne ou j'en suis quoi qu'il arrive » : au Monde 1 aussi,
        // deux niveaux finis, JOUER reprend au 3e, sans les images de l'histoire.
        let mut app = Application::new(true);
        app.sauvegarde = Sauvegarde::depuis_texte(&format!(
            "version=3\nniveau.{}=medaille:1\nniveau.{}=medaille:1",
            niveaux::niveaux()[0].cle,
            niveaux::niveaux()[1].cle
        ));
        app.ouvrir(Ecran::Titre);
        app.reagir(&valider(), &Commandes::default(), None);
        assert!(!app.partie_racontee);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.monde, 1);
        assert_eq!(app.monde.indice, 2);
    }

    #[test]
    fn les_menus_ont_les_bonnes_lignes() {
        assert_eq!(
            choix_pour(Ecran::Titre),
            vec![
                Choix::Jouer,
                Choix::NouvellePartiePlus,
                Choix::Carte,
                Choix::Boutique,
                Choix::Histoire,
                Choix::Aide
            ]
        );
        assert_eq!(
            choix_pour(Ecran::BilanNiveau),
            vec![Choix::RejouerNiveau, Choix::Carte]
        );
        assert_eq!(
            choix_pour(Ecran::Pause),
            vec![Choix::Reprendre, Choix::Recommencer, Choix::QuitterAuTitre]
        );
        assert_eq!(
            choix_pour(Ecran::Fin),
            vec![Choix::Rejouer, Choix::RetourTitre]
        );
        assert!(choix_pour(Ecran::Jeu).is_empty());
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
        // La carte attend un appui (carte 9.31) : le joueur valide.
        passer_la_carte(&mut app);
        assert_eq!(app.monde.infos.numero, 1);
    }

    /// Carte 9.31 (AGA : « qu'on soit obligé d'appuyer pour passer les textes ») : sans
    /// appui, la carte du niveau reste affichée, même 10 s, et le monde reste figé.
    #[test]
    fn la_carte_reste_affichee_dix_secondes_sans_appui() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        avancer(&mut app, 10 * 60);
        assert!(matches!(app.ecran, Ecran::Carte(_)), "{:?}", app.ecran);
        assert_eq!(app.monde.pas, 0, "le monde attend le joueur");
        // « PLUS DE VIES » non plus ne se passe pas seul.
        let mut app = en_jeu();
        for _ in 0..VIES_DEPART {
            tomber(&mut app);
        }
        avancer(&mut app, 10 * 60);
        assert!(matches!(app.ecran, Ecran::PlusDeVies(_)), "{:?}", app.ecran);
    }

    /// Carte 9.31 : un appui pendant le court délai d'ouverture (un saut encore tenu)
    /// est ignoré ; après le délai, il passe la carte, et « PLUS DE VIES » de même.
    #[test]
    fn un_appui_avant_le_delai_est_ignore_apres_il_passe() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        // Dès la fin des volets, la carte est là mais pas encore prête (l'image de
        // l'histoire d'avant est passée d'un appui, comme dans `jusqu_a`).
        for _ in 0..600 {
            if matches!(app.ecran, Ecran::Carte(_)) && !app.transition.est_active() {
                break;
            }
            if app.texte_en_attente() {
                app.reagir(&valider(), &Commandes::default(), None);
            }
            app.pas(&Entrees::default());
        }
        assert!(matches!(app.ecran, Ecran::Carte(_)));
        assert!(!histoire::prete(app.anim), "anim = {}", app.anim);
        app.reagir(&valider(), &Commandes::default(), None);
        assert!(matches!(app.ecran, Ecran::Carte(_)), "trop tôt : ignoré");
        while !histoire::prete(app.anim) {
            app.pas(&Entrees::default());
        }
        passer_la_carte(&mut app);
        // « PLUS DE VIES » : ouvert à anim = 0, un appui aussitôt est ignoré.
        for _ in 0..VIES_DEPART {
            tomber(&mut app);
        }
        assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
        app.reagir(&valider(), &Commandes::default(), None);
        assert!(!app.transition.est_active(), "trop tôt : ignoré");
        avancer(&mut app, histoire::DELAI_PAS);
        app.reagir(&valider(), &Commandes::default(), None);
        assert!(app.transition.est_active(), "les volets partent");
    }

    /// Carte 9.31 : l'invitation « continuer » n'apparaît qu'une fois l'appui accepté,
    /// puis clignote doucement (0,5 s allumée, 0,5 s éteinte), comme dans l'histoire.
    #[test]
    fn l_invitation_clignote_une_fois_l_appui_accepte() {
        assert!(!histoire::invite_visible(0));
        assert!(!histoire::invite_visible(histoire::DELAI_PAS - 1));
        assert!(histoire::invite_visible(histoire::DELAI_PAS));
        assert!(!histoire::invite_visible(histoire::DELAI_PAS + 30));
        assert!(histoire::invite_visible(histoire::DELAI_PAS + 60));
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
        avancer(&mut app, 1);
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
        // Reprendre, Recommencer, Quitter : on descend en bas en bouclant par le haut.
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
    fn l_icone_du_son_bascule_sans_quitter_le_menu() {
        // Carte 7.8b : plus de ligne SON ; un clic (ou toucher) sur l'icône en bas à
        // gauche, même un peu à côté, bascule le son et ne valide aucune ligne.
        let mut app = Application::new(true);
        let clic = Navigation {
            clic: Some((3.0, 177.0)),
            ..Navigation::default()
        };
        app.reagir(&clic, &Commandes::default(), None);
        assert!(app.son);
        assert_eq!(app.ecran, Ecran::Titre);
        assert!(!app.transition.est_active());
        app.reagir(&clic, &Commandes::default(), None);
        assert!(!app.son);
    }

    #[test]
    fn un_clic_sur_une_ligne_la_valide() {
        let mut app = Application::new(true);
        let d = ecrans::disposition_menu(Ecran::Titre, app.choix.len()).unwrap();
        let ligne = app.choix.iter().position(|&c| c == Choix::Aide).unwrap();
        let (x, y, _, h) = d.rect_ligne(ligne);
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
        let d = ecrans::disposition_menu(Ecran::Titre, app.choix.len()).unwrap();
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

    /// Le dernier niveau DU MONDE 1 (l'épilogue ; carte 9.6 : la table continue avec
    /// le Monde 2) mène à l'écran de fin, avec le bilan de la partie.
    #[test]
    fn finir_le_dernier_niveau_ouvre_l_ecran_de_fin_avec_le_bilan() {
        let mut app = en_jeu();
        let (monde, camera) = charger(niveaux::NB_NIVEAUX_MONDE_1 - 1, None);
        app.monde = monde;
        app.camera = camera;
        app.monde.etat.gouttes = 9;
        app.monde.etat.morts = 2;
        app.monde.etat.score = 900;
        app.pas_partie = 3600;
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| e == Ecran::Fin);
        // Le dernier niveau est l'épilogue (carte 5.17) : l'écran de fin du retour.
        assert_eq!(
            app.bilan,
            Bilan {
                pas: 3601,
                gouttes: 9,
                morts: 2,
                vies: VIES_DEPART,
                score: 900,
                gouttes_bonus: None,
                epilogue: true,
                lettres: Some([false; 5]),
                monde: 1,
            }
        );
        assert_eq!(app.choix, vec![Choix::Rejouer, Choix::RetourTitre]);
    }

    /// AGA : le Monde 1 fini (porte du Monde 2 ouverte), l'écran de fin propose
    /// CONTINUER en premier, qui lance le niveau 1 du Monde 2 sans passer par la carte.
    #[test]
    fn finir_le_monde_1_enchaine_sur_le_monde_2() {
        let mut app = en_jeu();
        app.sauvegarde = Sauvegarde::depuis_texte("version=3\nniveau.poursuite=medaille:1");
        let (monde, camera) = charger(niveaux::NB_NIVEAUX_MONDE_1 - 1, None);
        app.monde = monde;
        app.camera = camera;
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| e == Ecran::Fin);
        assert_eq!(
            app.choix,
            vec![Choix::Continuer, Choix::Rejouer, Choix::RetourTitre]
        );
        assert_eq!(app.choix[app.menu.selection()], Choix::Continuer);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.infos.monde, 2);
        assert_eq!(
            Some(app.monde.indice),
            levels::debut_monde(niveaux::niveaux(), 2)
        );
    }

    #[test]
    fn rejouer_remet_la_partie_a_zero() {
        let mut app = en_jeu();
        app.monde.etat.gouttes = 5;
        app.pas_partie = 1234;
        app.monde.termine = false;
        app.ecran = Ecran::Fin;
        app.choix = choix_pour(Ecran::Fin);
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

    /// Touche F2 du build de QA : le niveau d'essai « grêle » (hors de la table),
    /// avec ses quatre colonnes de grêle.
    #[test]
    fn le_raccourci_d_essai_charge_le_niveau_de_grele() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Essai3),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_GRELE);
        assert_eq!(app.monde.infos.nom, "Essai grêle");
        let greles = app
            .monde
            .entites
            .iter()
            .filter(|e| matches!(e, pepin_core::entities::Entite::Grele(_)))
            .count();
        assert_eq!(greles, 4);
    }

    /// Touche F3 du build de QA : le niveau d'essai « auvent » (hors de la table),
    /// avec ses six auvents.
    #[test]
    fn le_raccourci_d_essai_charge_le_niveau_des_auvents() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Essai4),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_AUVENT);
        assert_eq!(app.monde.infos.nom, "Essai auvent");
        let auvents = app
            .monde
            .entites
            .iter()
            .filter(|e| matches!(e, pepin_core::entities::Entite::Auvent(_)))
            .count();
        assert_eq!(auvents, 6);
    }

    /// Touche F5 du build de QA : le niveau d'essai « montgolfière » (hors de la
    /// table), avec ses deux montgolfières.
    #[test]
    fn le_raccourci_d_essai_charge_le_niveau_des_montgolfieres() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Essai6),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_MONTGOLFIERE);
        assert_eq!(app.monde.infos.nom, "Essai montgolfière");
        let montgolfieres = app
            .monde
            .entites
            .iter()
            .filter(|e| matches!(e, pepin_core::entities::Entite::Montgolfiere(_)))
            .count();
        assert_eq!(montgolfieres, 2);
    }

    /// Touche F6 du build de QA : le niveau d'essai « escargot » (hors de la
    /// table), avec ses deux escargots.
    #[test]
    fn le_raccourci_d_essai_charge_le_niveau_des_escargots() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Essai7),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_ESCARGOT);
        assert_eq!(app.monde.infos.nom, "Essai escargot");
        let escargots = app
            .monde
            .entites
            .iter()
            .filter(|e| matches!(e, pepin_core::entities::Entite::Escargot(_)))
            .count();
        assert_eq!(escargots, 2);
    }

    /// Touche F7 du build de QA : le niveau d'essai « chat » (hors de la table),
    /// avec son chat endormi.
    #[test]
    fn le_raccourci_d_essai_charge_le_niveau_du_chat() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Essai8),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_CHAT);
        assert_eq!(app.monde.infos.nom, "Essai chat");
        let chats = app
            .monde
            .entites
            .iter()
            .filter(|e| {
                matches!(e, pepin_core::entities::Entite::Ennemi(c)
                    if c.type_ennemi == pepin_core::entities::TypeEnnemi::Chat)
            })
            .count();
        assert_eq!(chats, 1);
    }

    /// Touche F10 du build de QA : l'arène du Grand Nuage (hors de la table), avec
    /// son nuage à 3 points de vie, et sans drapeau.
    #[test]
    fn le_raccourci_d_essai_charge_l_arene_du_grand_nuage() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Essai10),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_NUAGE);
        assert_eq!(app.monde.infos.nom, "Essai nuage");
        assert_eq!(app.monde.pv_nuage(), Some(3));
        assert!(app.monde.niveau.arrivee.is_none());
    }

    /// Touche F8 du build de QA : le niveau d'essai « horloge » (hors de la
    /// table), avec ses deux aiguilles.
    #[test]
    fn le_raccourci_d_essai_charge_le_niveau_de_l_horloge() {
        let mut app = en_jeu();
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Essai9),
        );
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.monde.indice, crate::monde::INDICE_ESSAI_HORLOGE);
        assert_eq!(app.monde.infos.nom, "Essai horloge");
        let aiguilles = app
            .monde
            .entites
            .iter()
            .filter(
                |e| matches!(e, pepin_core::entities::Entite::Mobile(m) if m.cercle().is_some()),
            )
            .count();
        assert_eq!(aiguilles, 2);
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
        for attendu in 1..niveaux::NB_NIVEAUX {
            app.reagir(
                &rien(),
                &Commandes::default(),
                Some(RaccourciNiveau::Suivant),
            );
            jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
            passer_la_carte(&mut app);
            assert_eq!(app.monde.indice, attendu);
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
        assert_eq!(app.monde.indice, niveaux::NB_NIVEAUX - 2);
    }

    /// Chaque niveau terminé mène là où le dit `levels::apres` (avec les gouttes de la
    /// partie) : le niveau suivant, ou l'écran de fin.
    #[test]
    fn la_suite_de_chaque_niveau_suit_la_table() {
        for indice in 0..niveaux::NB_NIVEAUX {
            for gouttes in [0, levels::gouttes_pour_le_bonus(niveaux::niveaux())] {
                let mut app = en_jeu();
                let (monde, camera) = charger(indice, None);
                app.monde = monde;
                app.camera = camera;
                app.monde.etat.gouttes = gouttes;
                app.monde.termine = true;
                avancer(&mut app, 1);
                match levels::apres(niveaux::niveaux(), indice, gouttes) {
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

    /// Les DEUX chemins au sortir du 10e niveau (le dernier normal, cartes 5.16, 5.17
    /// et 8.2) : avec les 30 gouttes des niveaux normaux, la carte suivante est celle du
    /// bonus (« NIVEAU BONUS », l'arc-en-ciel), puis celle de l'épilogue ; avec une de
    /// moins, directement l'épilogue (« ÉPILOGUE », la descente). L'épilogue fini,
    /// c'est l'écran de fin du retour : les gouttes du parcours sur 33, et celles du
    /// bonus à part s'il a été joué.
    #[test]
    fn apres_le_neuvieme_le_bonus_seulement_avec_toutes_les_gouttes_puis_l_epilogue() {
        use pepin_core::levels::nb_normaux;
        for (gouttes, bonus) in [(30, true), (29, false), (0, false)] {
            let mut app = en_jeu();
            let (monde, camera) = charger(9, None);
            app.monde = monde;
            app.camera = camera;
            assert_eq!(app.monde.infos.numero, 10);
            app.monde.etat.gouttes = gouttes;
            app.monde.termine = true;
            avancer(&mut app, 1);
            jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
            let mut gouttes_finales = gouttes;
            if bonus {
                assert_eq!(app.monde.indice, 10);
                assert_eq!(app.monde.infos.nom, niveaux::par_cle("arc").nom);
                assert_eq!(app.monde.infos.genre, GenreNiveau::Bonus);
                assert_eq!(
                    contenu::etiquette_niveau(app.monde.infos, nb_normaux(niveaux::niveaux())),
                    "NIVEAU BONUS"
                );
                // Le bonus fini avec 2 de ses gouttes (32 dans la partie) : l'épilogue.
                passer_la_carte(&mut app);
                app.monde.gouttes_niveau = [true, false, true];
                app.monde.etat.gouttes = 32;
                app.monde.termine = true;
                avancer(&mut app, 1);
                assert_eq!(app.gouttes_bonus, Some(2));
                jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
                gouttes_finales = 32;
            }
            assert_eq!(app.monde.indice, 11);
            assert_eq!(app.monde.infos.nom, niveaux::par_cle("descente").nom);
            assert_eq!(app.monde.infos.genre, GenreNiveau::Epilogue);
            assert_eq!(
                contenu::etiquette_niveau(app.monde.infos, nb_normaux(niveaux::niveaux())),
                "ÉPILOGUE"
            );
            assert_eq!(app.monde.etat.gouttes, gouttes_finales, "gouttes gardées");
            // L'épilogue fini avec ses 3 gouttes : l'écran de fin du retour.
            app.reagir(&valider(), &Commandes::default(), None);
            app.monde.etat.gouttes += 3;
            app.monde.termine = true;
            avancer(&mut app, 1);
            jusqu_a(&mut app, |e| e == Ecran::Fin);
            assert!(app.bilan.epilogue);
            if bonus {
                assert_eq!((app.bilan.gouttes, app.bilan.gouttes_bonus), (33, Some(2)));
            } else {
                assert_eq!(
                    (app.bilan.gouttes, app.bilan.gouttes_bonus),
                    (gouttes + 3, None)
                );
            }
        }
    }

    /// Une nouvelle partie oublie le bonus de la précédente.
    #[test]
    fn rejouer_oublie_les_gouttes_du_bonus() {
        let mut app = en_jeu();
        app.gouttes_bonus = Some(3);
        app.ecran = Ecran::Fin;
        app.choix = choix_pour(Ecran::Fin);
        app.menu = Menu::new(2);
        app.reagir(&valider(), &Commandes::default(), None);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        assert_eq!(app.gouttes_bonus, None);
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
        // La carte attend (carte 9.31), puis elle est passée sans que le focus soit
        // revenu : la pause s'ouvre au premier pas de jeu, AVANT que le monde ne bouge.
        avancer(&mut app, 600);
        assert!(matches!(app.ecran, Ecran::Carte(_)));
        app.reagir(&valider(), &Commandes::default(), None);
        avancer(&mut app, 2);
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
        app.reagir(&valider(), &Commandes::default(), None);
        avancer(&mut app, 12);
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
        let (monde, camera) = charger(niveaux::NB_NIVEAUX_MONDE_1 - 1, None);
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
        passer_la_carte(&mut app);
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
    // --- Vies (carte 6.2) ------------------------------------------------------------

    /// Fait tomber Pépin dans le vide : une mort au pas suivant (le vide tue même
    /// pendant la protection de réapparition).
    /// Fait tomber Pépin dans le vide, puis attend la fin du temps de mort (carte
    /// 9.24) ou l'écran « plus de vies ».
    fn tomber(app: &mut Application) {
        let bas = app.monde.niveau.hauteur_px();
        let x = app.monde.joueur.boite.x;
        app.monde.joueur.placer(x, bas + 100.0);
        avancer(app, 1);
        while app.ecran == Ecran::Jeu && app.monde.reapparition > 0 {
            avancer(app, 1);
        }
    }

    /// Pose Pépin sur chacune des gouttes d'or du niveau, une par pas.
    fn ramasser_les_gouttes(app: &mut Application) {
        let gouttes: Vec<(f32, f32)> = app
            .monde
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Goutte(g) => Some((g.boite.x, g.boite.y)),
                _ => None,
            })
            .collect();
        assert_eq!(gouttes.len(), 3);
        for (x, y) in gouttes {
            app.monde.joueur.placer(x, y);
            avancer(app, 1);
        }
        assert_eq!(app.monde.etat.gouttes_ici(), 3, "les trois gouttes prises");
    }

    /// Charge le niveau `indice` de la table dans une partie en cours (comme si on y
    /// était arrivé), avec la partie `etat`.
    fn aller_au_niveau(app: &mut Application, indice: usize, etat: EtatJeu) {
        app.recharger(indice, Some(etat));
        app.ecran = Ecran::Jeu;
    }

    fn indice_de(nom: &str) -> usize {
        niveaux::niveaux()
            .iter()
            .position(|n| n.nom == nom)
            .unwrap()
    }

    /// Carte 7.9 : REJOUER un niveau ne recrédite rien. La descente finie avec ses
    /// 3 gouttes rapporte la première fois ; rejouée de même, +0 (« 3 GOUTTES DÉJÀ
    /// COMPTÉES ») et le porte-monnaie ne bouge plus. Le meilleur temps reste noté.
    #[test]
    fn rejouer_un_niveau_ne_recredite_pas() {
        let mut app = Application::new(true);
        app.lancer(Suite::NiveauSeul(indice_de("La descente")));
        let mut poche = Vec::new();
        for _ in 0..2 {
            jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
            passer_la_carte(&mut app);
            ramasser_les_gouttes(&mut app);
            app.monde.termine = true;
            avancer(&mut app, 1);
            assert_eq!(app.ecran, Ecran::BilanNiveau);
            let r = app.resultat_niveau.clone().unwrap();
            poche.push((r.gain, r.deja_comptees, app.sauvegarde.gouttes_en_poche));
            app.valider_choix(Choix::RejouerNiveau);
        }
        let (gain, deja, en_poche) = poche[0];
        assert!(gain >= 3 && deja == 0 && en_poche == gain, "{poche:?}");
        assert_eq!(poche[1], (0, 3, en_poche), "rejouée : +0");
        assert!(app.sauvegarde.progres("descente").meilleur_temps.is_some());
    }

    #[test]
    fn une_partie_commence_avec_cinq_vies_et_chaque_mort_en_coute_une() {
        let mut app = en_jeu();
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
        tomber(&mut app);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART - 1);
        assert_eq!(app.monde.etat.morts, 1);
        assert_eq!(app.ecran, Ecran::Jeu, "Pépin réapparaît, on continue");
        // Le HUD les affiche, sans éclat (une vie perdue n'est pas une fête).
        let v = hud::vies(app.monde.etat.vies.restantes(), app.eclat_vie);
        assert_eq!((v.chiffre.as_str(), v.plus_un), ("4", None));
    }

    /// La dernière vie perdue : l'écran « PLUS DE VIES », puis le niveau EN COURS
    /// recommence depuis son début, avec cinq vies, sans les gouttes de ce niveau
    /// (celles d'avant gardées), le chrono qui continue et toutes les morts comptées.
    #[test]
    fn plus_de_vies_ouvre_l_ecran_puis_recommence_le_niveau_en_cours() {
        let mut app = en_jeu();
        // Niveau 1 fini avec deux gouttes : on arrive au niveau 2.
        app.monde.etat.gouttes = 2;
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(app.monde.infos.numero, 2);
        avancer(&mut app, 30);
        ramasser_les_gouttes(&mut app);
        assert_eq!(app.monde.etat.gouttes, 5);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1, "+1 vie");
        for _ in 0..VIES_DEPART {
            tomber(&mut app);
            assert_eq!(app.ecran, Ecran::Jeu);
        }
        tomber(&mut app);
        assert_eq!(app.ecran, Ecran::PlusDeVies(plus_de_vies::DUREE_PAS));
        let chrono = app.pas_partie;
        let morts = app.monde.etat.morts;
        assert_eq!(morts, u32::from(VIES_DEPART) + 1);
        // L'écran attend (le monde est figé), puis le joueur appuie (carte 9.31).
        let pas_monde = app.monde.pas;
        avancer(&mut app, 60);
        assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
        assert_eq!(app.monde.pas, pas_monde, "le monde ne bouge pas");
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        assert_eq!(app.monde.infos.numero, 2, "le même niveau");
        assert_eq!(app.monde.pas, 0, "tout neuf");
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
        assert_eq!(app.monde.etat.gouttes, 2, "celles du niveau 1 restent");
        assert_eq!(app.monde.etat.gouttes_ici(), 0);
        assert_eq!(
            app.gouttes_du_niveau(),
            0,
            "les gouttes du niveau sont revenues"
        );
        assert_eq!(app.monde.etat.morts, morts, "l'historique des morts");
        assert_eq!(app.pas_partie, chrono, "le chrono reprend où il était");
        assert_eq!(
            (app.monde.joueur.boite.x, app.monde.joueur.boite.y),
            app.monde.niveau.depart,
            "au départ du niveau"
        );
        // Les trois gouttes reprises ne redonnent pas de vie (déjà donnée ici).
        avancer(&mut app, 30);
        ramasser_les_gouttes(&mut app);
        assert_eq!(app.monde.etat.gouttes, 5);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
        // Le jeu continue : chrono et monde avancent.
        avancer(&mut app, 1);
        assert_eq!(app.ecran, Ecran::Jeu);
    }

    #[test]
    fn on_passe_l_ecran_plus_de_vies_d_un_appui() {
        let mut app = en_jeu();
        for _ in 0..VIES_DEPART {
            tomber(&mut app);
        }
        assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
        // L'appui est accepté après le court délai d'ouverture (carte 9.31).
        avancer(&mut app, histoire::DELAI_PAS);
        app.reagir(&valider(), &Commandes::default(), None);
        assert!(
            app.transition.est_active(),
            "les volets partent tout de suite"
        );
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
        assert_eq!(app.monde.pas, 0);
        // Un toucher (clic) aussi.
        for _ in 0..VIES_DEPART {
            tomber(&mut app);
        }
        let toucher = Navigation {
            clic: Some((160.0, 90.0)),
            ..Navigation::default()
        };
        avancer(&mut app, histoire::DELAI_PAS);
        app.reagir(&toucher, &Commandes::default(), None);
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        assert_eq!(app.monde.etat.morts, 2 * u32::from(VIES_DEPART));
    }

    /// Focus perdu (ou pause demandée) pendant l'écran « PLUS DE VIES » : l'écran
    /// finit, le niveau recommence, et la pause s'ouvre AVANT son premier pas.
    #[test]
    fn focus_perdu_ou_pause_pendant_plus_de_vies_la_pause_attend_le_niveau() {
        for par_la_pause in [false, true] {
            let mut app = en_jeu();
            for _ in 0..VIES_DEPART {
                tomber(&mut app);
            }
            assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
            if par_la_pause {
                app.reagir(&rien(), &pause(), None);
            } else {
                app.focus(false);
            }
            assert!(
                matches!(app.ecran, Ecran::PlusDeVies(_)),
                "pas de pause ici"
            );
            let chrono = app.pas_partie;
            jusqu_a(&mut app, |e| e == Ecran::Pause);
            assert_eq!(app.monde.pas, 0, "le niveau recommencé attend le joueur");
            assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
            assert_eq!(app.pas_partie, chrono);
            assert!(!app.boucles_sonores());
            // Reprise : le jeu repart normalement.
            app.reagir(&rien(), &pause(), None);
            avancer(&mut app, 3);
            assert_eq!(app.ecran, Ecran::Jeu);
            assert_eq!(app.monde.pas, 3);
        }
        // Focus perdu PUIS revenu pendant l'écran : aucune pause.
        let mut app = en_jeu();
        for _ in 0..VIES_DEPART {
            tomber(&mut app);
        }
        app.focus(false);
        app.focus(true);
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        avancer(&mut app, 3);
        assert_eq!((app.ecran, app.monde.pas), (Ecran::Jeu, 3));
    }

    /// Dans l'arène du Grand Nuage, un « PLUS DE VIES » le remet à ses trois points
    /// de vie (une simple mort, elle, les lui laisse perdus).
    #[test]
    fn plus_de_vies_rend_au_grand_nuage_ses_trois_points_de_vie() {
        let mut app = en_jeu();
        let etat = app.monde.etat.clone();
        aller_au_niveau(&mut app, indice_de(niveaux::par_cle("cyclone").nom), etat);
        assert_eq!(app.monde.pv_nuage(), Some(3));
        for e in &mut app.monde.entites {
            if let Entite::Nuage(g) = e {
                g.essouffler();
                g.toucher();
            }
        }
        assert_eq!(app.monde.pv_nuage(), Some(2));
        tomber(&mut app);
        assert_eq!(
            app.monde.pv_nuage(),
            Some(2),
            "une mort : il garde ses points perdus"
        );
        for _ in 1..VIES_DEPART {
            tomber(&mut app);
        }
        assert!(matches!(app.ecran, Ecran::PlusDeVies(_)));
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        assert_eq!(app.monde.pv_nuage(), Some(3), "le niveau recommence");
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
    }

    /// Le niveau bonus ne coûte aucune vie ; l'épilogue, si.
    #[test]
    fn le_niveau_bonus_ne_coute_aucune_vie() {
        let mut app = en_jeu();
        let etat = app.monde.etat.clone();
        let bonus = niveaux::niveaux()
            .iter()
            .position(|n| n.genre == GenreNiveau::Bonus)
            .unwrap();
        aller_au_niveau(&mut app, bonus, etat);
        for _ in 0..3 * VIES_DEPART {
            tomber(&mut app);
        }
        assert_eq!(app.ecran, Ecran::Jeu);
        assert_eq!(app.monde.etat.morts, 3 * u32::from(VIES_DEPART));
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART);
        // L'épilogue (le niveau d'après) en coûte de nouveau.
        let etat = app.monde.etat.clone();
        aller_au_niveau(&mut app, bonus + 1, etat);
        assert_eq!(app.monde.infos.genre, GenreNiveau::Epilogue);
        tomber(&mut app);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART - 1);
    }

    /// La troisième goutte d'un niveau : une vie de plus, et le petit signe du HUD
    /// (chiffre en or, « +1 ») pendant une seconde. Plafond : 9.
    #[test]
    fn la_troisieme_goutte_donne_une_vie_et_le_hud_le_montre() {
        let mut app = en_jeu();
        avancer(&mut app, 30);
        ramasser_les_gouttes(&mut app);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1);
        assert_eq!(app.eclat_vie, hud::DUREE_ECLAT_VIE);
        let v = hud::vies(app.monde.etat.vies.restantes(), app.eclat_vie);
        assert_eq!(v.chiffre, "6");
        assert!(v.plus_un.is_some());
        avancer(&mut app, u32::from(hud::DUREE_ECLAT_VIE));
        assert_eq!(app.eclat_vie, 0);
        assert!(hud::vies(6, app.eclat_vie).plus_un.is_none());
        // Recommencer (pause) : les gouttes reviennent, mais pas la vie (déjà donnée),
        // et la vie gagnée reste.
        app.reagir(&rien(), &pause(), None);
        app.valider_choix(Choix::Recommencer);
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1);
        avancer(&mut app, 30);
        ramasser_les_gouttes(&mut app);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART + 1);
        assert_eq!(app.eclat_vie, 0, "pas de signe sans vie gagnée");
        // Plafond : avec 9 vies, la vie du niveau suivant ne compte pas.
        let mut etat = app.monde.etat.clone();
        while etat.vies.restantes() < VIES_MAX {
            etat.nouveau_niveau((0.0, 0.0), GenreNiveau::Normal);
            let mut j = app.monde.joueur.clone();
            let trois = [Contact::Goutte(0), Contact::Goutte(1), Contact::Goutte(2)];
            etat.traiter(&mut j, &trois);
        }
        aller_au_niveau(&mut app, 1, etat);
        avancer(&mut app, 30);
        ramasser_les_gouttes(&mut app);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_MAX);
        assert_eq!(app.eclat_vie, 0);
    }

    /// « RECOMMENCER » (pause) ne rend pas les vies perdues dans le niveau.
    #[test]
    fn recommencer_ne_rend_pas_les_vies_perdues() {
        let mut app = en_jeu();
        tomber(&mut app);
        tomber(&mut app);
        app.reagir(&rien(), &pause(), None);
        app.valider_choix(Choix::Recommencer);
        jusqu_a(&mut app, |e| e == Ecran::Jeu);
        assert_eq!(app.monde.etat.vies.restantes(), VIES_DEPART - 2);
        assert_eq!(app.monde.etat.morts, 2);
    }

    /// L'écran de fin reçoit les vies restantes.
    #[test]
    fn le_bilan_garde_les_vies_restantes() {
        let mut app = en_jeu();
        let etat = app.monde.etat.clone();
        aller_au_niveau(&mut app, niveaux::NB_NIVEAUX_MONDE_1 - 1, etat);
        tomber(&mut app);
        app.monde.termine = true;
        avancer(&mut app, 1);
        jusqu_a(&mut app, |e| e == Ecran::Fin);
        assert_eq!(app.bilan.vies, VIES_DEPART - 1);
        assert_eq!(app.bilan.morts, 1);
    }

    // --- Histoire (carte 8.4) -----------------------------------------------------

    /// Avance jusqu'à la prochaine image de l'histoire ou carte de niveau (sans passer
    /// l'image) ; renvoie le chapitre montré (`None` : directement la carte).
    fn image_suivante(app: &mut Application) -> Option<Chapitre> {
        jusqu_a(app, |e| matches!(e, Ecran::Histoire(_) | Ecran::Carte(_)));
        match app.ecran {
            Ecran::Histoire(c) => Some(c),
            _ => None,
        }
    }

    /// Termine le niveau en cours (gouttes de la partie inchangées).
    fn finir_le_niveau(app: &mut Application) {
        if matches!(app.ecran, Ecran::Carte(_)) {
            passer_la_carte(app);
        }
        assert_eq!(app.ecran, Ecran::Jeu);
        app.monde.termine = true;
        avancer(app, 1);
    }

    /// JOUER : la première image avant la carte du niveau 1. Elle ignore les appuis un
    /// instant (un saut tenu ne la ferait pas disparaître sans être vue), puis un appui
    /// mène à la carte du niveau 1, déjà chargé ; le chrono n'a pas bougé.
    #[test]
    fn jouer_montre_la_premiere_image_avant_le_niveau_1() {
        let mut app = Application::new(true);
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(image_suivante(&mut app), Some(Chapitre::Depart));
        assert_eq!(app.monde.indice, 0);
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart), "trop tôt");
        avancer(&mut app, histoire::DELAI_PAS);
        // Rien ne la passe toute seule.
        avancer(&mut app, 600);
        assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart));
        // La fenêtre perd le focus : aucune pause (rien ne bouge derrière l'image).
        app.focus(false);
        assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart));
        assert!(!app.pause_differee);
        app.focus(true);
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(image_suivante(&mut app), None);
        assert_eq!(app.ecran, Ecran::Carte(carte::DUREE_PAS));
        assert_eq!(app.monde.indice, 0);
        assert_eq!(app.chrono(), 0, "l'histoire ne compte pas dans le chrono");
    }

    /// Une partie de JOUER à la fin : les trois images, chacune UNE fois, aux bons
    /// niveaux (avant le 1er, après le Grand Nuage, avant la descente), bonus joué ou
    /// non. Mourir sans vies ou « recommencer » ne les remontre pas. Score et gouttes
    /// ne bougent pas en les passant.
    #[test]
    fn une_partie_montre_les_trois_images_une_fois_chacune() {
        // Toutes les gouttes du MONDE 1 (carte 9.6 : pas celles de toute la table).
        let toutes = levels::gouttes_pour_le_bonus(niveaux::niveaux_monde_1());
        for gouttes in [0, toutes] {
            let mut app = Application::new(true);
            app.reagir(&valider(), &Commandes::default(), None);
            let mut vues = Vec::new();
            loop {
                if let Some(c) = image_suivante(&mut app) {
                    vues.push((c, app.monde.indice));
                    let e = &app.monde.etat;
                    let avant = (e.score, e.gouttes, e.morts, e.vies.restantes());
                    avancer(&mut app, histoire::DELAI_PAS);
                    app.reagir(&valider(), &Commandes::default(), None);
                    jusqu_a(&mut app, |e| matches!(e, Ecran::Carte(_)));
                    let e = &app.monde.etat;
                    let apres = (e.score, e.gouttes, e.morts, e.vies.restantes());
                    assert_eq!(apres, avant, "rien ne change en passant l'image");
                }
                if app.monde.indice == 1 {
                    // « Recommencer » ne remontre pas d'image.
                    passer_la_carte(&mut app);
                    app.valider_choix(Choix::Recommencer);
                    jusqu_a(&mut app, |e| e == Ecran::Jeu);
                }
                app.monde.etat.gouttes = gouttes;
                finir_le_niveau(&mut app);
                jusqu_a(&mut app, |e| {
                    matches!(e, Ecran::Histoire(_) | Ecran::Carte(_) | Ecran::Fin)
                });
                if app.ecran == Ecran::Fin {
                    break;
                }
            }
            let indice_de_la_suite_du_boss = indice_de(niveaux::par_cle("cyclone").nom) + 1;
            assert_eq!(
                vues,
                vec![
                    (Chapitre::Depart, 0),
                    (Chapitre::Lueur, indice_de_la_suite_du_boss),
                    (Chapitre::Soleil, niveaux::NB_NIVEAUX_MONDE_1 - 1),
                ],
                "gouttes : {gouttes}"
            );
        }
    }

    /// REJOUER (l'histoire vient d'être vue) et les raccourcis de niveau du build de
    /// QA : aucune image.
    #[test]
    fn rejouer_et_les_raccourcis_ne_racontent_pas_l_histoire() {
        let mut app = en_jeu();
        app.ecran = Ecran::Fin;
        app.choix = choix_pour(Ecran::Fin);
        app.menu = Menu::new(2);
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(image_suivante(&mut app), None, "REJOUER");
        let mut app = en_jeu();
        let descente = niveaux::NB_NIVEAUX_MONDE_1 - 1;
        app.reagir(
            &rien(),
            &Commandes::default(),
            Some(RaccourciNiveau::Aller(descente)),
        );
        assert_eq!(image_suivante(&mut app), None, "raccourci");
        assert_eq!(app.monde.indice, descente);
    }

    /// HISTOIRE au titre : les trois images dans l'ordre (sans rien charger), puis le
    /// titre, curseur sur HISTOIRE ; Échap y revient à tout moment.
    #[test]
    fn la_ligne_histoire_du_titre_montre_les_trois_images() {
        let mut app = Application::new(true);
        // Pas de ligne HISTOIRE avant une partie finie (carte 8.11) : 4 lignes.
        assert!(!app.choix.contains(&Choix::Histoire));
        assert_eq!(app.choix.len(), 4);
        app.sauvegarde.parties_finies = 1;
        app.ouvrir(Ecran::Titre);
        // NOUVELLE PARTIE + désactivée pour l'instant (AGA) : absente même jeu fini.
        assert_eq!(
            app.choix.contains(&Choix::NouvellePartiePlus),
            NOUVELLE_PARTIE_PLUS_OUVERTE
        );
        let ligne = app
            .choix
            .iter()
            .position(|&c| c == Choix::Histoire)
            .unwrap();
        app.menu.choisir(ligne);
        app.reagir(&valider(), &Commandes::default(), None);
        for attendu in Chapitre::TOUS {
            jusqu_a(&mut app, |e| e == Ecran::Histoire(attendu));
            avancer(&mut app, histoire::DELAI_PAS);
            app.reagir(&valider(), &Commandes::default(), None);
        }
        assert_eq!(app.ecran, Ecran::Titre);
        assert_eq!(app.choix[app.menu.selection()], Choix::Histoire);
        assert_eq!(app.chrono(), 0);
        // Échap, même tout de suite : retour au titre.
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Histoire(Chapitre::Depart));
        let retour = Navigation {
            retour: true,
            ..Navigation::default()
        };
        app.reagir(&retour, &Commandes::default(), None);
        assert_eq!(app.ecran, Ecran::Titre);
        assert_eq!(app.choix[app.menu.selection()], Choix::Histoire);
        // Puis JOUER raconte de nouveau l'histoire, et passe aux niveaux, quand il
        // repart du tout premier niveau (AGA : JOUER reprend où l'on en est ; une
        // partie finie ouvre tout le Monde 1, on y reprendrait donc au bout).
        app.sauvegarde = Sauvegarde::depuis_texte("version=3");
        app.menu.choisir(0);
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(image_suivante(&mut app), Some(Chapitre::Depart));
        avancer(&mut app, histoire::DELAI_PAS);
        app.reagir(&valider(), &Commandes::default(), None);
        assert_eq!(image_suivante(&mut app), None);
    }
}
