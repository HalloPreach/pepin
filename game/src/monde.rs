//! `Monde` : un niveau en cours de jeu (joueur, entités, état de partie).
//!
//! Il enchaîne, à chaque pas fixe de 1/60 s, les étapes dans l'ordre documenté
//! par `pepin_core::entities` : entités -> plateformes porteuses -> joueur ->
//! contacts -> état. Il garde aussi les positions du pas précédent pour que le
//! rendu puisse interpoler (affichage fluide même si l'écran n'est pas à 60 Hz).
//!
//! Notions Rust illustrées ici : struct qui POSSÈDE ses données, emprunts
//! partagés (`&self.niveau`) pendant qu'on modifie un autre champ, `Vec` de
//! `Contact` renvoyé par le cœur.

use pepin_core::aabb::Aabb;
use pepin_core::entities::contacts::{
    porter_par_plateformes, resoudre_contacts, verifier_vide, CauseMort, Contact,
};
use pepin_core::entities::pieuvre::{cases_englouties, GrilleSaisie};
use pepin_core::entities::{
    courants_des_entites, eaux_des_entites, reprendre_apres_une_mort, solides_des_entites,
    Actualisable, Entite,
};
use pepin_core::level::Niveau;
use pepin_core::levels::variantes;
use pepin_core::physics::{CourantAscendant, Eau, Environnement, Vec2};
use pepin_core::player::{Direction, Entrees, Evenements, Joueur};
use pepin_core::poursuite::MurDePluie;
use pepin_core::state::{EtatJeu, Ordre};
use pepin_core::zones::InfosNiveau;

use crate::mort::{AnimationMort, CauseDeMort, RIDEAU_FERME};

/// Sans temps de mort (`Monde::temps_de_mort` faux) : pas restants du court
/// clignotement de réapparition, le monde continuant de tourner.
pub const REAPPARITION_PAS: u16 = 18;
/// Durée (pas) de l'encre de la Pieuvre (carte 9.12a) : l'écran s'assombrit 0,75 s
/// quand elle a eu Pépin. Un effet visuel, rien de plus.
pub const ENCRE_PAS: u16 = 45;

#[derive(Clone)]
pub struct Monde {
    /// Place du niveau dans la table `NIVEAUX` (0 pour le premier).
    pub indice: usize,
    pub infos: &'static InfosNiveau,
    pub niveau: Niveau,
    pub entites: Vec<Entite>,
    pub joueur: Joueur,
    pub etat: EtatJeu,
    /// Nombre de pas écoulés depuis le début du niveau (sert aux animations).
    pub pas: u32,
    /// Événements du dernier pas (saut, atterrissage, début de plané).
    pub evenements: Evenements,
    /// Contacts du dernier pas (gouttes ramassées, morts...).
    pub contacts: Vec<Contact>,
    /// Dernière cause de mort (pour les effets).
    pub derniere_mort: Option<CauseMort>,
    /// Pas restants du temps de mort (carte 9.24, `crate::mort`) : l'animation de
    /// la cause puis le rideau. Tant qu'il dure, le monde est figé et les
    /// commandes ne répondent plus (0 = Pépin en jeu).
    pub reapparition: u16,
    /// La mort en cours d'animation (cause, endroit) : lue par le dessin.
    pub mort: AnimationMort,
    /// Le temps de mort (carte 9.24) est-il joué ? Oui dans la partie (`app`) ; non
    /// dans les simulations (robot de QA, mesure, fantôme de la course, démo) :
    /// leurs parcours enregistrés supposent la réapparition immédiate d'avant.
    pub temps_de_mort: bool,
    /// Pas restants de l'encre de la Pieuvre (0 = aucune, voir `ENCRE_PAS`).
    pub encre: u16,
    /// Où Pépin est mort (coin haut-gauche), pour jouer l'animation de mort.
    pub pos_mort: (f32, f32),
    /// Le joueur a-t-il touché le drapeau d'arrivée ?
    pub termine: bool,
    /// Gouttes d'or ramassées dans CE niveau (index 0 à 2).
    pub gouttes_niveau: [bool; 3],
    /// La lettre cachée du niveau (carte 8.6) a été prise dans cette tentative.
    pub lettre_prise: bool,
    /// Pas passés immobile au sol (voir `compter_immobile`) : au bout de 2 s, Pépin
    /// s'occupe (animation d'attente, `scene::image_attente`). Sert au dessin seulement.
    pub immobile: u32,
    /// Secrets (carte 8.5) : zones cachées déjà révélées (une par zone de
    /// `niveau.caches`), la zone révélée à CE pas (le son « secret ») et la fin
    /// par la sortie secrète `x` (elle compte comme une fin normale).
    pub caches_reveles: Vec<bool>,
    pub secret_revele: bool,
    pub sortie_secrete: bool,
    /// Le mur de pluie du niveau de poursuite (carte 8.2) : `None` ailleurs.
    pub mur: Option<MurDePluie>,
    /// Nouvelle partie + (carte 8.8) : le niveau a été chargé avec sa variante
    /// (`levels::variantes`) ; la pluie dessinée est plus forte.
    pub ng_plus: bool,
    precedent_joueur: (f32, f32),
    precedent_entites: Vec<(f32, f32)>,
}

