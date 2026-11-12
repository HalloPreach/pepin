//! Sprites des ennemis : flaques qui patrouillent (16x10) et corbeaux (16x12).

use super::Image;

pub const FLAQUE0: Image = &[
    "................",
    "................",
    "................",
    "....0000000.....",
    "...0D7DDDDD00...",
    "..0DD07DD07DDD0.",
    ".0DDD00DD00DDDD0",
    ".0EDDDDDDDDDDEE0",
    "..0EEEEEEEEEE00.",
    "...0000000000...",
];

pub const FLAQUE1: Image = &[
    "................",
    "................",
    ".....000000.....",
    "...0007DDDD000..",
    "..0DDDDDDDDDDD0.",
    ".0DDD07DDD07DDD0",
    ".0DDD00DDD00DDD0",
    ".0EEDDDDDDDDEEE0",
    "..0EEEEEEEEEEE0.",
    "...0000000000...",
];

pub const FLAQUE2: Image = &[
    "................",
    "................",
    "................",
    "....000000......",
    "...0D7DDDD000...",
    "..0DDDDDDDDDDD0.",
    ".0DDD07DD07DDDD0",
    ".0DDD00DD00DDDD0",
    ".0EEEDDDDDDEEEE0",
    "..000000000000..",
];

pub const FLAQUE_ECRASEE: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "..00000000000000",
    ".0D7DDDDDDDDDEE0",
    "0000000000000000",
];

pub const CORBEAU0: Image = &[
    "................",
    "................",
    "......000.......",
    "..0..03440......",
    ".0400344440.....",
    ".0440344440A0...",
    "..044444440AA0..",
    "...04444444000..",
    "....044444440...",
    ".....0044400....",
    "......0..0......",
    "................",
];

pub const CORBEAU1: Image = &[
    "................",
    "................",
    "................",
    "................",
    "......000.......",
    ".....03440......",
    "..00034444400...",
    ".0444444444A0...",
    "0444004444CAA0..",
    ".00000444440....",
    "......00440.....",
    ".......0..0.....",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("flaque0", FLAQUE0),
    ("flaque1", FLAQUE1),
    ("flaque2", FLAQUE2),
    ("flaque_ecrasee", FLAQUE_ECRASEE),
    ("corbeau0", CORBEAU0),
    ("corbeau1", CORBEAU1),
];

/// Images de la patrouille d'une flaque (3 images).
pub const FLAQUE: [&str; 3] = ["flaque0", "flaque1", "flaque2"];
/// Images d'un corbeau (battement d'ailes).
pub const CORBEAU: [&str; 2] = ["corbeau0", "corbeau1"];
