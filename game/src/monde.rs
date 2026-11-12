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
use pepin_core::entities::{solides_des_entites, Actualisable, Entite};
use pepin_core::level::Niveau;
use pepin_core::physics::Environnement;
use pepin_core::player::{Entrees, Evenements, Joueur};
use pepin_core::state::{EtatJeu, Ordre};
use pepin_core::zones::{self, InfosNiveau};

/// Durée (pas) de l'animation de réapparition de Pépin.
pub const REAPPARITION_PAS: u16 = 18;

pub struct Monde {
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
    precedent_joueur: (f32, f32),
    precedent_entites: Vec<(f32, f32)>,
}

impl Monde {
    /// Charge le niveau `indice` (0 à 4). Les niveaux sont validés par les tests du
    /// cœur : un échec ici est une erreur de programmation, d'où le `expect`.
    pub fn new(indice: usize) -> Self {
        let infos = zones::niveau(indice).expect("indice de niveau valide");
        let niveau = infos.charger().expect("niveau embarqué valide");
        let joueur = Joueur::new(niveau.depart.0, niveau.depart.1);
        let mut etat = EtatJeu::new(niveau.depart);
        etat.appliquer(Ordre::Demarrer)
            .expect("une partie neuve peut démarrer");
        let entites = niveau.entites.clone();
        let precedent_entites = entites.iter().map(Self::coin).collect();
        Self {
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
            };
            for e in &mut self.entites {
                e.actualiser(&env);
            }
        }
        // 2. Les plateformes ont bougé : on recalcule les solides, le joueur est porté.
        let solides = solides_des_entites(&self.entites);
        let env = Environnement {
            grille: &self.niveau,
            solides: &solides,
            courants: &self.niveau.courants,
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
                    self.pos_mort = (avant_mort.0, avant_mort.1.min(self.niveau.hauteur_px() - 16.0));
                    // Téléportation : pas d'interpolation entre l'ancien et le nouvel endroit.
                    self.precedent_joueur = (self.joueur.boite.x, self.joueur.boite.y);
                }
                _ => {}
            }
        }
        self.contacts = contacts;
        self.reapparition = self.reapparition.saturating_sub(1);

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
    use pepin_core::player::Direction;

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
        m.joueur.placer(m.niveau.largeur_px() / 2.0, m.niveau.hauteur_px() + 40.0);
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
}
