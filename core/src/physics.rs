//! Module `physics` : constantes de la physique et petits utilitaires d'intégration.
//!
//! Unités : pixels logiques et secondes. L'axe Y pointe vers le BAS.
//! Tout est calculé à pas FIXE (`DT` = 1/60 s) : le résultat ne dépend donc
//! jamais de la vitesse d'affichage, ce qui rend les tests reproductibles.
//!
//! Notions Rust illustrées ici (à reprendre dans le README) :
//! - `const` : valeurs connues à la compilation, nommées et documentées ;
//! - durées de vie (`'a`) : une structure qui EMPRUNTE des données doit dire
//!   combien de temps ces données vivent (`Environnement<'a>`) ;
//! - `&dyn Trait` et tranches `&[T]` : emprunter sans posséder.

use crate::aabb::{Aabb, Grille};

// ---------------------------------------------------------------------------
// Pas de temps
// ---------------------------------------------------------------------------

/// Fréquence de la simulation (pas par seconde).
pub const HZ: u32 = 60;
/// Durée d'un pas de simulation en secondes.
pub const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------------------
// Gravité et chute
// ---------------------------------------------------------------------------

/// Accélération vers le bas (px/s²).
pub const GRAVITE: f32 = 900.0;
/// Vitesse de chute maximale, parapluie fermé (px/s).
pub const VITESSE_MAX_CHUTE: f32 = 360.0;

// ---------------------------------------------------------------------------
// Course
// ---------------------------------------------------------------------------

/// Vitesse horizontale maximale (px/s).
pub const VITESSE_COURSE: f32 = 96.0;
/// Accélération au sol quand on appuie sur une direction (px/s²).
pub const ACCEL_SOL: f32 = 800.0;
/// Décélération au sol quand on lâche la direction (px/s²).
pub const FRICTION_SOL: f32 = 900.0;
/// Accélération en l'air (contrôle réduit).
pub const ACCEL_AIR: f32 = 500.0;
/// Décélération en l'air sans direction (l'élan se conserve presque).
pub const FRICTION_AIR: f32 = 200.0;

// ---------------------------------------------------------------------------
// Saut
// ---------------------------------------------------------------------------

/// Vitesse initiale du saut (px/s, vers le haut).
pub const VITESSE_SAUT: f32 = 320.0;
/// Saut à hauteur variable : si on relâche la touche pendant la montée, la
/// vitesse montante est ramenée à cette valeur (px/s) -> petit saut.
pub const VITESSE_COUPE_SAUT: f32 = 130.0;
/// Coyote time : nombre de pas (1/60 s) pendant lesquels on peut encore sauter
/// après avoir quitté un bord. Des entiers plutôt que des flottants : pas
/// d'erreur d'arrondi, comportement parfaitement déterministe.
pub const COYOTE_PAS: u8 = 6;
/// Jump buffer : nombre de pas pendant lesquels un appui trop précoce est mémorisé.
pub const TAMPON_SAUT_PAS: u8 = 6;

// ---------------------------------------------------------------------------
// Planer (parapluie ouvert)
// ---------------------------------------------------------------------------

/// Vitesse de chute plafonnée quand le parapluie est ouvert (px/s).
pub const VITESSE_PLANER_CHUTE: f32 = 36.0;
/// Freinage vertical quand on ouvre le parapluie en chute rapide (px/s²).
pub const FREIN_PLANER: f32 = 2400.0;
/// Vitesse horizontale maximale en planant (px/s) : léger contrôle seulement.
pub const VITESSE_PLANER_X: f32 = 72.0;
/// Accélération horizontale en planant.
pub const ACCEL_PLANER: f32 = 300.0;
/// Décélération horizontale en planant sans direction.
pub const FRICTION_PLANER: f32 = 150.0;

// ---------------------------------------------------------------------------
// Courants ascendants
// ---------------------------------------------------------------------------

/// Accélération vers le haut par défaut d'un courant ascendant (px/s²).
/// Supérieure à la gravité : le joueur monte.
pub const COURANT_ACCEL: f32 = 1500.0;
/// Vitesse de montée maximale obtenue grâce à un courant (px/s).
pub const VITESSE_MAX_COURANT: f32 = 150.0;

