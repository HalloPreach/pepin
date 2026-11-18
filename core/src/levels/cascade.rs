//! Monde 2, niveau 3 : « La grande cascade » (carte 9.8, allongée par la carte
//! 9.21b). Le niveau VERTICAL du Monde 2, à l'envers comme « La descente » : on
//! part du haut de la falaise et l'on DESCEND entre les chutes d'eau, de bassin en
//! bassin (onze, dont les trois marches et la mare), jusqu'au pied de la cascade.
//! 32 tuiles de large, 94 de haut (plus de huit écrans) : la caméra suit Pépin en
//! hauteur (carte 5.2).
//!
//! Les deux façons de traverser une chute d'eau (lettre `|`, carte 9.3) :
//! - parapluie OUVERT, on y descend lentement (56 px/s), en sécurité, et l'on
//!   retombe en barque dans le bassin du dessous ;
//! - parapluie FERMÉ, on est emporté vite (380 px/s) : dans un bassin, c'est la
//!   noyade (la chute tue à l'arrivée) ; sur la roche, rien.
//!
//! Le parcours, de haut en bas :
//! 1. le haut de la falaise : Pépin sort du GRAND TUYAU des égouts (carte 9.20c :
//!    sa passerelle est la corniche du départ, la voûte au-dessus est de la roche),
//!    et plane dans le BASSIN A (le panneau « planer » le rappelle), où l'eau du
//!    tuyau se déverse ; son courant `}` porte la barque vers la droite. La
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
//!    on y est encore quand elle tombe. Le puits débouche sur le palier de la
//!    salle des bras (3e lanterne), hors de la colonne de la stalactite ;
//! 7. LA SALLE DES BRAS (carte 9.21b) : deux bras de la cascade (colonnes 15-16
//!    et 7-8) tombent du plafond jusqu'au BASSIN E. On va vers la gauche de
//!    corniche en corniche, du palier à la corniche K1 puis à K2 (4e lanterne), en
//!    PLANANT à travers chaque bras (il freine la descente ; parapluie fermé, il
//!    emporte au bassin). Une mouette garde le bout de K1 : on l'écrase en s'y
//!    posant, ou on la laisse piquer avant de passer. Tombé dans le bassin E, on y
//!    flotte en barque : son courant ramène sous le palier, où des marches
//!    remontent ;
//! 8. LA GALERIE EN BARQUE (carte 9.21b) : au bout de K2, on se laisse tomber
//!    dans le puits de gauche jusqu'à la rive du BASSIN F, une longue galerie sous
//!    la falaise. On y rame (eau calme), puis le courant `}` emporte la barque vers
//!    la droite, sous un poisson qui saute : on attend qu'il replonge avant de se
//!    laisser emporter. Au bout, l'eau tombe (chute des colonnes 28-29) ;
//! 9. LE BASSIN DU CRABE (carte 9.21b) : on descend la chute en planant jusqu'au
//!    BASSIN G, une eau calme : on rame vers la gauche jusqu'au pied d'une berge.
//!    Sur la berge, un crabe (on lui rebondit dessus, ou on le saute) sous une
//!    stalactite ; au bout, on se laisse tomber à gauche ;
//! 10. LES MARCHES DE LA CASCADE (carte 9.21b) : une corniche (5e lanterne), puis
//!    trois petits bassins en escalier vers la droite, reliés par deux courtes
//!    chutes : la barque y passe par-dessus bord (BAS tenu en tombant, ou un saut
//!    et l'on replane), et l'on retombe en barque dans la marche du dessous (la
//!    deuxième est calme : on y rame). Au bout, une rive, et l'on tombe à droite ;
//! 11. LA MARE AUX NÉNUPHARS (carte 9.21b) : une rive (6e lanterne), puis la mare,
//!    vers la gauche, de nénuphar en nénuphar (chacun s'enfonce sous Pépin : on ne
//!    s'y attarde pas ; tombé à l'eau, BAS donne la barque, qui rame lentement).
//!    Une grenouille garde l'autre rive ; on se laisse tomber derrière elle ;
//! 12. le pied de la cascade (carte 9.20c) : de la corniche de gauche, on gagne le
//!    BASSIN D'ARRIVÉE, large, où tombe la grande chute du fond (la 3e goutte
//!    au-dessus de l'eau, sur le chemin) ; il part en RIVIÈRE vers la droite (`}`)
//!    jusqu'au drapeau posé sur la berge de droite (colonne 29).
//!
//! La GRANDE CHUTE (carte 9.20c) est un décor de fond, sur toute la hauteur
//! (`game/src/decor_cascade.rs`) : sans collision ni danger, plus sombre que les
//! chutes `|`, qui sont ses bras au premier plan.
//!
//! Six lanternes, chacune hors de tout danger : la corniche (10, 16), le couloir
//! (15, 25), le palier (24, 35), K2 (3, 40), la corniche des marches (1, 68), la
//! rive de la mare (29, 81). Ennemis : deux poissons (bassins B et F), une
//! mouette (K1), un crabe, une grenouille. Les oiseaux du décor (`K`) n'ont aucun
//! effet sur le jeu.

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
    intro: "L'eau ressort du grand tuyau au bord de la falaise et tombe de bassin en bassin.",
    mecanique: "Les chutes d'eau : parapluie ouvert, on y descend lentement jusqu'au \
                bassin du dessous ; parapluie fermé, elles emportent vite (et l'on se noie \
                si un bassin attend en bas).",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (32 x 94 tuiles). Légende : voir `crate::level`.
