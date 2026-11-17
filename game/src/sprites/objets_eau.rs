//! Sprites des mécaniques d'eau du Monde 2 (carte 9.3) : bulles, chutes d'eau,
//! nénuphar, stalactite, graduation et flèches de la marée. Même règle que les
//! autres fichiers de sprites : 1 caractère = 1 pixel, `.` transparent, chiffres
//! et lettres = indices de la palette (D cyan de la pluie, E cyan sombre, 7 blanc
//! cassé, B or, 0 ombre de contour, 2 à 5 ardoises).

use super::Image;

/// Bulle 5x5 (lettre `Y`) : un anneau de cyan clair, un reflet blanc cassé.
pub const BULLE0: Image = &[
    ".DDD.",
    "D7..D",
    "D...D",
    "D...D",
    ".DDD.",
];

/// Petite bulle 3x3.
pub const BULLE1: Image = &[
    ".D.",
    "D7D",
    ".D.",
];

/// Chute d'eau (lettre `|`), une case 16x16 : des filets de cyan clair sur le cyan
/// sombre, un reflet blanc cassé. Quatre images : le motif descend de 4 px à chaque
/// image (il boucle sur ses 16 rangées), l'eau coule.
pub const CHUTE0: Image = &[
    "EDDEEEDEEEEDDEEE",
    "EDDEEEDEEEEDDEEE",
    "EDEEEEDDEEEEDEEE",
    "EDEEE7DDEEEEDEEE",
    "EEEEEEDDEEEEDDEE",
    "EEDEEEEDEEE7DDEE",
    "EEDEEEEDEEEEEDEE",
    "EEDDEEEEEEEEEDEE",
    "EEDDEEEDEEEEEEEE",
    "EEEDEEEDEEEDEEEE",
    "E7EDEEEDDEEDEEEE",
    "EEEEEEEEDEEDDEEE",
    "EDEEEEEEDEEEDEEE",
    "EDEEEEEEEEEEDEEE",
    "EDDEEE7EEEEEEEEE",
    "EDDEEEDEEEEDDEEE",
];

pub const CHUTE1: Image = &[
    "EDEEEEEEDEEEDEEE",
    "EDEEEEEEEEEEDEEE",
    "EDDEEE7EEEEEEEEE",
    "EDDEEEDEEEEDDEEE",
    "EDDEEEDEEEEDDEEE",
    "EDDEEEDEEEEDDEEE",
    "EDEEEEDDEEEEDEEE",
    "EDEEE7DDEEEEDEEE",
    "EEEEEEDDEEEEDDEE",
    "EEDEEEEDEEE7DDEE",
    "EEDEEEEDEEEEEDEE",
    "EEDDEEEEEEEEEDEE",
    "EEDDEEEDEEEEEEEE",
    "EEEDEEEDEEEDEEEE",
    "E7EDEEEDDEEDEEEE",
    "EEEEEEEEDEEDDEEE",
];

pub const CHUTE2: Image = &[
    "EEDDEEEDEEEEEEEE",
    "EEEDEEEDEEEDEEEE",
    "E7EDEEEDDEEDEEEE",
    "EEEEEEEEDEEDDEEE",
    "EDEEEEEEDEEEDEEE",
    "EDEEEEEEEEEEDEEE",
    "EDDEEE7EEEEEEEEE",
    "EDDEEEDEEEEDDEEE",
    "EDDEEEDEEEEDDEEE",
    "EDDEEEDEEEEDDEEE",
    "EDEEEEDDEEEEDEEE",
    "EDEEE7DDEEEEDEEE",
    "EEEEEEDDEEEEDDEE",
    "EEDEEEEDEEE7DDEE",
    "EEDEEEEDEEEEEDEE",
    "EEDDEEEEEEEEEDEE",
];