// ---------------------------------------------------------------------------
// Joueur
// ---------------------------------------------------------------------------

/// Largeur de la boîte de collision du joueur.
pub const JOUEUR_L: f32 = 10.0;
/// Hauteur de la boîte de collision du joueur.
pub const JOUEUR_H: f32 = 14.0;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Vecteur 2D (vitesses...). `Default` donne `(0.0, 0.0)`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Zone rectangulaire où souffle un vent ascendant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CourantAscendant {
    /// Rectangle où le vent agit.
    pub zone: Aabb,
    /// Accélération vers le haut (px/s², valeur positive).
    pub acceleration: f32,
}

impl CourantAscendant {
    /// Courant avec l'accélération standard `COURANT_ACCEL`.
    pub fn new(zone: Aabb) -> Self {
        Self {
            zone,
            acceleration: COURANT_ACCEL,
        }
    }
}

/// Vitesse (px/s) du vent latéral, parapluie FERMÉ : le vent pousse Pépin
/// d'autant par seconde.
pub const VENT_LAT: f32 = 60.0;
/// Vitesse (px/s) du vent latéral en PLANANT (parapluie ouvert) : deux fois
/// plus forte que `VENT_LAT`. On s'en sert pour allonger un plané, ou on ferme
/// le parapluie pour résister au vent.
pub const VENT_LAT_PLANE: f32 = 2.0 * VENT_LAT;

// ---------------------------------------------------------------------------
// Auvents (lettre `A`) : rebond en diagonale, vers le haut et vers la droite
// ---------------------------------------------------------------------------

/// Vitesse verticale (px/s, vers le haut) donnée par un auvent. Hauteur du
/// rebond : 300² / (2 × 900) = 50 px (un peu plus de 3 tuiles), un peu moins
/// qu'un saut : l'auvent sert à AVANCER de boutique en boutique, pas à monter.
pub const VITESSE_AUVENT: f32 = 300.0;
/// Durée (pas) d'un vol d'auvent qui retombe à sa hauteur de départ :
/// 2 × 300 / 900 s = 2/3 s = 40 pas.
pub const VOL_AUVENT_PAS: u16 = 40;
/// Élan horizontal (px/s, vers la droite) donné par un auvent. Il est rangé À
/// PART de `vitesse.x` (comme le vent latéral) et s'éteint en ligne droite
/// pendant `VOL_AUVENT_PAS` pas : sans toucher à rien, Pépin avance de
/// 192 × (40/60) / 2 = 64 px, soit 4 tuiles, avant de retomber à sa hauteur de
/// départ. En gardant la droite, il va presque deux fois plus loin ; en gardant
/// la gauche, il retombe presque sur place. Des auvents espacés de 3 à 5 tuiles
/// s'enchaînent donc sans toucher le sol (voir `entities::auvent`).
pub const ELAN_AUVENT: f32 = 192.0;
/// Baisse de l'élan d'auvent à chaque pas (px/s) : il atteint 0 en `VOL_AUVENT_PAS`.
pub const BAISSE_ELAN_AUVENT: f32 = ELAN_AUVENT / VOL_AUVENT_PAS as f32;

/// Zone rectangulaire où souffle un vent LATERAL (lettre `<` ou `>` de la
/// grille). Le vent pousse Pépin à `VENT_LAT` (parapluie fermé), et deux fois
/// plus fort s'il plane (`VENT_LAT_PLANE`) : voir `player::actualiser`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VentLateral {
    /// Rectangle où le vent agit.
    pub zone: Aabb,
    /// Sens du vent : 1 (vers la droite, `>`), -1 (vers la gauche, `<`).
    pub sens: i8,
}

impl VentLateral {
    /// Zone de vent de `sens` (1 ou -1).
    pub fn new(zone: Aabb, sens: i8) -> Self {
        Self { zone, sens }
    }
}

