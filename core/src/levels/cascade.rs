//! Monde 2, niveau 3 : « La grande cascade » (carte 9.8). Le niveau VERTICAL du
//! Monde 2, à l'envers comme « La descente » : on part du haut de la falaise et
//! l'on DESCEND entre les chutes d'eau, de bassin en bassin, jusqu'au pied de la
//! cascade. 32 tuiles de large, 46 de haut (quatre écrans) : la caméra suit Pépin
//! en hauteur (carte 5.2). Difficulté : un cran au-dessus des égouts, celle de
//! l'Orage ou du Clocher du Monde 1.
//!
//! Les deux façons de traverser une chute d'eau (lettre `|`, carte 9.3) :
//! - parapluie OUVERT, on y descend lentement (56 px/s), en sécurité, et l'on
//!   retombe en barque dans le bassin du dessous ;
//! - parapluie FERMÉ, on est emporté vite (380 px/s) : dans un bassin, c'est la
//!   noyade (la chute tue à l'arrivée) ; sur la roche, rien.
//!
//! Le parcours, de haut en bas :
//! 1. le haut de la falaise : on plane du départ dans le BASSIN A (le panneau
//!    « planer » le rappelle) ; son courant `}` porte la barque vers la droite. La
//!    1re goutte flotte au-dessus (colonne 13) : on la prend d'un saut depuis la
//!    barque, et l'on replane pour retomber à bord ;
//! 2. au bout du bassin A, l'eau tombe : la CHUTE C1 (colonnes 27 et 28, dix
//!    rangées). Il FAUT y planer : on y descend lentement jusqu'au BASSIN B, où
//!    l'on retombe en barque (parapluie fermé, on s'y noie) ;
//! 3. le bassin B, courant `{` vers la gauche : un poisson y saute (colonne 20) ;
//!    on attend qu'il replonge. Au bout, la corniche (1re lanterne) ;
//! 4. la GOUTTE DERRIÈRE LA CHUTE : au bout de la corniche, la chute C2 (quatre
//!    colonnes, 2 à 5) tombe devant une niche creusée dans la falaise (colonnes 0
//!    et 1), où brille la 2e goutte. On la franchit d'un saut EN PLANANT : la
//!    chute freine la descente, on se pose dans la niche, une rangée plus bas.
//!    Parapluie fermé, la chute emporte Pépin vers le bas ;
//! 5. on redescend par la chute C2 en planant jusqu'au BASSIN C, dont le courant
//!    `}` porte la barque sous la falaise jusqu'au couloir (2e lanterne) ;
//! 6. LE PUITS : au bout du couloir, une stalactite pend au-dessus d'un puits
//!    d'une tuile de large où coule une chute. Elle tremble dès que Pépin passe
//!    dessous et tombe 0,75 s plus tard : il faut SE LAISSER EMPORTER VITE,
//!    parapluie fermé (on traverse le puits en un tiers de seconde). En planant,
//!    on y est encore quand elle tombe. En bas du puits, la salle et le palier
//!    (3e lanterne), hors de la colonne de la stalactite ;
//! 7. le pied de la cascade : on plane du palier dans le BASSIN D (la 3e goutte
//!    au-dessus de l'eau, sur le chemin) ; son courant `{` mène à la berge, où
//!    est le drapeau (colonne 2).
//!
//! Trois lanternes, chacune hors de tout danger : la corniche (10, 16), le
//! couloir (15, 25), le palier (27, 37). Deux ennemis seulement : le poisson du
//! bassin B et une mouette sur le rocher de la salle du bas (9, 34), qui plonge
//! quand Pépin passe à sa portée (on l'écrase par le dessus, ou on l'évite). Les
//! oiseaux du décor (`K`) n'ont aucun effet sur le jeu.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    monde: 2,
    // 2 dans la branche de la carte 9.8 (seul le caniveau le précède) ; le
    // coordinateur le range 3e, après les égouts, à la fusion.
    numero: 3,
    cle: "cascade",
    sans_planer: false,
    nom: "La grande cascade",
    theme: Theme::Cascade,
    genre: GenreNiveau::Normal,
    intro: "L'eau ressort au bord de la falaise et tombe de bassin en bassin.",
    mecanique: "Les chutes d'eau : parapluie ouvert, on y descend lentement jusqu'au \
                bassin du dessous ; parapluie fermé, elles emportent vite (et l'on se noie \
                si un bassin attend en bas).",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (32 x 46 tuiles). Légende : voir `crate::level`.
/// - Bassins : A rangée 8 (colonnes 7 à 26), B rangée 18 (14 à 28), C rangée 27
///   (1 à 12), D rangée 41 (6 à 16).
/// - Chutes : C1 colonnes 27-28 (rangées 8 à 17), C2 colonnes 2 à 5 (11 à 26),
///   le puits colonne 22 (25 à 31), sous la stalactite (22, 24).
/// - Gouttes : (13, 5), (1, 17) dans la niche, (12, 39) ; poisson (20, 17),
///   mouette (9, 34).
const ASCII: &str = r"
................................
................................
................................
................................
K.@..G........................K.
#######......o...............###
BBBBBBB......................BBB
BBBBBBB......................BBB
BBBBBBB~~~}}}}}}}}}}}}}}}}}||BBB
BBBBBBBBBBBBBBBBBBBBBBBBBBB||BBB
BBBBBBBBBBBBBBBBBBBBBBBBBBB||BBB
BB||||.....................||BBB
BB||||.....................||BBB
BB||||.....................||BBB
BB||||.....................||BBB
..||||.....................||BBB
..||||....L................||BBB
.o||||########......i......||BBB
BB||||BBBBBBBB{{{{{{{{{{{{{{{BBB
BB||||BBBBBBBBBBBBBBBBBBBBBBBBBB
BB||||BBBBBBBBBBBBBBBBBBBBBBBBBB
BB||||BBBBBBBBBBBBBBBBBBBBBBBBBB
BB||||BBBBBBBBBBBBBBBBBBBBBBBBBB
BB||||BBBBBBBBBBBBBBBBBBBBBBBBBB
BB||||................VBBBBBBBBB
BB||||.........L......|BBBBBBBBB
BB||||.......#########|BBBBBBBBB
B}}}}}}}}}}}}BBBBBBBBB|BBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBB|BBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBB|BBBBBBBBB
.............BBBBBBBBB|BBBBBBBBB
.............BBBBBBBBB|BBBBBBBBB
...............................B
...............................B
.........u.....................B
##########.....................B
...............................B
...........................L..KB
..................#############B
..X.........o....BBBBBBBBBBBBBBB
######...........BBBBBBBBBBBBBBB
BBBBBB{{{{{{{{{{{BBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
