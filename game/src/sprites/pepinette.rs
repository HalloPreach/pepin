//! Sprites de Pépinette : la version fille de Pépin (carte 7.1), premier skin ajouté.
//!
//! Même format que `pepin.rs` (1 caractère = 1 pixel, `.` transparent, `0`-`F` =
//! indice dans `palette::TOUTES`) et MÊMES images, une à une : mêmes tailles (16x16,
//! 20x20 parapluie ouvert, 10x10 pour l'icône des vies), même silhouette, mêmes pieds,
//! même parapluie au même endroit. Le jeu reste donc identique : seule la tenue change.
//!
//! Ce qui la distingue de Pépin, dans chaque image :
//! - un nœud corail (`8`) posé à gauche sur le haut de la tête ;
//! - des cils : un pixel sombre en haut, à l'extérieur de chaque œil ;
//! - une petite robe corail : les rangées d'ombre (`5`) sous le corps blanc ;
//! - un parapluie cyan de la pluie (`D`, plis `6`) au lieu du corail : on la
//!   reconnaît de loin. Les joues corail clair (`9`) restent.
//!
//! Les étincelles de la réapparition (`PEPINETTE_MORT2`) sont celles de Pépin.

use super::{Image, JeuImages};

pub const PEPINETTE_IDLE0: Image = &[
    "............000.",
    "..000.000...050.",
    "..0880880...0D0.",
    "...0077770..06D0",
    "...077777700DDD0",
    "..07707777007770",
    "..0777077050D6D0",
    "..07770770506DD0",
    "..077977955000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0888888880AAA0",
    "...08888880.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPINETTE_IDLE1: Image = &[
    "............000.",
    "............050.",
    ".000.000....0D0.",
    ".088088000..06D0",
    "..0077777700DDD0",
    "..07707777007770",
    "..0777077070D6D0",
    "..07770770506DD0",
    "..077977955000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0888888880AAA0",
    "...08888880.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPINETTE_COURSE0: Image = &[
    "............000.",
    "..000.000...050.",
    "..0880880...0D0.",
    "...0077770..06D0",
    "...077777700DDD0",
    "..07770707707770",
    "..0777707500D6D0",
    "..07777075006DD0",
    "..077797795000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0888888880AAA0",
    "...08888880.000.",
    "....000000......",
    "..0AA0..0AA0....",
    "...00....00.....",
];

pub const PEPINETTE_COURSE1: Image = &[
    "..000.000...000.",
    "..0880880...050.",
    "...0077770..0D0.",
    "...07777770.06D0",
    "..0777070770DDD0",
    "..07777075007770",
    "..0777707500D6D0",
    "..07779779506DD0",
    "..077777755000A0",
    "..077777755000A0",
    "..0888888880A0A0",
    "...088888800AAA0",
    "....000000..000.",
    "...0AA00AA0.....",
    "....00..00......",
    "................",
];

pub const PEPINETTE_COURSE2: Image = &[
    "............000.",
    "..000.000...050.",
    "..0880880...0D0.",
    "...0077770..06D0",
    "...077777700DDD0",
    "..07770707707770",
    "..0777707500D6D0",
    "..07777075006DD0",
    "..077797795000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0888888880AAA0",
    "...08888880.000.",
    "....000000......",
    "...00..0AA0.....",
    "........00......",
];

pub const PEPINETTE_COURSE3: Image = &[
    "..000.000...000.",
    "..0880880...050.",
    "...0077770..0D0.",
    "...07777770.06D0",
    "..0777070770DDD0",
    "..07777075007770",
    "..0777707500D6D0",
    "..07779779506DD0",
    "..077777755000A0",
    "..077777755000A0",
    "..0888888880A0A0",
    "...088888800AAA0",
    "....000000..000.",
    "...0AA00AA0.....",
    "....00..00......",
    "................",
];

pub const PEPINETTE_SAUT: Image = &[
    "..000.000...000.",
    "..0880880...050.",
    "...0077770..0D0.",
    "...07077770.06D0",
    "..0777077070DDD0",
    "..07770770507770",
    "..0779779550D6D0",
    "..07777775506DD0",
    "..077777755000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "...088888800AAA0",
    "....088880..000.",
    "....000000......",
    "....00..00......",
    "................",
];