/// Tout ce que le joueur « voit » du monde pendant un pas : un simple PRÊT
/// de données. Les trois champs sont des emprunts en lecture seule ; la durée
/// de vie `'a` garantit que le niveau reste vivant tant que l'`Environnement`
/// existe (le compilateur refuserait de le garder plus longtemps).
#[derive(Clone, Copy)]
pub struct Environnement<'a> {
    /// Tuiles du niveau.
    pub grille: &'a dyn Grille,
    /// Solides supplémentaires (plateformes mobiles, friables...).
    pub solides: &'a [Aabb],
    /// Zones de courant ascendant.
    pub courants: &'a [CourantAscendant],
    /// Zones de vent latéral.
    pub vents: &'a [VentLateral],
}

impl<'a> Environnement<'a> {
    /// Environnement sans solides, courants ni vents, juste la grille.
    pub fn simple(grille: &'a dyn Grille) -> Self {
        Self {
            grille,
            solides: &[],
            courants: &[],
            vents: &[],
        }
    }
}

// ---------------------------------------------------------------------------
// Fonctions utilitaires
// ---------------------------------------------------------------------------

/// Rapproche `valeur` de `cible` d'au plus `delta`, sans jamais dépasser la cible.
/// Brique de base de l'accélération ET de la friction.
pub fn approcher(valeur: f32, cible: f32, delta: f32) -> f32 {
    if valeur < cible {
        (valeur + delta).min(cible)
    } else {
        (valeur - delta).max(cible)
    }
}

/// Hauteur théorique (px) d'un saut complet : v² / 2g. Les niveaux s'en servent
/// pour vérifier qu'un saut est faisable (la simulation discrète s'en approche
/// à quelques pixels près).
pub fn hauteur_saut_theorique() -> f32 {
    VITESSE_SAUT * VITESSE_SAUT / (2.0 * GRAVITE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approcher_ne_depasse_pas_la_cible() {
        assert_eq!(approcher(0.0, 10.0, 3.0), 3.0);
        assert_eq!(approcher(9.0, 10.0, 3.0), 10.0);
        assert_eq!(approcher(0.0, -10.0, 4.0), -4.0);
        assert_eq!(approcher(-9.0, 0.0, 20.0), 0.0);
        assert_eq!(approcher(5.0, 5.0, 1.0), 5.0);
    }

    #[test]
    fn constantes_coherentes() {
        assert!((DT * HZ as f32 - 1.0).abs() < 1e-6);
        // Un saut complet monte d'environ 3,5 tuiles de 16 px.
        let h = hauteur_saut_theorique();
        assert!((50.0..60.0).contains(&h), "hauteur = {h}");
    }

    // Le vent doit gagner contre la gravité, sinon on ne monterait pas ;
    // planer doit ralentir la chute. Vérifié à la compilation.
    const _: () = assert!(COURANT_ACCEL > GRAVITE);
    const _: () = assert!(VITESSE_PLANER_CHUTE < VITESSE_MAX_CHUTE);
    // Le vent latéral en planant pousse deux fois plus fort que fermé.
    const _: () = assert!(VENT_LAT_PLANE == 2.0 * VENT_LAT);
    const _: () = assert!(VENT_LAT == 60.0);

    /// Auvent : le vol dure bien `VOL_AUVENT_PAS` (montée + descente à la même
    /// hauteur), et l'élan seul fait avancer de 4 tuiles pendant ce vol.
    #[test]
    fn constantes_de_l_auvent_coherentes() {
        let vol = 2.0 * VITESSE_AUVENT / GRAVITE * HZ as f32;
        assert_eq!(vol.round() as u16, VOL_AUVENT_PAS);
        let avance = ELAN_AUVENT * (VOL_AUVENT_PAS as f32 * DT) / 2.0;
        assert!((avance - 64.0).abs() < 1e-3, "avance = {avance}");
        // Le rebond d'auvent monte moins haut qu'un saut (il sert à avancer).
        let h = VITESSE_AUVENT * VITESSE_AUVENT / (2.0 * GRAVITE);
        assert!(h < hauteur_saut_theorique(), "h = {h}");
    }
}