/// Indice du niveau d'essai vertical (carte 5.2) : juste après la table `NIVEAUX`. Il
/// n'existe QUE dans les tests et dans le build de QA (fonctionnalité `debug_niveaux`,
/// touche 0) ; en production, cet indice est hors de la table comme n'importe quel autre
/// et le niveau d'essai n'est pas dans le binaire.
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI: usize = niveaux::NB_NIVEAUX;

/// Indice du niveau d'essai « vent » (carte 5.3) : juste après le précédent
/// (touche F1 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_VENT: usize = niveaux::NB_NIVEAUX + 1;

/// Indice du niveau d'essai « grêle » (carte 5.4) : juste après celui du vent
/// (touche F2 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_GRELE: usize = niveaux::NB_NIVEAUX + 2;

/// Indice du niveau d'essai « auvent » (carte 5.5) : juste après celui de la
/// grêle (touche F3 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_AUVENT: usize = niveaux::NB_NIVEAUX + 3;

/// Indice du niveau d'essai « ventilateur » (carte 5.6) : juste après celui de
/// l'auvent (touche F4 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_VENTILATEUR: usize = niveaux::NB_NIVEAUX + 4;

/// Indice du niveau d'essai « montgolfière » (carte 5.7) : juste après celui du
/// ventilateur (touche F5 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_MONTGOLFIERE: usize = niveaux::NB_NIVEAUX + 5;

/// Indice du niveau d'essai « escargot » (carte 5.8) : juste après celui des
/// montgolfières (touche F6 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_ESCARGOT: usize = niveaux::NB_NIVEAUX + 6;

/// Indice du niveau d'essai « chat » (carte 5.9) : juste après celui des
/// escargots (touche F7 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_CHAT: usize = niveaux::NB_NIVEAUX + 7;

/// Indice du niveau d'essai « horloge » (carte 5.10) : juste après celui du chat
/// (touche F8 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_HORLOGE: usize = niveaux::NB_NIVEAUX + 8;

/// Indice du niveau d'essai « nuage » (carte 5.11) : l'arène du Grand Nuage, juste
/// après celui de l'horloge (touche F10 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_NUAGE: usize = niveaux::NB_NIVEAUX + 9;
/// Niveau d'essai « eau » (carte 9.2 : la mécanique de la barque, le Monde 2),
/// tests et build de QA seulement.
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_EAU: usize = niveaux::NB_NIVEAUX + 10;
/// Niveau d'essai « eau 2 » (carte 9.3 : bulles, nénuphars, marée, stalactites,
/// chutes d'eau), tests et build de QA seulement (touche F12).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_EAU2: usize = niveaux::NB_NIVEAUX + 11;

/// Niveau d'essai « ennemis d'eau » (carte 9.4 : grenouille, poisson, crabe, rat,
/// mouette ; touche J), tests et build de QA seulement. L'indice `+ 11` est celui
/// de l'essai de la carte 9.3 (touche F12).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_ENNEMIS_EAU: usize = niveaux::NB_NIVEAUX + 12;

/// Niveau d'essai « pieuvre » (carte 9.12a : l'arène de la Pieuvre du port, boss du
/// Monde 2 ; touche L), tests et build de QA seulement.
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_PIEUVRE: usize = niveaux::NB_NIVEAUX + 13;

/// Niveau d'essai VENU DE L'EXTÉRIEUR (carte 10.1b) : le fichier RON posé dans le
/// `localStorage` du navigateur, lancé par `?essai=1` (voir `essai_externe.rs`).
/// Juste après les niveaux d'essai embarqués ; build de QA et tests seulement.
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_EXTERNE: usize = niveaux::NB_NIVEAUX + niveaux::NB_ESSAIS;

/// Infos du niveau `indice` : la table `NIVEAUX`, puis les quatorze niveaux
/// d'essai (`INDICE_ESSAI` vertical, `INDICE_ESSAI_VENT` vent,
/// `INDICE_ESSAI_GRELE` grêle, `INDICE_ESSAI_AUVENT` auvent,
/// `INDICE_ESSAI_VENTILATEUR` ventilateur, `INDICE_ESSAI_MONTGOLFIERE`
/// montgolfière, `INDICE_ESSAI_ESCARGOT` escargot, `INDICE_ESSAI_CHAT` chat,
/// `INDICE_ESSAI_HORLOGE` horloge, `INDICE_ESSAI_NUAGE` arène du Grand Nuage,
/// `INDICE_ESSAI_EAU` eau, `INDICE_ESSAI_EAU2` eau 2, `INDICE_ESSAI_ENNEMIS_EAU` ennemis d'eau,
/// `INDICE_ESSAI_PIEUVRE` arène de la Pieuvre, `INDICE_ESSAI_EXTERNE` le niveau
/// posé dans le navigateur), tests et build de QA seulement. `None` au-delà. Tous
/// sont lus dans leurs fichiers `.ron` par le crate `niveaux` (carte 10.1b).
pub fn infos_niveau(indice: usize) -> Option<&'static InfosNiveau> {
    niveaux::niveaux()
        .get(indice)
        .or_else(|| niveau_essai(indice))
}

