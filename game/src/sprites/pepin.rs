//! Sprites de Pépin : petite créature ronde avec un grand parapluie corail.
//!
//! 1 caractère = 1 pixel : `.` transparent, `0`-`9` `A`-`F` = indice dans
//! `palette::TOUTES` (voir `sprites/mod.rs`). Pépin fait 16x16 parapluie fermé
//! et 20x20 parapluie ouvert (planer). Boîte de collision : 10x14.
//!
//! Parapluie fermé (colonnes 11 à 15, rangées 0 à 12, le même dans toutes les
//! images où il est fermé, pour qu'il ne tremble pas pendant la course), tenu
//! par la poignée (carte 7.1b : il était à l'envers, crosse en l'air) :
//! - embout de métal (`5`) en haut ;
//! - toile corail repliée (`8`, plis en corail clair `9`) serrée par une sangle
//!   blanche (`7`), fine sous l'embout et plus large en bas, vers la poignée,
//!   comme un vrai parapluie roulé ;
//! - crosse orange (`A`) en bas, à hauteur du flanc, recourbée vers Pépin : la
//!   poignée qu'il tient.
//!
//! Seul le corps de Pépin bouge d'une image à l'autre (respiration, course) : le
//! parapluie reste à sa place, sauf dans `PEPIN_TAPOTE` où il descend d'un pixel
//! (petit coup de crosse vers le sol).

use super::{Image, JeuImages};

pub const PEPIN_IDLE0: Image = &[
    "............000.",
    "............050.",
    ".....0000...080.",
    "....077770..0980",
    "...0777777008880",
    "..07777777707770",
    "..07770770508980",
    "..07770770509880",
    "..077977955000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0555555550AAA0",
    "...05555550.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPIN_IDLE1: Image = &[
    "............000.",
    "............050.",
    "............080.",
    "....000000..0980",
    "...0777777008880",
    "..07777777707770",
    "..07770770708980",
    "..07770770509880",
    "..077977955000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0555555550AAA0",
    "...05555550.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPIN_COURSE0: Image = &[
    "............000.",
    "............050.",
    ".....0000...080.",
    "....077770..0980",
    "...0777777008880",
    "..07777777707770",
    "..07777075008980",
    "..07777075009880",
    "..077797795000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0555555550AAA0",
    "...05555550.000.",
    "....000000......",
    "..0AA0..0AA0....",
    "...00....00.....",
];

pub const PEPIN_COURSE1: Image = &[
    "............000.",
    ".....0000...050.",
    "....077770..080.",
    "...07777770.0980",
    "..07777777708880",
    "..07777075007770",
    "..07777075008980",
    "..07779779509880",
    "..077777755000A0",
    "..077777755000A0",
    "..0555555550A0A0",
    "...055555500AAA0",
    "....000000..000.",
    "...0AA00AA0.....",
    "....00..00......",
    "................",
];

pub const PEPIN_COURSE2: Image = &[
    "............000.",
    "............050.",
    ".....0000...080.",
    "....077770..0980",
    "...0777777008880",
    "..07777777707770",
    "..07777075008980",
    "..07777075009880",
    "..077797795000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0555555550AAA0",
    "...05555550.000.",
    "....000000......",
    "...00..0AA0.....",
    "........00......",
];

pub const PEPIN_COURSE3: Image = &[
    "............000.",
    ".....0000...050.",
    "....077770..080.",
    "...07777770.0980",
    "..07777777708880",
    "..07777075007770",
    "..07777075008980",
    "..07779779509880",
    "..077777755000A0",
    "..077777755000A0",
    "..0555555550A0A0",
    "...055555500AAA0",
    "....000000..000.",
    "...0AA00AA0.....",
    "....00..00......",
    "................",
];

pub const PEPIN_SAUT: Image = &[
    "............000.",
    ".....0000...050.",
    "....077770..080.",
    "...07777770.0980",
    "..07770770708880",
    "..07770770507770",
    "..07797795508980",
    "..07777775509880",
    "..077777755000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "...055555500AAA0",
    "....055550..000.",
    "....000000......",
    "....00..00......",
    "................",
];

