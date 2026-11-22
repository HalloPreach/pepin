//! Sprites de la Pieuvre du port, boss du Monde 2 (carte 9.12a) : sa tête (48x32,
//! en colère, touchée, vaincue), ses tentacules en tronçons (24x8) avec leur bout
//! recourbé (24x16), l'annonce (remous, ombre, bout qui pointe), la clé de la
//! victoire et ses points de vie dans le HUD. Carte 9.13c : la GÉANTE de la montée (sa
//! tête de 64x40, son œil, ses tentacules en tronçons ronds), le repère de la frappe
//! visée et les caisses de la secousse. Corail et violet de la palette, contour sombre,
//! aucune couleur ajoutée.

use super::Image;

/// Tête de la Pieuvre (48x32), en colère, 1re image : corps corail, reflet corail clair,
/// ombre violette en croissant à droite, bras recourbés au pied ; grands yeux blancs sous
/// des sourcils froncés, pupilles à gauche ; écume au ras de l'eau (dernière rangée).
pub const PIEUVRE_COLERE0: Image = &[
    "................................................",
    ".................00000000000000.................",
    "..............00088888888888888000..............",
    "............008888888888888888888800............",
    "...........088988888888888888888888F0...........",
    ".........00999999999888888998888888FF00.........",
    "........0999999999999888888888888888FFF0........",
    ".......099999999999999888888888888888FFF0.......",
    "......0899999999999998888888889988888FFFF0......",
    "......08899999999999899888888888888888FFF0......",
    ".....0888000889888888888888888888888000FFF0.....",
    "....08888000000888888888888888888000000FFFF0....",
    "....08888888000000888888888888000000988FFFF0....",
    "...088888888008000088888888880000800888FFFFF0...",
    "...088888880888888008888888800888888088FFFFF0...",
    "..088888880FFFFFFFFF08888880FFFFFFFFF08FFFFFF0..",
    "..0888888807770007770888888077700077708FFFFFF0..",
    "..08888888077700077708888880777000777088FFFFF0..",
    "..0888888807770007770888888077700077708FFFFFF0..",
    "..0888888807770007770888888077700077708FFFFFF0..",
    "..0888888880777777708888888807777777088FFFFFF0..",
    ".08888888888000000088888888880000000888FFFFFFF0.",
    "..0888888888888888888888888888888888888FFFFFF0..",
    "..0088888888888888888888888888888888888FFFFF00..",
    "....0888888888888888000000008888888888FFFFF0....",
    "...08888888888888880888888880888888888FFFFFF0...",
    ".008888888888888888888888888888888888FFFFFFFF00.",
    "0888888888888888888888888888888888888FFFFFFFFFF0",
    "0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0..",
    "..7DDD777DDD777DDD777DDD777DDD777DDD777DDD777D..",
];

/// En colère, 2e image : les pupilles regardent à droite, l'écume a bougé.
pub const PIEUVRE_COLERE1: Image = &[
    "................................................",
    ".................00000000000000.................",
    "..............00088888888888888000..............",
    "............008888888888888888888800............",
    "...........088988888888888888888888F0...........",
    ".........00999999999888888998888888FF00.........",
    "........0999999999999888888888888888FFF0........",
    ".......099999999999999888888888888888FFF0.......",
    "......0899999999999998888888889988888FFFF0......",
    "......08899999999999899888888888888888FFF0......",
    ".....0888000889888888888888888888888000FFF0.....",
    "....08888000000888888888888888888000000FFFF0....",
    "....08888888000000888888888888000000988FFFF0....",
    "...088888888008000088888888880000800888FFFFF0...",
    "...088888880888888008888888800888888088FFFFF0...",
    "..088888880FFFFFFFFF08888880FFFFFFFFF08FFFFFF0..",
    "..0888888807777700070888888077777000708FFFFFF0..",
    "..08888888077777000708888880777770007088FFFFF0..",
    "..0888888807777700070888888077777000708FFFFFF0..",
    "..0888888807777700070888888077777000708FFFFFF0..",
    "..0888888880777777708888888807777777088FFFFFF0..",
    ".08888888888000000088888888880000000888FFFFFFF0.",
    "..0888888888888888888888888888888888888FFFFFF0..",
    "..0088888888888888888888888888888888888FFFFF00..",
    "....0888888888888888000000008888888888FFFFF0....",
    "...08888888888888880888888880888888888FFFFFF0...",
    ".008888888888888888888888888888888888FFFFFFFF00.",
    "0888888888888888888888888888888888888FFFFFFFFFF0",
    "0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0..",
    "..D777DDD777DDD777DDD777DDD777DDD777DDD777DDD7..",
];