#[cfg(any(test, feature = "debug_niveaux"))]
fn niveau_essai(indice: usize) -> Option<&'static InfosNiveau> {
    if indice == INDICE_ESSAI {
        Some(niveaux::par_cle("essai_vertical"))
    } else if indice == INDICE_ESSAI_VENT {
        Some(niveaux::par_cle("essai_vent"))
    } else if indice == INDICE_ESSAI_GRELE {
        Some(niveaux::par_cle("essai_grele"))
    } else if indice == INDICE_ESSAI_AUVENT {
        Some(niveaux::par_cle("essai_auvent"))
    } else if indice == INDICE_ESSAI_VENTILATEUR {
        Some(niveaux::par_cle("essai_ventilateur"))
    } else if indice == INDICE_ESSAI_MONTGOLFIERE {
        Some(niveaux::par_cle("essai_montgolfiere"))
    } else if indice == INDICE_ESSAI_ESCARGOT {
        Some(niveaux::par_cle("essai_escargot"))
    } else if indice == INDICE_ESSAI_CHAT {
        Some(niveaux::par_cle("essai_chat"))
    } else if indice == INDICE_ESSAI_HORLOGE {
        Some(niveaux::par_cle("essai_horloge"))
    } else if indice == INDICE_ESSAI_NUAGE {
        Some(niveaux::par_cle("essai_nuage"))
    } else if indice == INDICE_ESSAI_EAU {
        Some(niveaux::par_cle("essai_eau"))
    } else if indice == INDICE_ESSAI_EAU2 {
        Some(niveaux::par_cle("essai_eau2"))
    } else if indice == INDICE_ESSAI_ENNEMIS_EAU {
        Some(niveaux::par_cle("essai_ennemis_eau"))
    } else if indice == INDICE_ESSAI_PIEUVRE {
        Some(niveaux::par_cle("essai_pieuvre"))
    } else if indice == INDICE_ESSAI_EXTERNE {
        crate::essai_externe::niveau()
    } else {
        None
    }
}

/// Production : aucun niveau en dehors de la table.
#[cfg(not(any(test, feature = "debug_niveaux")))]
fn niveau_essai(_indice: usize) -> Option<&'static InfosNiveau> {
    None
}

impl Monde {
    /// Charge le niveau `indice` de la table `NIVEAUX` (ou le niveau d'essai, voir
    /// `infos_niveau`). Les niveaux sont validés par les tests du crate `niveaux`
    /// (et `./build.sh`) : un échec ici est une erreur de programmation, d'où le `expect`. Toujours en partie normale
    /// (le fantôme de la course, carte 8.7, les tests) ; la partie passe par `charger`.
    pub fn new(indice: usize) -> Self {
        Self::charger(indice, false)
    }

    /// Comme `new`, en nouvelle partie + si `ng_plus` (carte 8.8) : la variante du
    /// niveau (ennemis déplacés, éclairs plus fréquents, Grand Nuage serré, vent de
    /// face sur les toits), validée par les tests du cœur.
    pub fn charger(indice: usize, ng_plus: bool) -> Self {
        let infos = infos_niveau(indice).expect("indice de niveau valide");
        let niveau = if ng_plus {
            variantes::charger_ng_plus(infos).expect("variante embarquée valide")
        } else {
            infos.charger().expect("niveau embarqué valide")
        };
        let joueur = Joueur::new(niveau.depart.0, niveau.depart.1);
        let mut etat = EtatJeu::new(niveau.depart);
        etat.appliquer(Ordre::Demarrer)
            .expect("une partie neuve peut démarrer");
        // Le genre du niveau dit s'il coûte des vies (pas le bonus, carte 6.2) :
        // compte aussi pour un niveau chargé seul (tests, mesure, build de QA).
        etat.nouveau_niveau(niveau.depart, infos.genre);
        let entites = niveau.entites.clone();
        let mur = MurDePluie::du_niveau(&niveau);
        let precedent_entites = entites.iter().map(Self::coin).collect();
        let caches_reveles = vec![false; niveau.caches.len()];
        Self {
            indice,
            infos,
            precedent_joueur: (joueur.boite.x, joueur.boite.y),
            niveau,
            entites,
            joueur,
            etat,
            pas: 0,
            evenements: Evenements::default(),
            contacts: Vec::new(),
            derniere_mort: None,
            reapparition: 0,
            mort: AnimationMort::default(),
            temps_de_mort: false,
            encre: 0,
            pos_mort: (0.0, 0.0),
            termine: false,
            gouttes_niveau: [false; 3],
            lettre_prise: false,
            immobile: 0,
            caches_reveles,
            secret_revele: false,
            sortie_secrete: false,
            mur,
            ng_plus,
            precedent_entites,
        }
    }

    fn coin(e: &Entite) -> (f32, f32) {
        let b = e.boite();
        (b.x, b.y)
    }