pub const PEPIN_CHUTE: Image = &[
    "............000.",
    "............050.",
    ".....0000...080.",
    "....077770..0980",
    "...0777777008880",
    "..07777777707770",
    "..07777775508980",
    "..07770770509880",
    "..077707705000A0",
    "..077977955000A0",
    "..0777777550A0A0",
    "..0555555550AAA0",
    "...05550550.000.",
    "....000000......",
    "..0AA0..0AA0....",
    "...00....00.....",
];

// --- Attente : après 2 s immobile au sol (voir `scene::image_attente`) ---
// Mêmes pieds et même silhouette que `PEPIN_IDLE0` : on passe de l'une à l'autre
// sans que Pépin saute. Seuls changent les yeux ou la place du parapluie.

/// Pépin cligne des yeux : chaque œil (2 px de haut) devient un trait de 2 px.
pub const PEPIN_CLIGNE: Image = &[
    "............000.",
    "............050.",
    ".....0000...080.",
    "....077770..0980",
    "...0777777008880",
    "..07777777707770",
    "..07777777508980",
    "..07700700509880",
    "..077977955000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0555555550AAA0",
    "...05555550.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

/// Pépin lève les yeux vers le ciel (va-t-il pleuvoir ?) : yeux et joues remontés d'un pixel.
pub const PEPIN_CIEL: Image = &[
    "............000.",
    "............050.",
    ".....0000...080.",
    "....077770..0980",
    "...0777777008880",
    "..07770770707770",
    "..07770770508980",
    "..07797797509880",
    "..077777755000A0",
    "..077777755000A0",
    "..0777777550A0A0",
    "..0555555550AAA0",
    "...05555550.000.",
    "....000000......",
    "...0AA00AA0.....",
    "....00..00......",
];

/// Pépin tapote le sol avec son parapluie : le parapluie descend d'un pixel et l'embout
/// touche le sol (en alternance avec `PEPIN_IDLE0`, ça fait « tac, tac »).
pub const PEPIN_TAPOTE: Image = &[
    "................",
    "............000.",
    ".....0000...050.",
    "....077770..080.",
    "...07777770.0980",
    "..07777777708880",
    "..07770770507770",
    "..07770770508980",
    "..07797795509880",
    "..077777755000A0",
    "..077777755000A0",
    "..0555555550A0A0",
    "...055555500AAA0",
    "....000000..000.",
    "...0AA00AA0.....",
    "....00..00......",
];

pub const PEPIN_MORT0: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    ".......C........",
    "................",
    "..B..........B..",
    "................",
    "..0000000000....",
    ".077777777770...",
    ".077707770770...",
    ".055505550550...",
    "..0090009000....",
    "................",
];

pub const PEPIN_MORT1: Image = &[
    "................",
    "................",
    "...B........B...",
    ".......B........",
    "................",
    ".......C........",
    ".B............B.",
    "....C......C....",
    "................",
    "................",
    "....000000......",
    "...07707700.....",
    "...05505500.....",
    "....090090......",
    "................",
    "................",
];

pub const PEPIN_MORT2: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    ".......B........",
    "....B......B....",
    "........C.......",
    "................",
    "..B..........B..",
    "...C........C...",
    "................",
    ".....B....B.....",
    "......C..C......",
    "................",
    "......6666......",
];

pub const PEPIN_PLANE0: Image = &[
    "......00000000......",
    "....099998888880....",
    "...09999988888880...",
    "..0999098808808880..",
    ".099990988088088880.",
    "09999909880880888880",
    "08008008008008008008",
    ".0..0..0..0..0..0..0",
    ".........00.........",
    ".........00.........",
    "......00000000......",
    ".....0777777770.....",
    ".....0770770770.....",
    ".....0770770770.....",
    ".....0797797550.....",
    ".....0777777550.....",
    "......05555550......",
    ".......000000.......",
    "....................",
    ".......00..00.......",
];

