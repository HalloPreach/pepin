//! Sprites des ennemis : flaques qui patrouillent (16x10), corbeaux (16x12) et
//! pigeons (16x12).

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

/// Pigeon perché (16x12, tourné vers la droite) : gris-bleu foncé (ardoise), cou
/// irisé (sarcelle et cyan), œil orange sous un sourcil sombre, deux barres noires
/// sur l'aile, pattes corail clair. Nettement plus gros et plus sombre que l'oiseau
/// de décor blanc (12x8, `objets_decor::OISEAU0`), qui est inoffensif.
pub const PIGEON0: Image = &[
    "..........000...",
    ".........02000..",
    ".........02A2100",
    ".........0EE200.",
    "......0000DEE20.",
    "....0034443EE220",
    "...0033444322220",
    "..01300300322220",
    ".011333333222220",
    "0110222222222220",
    "000.00000000000.",
    ".......0...0....",
];

/// Pigeon perché, 2e image : il hoche la tête (tête plus basse et en avant).
pub const PIGEON1: Image = &[
    "................",
    "..........000...",
    ".........02000..",
    "......00002A2100",
    "....00344DEE200.",
    "...003344EDE220.",
    "...0033444322220",
    "..01300300322220",
    ".011333333222220",
    "0110222222222220",
    "000.00000000000.",
    ".......0...0....",
];

/// Annonce (0,3 s avant la plongée) : ailes dressées, œil rouge corail ; la scène
/// le fait aussi trembler d'un pixel.
pub const PIGEON_ANNONCE: Image = &[
    ".0...0..........",
    "040.040..0000...",
    "0440444002000...",
    "04444444028210..",
    ".04443344EE2000.",
    "..0034443DEE220.",
    "...0033444322220",
    "..01300300322220",
    ".011333333222220",
    "0110222222222220",
    "000.00000000000.",
    ".......0...0....",
];

/// Plongée en diagonale : tête en avant vers le bas, ailes repliées vers l'arrière.
pub const PIGEON_PLONGEE: Image = &[
    "00..............",
    "010..000........",
    "0130044400......",
    ".013344440......",
    "..0133444400....",
    "..01333344300...",
    "...0133332220...",
    "....013322EEE0..",
    ".....02222DEE0..",
    "......0022EDE200",
    "........0022A210",
    "..........00000.",
];

/// Remontée vers le perchoir : ailes levées, corps incliné vers le haut. La scène
/// le retourne pour qu'il regarde son perchoir (il revient en arrière).
pub const PIGEON_REMONTEE: Image = &[
    ".....00....00...",
    "....0440..0440..",
    "....04440.04440.",
    ".....04440044440",
    "......0444034400",
    "......0033DEE200",
    ".....0333222EA10",
    "...0033322222000",
    "..01133222220...",
    ".0110222222220..",
    "0110.00000000...",
    "000.............",
];

/// Pigeon écrasé : aplati, plumes ébouriffées.
pub const PIGEON_ECRASE: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "..0..0.0..000...",
    ".040040400222000",
    "011333222DEE2A10",
    "0000000000000000",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("flaque0", FLAQUE0),
    ("flaque1", FLAQUE1),
    ("flaque2", FLAQUE2),
    ("flaque_ecrasee", FLAQUE_ECRASEE),
    ("corbeau0", CORBEAU0),
    ("corbeau1", CORBEAU1),
    ("pigeon0", PIGEON0),
    ("pigeon1", PIGEON1),
    ("pigeon_annonce", PIGEON_ANNONCE),
    ("pigeon_plongee", PIGEON_PLONGEE),
    ("pigeon_remontee", PIGEON_REMONTEE),
    ("pigeon_ecrase", PIGEON_ECRASE),
];

/// Images de la patrouille d'une flaque (3 images).
pub const FLAQUE: [&str; 3] = ["flaque0", "flaque1", "flaque2"];
/// Images d'un corbeau (battement d'ailes).
pub const CORBEAU: [&str; 2] = ["corbeau0", "corbeau1"];
/// Pigeon perché : il hoche la tête (2 images).
pub const PIGEON: [&str; 2] = ["pigeon0", "pigeon1"];