/// Touchée : yeux plissés en chevrons, bouche ouverte, joues marquées d'or pâle.
pub const PIEUVRE_TOUCHEE: Image = &[
    "................................................",
    ".................00000000000000.................",
    "..............00088888888888888000..............",
    "............008888888888888888888800............",
    "...........088988888888888888888888F0...........",
    ".........00999999999888888998888888FF00.........",
    "........0999999999999888888888888888FFF0........",
    ".......099999999999999888888888888888FFF0.......",
    "......0899999999999998888888889988888FFFF0......",
    "......08899999999999899888888888888888FFF0......",
    ".....088888888988888888888888888888888FFFF0.....",
    "....08888888888888888888888888888888888FFFF0....",
    "....08888888888888888888888888888889988FFFF0....",
    "...088888888888888888888888888888888888FFFFF0...",
    "...088888888008888888888888888888800888FFFFF0...",
    "..0888888888800888888888888888888008888FFFFFF0..",
    "..0888888888880088888888888888880088888FFFFFF0..",
    "..08888888888880088888888888888008888888FFFFF0..",
    "..0888888888888008888888888888800888888FFFFFF0..",
    "..0888888888880088888888888888880088888FFFFFF0..",
    "..0888888888800888888888888888888008888FFFFFF0..",
    ".08888888888008888888888888888888800888FFFFFFF0.",
    "..088888CC8888888888888888888888888888CCFFFFF0..",
    "..0088888888888888888000000888888888888FFFFF00..",
    "....0888888888888888801111088888888888FFFFF0....",
    "...08888888888888888801111088888888888FFFFFF0...",
    ".008888888888888888880000008888888888FFFFFFFF00.",
    "0888888888888888888888888888888888888FFFFFFFFFF0",
    "0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0..",
    "..7DDD777DDD777DDD777DDD777DDD777DDD777DDD777D..",
];

/// Vaincue : yeux en croix, bouche ondulée (elle replonge avec cette image).
pub const PIEUVRE_VAINCUE: Image = &[
    "................................................",
    ".................00000000000000.................",
    "..............00088888888888888000..............",
    "............008888888888888888888800............",
    "...........088988888888888888888888F0...........",
    ".........00999999999888888998888888FF00.........",
    "........0999999999999888888888888888FFF0........",
    ".......099999999999999888888888888888FFF0.......",
    "......0899999999999998888888889988888FFFF0......",
    "......08899999999999899888888888888888FFF0......",
    ".....088888888988888888888888888888888FFFF0.....",
    "....08888888888888888888888888888888888FFFF0....",
    "....08888888888888888888888888888889988FFFF0....",
    "...088888888888888888888888888888888888FFFFF0...",
    "...088888888088880888888888880888808888FFFFF0...",
    "..0888888888808808888888888888088088888FFFFFF0..",
    "..0888888888880088888888888888800888888FFFFFF0..",
    "..08888888888800888888888888888008888888FFFFF0..",
    "..0888888888808808888888888888088088888FFFFFF0..",
    "..0888888888088880888888888880888808888FFFFFF0..",
    "..0888888888888888888888888888888888888FFFFFF0..",
    ".08888888888888888888888888888888888888FFFFFFF0.",
    "..0888888888888888888888888888888888888FFFFFF0..",
    "..0088888888888888888888888888888888888FFFFF00..",
    "....0888888888888888080808080888888888FFFFF0....",
    "...08888888888888880808080808888888888FFFFFF0...",
    ".008888888888888888888888888888888888FFFFFFFF00.",
    "0888888888888888888888888888888888888FFFFFFFFFF0",
    "0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0..",
    "..7DDD777DDD777DDD777DDD777DDD777DDD777DDD777D..",
];