pub const PEPIN_PLANE1: Image = &[
    "......00000000......",
    "....099998888880....",
    "...09999988888880...",
    "..0999098808808880..",
    ".099990988088088880.",
    "09999909880880888880",
    "80080080080080080080",
    "0..0..0..0..0..0..0.",
    ".........00.........",
    ".........00.........",
    "....................",
    "......00000000......",
    ".....0777777770.....",
    ".....0770770770.....",
    ".....0770770770.....",
    ".....0797797550.....",
    ".....0777777550.....",
    "......05555550......",
    ".......000000.......",
    ".......00..00.......",
];

/// Barque (carte 9.2) : le parapluie corail RETOURNÉ devient une coque, et Pépin
/// s'assied dedans : la tête (yeux, joues) dépasse du bord, le bord clair (9) est
/// festonné comme le bord de la toile, deux baleines (9) descendent vers le fond,
/// et la crosse (A) sort dessous comme une quille. Posée comme une image 16x16, le
/// coin haut-gauche à (milieu-8, bas-16) : le bas de la coque est à la surface de
/// l'eau, la crosse dessous, à hauteur des pieds (3 px sous la surface).
pub const PEPIN_BARQUE: Image = &[
    "................",
    "................",
    ".....000000.....",
    "....07777770....",
    "...0777777770...",
    "...0770770770...",
    "...0797777970...",
    ".00077777777000.",
    "0999099999909990",
    "0888888888888880",
    ".08898888889880.",
    "..088988889880..",
    "...0088998800...",
    ".....000A000....",
    ".......0A0......",
    "......0AA0......",
];

/// Icône des vies dans le HUD (10x10, carte 6.2) : la tête de Pépin, corps blanc,
/// yeux, joues corail et ombre de brume, comme le vrai, cernée de sombre pour se lire
/// sur tous les ciels.
pub const VIE: Image = &[
    "..000000..",
    ".07777770.",
    "0777777770",
    "0777077050",
    "0777077050",
    "0779779550",
    "0777777550",
    "0555555550",
    ".05555550.",
    "..000000..",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("pepin_idle0", PEPIN_IDLE0),
    ("pepin_idle1", PEPIN_IDLE1),
    ("pepin_course0", PEPIN_COURSE0),
    ("pepin_course1", PEPIN_COURSE1),
    ("pepin_course2", PEPIN_COURSE2),
    ("pepin_course3", PEPIN_COURSE3),
    ("pepin_saut", PEPIN_SAUT),
    ("pepin_chute", PEPIN_CHUTE),
    ("pepin_cligne", PEPIN_CLIGNE),
    ("pepin_ciel", PEPIN_CIEL),
    ("pepin_tapote", PEPIN_TAPOTE),
    ("pepin_mort0", PEPIN_MORT0),
    ("pepin_mort1", PEPIN_MORT1),
    ("pepin_mort2", PEPIN_MORT2),
    ("pepin_plane0", PEPIN_PLANE0),
    ("pepin_plane1", PEPIN_PLANE1),
    ("pepin_barque", PEPIN_BARQUE),
    ("vie", VIE),
];

/// Images de la course (4 images en boucle).
pub const COURSE: [&str; 4] = ["pepin_course0", "pepin_course1", "pepin_course2", "pepin_course3"];
/// Images du repos (respiration).
pub const REPOS: [&str; 2] = ["pepin_idle0", "pepin_idle1"];
/// Images de l'attente (après 2 s immobile) : cligner, regarder le ciel, tapoter le sol.
pub const ATTENTE: [&str; 3] = ["pepin_cligne", "pepin_ciel", "pepin_tapote"];
/// Images du parapluie ouvert (les franges oscillent).
pub const PLANER: [&str; 2] = ["pepin_plane0", "pepin_plane1"];
/// Images de la réapparition (étincelles qui se rassemblent puis Pépin).
pub const REAPPARITION: [&str; 3] = ["pepin_mort0", "pepin_mort1", "pepin_mort2"];

/// Le jeu d'images de Pépin, animation par animation (voir `skins`).
pub const IMAGES: JeuImages = JeuImages {
    repos: REPOS,
    course: COURSE,
    saut: "pepin_saut",
    chute: "pepin_chute",
    attente: ATTENTE,
    planer: PLANER,
    barque: "pepin_barque",
    reapparition: REAPPARITION,
    vie: "vie",
};