pub const PEPINETTE_CHUTE: Image = &[
    "............000.",
    "..000.000...050.",
    "..0880880...0D0.",
    "...0077770..06D0",
    "...077777700DDD0",
    "..07777777707770",
    "..0770777500D6D0",
    "..07770770506DD0",
    "..077707705000A0",
    "..077977955000A0",
    "..0777777550A0A0",
    "..0888888880AAA0",
    "...08880880.000.",
    "....000000......",
    "..0AA0..0AA0....",
    "...00....00.....",
];

pub const PEPINETTE_CLIGNE: Image = &[
    "............000.",
    "..000.000...050.",
    "..0880880...0D0.",
    "...0077770..06D0",
    "...077777700DDD0",
    "..07777777707770",
    "..0707777700D6D0",
    "..07700700506DD0",
    "..077977955000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0888888880AAA0",
    "...08888880.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPINETTE_CIEL: Image = &[
    "............000.",
    "..000.000...050.",
    "..0880880...0D0.",
    "...0077770..06D0",
    "...070777700DDD0",
    "..07770770707770",
    "..0777077050D6D0",
    "..07797797506DD0",
    "..077777755000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0888888880AAA0",
    "...08888880.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPINETTE_TAPOTE: Image = &[
    "................",
    "..000.000...000.",
    "..0880880...050.",
    "...0077770..0D0.",
    "...07777770.06D0",
    "..0770777700DDD0",
    "..07770770507770",
    "..0777077050D6D0",
    "..07797795506DD0",
    "..077777755000A0",
    "..077777755000A0",
    "..0888888880A0A0",
    "...088888800AAA0",
    "....000000..000.",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPINETTE_MORT0: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    ".......C........",
    "................",
    "..B..........B..",
    "000.000.........",
    "088088000000....",
    ".077077777070...",
    ".077707770770...",
    ".055505550550...",
    "..0090009000....",
    "................",
];

pub const PEPINETTE_MORT1: Image = &[
    "................",
    "................",
    "...B........B...",
    ".......B........",
    "................",
    ".......C........",
    ".B............B.",
    "....C......C....",
    "................",
    ".000.000........",
    ".088088000......",
    "..007707700.....",
    "...05505500.....",
    "....090090......",
    "................",
    "................",
];

/// Les étincelles de la réapparition : les mêmes que pour Pépin.
pub const PEPINETTE_MORT2: Image = super::pepin::PEPIN_MORT2;

pub const PEPINETTE_PLANE0: Image = &[
    "......00000000......",
    "....06666DDDDDD0....",
    "...066666DDDDDDD0...",
    "..066606DD0DD0DDD0..",
    ".0666606DD0DD0DDDD0.",
    "06666606DD0DD0DDDDD0",
    "0D00D00D00D00D00D00D",
    ".0..0..0..0..0..0..0",
    ".........00.........",
    "...000.0000.........",
    "...08808800000......",
    "....00707777070.....",
    ".....0770770770.....",
    ".....0770770770.....",
    ".....0797797550.....",
    ".....0777777550.....",
    "......08888880......",
    ".......000000.......",
    "....................",
    ".......00..00.......",
];

pub const PEPINETTE_PLANE1: Image = &[
    "......00000000......",
    "....06666DDDDDD0....",
    "...066666DDDDDDD0...",
    "..066606DD0DD0DDD0..",
    ".0666606DD0DD0DDDD0.",
    "06666606DD0DD0DDDDD0",
    "D00D00D00D00D00D00D0",
    "0..0..0..0..0..0..0.",
    ".........00.........",
    ".........00.........",
    "...000.000..........",
    "...08808800000......",
    "....00707777070.....",
    ".....0770770770.....",
    ".....0770770770.....",
    ".....0797797550.....",
    ".....0777777550.....",
    "......08888880......",
    ".......000000.......",
    ".......00..00.......",
];