    /// Un pas de simulation (1/60 s) avec les entrées données.
    pub fn avancer(&mut self, entrees: &Entrees) {
        self.precedent_joueur = (self.joueur.boite.x, self.joueur.boite.y);
        self.precedent_entites = self.entites.iter().map(Self::coin).collect();
        // Temps de mort (carte 9.24) : le monde est figé, les entrées ignorées.
        if self.temps_de_mort && self.reapparition > 0 {
            self.pas_de_mort();
            return;
        }

        // La Pieuvre du port (carte 9.12f) voit où est Pépin : elle vise sa
        // plateforme, et ne la fait jamais remonter dans lui.
        for e in &mut self.entites {
            if let Entite::Pieuvre(p) = e {
                p.voir_pepin(self.joueur.boite);
            }
        }
        // 1. Les entités avancent (plateformes, ennemis, éclairs...).
        let solides = solides_des_entites(&self.entites);
        {
            let trous = cases_englouties(&self.entites);
            let grille = GrilleSaisie {
                grille: &self.niveau,
                trous: &trous,
            };
            let env = Environnement {
                grille: &grille,
                solides: &solides,
                courants: &self.niveau.courants,
                vents: &self.niveau.vents,
                eaux: &[],
            };
            for e in &mut self.entites {
                e.actualiser(&env);
            }
        }
        // Le mur de pluie avance avec elles (carte 8.2).
        if let Some(mur) = &mut self.mur {
            mur.avancer(&self.joueur.boite);
        }
        // 2. Les plateformes ont bougé : on recalcule les solides, le joueur est porté.
        let solides = solides_des_entites(&self.entites);
        // Les courants des ventilateurs (actifs uniquement pendant leur phase de
        // souffle) et celui du Grand Nuage essoufflé s'ajoutent aux courants
        // statiques du niveau : ce sont eux qui portent Pépin vers le haut.
        let courants: Vec<CourantAscendant> = self
            .niveau
            .courants
            .iter()
            .cloned()
            .chain(courants_des_entites(&self.entites))
            .collect();
        // La marée (carte 9.3) : son eau, qui monte et descend, s'ajoute à celle
        // du niveau. Seulement dans un niveau qui en a une : ailleurs, l'eau du
        // niveau sert telle quelle (rien ne change au Monde 1).
        let a_une_maree = self.entites.iter().any(|e| matches!(e, Entite::Maree(_)));
        let eaux_du_moment = if a_une_maree {
            eaux_des_entites(&self.niveau.eaux, &self.entites)
        } else {
            Vec::new()
        };
        // La plateforme que la Pieuvre a emportée sous l'eau ne porte plus : on
        // perce la grille de ses cases, le temps de la plongée (carte 9.12f).
        let trous = cases_englouties(&self.entites);
        let grille = GrilleSaisie {
            grille: &self.niveau,
            trous: &trous,
        };
        let env = Environnement {
            grille: &grille,
            solides: &solides,
            courants: &courants,
            vents: &self.niveau.vents,
            // L'eau (carte 9.2) : la barque flotte, la noyade tue. Seuls le
            // joueur et ses états (barque/noyade) en sont affectés : les
            // entités (plateformes, ennemis) ne s'y baignent pas.
            eaux: if a_une_maree {
                &eaux_du_moment
            } else {
                &self.niveau.eaux
            },
        };
        porter_par_plateformes(&mut self.joueur, &self.entites, &env);
        // 3. Le joueur bouge.
        // Avec les crochets du niveau (carte 11.1) : sans crochet, c'est
        // exactement `actualiser`.
        self.evenements =
            self.joueur
                .actualiser_avec_crochets(entrees, &env, &self.niveau.crochets);
        // 4. Contacts (+ chute dans le vide).
        let mut contacts = resoudre_contacts(&mut self.joueur, &mut self.entites);
        if !contacts.iter().any(|c| matches!(c, Contact::Mort(_))) {
            if let Some(mort) = verifier_vide(&self.joueur, self.niveau.hauteur_px()) {
                contacts.push(mort);
            }
        }
        // Le mur de pluie tue au contact (carte 8.2).
        if self.mur.is_some_and(|m| m.touche(&self.joueur.boite))
            && !contacts.iter().any(|c| matches!(c, Contact::Mort(_)))
        {
            contacts.push(Contact::Mort(CauseMort::Pluie));
        }
        // 5. L'état de partie digère les contacts (score, checkpoint, respawn).
        //    On retient où Pépin se trouvait : l'animation de mort se joue là.
        let avant_mort = self.joueur.boite;
        let eaux = if a_une_maree {
            &eaux_du_moment
        } else {
            &self.niveau.eaux
        };
        let surface = surface_sous(eaux, &avant_mort);
        let regarde_droite = self.joueur.regarde_droite;
        self.etat.traiter(&mut self.joueur, &contacts);
        for c in &contacts {
            match *c {
                Contact::Goutte(i) => {
                    if let Some(g) = self.gouttes_niveau.get_mut(i as usize) {
                        *g = true;
                    }
                }
                Contact::Lettre => self.lettre_prise = true,
                Contact::Mort(cause) => {
                    // Le cœur a déjà replacé Pépin à la lanterne. Avec le temps de
                    // mort, le monde se remet en place derrière le rideau
                    // (`pas_de_mort`), pas avant : on voit l'animation de mort dans
                    // le monde tel qu'il était.
                    self.derniere_mort = Some(cause);
                    let anime = CauseDeMort::depuis(cause);
                    self.mort = AnimationMort {
                        cause: anime,
                        boite: (avant_mort.x, avant_mort.y, avant_mort.w, avant_mort.h),
                        surface: surface.unwrap_or(avant_mort.bas()),
                        regarde_droite,
                    };
                    if self.temps_de_mort {
                        self.reapparition = anime.duree_totale();
                    } else {
                        self.remettre_en_place();
                        self.reapparition = REAPPARITION_PAS;
                    }
                    // La Pieuvre a eu Pépin : l'encre (carte 9.12a).
                    if self.derniere_mort == Some(CauseMort::Pieuvre) {
                        self.encre = ENCRE_PAS;
                    }
                    // Mort dans le vide : on garde l'animation visible, au bas du niveau.
                    self.pos_mort = (
                        avant_mort.x,
                        avant_mort.y.min(self.niveau.hauteur_px() - 16.0),
                    );
                    // Téléportation : pas d'interpolation entre l'ancien et le nouvel endroit.
                    self.precedent_joueur = (self.joueur.boite.x, self.joueur.boite.y);
                }
                _ => {}
            }
        }
        self.contacts = contacts;
        if !self.temps_de_mort {
            self.reapparition = self.reapparition.saturating_sub(1);
        }
        self.encre = self.encre.saturating_sub(1);
        // Après la réapparition éventuelle : un Pépin replacé n'est pas encore au sol.
        self.immobile = compter_immobile(self.immobile, &self.joueur, entrees);

        // Fin du niveau : le drapeau touché, ou le boss vaincu (le ciel s'est
        // déchiré, ou la Pieuvre a replongé : leur arène n'a pas de drapeau).
        let drapeau = self
            .niveau
            .arrivee
            .is_some_and(|a| self.joueur.boite.chevauche(&a));
        let nuage_vaincu = self.entites.iter().any(Entite::boss_vaincu);
        // Secrets (carte 8.5) : une zone cachée que Pépin touche se révèle pour de
        // bon ; la sortie secrète finit le niveau comme le drapeau.
        self.secret_revele = false;
        for (z, revele) in self.niveau.caches.iter().zip(&mut self.caches_reveles) {
            if !*revele && z.touche(&self.joueur.boite) {
                *revele = true;
                self.secret_revele = true;
            }
        }
        let secrete = self
            .niveau
            .sortie_secrete
            .is_some_and(|a| self.joueur.boite.chevauche(&a));
        if drapeau || nuage_vaincu || secrete {
            self.sortie_secrete = secrete && !drapeau;
            self.termine = true;
        }
        self.pas += 1;
    }

