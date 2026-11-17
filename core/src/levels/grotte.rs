//! Monde 2, niveau 4 : « La grotte aux bulles » (carte 9.9). Sous la ville, la
//! pluie a creusé une grotte SÈCHE (thème sans pluie) : on y MONTE, porté par des
//! colonnes de bulles, sous des stalactites qui tremblent puis tombent. Demande
//! d'AGA : « ascensions en bulles, stalactites qui tombent ».
//!
//! Lecture du niveau (36 x 48 tuiles, plus de quatre écrans de haut ; la caméra
//! suit Pépin en hauteur), de bas en haut, en zigzag :
//! 1. le SOL DU BAS (rangée 46) : un bassin (colonnes 8 à 12) et son nénuphar, la
//!    1re goutte au-dessus ; un crabe patrouille le sol, une stalactite (colonne
//!    15) pend sous la corniche P1. Au bout, la 1re colonne de bulles (colonnes
//!    30-31) soulève Pépin jusqu'à P1 (rangée 35), à sa gauche ;
//! 2. P1, vers la GAUCHE : la lanterne 1 (colonne 27), puis le COULOIR des
//!    stalactites (colonnes 11 et 16), bas de plafond : on y court sans
//!    s'arrêter. Au bout, la 2e colonne (colonnes 4-5) part de P1 : on la
//!    traverse pour se placer contre le mur, puis on y entre vers la DROITE ;
//! 3. le DESSUS DU ROCHER (rangée 23), vers la DROITE : en sortant des bulles, on
//!    passe sous le surplomb et ses deux stalactites (colonnes 8 et 11), la 2e
//!    goutte entre elles ; la lanterne 2 (colonne 13), un bassin (colonnes 15 à
//!    18) et une grenouille sur sa rive (colonne 20) ;
//! 4. le MOMENT FORT : la 3e colonne (colonnes 22-23) monte jusqu'à la rangée 9,
//!    la 4e (colonnes 28-29), SUSPENDUE, porte jusqu'au drapeau. Entre les deux,
//!    rien où se poser : on sort du haut de la 3e colonne EN PLANANT pour passer
//!    dans la 4e, EN L'AIR, sous une stalactite (colonne 25). Raté : on retombe
//!    sur la corniche de repli (rangée 26, lanterne 3), d'où l'on reprend la 3e ;
//! 5. le DRAPEAU, en haut à droite (colonne 33), sur un rocher accroché au mur :
//!    en sortant de la 4e colonne, on passe sous une dernière stalactite
//!    (colonne 30) ;
//! 6. la GOUTTE RISQUÉE, avant le drapeau : depuis le rocher du drapeau, un saut
//!    planant vers la GAUCHE à travers le haut de la 4e colonne (elle soulève),
//!    sous la stalactite du passage, puis un long plané au-dessus du vide jusqu'à
//!    la petite corniche P5 (rangée 7) et sa goutte ; retour par un saut planant
//!    dans la 4e colonne. Raté : la chute finit au rocher, voire dans son bassin.
//!
//! Sept stalactites, deux ennemis (le crabe, la grenouille), trois lanternes,
//! deux oiseaux de décor (les hirondelles de la grotte). Aucune lanterne dans la
//! colonne d'une stalactite.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Sa place dans le Monde 2 de SA branche (après le caniveau) : le coordinateur
    // la range à la 4e place à la fusion (recette, point 3).
    numero: 2,
    cle: "grotte",
    sans_planer: false,
    nom: "La grotte aux bulles",
    theme: Theme::Grotte,
    genre: GenreNiveau::Normal,
    intro: "Sous la ville, la pluie a creusé une grotte. Les bulles montent : Pépin aussi.",
    mecanique: "Ascensions en bulles (passer d'une colonne à l'autre en planant) ; stalactites qui tremblent puis tombent : ne pas s'arrêter dessous.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (36 x 48 tuiles). Légende : voir `crate::level`.
