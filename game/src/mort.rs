//! Le temps de mort de Pépin (carte 9.24) : avant de réapparaître à la lanterne,
//! une courte animation SELON LA CAUSE, puis un rideau.
//!
//! Le cœur (`EtatJeu::traiter`) replace Pépin à la lanterne au pas même de la mort :
//! vies, protection de 1 s et « PLUS DE VIES » ne changent pas. C'est le `Monde`
//! qui, ensuite, se fige pendant `duree_totale` pas (ennemis, boss, plateformes et
//! commandes compris) : seul l'effet de mort bouge. Le dessin est dans
//! `dessin_mort`.
//!
//! Déroulé (à 60 pas par seconde) :
//! - l'animation de la cause : noyade 1,2 s, dégât 1,0 s, chute dans le vide 0,6 s ;
//! - le rideau, 0,3 s : il se ferme sur l'endroit de la mort, le monde se remet en
//!   place derrière lui (`Monde::avancer`), la caméra saute à la lanterne, il
//!   s'ouvre.
//!
//! Notions Rust illustrées ici : `enum` sans données et `match` exhaustif (une
//! nouvelle cause du cœur oblige à choisir son animation), `impl Default`.

use pepin_core::entities::contacts::CauseMort;

/// Durée (pas) de l'animation de noyade : 1,2 s.
pub const NOYADE_PAS: u16 = 72;
/// Durée (pas) de l'animation de dégât : 1,0 s.
pub const DEGAT_PAS: u16 = 60;
/// Durée (pas) de la chute dans le vide : 0,6 s.
pub const VIDE_PAS: u16 = 36;
/// Durée (pas) du rideau : 0,15 s pour se fermer, autant pour s'ouvrir.
pub const RIDEAU_PAS: u16 = 18;
/// Reste du rideau (pas) quand il est tout fermé : le monde se remet en place.
pub const RIDEAU_FERME: u16 = RIDEAU_PAS / 2;

/// L'animation à jouer : trois familles de causes suffisent au dessin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CauseDeMort {
    /// Entré dans l'eau parapluie fermé.
    Noyade,
    /// Ennemi, éclair, grêle, tentacule, pointes, mur de pluie.
    Degat,
    /// Tombé sous le bas du niveau.
    #[default]
    Vide,
}

impl CauseDeMort {
    /// La famille d'une cause de mort du cœur.
    pub fn depuis(cause: CauseMort) -> Self {
        match cause {
            CauseMort::Noyade => CauseDeMort::Noyade,
            CauseMort::Vide => CauseDeMort::Vide,
            CauseMort::Ennemi
            | CauseMort::Pointe
            | CauseMort::Eclair
            | CauseMort::Grele
            | CauseMort::Pluie
            | CauseMort::Pieuvre => CauseDeMort::Degat,
        }
    }

    /// Durée (pas) de l'animation de cette cause, sans le rideau.
    pub fn duree(self) -> u16 {
        match self {
            CauseDeMort::Noyade => NOYADE_PAS,
            CauseDeMort::Degat => DEGAT_PAS,
            CauseDeMort::Vide => VIDE_PAS,
        }
    }

    /// Durée (pas) de tout le temps de mort : animation puis rideau.
    pub fn duree_totale(self) -> u16 {
        self.duree() + RIDEAU_PAS
    }
}

/// Ce que le dessin doit savoir d'une mort, noté au pas de la mort.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AnimationMort {
    pub cause: CauseDeMort,
    /// Boîte de Pépin juste avant la mort (coin haut-gauche, puis largeur et hauteur).
    pub boite: (f32, f32, f32, f32),
    /// Noyade : hauteur (px) de la surface de l'eau où Pépin s'enfonce.
    pub surface: f32,
    /// Pépin regardait-il à droite ? (Un dégât l'éjecte en arrière.)
    pub regarde_droite: bool,
}

impl AnimationMort {
    /// Pas écoulés depuis la mort, `reste` étant le compteur du `Monde`
    /// (`Monde::reapparition`, pas restants du temps de mort).
    pub fn ecoule(&self, reste: u16) -> u16 {
        self.cause.duree_totale().saturating_sub(reste)
    }
}

/// Hauteur (px) du rideau, en haut comme en bas de l'écran, pour `reste` pas
/// restants du temps de mort : 0 pendant l'animation, la moitié de l'écran quand
/// il est fermé (à `RIDEAU_FERME`), puis il se rouvre.
pub fn hauteur_rideau(reste: u16, demi_ecran: i32) -> i32 {
    if reste == 0 || reste > RIDEAU_PAS {
        return 0;
    }
    // Distance (en pas) au moment où le rideau est fermé.
    let ecart = i32::from(reste.abs_diff(RIDEAU_FERME));
    demi_ecran * (i32::from(RIDEAU_FERME) - ecart) / i32::from(RIDEAU_FERME)
}