    /// Un pas du temps de mort (carte 9.24) : rien ne bouge (entités, boss, mur,
    /// Pépin), seul le compteur avance. Rideau fermé (`RIDEAU_FERME`), le monde se
    /// remet en place comme il le faisait au pas de la mort.
    fn pas_de_mort(&mut self) {
        self.reapparition -= 1;
        if self.reapparition == RIDEAU_FERME {
            self.remettre_en_place();
            // Les entités replacées sautent : pas d'interpolation.
            self.precedent_entites = self.entites.iter().map(Self::coin).collect();
        }
        self.evenements = Evenements::default();
        self.contacts.clear();
        self.secret_revele = false;
        self.encre = self.encre.saturating_sub(1);
        self.pas += 1;
    }

    /// Après une mort, Pépin déjà replacé : le Grand Nuage reprend son combat au
    /// début (il garde ses points perdus) et Pépin réapparaît au début de l'arène ;
    /// les dalles friables se reforment (carte 5.16), sauf sous Pépin ; le mur
    /// recule à distance fixe derrière lui : jamais de mort en boucle (carte 8.2).
    fn remettre_en_place(&mut self) {
        reprendre_apres_une_mort(&mut self.entites, &self.joueur.boite);
        if let Some(mur) = &mut self.mur {
            mur.replacer(&self.joueur.boite);
        }
    }

    /// Points de vie du boss du niveau (`None` s'il n'y en a pas) : le HUD les
    /// affiche. Le boss : le Grand Nuage, ou la Pieuvre du port (carte 9.12a,
    /// `Entite::points_du_boss`) ; le nom est resté celui du premier boss.
    pub fn pv_nuage(&self) -> Option<u8> {
        self.points_nuage().map(|(pv, _)| pv)
    }

    /// Points de vie du boss et son maximum, `(pv, pv_max)` : le HUD montre
    /// `pv_max` cases (2 en mode facile, carte 8.14).
    pub fn points_nuage(&self) -> Option<(u8, u8)> {
        self.entites.iter().find_map(Entite::points_du_boss)
    }

    /// La Pieuvre du port (carte 9.12a) est-elle le boss du niveau ? (Le HUD dessine
    /// alors ses points de vie en petites pieuvres.)
    pub fn boss_pieuvre(&self) -> bool {
        self.entites.iter().any(|e| matches!(e, Entite::Pieuvre(_)))
    }

    /// Mode facile du boss (carte 8.14) : le boss du niveau (Grand Nuage ou Pieuvre)
    /// passe à deux points de vie. Sans boss, rien.
    pub fn passer_nuage_en_facile(&mut self) {
        for e in &mut self.entites {
            e.passer_boss_en_facile();
        }
    }

    /// Le niveau a-t-il de l'eau (étendues, ou marée qui en apporte, cartes 9.2 et
    /// 9.3) ? C'est là seulement que le bouton tactile BAS (retourner le parapluie,
    /// carte 9.2b) est montré.
    pub fn a_de_l_eau(&self) -> bool {
        !self.niveau.eaux.is_empty() || self.entites.iter().any(|e| matches!(e, Entite::Maree(_)))
    }