/// - Murs (colonnes 0 et 35) et plafond (rangée 0) de briques ; sol du bas en
///   rangée 46, départ en colonne 2.
/// - Bulles `Y` : C1 (30-31, rangées 34 à 45), C2 (4-5, 22 à 34), C3 (22-23, 9
///   à 25), C4 (28-29, 3 à 20, suspendue). Le haut de C1, C2 et C4 est une rangée
///   au-dessus de la corniche où l'on en sort.
/// - Stalactites `V` : (15, 36), (11, 31), (16, 31), (8, 19), (11, 19), (25, 1),
///   (30, 1), chacune sous un solide.
/// - Corniches : P1 (rangée 35, 1 à 29), rocher (dessus en rangée 23, 6 à 21),
///   repli (rangée 26, 24 à 27), P5 (rangée 7, 16 à 19), drapeau (rangée 4, 30 à 34).
/// - Décor : deux oiseaux `K` (colonne 4 au sol du bas, colonne 20 sur P1).
const ASCII: &str = r"
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
B........................V....V....B
B..................................B
B...........................YY...X.B
B...........................YY#####B
B...........................YYBBBBBB
B...............o...........YYBBBBBB
B...............####........YY.....B
B...........................YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B.....................YY....YY.....B
B......BBBBBB.........YY....YY.....B
B......BBBBBB.........YY....YY.....B
B.......V..V..........YY....YY.....B
B.....................YY....YY.....B
B.....................YY...........B
B...YY....o..L......j.YY...........B
B...YY#########~~~~###YY...........B
B...YYBBBBBBBBBBBBBBBBYY...........B
B...YYBBBBBBBBBBBBBBBBYY...L.......B
B...YYBBBBBBBBBBBBBBBB..####.......B
B...YYBBBBBBBBBBBBBBBB.............B
B...YYBBBBBBBBBBBBBBBB.............B
B...YYBBBBBBBBBBBBBBBB.............B
B...YYBBBBBBBBBBBBBBBB.............B
B...YY.....V....V..................B
B...YY.............................B
B...YY.............................B
B...YY..............K......L..YY...B
B#############################YY...B
B..............V..............YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.............................YY...B
B.........o...................YY...B
B.............................YY...B
B.............................YY...B
B.@.K.....n...........k.......YY...B
B#######~~~~~######################B
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faisabilite::{analyser, Options};

    /// Les bulles sont LA mécanique du niveau (demande d'AGA). Sans elles
    /// (courants coupés), le robot de faisabilité ne quitte pas le bas de la
    /// grotte : ni drapeau, ni goutte risquée (la 1re dans l'ordre de lecture).
    /// Et le passage de la 3e colonne (22-23) à la 4e (28-29) se fait EN L'AIR :
    /// rien où se poser entre elles (colonnes 24 à 27) au-dessus de la lanterne
    /// de la corniche de repli (rangée 25), sauf la stalactite qui y pend.
    #[test]
    fn les_bulles_font_le_niveau() {
        let niv = INFOS.charger().unwrap();
        let sans_bulles = Options {
            courants: false,
            ..Options::default()
        };
        let r = analyser(&niv, niv.depart, sans_bulles);
        assert!(!r.arrivee, "le drapeau sans les bulles");
        assert!(!r.gouttes[0], "la goutte risquée sans les bulles");

        let g: Vec<Vec<char>> = ASCII.trim().lines().map(|l| l.chars().collect()).collect();
        for (rangee, ligne) in g.iter().enumerate().take(25).skip(1) {
            for (x, c) in ligne.iter().enumerate().take(28).skip(24) {
                assert!(
                    matches!(c, '.' | 'V'),
                    "colonne {x}, rangée {rangee} : un appui entre les deux colonnes"
                );
            }
        }
        assert!(g[9][22] == 'Y' && g[8][22] == '.', "haut de la 3e colonne");
        assert!(g[3][28] == 'Y' && g[20][28] == 'Y', "4e colonne");
    }
}
