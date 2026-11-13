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
use pepin_core::entities::{courants_des_ventilateurs, solides_des_entites, Actualisable, Entite};
use pepin_core::level::Niveau;
use pepin_core::physics::{CourantAscendant, Environnement, Vec2};
use pepin_core::player::{Direction, Entrees, Evenements, Joueur};
use pepin_core::state::{EtatJeu, Ordre};
use pepin_core::zones::{self, InfosNiveau};

/// Durée (pas) de l'animation de réapparition de Pépin.
pub const REAPPARITION_PAS: u16 = 18;

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
    /// Pas restants de l'animation de réapparition (0 = aucune).
    pub reapparition: u16,
    /// Où Pépin est mort (coin haut-gauche), pour jouer l'animation de mort.
    pub pos_mort: (f32, f32),
    /// Le joueur a-t-il touché le drapeau d'arrivée ?
    pub termine: bool,
    /// Gouttes d'or ramassées dans CE niveau (index 0 à 2).
    pub gouttes_niveau: [bool; 3],
    /// Pas passés immobile au sol (voir `compter_immobile`) : au bout de 2 s, Pépin
    /// s'occupe (animation d'attente, `scene::image_attente`). Sert au dessin seulement.
    pub immobile: u32,
    precedent_joueur: (f32, f32),
    precedent_entites: Vec<(f32, f32)>,
}

/// Indice du niveau d'essai vertical (carte 5.2) : juste après la table `NIVEAUX`. Il
/// n'existe QUE dans les tests et dans le build de QA (fonctionnalité `debug_niveaux`,
/// touche 0) ; en production, cet indice est hors de la table comme n'importe quel autre
/// et le niveau d'essai n'est pas dans le binaire.
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI: usize = pepin_core::levels::NB_NIVEAUX;

/// Indice du niveau d'essai « vent » (carte 5.3) : juste après le précédent
/// (touche F1 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_VENT: usize = pepin_core::levels::NB_NIVEAUX + 1;

/// Indice du niveau d'essai « grêle » (carte 5.4) : juste après celui du vent
/// (touche F2 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_GRELE: usize = pepin_core::levels::NB_NIVEAUX + 2;

/// Indice du niveau d'essai « auvent » (carte 5.5) : juste après celui de la
/// grêle (touche F3 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_AUVENT: usize = pepin_core::levels::NB_NIVEAUX + 3;

/// Indice du niveau d'essai « ventilateur » (carte 5.6) : juste après celui de
/// l'auvent (touche F4 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_VENTILATEUR: usize = pepin_core::levels::NB_NIVEAUX + 4;

/// Indice du niveau d'essai « montgolfière » (carte 5.7) : juste après celui du
/// ventilateur (touche F5 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_MONTGOLFIERE: usize = pepin_core::levels::NB_NIVEAUX + 5;

/// Indice du niveau d'essai « escargot » (carte 5.8) : juste après celui des
/// montgolfières (touche F6 du build de QA).
#[cfg(any(test, feature = "debug_niveaux"))]
pub const INDICE_ESSAI_ESCARGOT: usize = pepin_core::levels::NB_NIVEAUX + 6;

/// Infos du niveau `indice` : la table `NIVEAUX`, puis les sept niveaux
/// d'essai (`INDICE_ESSAI` vertical, `INDICE_ESSAI_VENT` vent,
/// `INDICE_ESSAI_GRELE` grêle, `INDICE_ESSAI_AUVENT` auvent,
/// `INDICE_ESSAI_VENTILATEUR` ventilateur, `INDICE_ESSAI_MONTGOLFIERE`
/// montgolfière, `INDICE_ESSAI_ESCARGOT` escargot) — tests et build de QA
/// seulement. `None` au-delà.
pub fn infos_niveau(indice: usize) -> Option<&'static InfosNiveau> {
    zones::niveau(indice).or_else(|| niveau_essai(indice))
}