/// Bout du tentacule (24x16) : la tige s'affine et se recourbe en crosse ; son pied se
/// raccorde au tronçon (`tentacule0`).
pub const TENTACULE_BOUT: Image = &[
    "........0000............",
    "......00888800..........",
    ".....0888888880.........",
    ".....0888008880.........",
    "....08880..08880........",
    "....0980..08888F00......",
    "....0980.0888888FF0.....",
    "....000808888888FF0.....",
    ".......0988888CCFF0.....",
    ".......0988888888FF0....",
    "......09888888888FF0....",
    ".....0988CC888888FF0....",
    ".....0988888888888FF0...",
    "....09888888888888FF0...",
    "...098888888CC8888FF0...",
    "..0098888888CC8888FF00..",
];

/// Tronçon de tentacule (24x8), répété de la surface jusqu'au bout : corps corail, reflet
/// à gauche, ombre violette à droite, deux ventouses d'or pâle. 1re image.
pub const TENTACULE0: Image = &[
    "..0998888888888888FFF0..",
    "..0998889988888888FFF0..",
    "..099888CC88888888FFF0..",
    "..099888CC88888888FFF0..",
    "..0998888888899888FFF0..",
    "..09988888888CC888FFF0..",
    "..09988888888CC888FFF0..",
    "..099F8FF8FF8FF8FFFFF0..",
];

/// Tronçon, 2e image : les ventouses ont glissé (le tentacule ondule).
pub const TENTACULE1: Image = &[
    "..09988888888CC888FFF0..",
    "..09988888888CC888FFF0..",
    "..0998888888888888FFF0..",
    "..0998888888888888FFF0..",
    "..0998889988888888FFF0..",
    "..099888CC88888888FFF0..",
    "..099888CC88888888FFF0..",
    "..099F8FF8FF8FF8FFFFF0..",
];

/// Annonce, 2e moitié (24x12) : le bout du tentacule pointe hors de l'eau, dans l'écume.
pub const TENTACULE_POINTE0: Image = &[
    "........0000............",
    "......00888800..........",
    ".....0888888880.........",
    ".....0888008880.........",
    "....08880..08880........",
    "....0980..08888F00......",
    "....0980.0888888FF0.....",
    "....000808888888FF0.....",
    ".......0988888CCFF0.....",
    ".......0988888888FF0....",
    "......09888888888FF0....",
    "...7DD7DD7DD7DD7DD7DD...",
];

/// Le bout qui pointe, 2e image (l'écume bouge).
pub const TENTACULE_POINTE1: Image = &[
    "........0000............",
    "......00888800..........",
    ".....0888888880.........",
    ".....0888008880.........",
    "....08880..08880........",
    "....0980..08888F00......",
    "....0980.0888888FF0.....",
    "....000808888888FF0.....",
    ".......0988888CCFF0.....",
    ".......0988888888FF0....",
    "......09888888888FF0....",
    "...DD7DD7DD7DD7DD7DD7...",
];

/// Remous sur l'eau (32x6) : annonce d'un tentacule (1re moitié) et de la tête qui
/// remonte. 1re image.
pub const REMOUS_PIEUVRE0: Image = &[
    "......7....................7....",
    "..77......77......777.....77....",
    ".7DD7....7DD7....7DD7....7DD7...",
    "7DDDD7..7DDDD7..7DDDD7..7DDDD7..",
    "DDDDDD77DDDDDD77DDDDDD77DDDDDD77",
    "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
];

/// Remous, 2e image.
pub const REMOUS_PIEUVRE1: Image = &[
    "........7..................7....",
    "77......77......777.....77......",
    "DD7....7DD7....7DD7....7DD7....7",
    "DDD7..7DDDD7..7DDDD7..7DDDD7..7D",
    "DDDD77DDDDDD77DDDDDD77DDDDDD77DD",
    "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
];

/// L'ombre de la pieuvre sous la surface (32x4), avec les remous.
pub const OMBRE_PIEUVRE: Image = &[
    ".....EEEEEEEEEEEEEEEEEEEEEE.....",
    ".EEEEE11111111111111111111EEEEE.",
    ".EEEEE11111111111111111111EEEEE.",
    ".....EEEEEEEEEEEEEEEEEEEEEE.....",
];

/// La clé que la Pieuvre vaincue lâche en replongeant (14x8), or.
pub const CLE_PIEUVRE: Image = &[
    "..000.........",
    ".0BBB0........",
    "0B0.0B00000000",
    "0B0.0BBBBBBBB0",
    "0BC.CBCCCCCCC0",
    ".0CCC00000B0B0",
    "..000.....0.0.",
    "..............",
];