    /// Position interpolée de Pépin (coin haut-gauche), `alpha` entre 0 et 1.
    pub fn joueur_interpole(&self, alpha: f32) -> (f32, f32) {
        interpoler(
            self.precedent_joueur,
            (self.joueur.boite.x, self.joueur.boite.y),
            alpha,
        )
    }

    /// Boîte interpolée de l'entité `i` (même taille, position entre deux pas).
    pub fn boite_interpolee(&self, i: usize, alpha: f32) -> Aabb {
        let b = self.entites[i].boite();
        let (x, y) = interpoler(
            self.precedent_entites.get(i).copied().unwrap_or((b.x, b.y)),
            (b.x, b.y),
            alpha,
        );
        Aabb::new(x, y, b.w, b.h)
    }
}

/// Minuterie de l'attente : nombre de pas que Pépin vient de passer immobile au sol.
/// `avant` : la valeur du pas précédent. Elle repart de zéro au premier mouvement : une
/// touche de direction, de saut ou BAS enfoncée (même contre un mur, où il ne bouge pas), une
/// vitesse non nulle, ou les pieds qui quittent le sol (saut, chute, réapparition).
/// Elle ne fait que LIRE le joueur (`&Joueur`) : aucun effet sur la physique. Porté par
/// une plateforme mobile sans toucher à rien, Pépin est immobile (il attend, debout).
/// Fonction pure.
pub fn compter_immobile(avant: u32, joueur: &Joueur, entrees: &Entrees) -> u32 {
    let bouge = !joueur.au_sol
        || joueur.vitesse != Vec2::default()
        || entrees.direction != Direction::Immobile
        || entrees.saut
        // BAS tenu (carte 9.2b) : il tient son parapluie retourné, il n'attend pas.
        || entrees.bas;
    if bouge {
        0
    } else {
        avant.saturating_add(1)
    }
}

/// Noyade (carte 9.24) : la surface (px) de l'eau où la boîte `b` de Pépin est entrée.
fn surface_sous(eaux: &[Eau], b: &Aabb) -> Option<f32> {
    let (cx, _) = b.centre();
    eaux.iter()
        .find(|e| cx >= e.zone.x && cx <= e.zone.x + e.zone.w && b.chevauche(&e.zone))
        .map(|e| e.zone.y)
}

