//! Module `entities` : tout ce qui vit dans un niveau en dehors du joueur et des tuiles.
//!
//! Le module est découpé en petits fichiers (un par famille d'entités) :
//! - `ennemi` : les flaques patrouilleuses (écrasables) et les corbeaux volants ;
//! - `pointe`, `eclair`, `grele` : les dangers fixes, temporisés et continus ;
//! - `friable`, `mobile` : les plateformes qui s'effritent ou qui voyagent ;
//! - `champignon` : le trampoline ; `auvent` : la toile qui relance en diagonale ;
//! - `collectibles` : les gouttes d'or et les lanternes (checkpoints) ;
//! - `contacts` : ce qui se passe quand le joueur touche une entité.
//!
//! Ordre conseillé pour UN pas de simulation (60 Hz), à respecter par le jeu :
//! 1. `Entite::actualiser` sur chaque entité (les plateformes bougent, les éclairs avancent) ;
//! 2. `contacts::porter_par_plateformes` (le joueur suit sa plateforme mobile) ;
//! 3. `Joueur::actualiser` (le joueur bouge) ;
//! 4. `contacts::resoudre_contacts` (ramassage, écrasement, dégâts...) ;
//! 5. `EtatJeu::traiter` (score, checkpoint, mort et réapparition).
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `trait` : le contrat `Actualisable` (« je sais avancer d'un pas de temps ») ;
//! - `enum` à données : `Entite` est UNE des dix familles, chacune portant ses
//!   propres champs ; `match` oblige à traiter toutes les variantes ;
//! - ownership : l'enum POSSÈDE l'entité qu'il contient (pas de pointeur partagé) ;
//! - emprunts : `&self` pour lire, `&mut self` pour modifier, `&[T]` / `&mut [T]`
//!   pour parcourir une tranche sans en prendre possession.

pub mod auvent;
pub mod champignon;
pub mod collectibles;
pub mod contacts;
pub mod eclair;
pub mod ennemi;
pub mod friable;
pub mod grele;
pub mod mobile;
pub mod pointe;
pub mod ventilateur;

use crate::aabb::Aabb;
use crate::physics::{CourantAscendant, Environnement};

pub use auvent::Auvent;
pub use champignon::Champignon;
pub use collectibles::{GoutteDor, Lanterne};
pub use eclair::Eclair;
pub use ennemi::{Ennemi, TypeEnnemi};
pub use friable::PlateformeFriable;
pub use grele::Grele;
pub use mobile::PlateformeMobile;
pub use pointe::Pointe;
pub use ventilateur::Ventilateur;

/// Contrat : « savoir avancer d'un pas de simulation fixe (1/60 s) ».
///
/// Un `trait` décrit un comportement que plusieurs types peuvent offrir. Le jeu
/// peut ainsi boucler sur toutes les entités sans savoir de quel type elles sont.
///
/// - `&mut self` : on a besoin d'un emprunt EXCLUSIF en écriture (l'entité change) ;
/// - `env: &Environnement` : un simple PRÊT en lecture du niveau (grille, solides).
pub trait Actualisable {
    /// Avance l'entité d'un pas de temps (`physics::DT`).
    fn actualiser(&mut self, env: &Environnement<'_>);
}

/// Une entité du niveau. Chaque variante contient (possède) une structure différente.
///
/// Avantage de l'enum face à une collection d'objets de types variés : tout est
/// rangé dans un seul `Vec<Entite>`, contigu en mémoire, sans allocation par entité.
#[derive(Debug, Clone)]
pub enum Entite {
    Ennemi(Ennemi),
    Pointe(Pointe),
    Eclair(Eclair),
    Grele(Grele),
    Friable(PlateformeFriable),
    Mobile(PlateformeMobile),
    Champignon(Champignon),
    Auvent(Auvent),
    Ventilateur(Ventilateur),
    Goutte(GoutteDor),
    Lanterne(Lanterne),
}

/// `Entite` implémente le trait en DÉLÉGUANT à la variante réelle.
impl Actualisable for Entite {
    fn actualiser(&mut self, env: &Environnement<'_>) {
        // `match self` avec `self: &mut Entite` : chaque `e` est un `&mut` vers
        // l'entité interne (le compilateur gère les emprunts pour nous).
        match self {
            Entite::Ennemi(e) => e.actualiser(env),
            Entite::Pointe(e) => e.actualiser(env),
            Entite::Eclair(e) => e.actualiser(env),
            Entite::Grele(e) => e.actualiser(env),
            Entite::Friable(e) => e.actualiser(env),
            Entite::Mobile(e) => e.actualiser(env),
            Entite::Champignon(e) => e.actualiser(env),
            Entite::Auvent(e) => e.actualiser(env),
            Entite::Ventilateur(e) => e.actualiser(env),
            Entite::Goutte(e) => e.actualiser(env),
            Entite::Lanterne(e) => e.actualiser(env),
        }
    }
}

impl Entite {
    /// Boîte principale de l'entité (pour l'affichage et les tests rapides).
    pub fn boite(&self) -> Aabb {
        match self {
            Entite::Ennemi(e) => e.boite,
            Entite::Pointe(e) => e.boite,
            Entite::Eclair(e) => e.zone,
            Entite::Grele(e) => e.zone,
            Entite::Friable(e) => e.boite,
            Entite::Mobile(e) => e.boite,
            Entite::Champignon(e) => e.boite,
            Entite::Auvent(e) => e.boite,
            Entite::Ventilateur(e) => e.boite,
            Entite::Goutte(e) => e.boite,
            Entite::Lanterne(e) => e.boite,
        }
    }

    /// Boîte SOLIDE de l'entité si elle en a une en ce moment (sinon `None`).
    /// `Option<Aabb>` : « une boîte, ou rien » — pas de valeur magique.
    pub fn solide(&self) -> Option<Aabb> {
        match self {
            Entite::Friable(p) => p.solide(),
            Entite::Mobile(p) => Some(p.boite),
            Entite::Ventilateur(v) => Some(v.boite),
            _ => None,
        }
    }
}

/// Rassemble les boîtes solides de toutes les entités, à passer à
/// `Environnement::solides`. On EMPRUNTE la tranche (`&[Entite]`) : l'appelant
/// garde ses entités, et la fonction renvoie un nouveau `Vec` qu'il possédera.
pub fn solides_des_entites(entites: &[Entite]) -> Vec<Aabb> {
    entites.iter().filter_map(Entite::solide).collect()
}

/// Rassemble les courants ascendants des ventilateurs qui SOUFFLENT en ce
/// moment, à ajouter aux courants du niveau (`niveau.courants`). Le courant
/// d'un ventilateur n'existe que pendant sa phase de souffle : on ne le met
/// dans `niveau.courants` (statique) que s'il est actif.
pub fn courants_des_ventilateurs(entites: &[Entite]) -> Vec<CourantAscendant> {
    entites
        .iter()
        .filter_map(|e| match e {
            Entite::Ventilateur(v) => v.courant_actif(),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aabb::GrilleTuiles;

    #[test]
    fn solides_ne_garde_que_les_entites_solides() {
        let mut entites = vec![
            Entite::Goutte(GoutteDor::new(0, 0.0, 0.0)),
            Entite::Mobile(PlateformeMobile::new(
                Aabb::new(10.0, 10.0, 32.0, 8.0),
                (10.0, 10.0),
                (50.0, 10.0),
                20.0,
            )),
            Entite::Friable(PlateformeFriable::new(Aabb::new(60.0, 10.0, 16.0, 8.0))),
        ];
        assert_eq!(solides_des_entites(&entites).len(), 2);

        // Une plateforme friable absente n'est plus solide.
        if let Entite::Friable(f) = &mut entites[2] {
            f.toucher();
        }
        let grille = GrilleTuiles::vide(10, 10);
        let env = Environnement::simple(&grille);
        for _ in 0..friable::EFFRITEMENT_PAS {
            entites[2].actualiser(&env);
        }
        assert_eq!(solides_des_entites(&entites).len(), 1);
    }

    #[test]
    fn actualiser_delegue_a_la_variante() {
        let grille = GrilleTuiles::vide(10, 10);
        let env = Environnement::simple(&grille);
        let mut e = Entite::Eclair(Eclair::new(Aabb::new(0.0, 0.0, 16.0, 64.0), 100, 1));
        e.actualiser(&env);
        match &e {
            Entite::Eclair(x) => assert!(x.en_annonce(), "1 pas : la phase d'annonce commence"),
            _ => unreachable!(),
        }
        assert_eq!(e.boite().h, 64.0);
    }
}