/// Point de vie de la Pieuvre dans le HUD (10x6) : une petite pieuvre corail.
pub const PV_PIEUVRE: Image = &[
    "...0000...",
    "..088880..",
    ".08708780.",
    ".08888880.",
    "0808080800",
    ".0.0.0.0..",
];

/// Point de vie perdu (10x6) : la même, éteinte (ardoise).
pub const PV_PIEUVRE_PERDU: Image = &[
    "...0000...",
    "..022220..",
    ".02302320.",
    ".02222220.",
    "0202020200",
    ".0.0.0.0..",
];

/// Tentacule ENROULÉ sur le dessus d'une plateforme saisie (16x6, carte 9.12f),
/// posé case par case : corps corail, reflet corail clair, ventouses d'or pâle,
/// ombre violette dessous. 1re image.
pub const TENTACULE_ENROULE0: Image = &[
    ".00000000000000.",
    "0998899888998890",
    "08CC8888CC888CC0",
    "0888888888888880",
    "0FFFF8FFFF8FFFF0",
    ".00000000000000.",
];

/// Tentacule enroulé, 2e image : les ventouses ont glissé (il serre).
pub const TENTACULE_ENROULE1: Image = &[
    ".00000000000000.",
    "0889988899888990",
    "0888CC8888CC8880",
    "0888888888888880",
    "0FF8FFFF8FFFF8F0",
    ".00000000000000.",
];

/// Une bulle qui remonte sous une plateforme saisie (4x4, carte 9.12g) : contour
/// sombre, reflet blanc cassé, cyan. Elle crève à la surface.
pub const BULLE_SAISIE: Image = &[
    ".00.",
    "07D0",
    "0DD0",
    ".00.",
];

/// Bout gauche du cercle de remous de la saisie (4x5, carte 9.12g ; le bout droit
/// est son miroir) : l'anneau d'écume posé à plat sur la surface, la 3e rangée au
/// ras de l'eau. Entre les deux bouts, deux traits d'écume (`dessiner_saisie`).
pub const ONDE_SAISIE: Image = &[
    "..00",
    ".077",
    "07..",
    ".077",
    "..00",
];

/// Lèvre d'écume au bord gauche du creux (6x4, carte 9.12g ; le bord droit est son
/// miroir) : l'eau bourrelée qui tourne, contour sombre au dehors, la dernière
/// rangée au ras de la surface. Pas de gerbe : 3 px au-dessus de l'eau, pas plus.
pub const BORD_CREUX_SAISIE: Image = &[
    "...00.",
    "..0770",
    ".07777",
    "0777DD",
];