pub const CHUTE3: Image = &[
    "EEEEEEDDEEEEDDEE",
    "EEDEEEEDEEE7DDEE",
    "EEDEEEEDEEEEEDEE",
    "EEDDEEEEEEEEEDEE",
    "EEDDEEEDEEEEEEEE",
    "EEEDEEEDEEEDEEEE",
    "E7EDEEEDDEEDEEEE",
    "EEEEEEEEDEEDDEEE",
    "EDEEEEEEDEEEDEEE",
    "EDEEEEEEEEEEDEEE",
    "EDDEEE7EEEEEEEEE",
    "EDDEEEDEEEEDDEEE",
    "EDDEEEDEEEEDDEEE",
    "EDDEEEDEEEEDDEEE",
    "EDEEEEDDEEEEDEEE",
    "EDEEE7DDEEEEDEEE",
];

/// Écume au pied d'une chute d'eau (16x4), deux images qui alternent.
pub const ECUME0: Image = &[
    "..7..7D..7...7..",
    ".7D77DD77D7.7D7.",
    "7DDDDDDDDDDDDDD7",
    "DDEDDDEDDDEDDDED",
];

pub const ECUME1: Image = &[
    ".7...7..7D..7...",
    "7D7.7D77DD77D77.",
    "7DDDDDDDDDDDDDD7",
    "DEDDDEDDDEDDDEDD",
];

/// Nénuphar (lettre `n`) : une feuille 16x4 vue de profil, échancrée, cernée de
/// sombre (cyan clair et cyan sombre : la palette n'a pas de vert). Seules ses
/// rangées au-dessus de l'eau se voient (la surface est redessinée par-dessus).
pub const NENUPHAR: Image = &[
    ".000000..000000.",
    "0DDDDDD00DDDEDD0",
    "0EDDDEDDDDDDDDE0",
    ".00000000000000.",
];

/// Stalactite (lettre `V`), 10x14 : une pointe de pierre (ardoises) accrochée au
/// plafond, cernée de sombre.
pub const STALACTITE: Image = &[
    "0000000000",
    "0344554320",
    "0344443320",
    ".03444320.",
    ".03443320.",
    ".0344320..",
    "..034320..",
    "..034320..",
    "..03420...",
    "...0420...",
    "...0320...",
    "...030....",
    "....00....",
    "....0.....",
];

/// Éclats d'une stalactite brisée au sol (16x4).
pub const STALACTITE_ECLATS: Image = &[
    "..0.....0...0...",
    ".040...030.040..",
    "03440.04430.0430",
    "0000000000000000",
];

/// Graduation de la marée (lettre `=`), 4x16 par case : une règle au mur, un
/// trait long par case, un moyen par demi-case, un court par quart.
pub const GRADUATION: Image = &[
    "0CCC",
    "0...",
    "0...",
    "0...",
    "0C..",
    "0...",
    "0...",
    "0...",
    "0CC.",
    "0...",
    "0...",
    "0...",
    "0C..",
    "0...",
    "0...",
    "0...",
];

/// Flèche de la marée (7x7, or) : l'eau va monter (clignote pendant l'annonce).
pub const MAREE_MONTE: Image = &[
    "...0...",
    "..0B0..",
    ".0BBB0.",
    "0BBBBB0",
    "000B000",
    "..0B0..",
    "..000..",
];

/// La même flèche vers le bas : l'eau va descendre.
pub const MAREE_DESCEND: Image = &[
    "..000..",
    "..0B0..",
    "000B000",
    "0BBBBB0",
    ".0BBB0.",
    "..0B0..",
    "...0...",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("bulle0", BULLE0),
    ("bulle1", BULLE1),
    ("chute0", CHUTE0),
    ("chute1", CHUTE1),
    ("chute2", CHUTE2),
    ("chute3", CHUTE3),
    ("ecume0", ECUME0),
    ("ecume1", ECUME1),
    ("nenuphar", NENUPHAR),
    ("stalactite", STALACTITE),
    ("stalactite_eclats", STALACTITE_ECLATS),
    ("graduation", GRADUATION),
    ("maree_monte", MAREE_MONTE),
    ("maree_descend", MAREE_DESCEND),
];

/// Bulles : la grosse, puis la petite.
pub const BULLES: [&str; 2] = ["bulle0", "bulle1"];
/// Chute d'eau : 4 images (le motif descend).
pub const CHUTE: [&str; 4] = ["chute0", "chute1", "chute2", "chute3"];
/// Écume au pied d'une chute d'eau : 2 images.
pub const ECUME: [&str; 2] = ["ecume0", "ecume1"];
