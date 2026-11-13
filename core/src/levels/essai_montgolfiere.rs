//! Niveau d'essai « montgolfière » : une petite tour de nuages que l'on gravit
//! en deux montgolfières (carte 5.7 du plan de capture). Hors de la table
//! `NIVEAUX`, il n'existe QUE dans les tests et dans le build de QA
//! (fonctionnalité `debug_niveaux`, touche F5) ; en production il n'est pas
//! compilé dans le binaire, comme les autres niveaux d'essai.
//!
//! Les deux façons de s'arrêter en haut :
//! - la première montgolfière monte de sa hauteur maximale (8 tuiles) : rien
//!   au-dessus d'elle ;
//! - la seconde s'arrête plus tôt, sous un nuage (le haut du ballon le touche).
//! Chaque fois, l'étage suivant est trop haut pour un saut : seule la nacelle y
//! porte Pépin. Quand il la quitte, elle redescend doucement à son quai.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 0,
    nom: "Essai montgolfière",
    theme: Theme::Altitude,
    genre: GenreNiveau::Normal,
    intro: "Une montgolfière monte quand Pépin se tient dans sa nacelle, et redescend doucement quand il la quitte.",
    mecanique: "Gravir deux étages de nuages en montgolfière : la première monte de 8 tuiles, la seconde s'arrête sous un nuage.",
    duree_cible_s: 30,
    ascii: ASCII,
};

/// 28 x 18 tuiles, thème Altitude.
/// - Quai de départ : colonnes 0 à 5, lignes 16-17 (dessus à y = 256).
/// - Montgolfière 1 en (6, 16), au ras du quai : rien au-dessus de sa colonne,
///   elle monte de 8 tuiles (nacelle de y = 256 à y = 128).
/// - Étage B : colonnes 8 à 15, lignes 9-10 (dessus à y = 144, 16 px sous la
///   nacelle arrivée en haut ; la colonne 7 reste vide : la tête de Pépin ne
///   cogne rien pendant la montée). Hors de portée d'un saut depuis le quai
///   (112 px).
/// - Montgolfière 2 en (16, 9), au ras de l'étage B : un nuage en ligne 1 au-dessus
///   d'elle arrête le haut du ballon (nacelle de y = 144 à y = 64, 5 tuiles).
/// - Étage D (drapeau) : colonnes 18 à 25, lignes 5-6 (dessus à y = 80, 64 px
///   au-dessus de l'étage B : hors de portée d'un saut).
/// - Gouttes : quai (colonne 4), étage B (colonne 11), étage D (colonne 19), sur
///   le chemin des deux parcours du robot de QA (le plus rapide, qui saute de la
///   nacelle dès que l'étage est à portée, et celui qui monte jusqu'en haut) : il
///   les ramasse en marchant d'une montgolfière à l'autre, puis vers le drapeau.
///   Lanternes : quai (2), étage B (9), étage D (18), une par étage.
const ASCII: &str = r"
............................
...............NNNN.........
............................
............................
..................Lo....X...
..................NNNNNNNN..
..................NNNNNNNN..
............................
.........L.o................
........NNNNNNNNb...........
........NNNNNNNN............
............................
............................
............................
............................
.@L.o.......................
NNNNNNb.....................
NNNNNN......................
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entite;

    /// Les deux montgolfières sont là où le commentaire le dit : la première
    /// monte de 8 tuiles, la seconde s'arrête sous le nuage de la ligne 1.
    #[test]
    fn deux_montgolfieres_et_leurs_sommets() {
        let n = INFOS.charger().unwrap();
        let alt: Vec<(f32, f32, f32)> = n
            .entites
            .iter()
            .filter_map(|e| match e {
                Entite::Montgolfiere(m) => Some((m.boite.x, m.altitudes().0, m.altitudes().1)),
                _ => None,
            })
            .collect();
        assert_eq!(alt, vec![(256.0, 144.0, 64.0), (96.0, 256.0, 128.0)]);
    }
}