/// La tête de la Pieuvre GÉANTE de la montée (64x40, carte 9.13c), 1re image : le manteau
/// corail (reflet corail clair en haut à gauche, ombre violette en croissant à droite, taches
/// claires), le grand œil blanc cerné sous un sourcil froncé (la pupille est posée à part :
/// elle suit Pépin), et cinq racines de tentacules au ras de l'eau, ventouses d'or pâle.
pub const GEANTE0: Image = &[
    "............................000000000000........................",
    "........................00008888888888880000....................",
    ".....................00088888888888888888888000.................",
    "...................00999999888888888888888888FF00...............",
    ".................0099999999998888888888888888FFFF00.............",
    "...............0099999999999999888888888888888FFFFF00...........",
    "..............099999999999999988888888889888888FFFFFF0..........",
    ".............0999999999999999998888888888888888FFFFFFF0.........",
    "............099999999999999999988888888888888888FFFFFFF0........",
    "...........0000000000009999999988888888888889888FFFFFFFF0.......",
    "..........08000000000000099999988888888888888888FFFFFFFFF0......",
    ".........0800077700000000000999888888898888888888FFFFFFFFF0.....",
    ".........0000777777777700000000888888888888888888FFFFFFFFF0.....",
    "........00007777777777777700000888888888888888888FFFFFFFFFF0....",
    "........000777777777777777700888888888888888888988FFFFFFFFF0....",
    ".......0800777777777777777700888888888888888888888FFFFFFFFFF0...",
    ".......0000777777777777777700088888888888888888888FFFFFFFFFF0...",
    ".......0000777777777777777700088888888888888888888FFFFFFFFFF0...",
    ".......0800777777777777777700888888888888888888888FFFFFFFFFF0...",
    ".......0800777777777777777700888888888888898888888FFFFFFFFFF0...",
    ".......0800077777777777777000888888888888888888888FFFFFFFFFF0...",
    ".......0880007777777777770008888888888888888888888FFFFFFFFFF0...",
    ".......0888000777777777700088888888888888888888888FFFFFFFFFF0...",
    ".......0888880000000000008888888888888888888888888FFFFFFFFFF0...",
    ".0000008888888800000000888888888888888888888888888FFFFFFFFFF0...",
    "..088888888888888888888888888888888888888888888888FFFFFFFFFF0...",
    "..088888888888888888888888888888888888888888888888FFFFFFFFFF0...",
    "..088888888888888888888888888888888888888888888888FFFFFFFFF0....",
    "..0888888880008888888880008888888880000888888888000FFFFFFFF0....",
    "..088888880...088888880...088888880....088888880...0FFFFFFF0....",
    "..088888880...088888880...088888880....088888880...0FFFFFFF0....",
    "..088888880...088888880...088888880....088888880...0FFFFFFF0....",
    "..088888C80...088888C80...088888C80....088C88880...0FFCFFFF0....",
    "..088888F0....088888F0....088888F0......0888FFF0....0FFFFFF0....",
    "..088888F0....088888F0....088888F0......0888FFF0....0FFFFFF0....",
    "..088888C0....088888C0....088888C0......08C8FFF0....0FCFFFF0....",
    "..0888880.....0888880.....0888880........088FFF0.....0FFFFF0....",
    "..0888880.....0888880.....0888880........088FFF0.....0FFFFF0....",
    "..088880......088880......088880..........08FFF0......0FFFF0....",
    "..000000......000000......000000..........000000......000000....",
];

/// La tête de la géante, 2e image : les racines ondulent d'un pixel.
pub const GEANTE1: Image = &[
    "............................000000000000........................",
    "........................00008888888888880000....................",
    ".....................00088888888888888888888000.................",
    "...................00999999888888888888888888FF00...............",
    ".................0099999999998888888888888888FFFF00.............",
    "...............0099999999999999888888888888888FFFFF00...........",
    "..............099999999999999988888888889888888FFFFFF0..........",
    ".............0999999999999999998888888888888888FFFFFFF0.........",
    "............099999999999999999988888888888888888FFFFFFF0........",
    "...........0000000000009999999988888888888889888FFFFFFFF0.......",
    "..........08000000000000099999988888888888888888FFFFFFFFF0......",
    ".........0800077700000000000999888888898888888888FFFFFFFFF0.....",
    ".........0000777777777700000000888888888888888888FFFFFFFFF0.....",
    "........00007777777777777700000888888888888888888FFFFFFFFFF0....",
    "........000777777777777777700888888888888888888988FFFFFFFFF0....",
    ".......0800777777777777777700888888888888888888888FFFFFFFFFF0...",
    ".......0000777777777777777700088888888888888888888FFFFFFFFFF0...",
    ".......0000777777777777777700088888888888888888888FFFFFFFFFF0...",
    ".......0800777777777777777700888888888888888888888FFFFFFFFFF0...",
    ".......0800777777777777777700888888888888898888888FFFFFFFFFF0...",
    ".......0800077777777777777000888888888888888888888FFFFFFFFFF0...",
    ".......0880007777777777770008888888888888888888888FFFFFFFFFF0...",
    ".......0888000777777777700088888888888888888888888FFFFFFFFFF0...",
    ".......0888880000000000008888888888888888888888888FFFFFFFFFF0...",
    ".0000008888888800000000888888888888888888888888888FFFFFFFFFF0...",
    "..088888888888888888888888888888888888888888888888FFFFFFFFFF0...",
    "..088888888888888888888888888888888888888888888888FFFFFFFFFF0...",
    "..088888888888888888888888888888888888888888888888FFFFFFFFF0....",
    "..0888888888008888888880008888888888008888888888000FFFFFFFF0....",
    "..0888888880..088888880...0888888880..0888888880...0FFFFFFF0....",
    "..0888888880..088888880...0888888880..0888888880...0FFFFFFF0....",
    "...08888880...08888880.....08888880....08888880.....0FFFFFF0....",
    "...08888C80...08888C80.....08888C80....08C88880.....0FCFFFF0....",
    "...08888FF0...088888F0.....08888FF0....08888FF0.....0FFFFFF0....",
    "...08888FF0..08888880......08888FF0....08888FF0......0FFFFFF0...",
    "...08888C0...088888C0......08888C0......0C88FF0......0CFFFFF0...",
    "...08888F0...0888880.......08888F0......0888FF0.......0FFFFF0...",
    "...08888F0...0888880.......08888F0......0888FF0.......0FFFFF0...",
    "...088880....0888880.......088880........088FF0.......0FFFFF0...",
    "...000000....000000........000000........000000........000000...",
];

