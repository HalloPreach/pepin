//! Sprites de la Pieuvre du port, boss du Monde 2 (carte 9.12a) : sa tête (48x32,
//! en colère, touchée, vaincue), ses tentacules en tronçons (24x8) avec leur bout
//! recourbé (24x16), l'annonce (remous, ombre, bout qui pointe), la clé de la
//! victoire et ses points de vie dans le HUD. Corail et violet de la palette, contour
//! sombre, aucune couleur ajoutée.

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
    ("cle_pieuvre", CLE_PIEUVRE),
    ("pv_pieuvre", PV_PIEUVRE),
    ("pv_pieuvre_perdu", PV_PIEUVRE_PERDU),
];