/// - Bassins : A rangée 8 (colonnes 7 à 26), B rangée 18 (14 à 28), C rangée 27
///   (1 à 12), E rangée 46 (6 à 24), F rangée 53 (2 à 27), G rangée 62 (12 à 29),
///   les marches rangées 69 (3 à 10), 72 (11 à 20) et 75 (21 à 27), la mare
///   rangée 82 (4 à 26), D (l'arrivée, qui part en rivière) rangée 90 (6 à 31).
/// - Le grand tuyau du départ : voûte rangées 0 (colonnes 0 à 4) et 1 (0 à 6), passerelle
///   rangée 5 ; drapeau (29, 88) sur la berge de droite.
/// - Chutes : C1 colonnes 27-28 (rangées 8 à 17), C2 colonnes 2 à 5 (11 à 26),
///   le puits colonne 22 (25 à 31), sous la stalactite (22, 24) ; les bras de la
///   salle, colonnes 15-16 et 7-8 (31 à 45) ; la chute du bassin F, colonnes
///   28-29 (53 à 59) ; celles des marches, colonnes 11-12 (69 à 71) et 21-22 (72
///   à 74).
/// - Gouttes : (13, 5), (1, 17) dans la niche, (12, 88) ; poissons (20, 17) et
///   (25, 52) ; mouette (10, 37) ; crabe (7, 60) ; grenouille (3, 81) ;
///   stalactite de la berge (5, 56) ; nénuphars rangée 81 (colonnes 7 à 23).
/// - La rive du puits de gauche (colonnes 0-1, rangée 52) : on s'y pose avant la
///   galerie. Aucun ennemi d'eau au-dessus d'une eau CALME : arrêté là, le robot
///   de la mesure ne retrouverait plus l'heure de l'ennemi (le poisson du bassin F
///   saute au-dessus du courant).
const ASCII: &str = r"
BBBBB...........................
BBBBBBB.........................
................................
................................
..@..G........................K.
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
BBBBBBBBBBBBBBBBBBBBBB|BBBBBBBBB
.......||......||.....|........B
.......||......||..............B
.......||......||..............B
.......||......||..............B
.......||......||.......L......B
.......||......||.#########....B
.......||.u....||..............B
.......||.####.||..............B
.......||......||..........##..B
...L...||......||..............B
..####.||......||..............B
..BBBB.||......||............##B
..BBBB.||......||..............B
..BBBB.||......||..............B
..BBBB.||......||........######B
..BBBB}}}}}}}}}}}}}}}}}}}BBBBBBB
..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
..............................BB
..............................BB
..............................BB
##.......................i....BB
BB~~~~~~~~~~~~~~~~}}}}}}}}}}||BB
BBBBBBBBBBBBBBBBBBBBBBBBBBBB||BB
BBBBBBBBBBBBBBBBBBBBBBBBBBBB||BB
.....V......................||BB
............................||BB
............................||BB
............................||BB
.......k....................BBBB
..##########..................BB
..BBBBBBBBBB~~~~~~~~~~~~~~~~~~BB
..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
...............................B
.L.............................B
###}}}}}}}}||..................B
BBBBBBBBBBB||..................B
BBBBBBBBBBB||..................B
BBBBBBBBBBB~~~~~~~~~~||........B
BBBBBBBBBBBBBBBBBBBBB||........B
BBBBBBBBBBBBBBBBBBBBB||.....##.B
BBBBBBBBBBBBBBBBBBBBB}}}}}}}BB.B
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.B
...............................B
...............................B
...............................B
...............................B
...j...n...n...n...n...n.....L.B
..##~~~~~~~~~~~~~~~~~~~~~~~####B
..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
...............................B
...............................B
...............................B
...............................B
######......o................X.B
BBBBBB.....................####B
BBBBBB}}}}}}}}}}}}}}}}}}}}}}}}}}
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