/// Interpolation linéaire ; au-delà de 32 px d'écart c'est une téléportation
/// (réapparition) : on n'interpole pas.
fn interpoler(avant: (f32, f32), apres: (f32, f32), alpha: f32) -> (f32, f32) {
    if (apres.0 - avant.0).abs() > 32.0 || (apres.1 - avant.1).abs() > 32.0 {
        return apres;
    }
    (
        avant.0 + (apres.0 - avant.0) * alpha,
        avant.1 + (apres.1 - avant.1) * alpha,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_niveau_1_se_charge_et_pepin_tombe_au_sol() {
        let mut m = Monde::new(0);
        for _ in 0..30 {
            m.avancer(&Entrees::default());
        }
        assert!(m.joueur.au_sol, "Pépin doit se poser sur le toit");
        assert_eq!(m.pas, 30);
    }

    #[test]
    fn courir_a_droite_avance_et_sauter_decolle() {
        let mut m = Monde::new(0);
        for _ in 0..30 {
            m.avancer(&Entrees::default());
        }
        let x0 = m.joueur.boite.x;
        let droite = Entrees {
            direction: Direction::Droite,
            saut: false,
            bas: false,
        };
        for _ in 0..30 {
            m.avancer(&droite);
        }
        assert!(m.joueur.boite.x > x0 + 20.0);
        let y0 = m.joueur.boite.y;
        m.avancer(&Entrees {
            direction: Direction::Droite,
            saut: true,
            bas: false,
        });
        for _ in 0..5 {
            m.avancer(&Entrees {
                direction: Direction::Droite,
                saut: true,
                bas: false,
            });
        }
        assert!(m.joueur.boite.y < y0 - 5.0, "le saut décolle");
    }

    #[test]
    fn tomber_dans_le_vide_fait_reapparaitre_au_depart() {
        let mut m = Monde::new(0);
        m.joueur
            .placer(m.niveau.largeur_px() / 2.0, m.niveau.hauteur_px() + 40.0);
        m.avancer(&Entrees::default());
        assert_eq!(m.etat.morts, 1);
        assert_eq!(m.joueur.boite.x, m.niveau.depart.0);
        assert!(m.reapparition > 0);
    }

    /// Avance (sans commande) jusqu'à la fin du temps de mort (carte 9.24).
    fn finir_la_mort(m: &mut Monde) {
        while m.reapparition > 0 {
            m.avancer(&Entrees::default());
        }
    }

    /// Carte 9.24 : Pépin, déjà replacé par le cœur, ne réapparaît qu'à l'ouverture
    /// du rideau, après l'animation de la cause ; sa protection de 1 s commence alors.
    #[test]
    fn pas_de_reapparition_avant_la_fin_de_l_animation() {
        use crate::dessin_mort::pepin_cache;
        use crate::mort::{CauseDeMort, RIDEAU_FERME};
        use pepin_core::player::PROTECTION_REAPPARITION_PAS;
        let mut m = Monde::new(0);
        m.temps_de_mort = true;
        m.joueur
            .placer(m.niveau.largeur_px() / 2.0, m.niveau.hauteur_px() + 40.0);
        m.avancer(&Entrees::default());
        assert_eq!(m.mort.cause, CauseDeMort::Vide);
        let total = CauseDeMort::Vide.duree_totale();
        assert_eq!(m.reapparition, total);
        let mut caches = 0;
        while m.reapparition > 0 {
            caches += u16::from(pepin_cache(m.reapparition));
            m.avancer(&Entrees::default());
        }
        assert_eq!(caches, total - RIDEAU_FERME, "caché jusqu'au rideau fermé");
        assert_eq!(m.joueur.protection_restante(), PROTECTION_REAPPARITION_PAS);
    }

    /// Carte 9.24 : pendant le temps de mort, les commandes sont ignorées et le
    /// monde est figé (Pépin et les entités) ; ensuite, Pépin répond de nouveau.
    #[test]
    fn les_commandes_sont_ignorees_pendant_l_animation() {
        let mut m = Monde::new(0);
        m.temps_de_mort = true;
        for _ in 0..30 {
            m.avancer(&Entrees::default());
        }
        m.joueur
            .placer(m.niveau.largeur_px() / 2.0, m.niveau.hauteur_px() + 40.0);
        m.avancer(&Entrees::default());
        let pepin = (m.joueur.boite.x, m.joueur.boite.y);
        let entites: Vec<(f32, f32)> = m.entites.iter().map(Monde::coin).collect();
        while m.reapparition > crate::mort::RIDEAU_FERME {
            m.avancer(&SAUT);
            assert_eq!((m.joueur.boite.x, m.joueur.boite.y), pepin);
            // Au pas du rideau fermé, les ennemis rentrent chez eux (carte 9.29) :
            // seul ce pas-là les déplace.
            if m.reapparition > crate::mort::RIDEAU_FERME {
                assert_eq!(
                    m.entites.iter().map(Monde::coin).collect::<Vec<_>>(),
                    entites
                );
            }
        }
        while m.reapparition > 0 {
            m.avancer(&DROITE);
        }
        assert_eq!((m.joueur.boite.x, m.joueur.boite.y), pepin);
        for _ in 0..10 {
            m.avancer(&DROITE);
        }
        assert!(m.joueur.boite.x > pepin.0, "Pépin repart");
    }

    /// Le bonus (carte 5.16), sans lanterne : une chute ramène au départ, et le pont
    /// friable s'y est déjà reformé en entier (sans attendre les 3 s d'absence
    /// d'une dalle tombée), pour repartir aussitôt.
    #[test]
    fn dans_le_bonus_une_chute_ramene_au_depart_sur_un_pont_reforme() {
        use pepin_core::entities::friable::EtatFriable;
        use pepin_core::levels::GenreNiveau;
        let i = niveaux::niveaux()
            .iter()
            .position(|n| n.genre == GenreNiveau::Bonus)
            .unwrap();
        let mut m = Monde::new(i);
        m.temps_de_mort = true;
        // Le début du pont est tombé, une autre dalle s'effrite.
        let mut friables = 0;
        for e in m.entites.iter_mut() {
            if let Entite::Friable(f) = e {
                friables += 1;
                f.etat = if friables < 10 {
                    EtatFriable::Absente(150)
                } else {
                    EtatFriable::Effritement(12)
                };
            }
        }
        assert!(friables > 100, "{friables} dalles");
        m.joueur
            .placer(m.niveau.largeur_px() / 2.0, m.niveau.hauteur_px() + 40.0);
        m.avancer(&Entrees::default());
        assert_eq!(m.etat.morts, 1);
        // Le pont se reforme derrière le rideau du temps de mort (carte 9.24).
        finir_la_mort(&mut m);
        assert_eq!(
            (m.joueur.boite.x, m.joueur.boite.y),
            m.niveau.depart,
            "au départ"
        );
        assert!(m.entites.iter().all(|e| match e {
            Entite::Friable(f) => f.etat == EtatFriable::Stable,
            _ => true,
        }));
    }

    #[test]
    fn l_interpolation_ignore_les_teleportations() {
        assert_eq!(interpoler((0.0, 0.0), (10.0, 0.0), 0.5), (5.0, 0.0));
        assert_eq!(interpoler((0.0, 0.0), (500.0, 0.0), 0.5), (500.0, 0.0));
    }

    // --- Minuterie de l'attente (carte 4.3) ---

    const DROITE: Entrees = Entrees {
        direction: Direction::Droite,
        saut: false,
        bas: false,
    };
    const SAUT: Entrees = Entrees {
        direction: Direction::Immobile,
        saut: true,
        bas: false,
    };

    /// Pépin posé, immobile (le vrai niveau 1 après 30 pas sans toucher à rien).
    fn pepin_pose() -> Monde {
        let mut m = Monde::new(0);
        for _ in 0..30 {
            m.avancer(&Entrees::default());
        }
        assert!(m.joueur.au_sol);
        m
    }

    #[test]
    fn la_minuterie_compte_les_pas_immobiles_au_sol() {
        let j = pepin_pose().joueur;
        assert_eq!(compter_immobile(0, &j, &Entrees::default()), 1);
        assert_eq!(compter_immobile(119, &j, &Entrees::default()), 120);
        // Jamais de débordement, même après des heures d'attente.
        assert_eq!(
            compter_immobile(u32::MAX, &j, &Entrees::default()),
            u32::MAX
        );
    }

    #[test]
    fn la_minuterie_repart_de_zero_au_premier_mouvement() {
        let pose = pepin_pose().joueur;
        // Une touche suffit, même si Pépin n'a pas encore bougé (ou bute contre un mur).
        assert_eq!(compter_immobile(500, &pose, &DROITE), 0);
        assert_eq!(compter_immobile(500, &pose, &SAUT), 0);
        // Une vitesse (il glisse encore après avoir lâché la touche).
        let mut glisse = pose.clone();
        glisse.vitesse.x = 3.0;
        assert_eq!(compter_immobile(500, &glisse, &Entrees::default()), 0);
        // En l'air (saut, chute, juste après une réapparition).
        let mut en_l_air = pose.clone();
        en_l_air.au_sol = false;
        assert_eq!(compter_immobile(500, &en_l_air, &Entrees::default()), 0);
    }

    /// Dans le vrai jeu : 3 s sans toucher à rien, la minuterie dépasse 2 s ; un seul pas
    /// avec une touche et elle retombe à zéro. Pépin, lui, ne bouge pas d'un pixel pendant
    /// l'attente : la minuterie ne change rien au jeu.
    #[test]
    fn dans_le_jeu_l_attente_commence_apres_2_s_et_cesse_au_premier_mouvement() {
        let mut m = pepin_pose();
        let (x, y) = (m.joueur.boite.x, m.joueur.boite.y);
        for _ in 0..180 {
            m.avancer(&Entrees::default());
        }
        assert!(m.immobile >= 120, "immobile = {}", m.immobile);
        assert_eq!((m.joueur.boite.x, m.joueur.boite.y), (x, y));
        m.avancer(&DROITE);
        assert_eq!(m.immobile, 0);
        // Il lâche tout : la minuterie repart de zéro (et non de 180) dès qu'il est arrêté.
        for _ in 0..30 {
            m.avancer(&Entrees::default());
        }
        assert!((1..=30).contains(&m.immobile), "immobile = {}", m.immobile);
    }

    /// La minuterie ne touche à rien : deux mondes qui reçoivent les mêmes touches restent
    /// identiques (joueur, entités, état), qu'on lise la minuterie ou non.
    #[test]
    fn la_minuterie_ne_change_pas_la_simulation() {
        let mut a = Monde::new(0);
        let mut b = Monde::new(0);
        let touches = [Entrees::default(), DROITE, SAUT, Entrees::default()];
        for pas in 0..600usize {
            let e = touches[(pas / 50) % touches.len()];
            a.avancer(&e);
            b.avancer(&e);
            b.immobile = 0; // b oublie sa minuterie à chaque pas
        }
        assert_eq!(a.joueur.boite, b.joueur.boite);
        assert_eq!(a.joueur.vitesse, b.joueur.vitesse);
        assert_eq!(a.etat.morts, b.etat.morts);
        assert_eq!(a.entites.len(), b.entites.len());
        for (ea, eb) in a.entites.iter().zip(&b.entites) {
            assert_eq!(ea.boite(), eb.boite());
        }
    }

    /// Le cas trouvé par la mesure 3.1, dans le vrai niveau 2 : Pépin attend, immobile,
    /// à la lanterne de la colonne 104 ; la flaque de la colonne 108 passe dessus, va
    /// au mur, revient. Avant la protection : une mort à chaque pas où la flaque
    /// recouvre la lanterne (des dizaines d'affilée). Maintenant : au plus une mort par
    /// passage, et jamais deux à moins d'une protection d'intervalle.
    #[test]
    fn la_flaque_sur_la_lanterne_104_du_niveau_2_ne_tue_plus_en_boucle() {
        use pepin_core::player::PROTECTION_REAPPARITION_PAS;
        let mut m = Monde::new(1);
        let point = m
            .entites
            .iter()
            .find_map(|e| match e {
                Entite::Lanterne(l) if (l.boite.x / 16.0) as i32 == 104 => {
                    Some(l.point_de_reapparition())
                }
                _ => None,
            })
            .expect("lanterne de la colonne 104");
        m.joueur.placer(point.0, point.1);
        let mut morts = Vec::new();
        // Un aller-retour de la flaque devant la lanterne (mur à 2 tuiles) : 6 s.
        for pas in 0..360u32 {
            let avant = m.etat.morts;
            m.avancer(&Entrees::default());
            if m.etat.morts > avant {
                morts.push(pas);
            }
        }
        assert_eq!(m.etat.checkpoint(), point, "lanterne 104 allumée");
        assert!(!morts.is_empty(), "la flaque passe bien sur Pépin");
        assert!(morts.len() <= 2, "morts aux pas {morts:?}");
        assert!(
            morts
                .windows(2)
                .all(|w| w[1] - w[0] >= PROTECTION_REAPPARITION_PAS as u32),
            "deux morts trop rapprochées : {morts:?}"
        );
        assert_eq!(
            m.joueur.boite.x, point.0,
            "Pépin est toujours à la lanterne"
        );
    }
}
