//! Niveau d'essai « horloge » : deux aiguilles d'horloge à gravir (carte 5.10 du
//! plan de capture), avant-goût de la cage de l'horloge du futur clocher. Hors de
//! la table `NIVEAUX`, il n'existe QUE dans les tests et dans le build de QA
//! (fonctionnalité `debug_niveaux`, touche F8) ; en production il n'est pas
//! compilé dans le binaire, comme les autres niveaux d'essai.
//!
//! Chaque aiguille est une plateforme qui tourne autour de son pivot `h` dans le
//! sens des aiguilles d'une montre, le dessus toujours horizontal. On saute dessus
//! quand elle passe au plus bas, on se laisse monter, puis on saute sur l'étage
//! suivant, trop haut pour un saut depuis le bas.

use crate::level::ReglageAiguille;
use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai horloge",
    theme: Theme::Orage,
    genre: GenreNiveau::Normal,
    intro: "Les aiguilles de l'horloge tournent : monte dessus, elles t'emmènent en haut.",
    mecanique:
        "Sauter sur une aiguille au plus bas, se laisser porter, sauter sur l'étage suivant.",
    duree_cible_s: 30,
    ascii: ASCII,
    aiguilles: AIGUILLES,
};

/// Réglage des deux aiguilles, dans l'ordre de lecture de la grille (de haut en
/// bas) : la 1re rencontrée est la petite, en haut.
///
/// - 1re `h` (ligne 5, colonne 28) : la petite aiguille de la tour, 3 tuiles de
///   rayon, 36 px/s (un tour en 8,4 s), partie de 9 h ;
/// - 2e `h` (ligne 10, colonne 10) : la grande aiguille du bas, 4 tuiles de
///   rayon, 40 px/s (un tour en 10 s), partie de 4 h (elle descend vers Pépin).
const AIGUILLES: &[ReglageAiguille] = &[
    ReglageAiguille {
        rayon: 3.0,
        vitesse: 36.0,
        phase: 9.0,
    },
    ReglageAiguille {
        rayon: 4.0,
        vitesse: 40.0,
        phase: 4.0,
    },
];

/// 42 x 18 tuiles, thème Orage (le violet du futur clocher).
/// - Sol du départ : lignes 16-17, colonnes 0 à 15 (dessus à y = 256).
/// - Grande aiguille : pivot en (10, 10), soit (168, 168) ; à 6 h son dessus est
///   à y = 228 (un saut depuis le sol), à midi à y = 100.
/// - Tour B : colonnes 16 à 22, de la ligne 6 au bas (dessus à y = 96, 160 px
///   au-dessus du sol : hors de portée d'un saut). On y saute depuis la grande
///   aiguille, vers 1 h ou 2 h.
/// - Petite aiguille : pivot en (28, 5), soit (456, 88) ; à 9 h son dessus est à
///   y = 84, un saut depuis le bord de la tour B ; à 1 h, à y = 42.
/// - Tour C (drapeau) : colonnes 33 à 41, de la ligne 2 au bas (dessus à y = 32,
///   64 px au-dessus de la tour B : plus qu'un saut). On y saute depuis la petite
///   aiguille, vers midi ou 1 h.
/// - Entre les tours B et C, un gouffre (colonnes 23 à 32) : y tomber tue, on
///   reprend à la lanterne de la tour B. Sous la grande aiguille, le sol : une
///   chute y ramène au départ de l'aiguille, sans mourir.
/// - Gouttes : sol (colonne 5), tour B (colonne 20), tour C (colonne 33, au bord
///   où l'on atterrit en sautant de la petite aiguille), sur le chemin du robot de
///   QA. Lanternes : départ (2), tour B (17), tour C (34).
const ASCII: &str = r"
..........................................
.................................oL.....X.
.................................#########
.................................#########
.................................#########
.................L..o.......h....#########
................#######..........#########
................#######..........#########
................#######..........#########
................#######..........#########
..........h.....#######..........#########
................#######..........#########
................#######..........#########
................#######..........#########
................#######..........#########
.@L..o..........#######..........#########
#######################..........#########
#######################..........#########
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entite;

    /// Les deux aiguilles sont là où le commentaire le dit : pivots, rayons, heures
    /// de départ.
    #[test]
    fn deux_aiguilles_a_leur_place() {
        let n = INFOS.charger().unwrap();
        let aiguilles: Vec<((f32, f32), f32, f32)> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Mobile(m) => m.cercle().map(|c| (c.pivot, c.rayon, c.tour())),
                _ => None,
            })
            .collect();
        assert_eq!(
            aiguilles,
            vec![
                ((456.0, 88.0), 48.0, 0.75),
                ((168.0, 168.0), 64.0, 4.0 / 12.0)
            ]
        );
        assert_eq!((n.largeur(), n.hauteur()), (42, 18));
    }
}