/// Icône des vies du HUD quand Pépinette est active : sa tête, nœud et cils compris.
pub const VIE_PEPINETTE: Image = &[
    "000.0000..",
    "088088070.",
    "0770777700",
    "0777077050",
    "0777077050",
    "0779779550",
    "0777777550",
    "0888888880",
    ".08888880.",
    "..000000..",
];

/// Parapluie RETOURNÉ de Pépinette (carte 9.2b), SOUS elle (carte 9.2c) : comme
/// celui de Pépin, debout dans la coque de `PEPINETTE_BARQUE` (toile cyan, bord et
/// baleines en 6, crosse dessous), son nœud corail et sa robe au-dessus. 16x16.
pub const PEPINETTE_RETOURNE: Image = &[
    ".000.0000.......",
    ".08808800000....",
    "..00707777070...",
    "...0770770770...",
    "...0770770770...",
    "...0797797550...",
    "...0888888880...",
    ".00088888888000.",
    "0666066666606660",
    "0DDDDDDDDDDDDDD0",
    ".0DD6DDDDDD6DD0.",
    "..0DD6DDDD6DD0..",
    "...00DD66DD00...",
    ".....000A000....",
    ".......0A0......",
    "......0AA0......",
];

/// Barque de Pépinette (carte 9.2) : la même coque que celle de Pépin, dans la toile
/// cyan de son parapluie (D, bord et baleines en 6), son nœud corail (8) sur la tête.
/// Comme la barque de Pépin, la crosse (A) pointe vers le bas.
pub const PEPINETTE_BARQUE: Image = &[
    "................",
    "....000.000.....",
    "....0880880.....",
    "....07777770....",
    "...0777777770...",
    "...0770770770...",
    "...0797777970...",
    ".00077777777000.",
    "0666066666606660",
    "0DDDDDDDDDDDDDD0",
    ".0DD6DDDDDD6DD0.",
    "..0DD6DDDD6DD0..",
    "...00DD66DD00...",
    ".....000A000....",
    ".......0A0......",
    "......0AA0......",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("pepinette_idle0", PEPINETTE_IDLE0),
    ("pepinette_idle1", PEPINETTE_IDLE1),
    ("pepinette_course0", PEPINETTE_COURSE0),
    ("pepinette_course1", PEPINETTE_COURSE1),
    ("pepinette_course2", PEPINETTE_COURSE2),
    ("pepinette_course3", PEPINETTE_COURSE3),
    ("pepinette_saut", PEPINETTE_SAUT),
    ("pepinette_chute", PEPINETTE_CHUTE),
    ("pepinette_cligne", PEPINETTE_CLIGNE),
    ("pepinette_ciel", PEPINETTE_CIEL),
    ("pepinette_tapote", PEPINETTE_TAPOTE),
    ("pepinette_mort0", PEPINETTE_MORT0),
    ("pepinette_mort1", PEPINETTE_MORT1),
    ("pepinette_mort2", PEPINETTE_MORT2),
    ("pepinette_plane0", PEPINETTE_PLANE0),
    ("pepinette_plane1", PEPINETTE_PLANE1),
    ("pepinette_retourne", PEPINETTE_RETOURNE),
    ("pepinette_barque", PEPINETTE_BARQUE),
    ("vie_pepinette", VIE_PEPINETTE),
];

/// Le jeu d'images de Pépinette, animation par animation (voir `skins`).
pub const IMAGES: JeuImages = JeuImages {
    repos: ["pepinette_idle0", "pepinette_idle1"],
    course: ["pepinette_course0", "pepinette_course1", "pepinette_course2", "pepinette_course3"],
    saut: "pepinette_saut",
    chute: "pepinette_chute",
    attente: ["pepinette_cligne", "pepinette_ciel", "pepinette_tapote"],
    planer: ["pepinette_plane0", "pepinette_plane1"],
    retourne: "pepinette_retourne",
    barque: "pepinette_barque",
    reapparition: ["pepinette_mort0", "pepinette_mort1", "pepinette_mort2"],
    vie: "vie_pepinette",
};