/// La paupière de la géante (18x14, posée sur l'œil) : elle PLISSE l'œil pendant
/// l'annonce d'une attaque (le haut de l'œil fermé, bord sombre).
pub const PAUPIERE_GEANTE: Image = &[
    "..................",
    "....9999999999....",
    "...999999999999...",
    "..88888888888888..",
    ".8888888888888888.",
    ".8888888888888888.",
    ".8888888888888888.",
    ".0000000000000000.",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
];

/// L'œil SONNÉ de la géante (18x14, posé sur l'œil) : une spirale sombre sur le blanc.
pub const OEIL_SONNE: Image = &[
    "..................",
    "....7777077070....",
    "...777077777777...",
    "..77077777777770..",
    ".7707777007077777.",
    ".7777770777700777.",
    ".7077707777770777.",
    ".7777700700777077.",
    ".7077770000777777.",
    ".7707777777770777.",
    "..77777777777077..",
    "...770077707077...",
    "....7777077777....",
    "..................",
];

/// La pupille de la géante (6x6) : elle suit Pépin dans le blanc de l'œil.
pub const PUPILLE_GEANTE: Image = &[
    ".0000.",
    "070000",
    "000000",
    "000000",
    "000000",
    ".0000.",
];

/// La pupille ROUGE (8x8) : l'œil rougit pendant l'annonce d'une attaque.
pub const PUPILLE_ROUGE: Image = &[
    "..8888..",
    ".880088.",
    "88000088",
    "80700008",
    "80000008",
    "88000088",
    ".880088.",
    "..8888..",
];

/// La bouche de la géante qui HURLE (12x8), quand on la frappe ou qu'elle se jette.
pub const BOUCHE_GEANTE: Image = &[
    "...000000...",
    "..01111110..",
    ".0111111110.",
    "011CC11CC110",
    "01111111110.",
    "0111111110..",
    ".00111110...",
    "...00000....",
];

/// Un tronçon rond de tentacule de la géante, près du corps (10x10) : corail, reflet,
/// ombre violette, ventouse d'or pâle. Les tentacules sont des chapelets de tronçons, du
/// corps jusqu'au bout, de plus en plus fins (`dessin_montee`).
pub const BRAS_GROS: Image = &[
    "...0000...",
    ".00999800.",
    ".09998880.",
    "0999888880",
    "0998888880",
    "0988CC88F0",
    "0888888FF0",
    ".08888FF0.",
    ".0088FF00.",
    "...0000...",
];

/// Tronçon du milieu (8x8).
pub const BRAS_MOYEN: Image = &[
    "..0000..",
    ".099880.",
    "09988880",
    "09888880",
    "088CC8F0",
    "08888FF0",
    ".088FF0.",
    "..0000..",
];

/// Tronçon fin, vers le bout (6x6).
pub const BRAS_FIN: Image = &[
    ".0000.",
    "009800",
    "098880",
    "0888F0",
    "008F00",
    ".0000.",
];

/// Le bout du tentacule (4x4).
pub const BRAS_POINTE: Image = &[
    ".00.",
    "0890",
    "08F0",
    ".00.",
];

/// Le repère ROUGE de la frappe visée (16x16) : quatre coins et un centre corail,
/// cernés de sombre, sur la place visée. 1re image.
pub const REPERE_FRAPPE0: Image = &[
    "88880......08888",
    "8000........0008",
    "80............08",
    "80............08",
    "0..............0",
    "......0000......",
    ".....088880.....",
    ".....080080.....",
    ".....080080.....",
    ".....088880.....",
    "......0000......",
    "0..............0",
    "80............08",
    "80............08",
    "8000........0008",
    "88880......08888",
];