#[cfg(any(test, feature = "debug_niveaux"))]
fn niveau_essai(indice: usize) -> Option<&'static InfosNiveau> {
    if indice == INDICE_ESSAI {
        Some(&pepin_core::levels::essai_vertical::INFOS)
    } else if indice == INDICE_ESSAI_VENT {
        Some(&pepin_core::levels::essai_vent::INFOS)
    } else if indice == INDICE_ESSAI_GRELE {
        Some(&pepin_core::levels::essai_grele::INFOS)
    } else if indice == INDICE_ESSAI_AUVENT {
        Some(&pepin_core::levels::essai_auvent::INFOS)
    } else if indice == INDICE_ESSAI_VENTILATEUR {
        Some(&pepin_core::levels::essai_ventilateur::INFOS)
    } else if indice == INDICE_ESSAI_MONTGOLFIERE {
        Some(&pepin_core::levels::essai_montgolfiere::INFOS)
    } else if indice == INDICE_ESSAI_ESCARGOT {
        Some(&pepin_core::levels::essai_escargot::INFOS)
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
    /// `infos_niveau`). Les niveaux sont validés par les tests du cœur : un échec ici est
    /// une erreur de programmation, d'où le `expect`.
    pub fn new(indice: usize) -> Self {
        let infos = infos_niveau(indice).expect("indice de niveau valide");
        let niveau = infos.charger().expect("niveau embarqué valide");
        let joueur = Joueur::new(niveau.depart.0, niveau.depart.1);
        let mut etat = EtatJeu::new(niveau.depart);
        etat.appliquer(Ordre::Demarrer)
            .expect("une partie neuve peut démarrer");
        let entites = niveau.entites.clone();
        let precedent_entites = entites.iter().map(Self::coin).collect();
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
            pos_mort: (0.0, 0.0),
            termine: false,
            gouttes_niveau: [false; 3],
            immobile: 0,
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

        // 1. Les entités avancent (plateformes, ennemis, éclairs...).
        let solides = solides_des_entites(&self.entites);
        {
            let env = Environnement {
                grille: &self.niveau,
                solides: &solides,
                courants: &self.niveau.courants,
                vents: &self.niveau.vents,
            };
            for e in &mut self.entites {
                e.actualiser(&env);
            }
        }
        // 2. Les plateformes ont bougé : on recalcule les solides, le joueur est porté.
        let solides = solides_des_entites(&self.entites);
        // Les courants des ventilateurs (actifs uniquement pendant leur phase de
        // souffle) s'ajoutent aux courants statiques du niveau : c'est ce courant
        // qui porte Pépin vers le haut quand l'hélice souffle.
        let courants: Vec<CourantAscendant> = self
            .niveau
            .courants
            .iter()
            .cloned()
            .chain(courants_des_ventilateurs(&self.entites))
            .collect();
        let env = Environnement {
            grille: &self.niveau,
            solides: &solides,
            courants: &courants,
            vents: &self.niveau.vents,
        };
        porter_par_plateformes(&mut self.joueur, &self.entites, &env);
        // 3. Le joueur bouge.
        self.evenements = self.joueur.actualiser(entrees, &env);
        // 4. Contacts (+ chute dans le vide).
        let mut contacts = resoudre_contacts(&mut self.joueur, &mut self.entites);
        if !contacts.iter().any(|c| matches!(c, Contact::Mort(_))) {
            if let Some(mort) = verifier_vide(&self.joueur, self.niveau.hauteur_px()) {
                contacts.push(mort);
            }
        }
        // 5. L'état de partie digère les contacts (score, checkpoint, respawn).
        //    On retient où Pépin se trouvait : l'animation de mort se joue là.
        let avant_mort = (self.joueur.boite.x, self.joueur.boite.y);
        self.etat.traiter(&mut self.joueur, &contacts);
        for c in &contacts {
            match *c {
                Contact::Goutte(i) => {
                    if let Some(g) = self.gouttes_niveau.get_mut(i as usize) {
                        *g = true;
                    }
                }
                Contact::Mort(cause) => {
                    self.derniere_mort = Some(cause);
                    self.reapparition = REAPPARITION_PAS;
                    // Mort dans le vide : on garde l'animation visible, au bas du niveau.
                    self.pos_mort = (
                        avant_mort.0,
                        avant_mort.1.min(self.niveau.hauteur_px() - 16.0),
                    );
                    // Téléportation : pas d'interpolation entre l'ancien et le nouvel endroit.
                    self.precedent_joueur = (self.joueur.boite.x, self.joueur.boite.y);
                }
                _ => {}
            }
        }
        self.contacts = contacts;
        self.reapparition = self.reapparition.saturating_sub(1);
        // Après la réapparition éventuelle : un Pépin replacé n'est pas encore au sol.
        self.immobile = compter_immobile(self.immobile, &self.joueur, entrees);

        if self.joueur.boite.chevauche(&self.niveau.arrivee) {
            self.termine = true;
        }
        self.pas += 1;
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
/// touche de direction ou de saut enfoncée (même contre un mur, où il ne bouge pas), une
/// vitesse non nulle, ou les pieds qui quittent le sol (saut, chute, réapparition).
/// Elle ne fait que LIRE le joueur (`&Joueur`) : aucun effet sur la physique. Porté par
/// une plateforme mobile sans toucher à rien, Pépin est immobile (il attend, debout).
/// Fonction pure.
pub fn compter_immobile(avant: u32, joueur: &Joueur, entrees: &Entrees) -> u32 {
    let bouge = !joueur.au_sol
        || joueur.vitesse != Vec2::default()
        || entrees.direction != Direction::Immobile
        || entrees.saut;
    if bouge {
        0
    } else {
        avant.saturating_add(1)
    }
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
        };
        for _ in 0..30 {
            m.avancer(&droite);
        }
        assert!(m.joueur.boite.x > x0 + 20.0);
        let y0 = m.joueur.boite.y;
        m.avancer(&Entrees {
            direction: Direction::Droite,
            saut: true,
        });
        for _ in 0..5 {
            m.avancer(&Entrees {
                direction: Direction::Droite,
                saut: true,
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

    #[test]
    fn l_interpolation_ignore_les_teleportations() {
        assert_eq!(interpoler((0.0, 0.0), (10.0, 0.0), 0.5), (5.0, 0.0));
        assert_eq!(interpoler((0.0, 0.0), (500.0, 0.0), 0.5), (500.0, 0.0));
    }

    // --- Minuterie de l'attente (carte 4.3) ---

    const DROITE: Entrees = Entrees {
        direction: Direction::Droite,
        saut: false,
    };
    const SAUT: Entrees = Entrees {
        direction: Direction::Immobile,
        saut: true,
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
