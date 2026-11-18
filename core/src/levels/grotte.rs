//! Monde 2, niveau 4 : « La grotte aux bulles » (carte 9.9, refaite à la carte
//! 9.20d). DERRIÈRE LA CASCADE, une grotte : on entre en traversant le rideau
//! d'eau de la chute (au pied, à gauche), puis on y MONTE, porté par des colonnes
//! de bulles qui jaillissent de SOURCES bouillonnantes (une vasque d'eau au pied
//! de la 1re et de la 3e, une fissure dans la roche pour la 2e et la 4e), sous
//! des stalactites qui tremblent puis tombent. On ressort en haut, au jour, au
//! bord de la rivière.
//! La pluie est restée dehors (thème sans pluie). Demande d'AGA : « ascensions en
//! bulles, stalactites qui tombent » ; puis (9.20) : le niveau ressemble à son nom.
//!
//! Lecture du niveau (36 x 48 tuiles, plus de quatre écrans de haut ; la caméra
//! suit Pépin en hauteur), de bas en haut, en zigzag :
//! 1. le SOL DU BAS (rangée 46) : l'ENTRÉE, le mur de gauche ouvert (rangées 38 à
//!    45) sur le rideau d'eau de la cascade, que Pépin vient de traverser ; un
//!    bassin (colonnes 8 à 12) et son nénuphar, la 1re goutte au-dessus ; un crabe
//!    patrouille le sol, une stalactite (colonne 15) pend sous la corniche P1. Au
//!    bout, la 1re colonne de bulles (colonnes 30-31) jaillit de sa vasque (rangée
//!    46) et soulève Pépin jusqu'à P1 (rangée 35), à sa gauche ;
//! 2. P1, vers la GAUCHE : la lanterne 1 (colonne 27), puis le COULOIR des
//!    stalactites (colonnes 11 et 16), bas de plafond : on y court sans
//!    s'arrêter. Au bout, la 2e colonne (colonnes 4-5) jaillit d'une fissure du
//!    sol de P1 : on la traverse pour se placer contre le mur, puis on y entre
//!    vers la DROITE ;
//! 3. le DESSUS DU ROCHER (rangée 23), vers la DROITE : en sortant des bulles, on
//!    passe sous le surplomb et ses deux stalactites (colonnes 8 et 11), la 2e
//!    goutte entre elles ; la lanterne 2 (colonne 13), un bassin (colonnes 15 à
//!    18) et une grenouille sur sa rive (colonne 20) ;
//! 4. le MOMENT FORT : la 3e colonne (colonnes 22-23), née de sa vasque au bord
//!    de la corniche de repli (rangée 26), monte jusqu'à la rangée 9 ; la 4e
//!    (colonnes 28-29) sort d'une FISSURE au sommet d'un pilier de roche (rangées
//!    21 à 26) et porte jusqu'au drapeau. Entre les deux, rien où se poser : on
//!    sort du haut de la 3e colonne EN PLANANT pour passer dans la 4e, EN L'AIR,
//!    sous une stalactite (colonne 25). Raté : on retombe sur la corniche de
//!    repli (lanterne 3), d'où l'on reprend la 3e ;
//! 5. le DRAPEAU, en haut à droite (colonne 33), sur un rocher accroché au mur :
//!    en sortant de la 4e colonne, on passe sous une dernière stalactite
//!    (colonne 30). La SORTIE : le mur de droite ouvert au-dessus du rocher
//!    (rangées 1 à 3), le jour qui se lève et les herbes de la rive ;
//! 6. la GOUTTE RISQUÉE, avant le drapeau : depuis le rocher du drapeau, un saut
//!    planant vers la GAUCHE à travers le haut de la 4e colonne (elle soulève),
//!    sous la stalactite du passage, puis un long plané au-dessus du vide jusqu'à
//!    la petite corniche P5 (rangée 7) et sa goutte ; retour par un saut planant
//!    dans la 4e colonne. Raté : la chute finit au pilier ou sur la corniche.
//!
//! Sept stalactites, deux ennemis (le crabe, la grenouille), trois lanternes,
//! deux oiseaux de décor (les hirondelles de la grotte). Aucune lanterne dans la
//! colonne d'une stalactite. Rendu des sources, du rideau d'eau et de la sortie :
//! `game/src/decor_grotte.rs`.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // Sa place dans le Monde 2 de SA branche (après le caniveau) : le coordinateur
    // la range à la 4e place à la fusion (recette, point 3).
    numero: 4,
    cle: "grotte",
    sans_planer: false,
    nom: "La grotte aux bulles",
    theme: Theme::Grotte,
    genre: GenreNiveau::Normal,
    intro: "Derrière la cascade, une grotte. Des sources bouillonnent : les bulles te portent.",
    mecanique: "Ascensions en bulles (passer d'une colonne à l'autre en planant) ; stalactites qui tremblent puis tombent : ne pas s'arrêter dessous.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (36 x 48 tuiles). Légende : voir `crate::level`.
/// - Murs (colonnes 0 et 35) et plafond (rangée 0) de briques ; sol du bas en
///   rangée 46, départ en colonne 2. Deux ouvertures dans les murs (le bord de la
///   carte reste un mur pour la collision) : l'entrée (colonne 0, rangées 38 à
///   45, le rideau de la cascade) et la sortie (colonne 35, rangées 1 à 3, le jour).
/// - Bulles `Y` : C1 (30-31, rangées 34 à 45), C2 (4-5, 22 à 34), C3 (22-23, 9
///   à 25), C4 (28-29, 3 à 20). Le haut de C1, C2 et C4 est une rangée au-dessus
///   de la corniche où l'on en sort.
/// - Sources (carte 9.20d) : une vasque d'eau `~` de deux cases au pied de C1
///   (rangée 46) et de C3 (rangée 26, au bout de la corniche de repli) : les
///   bulles en partent (la colonne descend jusqu'au fond de l'eau,
///   `level::ascii`). C2 sort d'une fissure du sol de P1 (une vasque y changeait
///   le passage du rocher : Habitué 0,04 au lieu de 0,02, mesure de la carte),
///   C4 d'une fissure au sommet d'un pilier de roche (28-29, rangées 21 à 26).
/// - Stalactites `V` : (15, 36), (11, 31), (16, 31), (8, 19), (11, 19), (25, 1),
///   (30, 1), chacune sous un solide.
/// - Corniches : P1 (rangée 35, 1 à 29), rocher (dessus en rangée 23, 6 à 21),
///   repli (rangée 26, 24 à 27), P5 (rangée 7, 16 à 19), drapeau (rangée 4, 30 à 34).
/// - Décor : deux oiseaux `K` (colonne 4 au sol du bas, colonne 20 sur P1).
const ASCII: &str = r"
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
B........................V....V.....
B...................................
B...........................YY...X..
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
B.....................YY....BB.....B
B...YY....o..L......j.YY....BB.....B
B...YY#########~~~~###YY....BB.....B
B...YYBBBBBBBBBBBBBBBBYY....BB.....B
B...YYBBBBBBBBBBBBBBBBYY...LBB.....B
B...YYBBBBBBBBBBBBBBBB~~####BB.....B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YYBBBBBBBBBBBBBBBBBB...........B
B...YY.....V....V..................B
B...YY.............................B
B...YY.............................B
B...YY..............K......L..YY...B
B#############################YY...B
B..............V..............YY...B
B.............................YY...B
..............................YY...B
..............................YY...B
..............................YY...B
..............................YY...B
..........o...................YY...B
..............................YY...B
..............................YY...B
..@.K.....n...........k.......YY...B
B#######~~~~~#################~~###B
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