/// Le repère, 2e image : les coins se resserrent (il palpite).
pub const REPERE_FRAPPE1: Image = &[
    ".0000......0000.",
    "088880....088880",
    "08000......00080",
    "080..........080",
    "080..........080",
    ".0....0000....0.",
    ".....088880.....",
    ".....080080.....",
    ".....080080.....",
    ".....088880.....",
    ".0....0000....0.",
    "080..........080",
    "080..........080",
    "08000......00080",
    "088880....088880",
    ".0000......0000.",
];

/// Une caisse qui tombe du haut de la tour pendant une secousse (16x16) : planches
/// orange, croisillon d'or, contour sombre.
pub const CAISSE_CHUTE: Image = &[
    "0000000000000000",
    "0AAAAAAAAAAAAAA0",
    "0A000000000000A0",
    "0A0BAAAAAAAA0BA0",
    "0A0AB2AAAAAA0AA0",
    "0A0A0BAAAAA0AAA0",
    "0A0AA0BAAA0AAAA0",
    "0A0AAA0BA0AAAAA0",
    "0A0AAAA0B0AAAAA0",
    "0A0AAA0A0B0AAAA0",
    "0A0AA0AAAA0BAAA0",
    "0A0A0AAAAAA0BAA0",
    "0A00AAAAAAAA0BA0",
    "0A000000000000A0",
    "0AAAAAAAAAAAAAA0",
    "0000000000000000",
];

/// L'ombre d'une caisse qui va tomber (16x4), posée sur le sol où elle se brisera.
pub const OMBRE_CAISSE: Image = &[
    ".00000000000000.",
    "0000000000000000",
    "0000000000000000",
    ".00000000000000.",
];

/// Les éclats d'une caisse brisée (16x6).
pub const ECLATS_CAISSE: Image = &[
    "..0A0.....0B0...",
    ".0AA0..0..0AA0..",
    "..00..0A0..00.0.",
    "0A0....0..0A0.0A",
    "00.0A0.....00..0",
    "...00...........",
];

/// Tous les sprites de la Pieuvre, par nom.
pub const SPRITES: &[(&str, Image)] = &[
    ("pieuvre_colere0", PIEUVRE_COLERE0),
    ("pieuvre_colere1", PIEUVRE_COLERE1),
    ("pieuvre_touchee", PIEUVRE_TOUCHEE),
    ("pieuvre_vaincue", PIEUVRE_VAINCUE),
    ("tentacule_bout", TENTACULE_BOUT),
    ("tentacule0", TENTACULE0),
    ("tentacule1", TENTACULE1),
    ("tentacule_enroule0", TENTACULE_ENROULE0),
    ("tentacule_enroule1", TENTACULE_ENROULE1),
    ("tentacule_pointe0", TENTACULE_POINTE0),
    ("tentacule_pointe1", TENTACULE_POINTE1),
    ("remous_pieuvre0", REMOUS_PIEUVRE0),
    ("remous_pieuvre1", REMOUS_PIEUVRE1),
    ("ombre_pieuvre", OMBRE_PIEUVRE),
    ("bulle_saisie", BULLE_SAISIE),
    ("onde_saisie", ONDE_SAISIE),
    ("bord_creux_saisie", BORD_CREUX_SAISIE),
    ("cle_pieuvre", CLE_PIEUVRE),
    ("pv_pieuvre", PV_PIEUVRE),
    ("pv_pieuvre_perdu", PV_PIEUVRE_PERDU),
    // Carte 9.13c : la géante de la montée.
    ("geante0", GEANTE0),
    ("geante1", GEANTE1),
    ("paupiere_geante", PAUPIERE_GEANTE),
    ("oeil_sonne", OEIL_SONNE),
    ("pupille_geante", PUPILLE_GEANTE),
    ("pupille_rouge", PUPILLE_ROUGE),
    ("bouche_geante", BOUCHE_GEANTE),
    ("bras_gros", BRAS_GROS),
    ("bras_moyen", BRAS_MOYEN),
    ("bras_fin", BRAS_FIN),
    ("bras_pointe", BRAS_POINTE),
    ("repere_frappe0", REPERE_FRAPPE0),
    ("repere_frappe1", REPERE_FRAPPE1),
    ("caisse_chute", CAISSE_CHUTE),
    ("ombre_caisse", OMBRE_CAISSE),
    ("eclats_caisse", ECLATS_CAISSE),
];
