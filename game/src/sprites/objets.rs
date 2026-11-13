//! Sprites des objets : gouttes d'or, lanternes, champignons, pointes, plateformes,
//! panneaux du tutoriel et drapeau d'arrivée.

use super::Image;

/// Goutte d'or 10x10 : la goutte de 8x8 (boîte de ramassage) cernée de 2 px de contour sombre
/// (le contour d'origine + un anneau), pour rester lisible devant tous les ciels, même dorés.
pub const GOUTTE0: Image = &[
    "....00....",
    "...0000...",
    "..00CB00..",
    "..00BB00..",
    ".00BCBB00.",
    "00BBCBBA00",
    "00BBBBAA00",
    ".00BAAA00.",
    "..000000..",
    "...0000...",
];

pub const GOUTTE1: Image = &[
    "....00....",
    "...0000...",
    "..00BB00..",
    "..00BB00..",
    ".00BBCB00.",
    "00BBBCBA00",
    "00BBBBAA00",
    ".00BAAA00.",
    "..000000..",
    "...0000...",
];

pub const GOUTTE2: Image = &[
    "....00....",
    "...0000.C.",
    "..00BB00..",
    "..00BB00..",
    ".00BBBB00.",
    "00BCBBBA00",
    "00BCBBAA00",
    ".00BAAA00.",
    "..000000..",
    "...0000...",
];

pub const GOUTTE3: Image = &[
    "....00....",
    ".C.0000...",
    "..00BB00..",
    "..00BB00..",
    ".00BBBB00.",
    "00BBBBBA00",
    "00BBBCAA00",
    ".00BACA00.",
    "..000000..",
    "...0000...",
];

/// Lanterne éteinte : vitre claire (ARDOISE_PALE/BRUME) pour se lire devant les immeubles du fond.
pub const LANTERNE0: Image = &[
    "...00...",
    "..0330..",
    ".033330.",
    ".045540.",
    ".045540.",
    ".045540.",
    ".033330.",
    "..0330..",
    "...00...",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    ".000000.",
];

pub const LANTERNE1: Image = &[
    "...00...",
    "..0330..",
    ".033330.",
    ".0BCCB0.",
    ".0BCCB0.",
    ".0BCCB0.",
    ".033330.",
    "..0330..",
    "...00...",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    "..0220..",
    ".000000.",
];

pub const CHAMPIGNON0: Image = &[
    "................",
    "................",
    "................",
    "....00000000....",
    "..0088889988800.",
    ".088988888888880",
    ".088888898888880",
    ".00000000000000.",
    "...00755570000..",
    ".....075570.....",
    ".....075570.....",
    "....00755700....",
];

pub const CHAMPIGNON1: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "....00000000....",
    "..0088889988800.",
    ".088988888888880",
    ".088888898888880",
    ".00000755570000.",
    "....00755700....",
];

pub const POINTE: Image = &[
    "...00...",
    "..0760..",
    "..0760..",
    ".076540.",
    ".076540.",
    "07765540",
    "07665540",
    "00000000",
];

pub const FRIABLE0: Image = &[
    "................",
    "..000..0000.....",
    ".04770076670....",
    "0477666666660...",
    "0566666666655000",
    ".055555555555550",
    "..05555555550...",
    "...0000000000...",
];

pub const FRIABLE1: Image = &[
    "................",
    "..000..0000.....",
    ".04770076670....",
    "0477060666660...",
    "0566606066655000",
    ".055050555505550",
    "..05500555000...",
    "...0000000000...",
];

/// Plateforme friable des niveaux 1 à 3 : dalle d'ardoise claire à angles écornés, avec une fine fêlure.
pub const FRIABLE_ARDOISE0: Image = &[
    ".00000000000000.",
    "0777777777777770",
    "0666666066666660",
    "0666660066666650",
    "0555550555555550",
    "0555555055555450",
    "0444444444444440",
    ".00000000000000.",
];

/// Dalle d'ardoise qui s'effrite : la fêlure s'ouvre de part en part.
pub const FRIABLE_ARDOISE1: Image = &[
    ".00000000000000.",
    "0777777007777770",
    "0666660000666660",
    "0666600..0066650",
    "0555500..0055550",
    "055500....005550",
    "0440000..0000440",
    ".00000....00000.",
];

/// Plateforme mobile 32x8 : le dessin couvre toute la boîte de collision (8 px de haut).
pub const MOBILE: Image = &[
    "00000000000000000000000000000000",
    "04444444444444444444444444444440",
    "03333333333333333333333333333330",
    "032222A222222222222222A222222230",
    "03222222222222222222222222222230",
    "02222222222222222222222222222220",
    "01111111111111111111111111111110",
    "00000000000000000000000000000000",
];

pub const PANNEAU_DEPLACER: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032222222230..",
    "..032722227230..",
    "..037777777730..",
    "..032722227230..",
    "..032222222230..",
    "..032222222230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_SAUTER: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032227722230..",
    "..032277772230..",
    "..032727727230..",
    "..032227722230..",
    "..032227722230..",
    "..032288882230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_PLANER: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032288882230..",
    "..032889988230..",
    "..038888888830..",
    "..032227722230..",
    "..032227722230..",
    "..032272272230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_CHAMPIGNON: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032227722230..",
    "..032277772230..",
    "..032227722230..",
    "..032888888230..",
    "..038899888830..",
    "..032275572230..",
    "..032222222230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const PANNEAU_VENT: Image = &[
    "..000000000000..",
    "..044444444440..",
    "..032D2222D230..",
    "..032D2DD2D230..",
    "..032D2D22D230..",
    "..032D2DD2D230..",
    "..032D22D2D230..",
    "..032D2DD2D230..",
    "..032227722230..",
    "..033333333330..",
    "..000000000000..",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    ".......33.......",
    "......0000......",
];

pub const DRAPEAU0: Image = &[
    ".0BB0...........",
    ".0BB0...........",
    ".00000000000....",
    "..54CBBBBBB0....",
    "..54CBBBBBBB0...",
    "..54CBBBBBBBB0..",
    "..54CBBBBBBBBB0.",
    "..54CBBBBBBBBBB0",
    "..54CBBBBBBBBB0.",
    "..54CBBBBBBBB0..",
    "..54CBBBBBBB0...",
    "..54CBBBBBB0....",
    "..54CBBBBB0.....",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "03333330........",
    "03333330........",
    "00000000........",
];

pub const DRAPEAU1: Image = &[
    ".0BB0...........",
    ".0BB0...........",
    ".0000000000.....",
    "..54CBBBBB0.....",
    "..54CBBBBBBBB0..",
    "..54CBBBBBBB0...",
    "..54CBBBBBBBBBB0",
    "..54CBBBBBBBBB0.",
    "..54CBBBBBBBBBB0",
    "..54CBBBBBBB0...",
    "..54CBBBBBBBB0..",
    "..54CBBBBB0.....",
    "..54CBBBBBB0....",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "..54............",
    "03333330........",
    "03333330........",
    "00000000........",
];

/// Grêlon 4x4 : une bille de glace de 2x2 px (reflet blanc cassé, nuage, ombre de
/// brume) cernée d'un contour sombre. Sur les ciels sombres la glace claire se
/// détache, sur les ciels dorés c'est le contour. Quatre images : le reflet fait le
/// tour de la bille (elle roule en tombant).
pub const GRELON0: Image = &[
    ".00.",
    "0760",
    "0650",
    ".00.",
];

pub const GRELON1: Image = &[
    ".00.",
    "0670",
    "0560",
    ".00.",
];

pub const GRELON2: Image = &[
    ".00.",
    "0560",
    "0670",
    ".00.",
];

pub const GRELON3: Image = &[
    ".00.",
    "0650",
    "0760",
    ".00.",
];

/// Éclat d'un grêlon qui se brise sur ce qui l'arrête (toit, parapluie ouvert) :
/// deux miettes de glace d'un pixel qui s'écartent, chacune cernée de sombre.
pub const GRELE_ECLAT: Image = &[
    ".0...0.",
    "070.070",
    ".0...0.",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("goutte0", GOUTTE0),
    ("goutte1", GOUTTE1),
    ("goutte2", GOUTTE2),
    ("goutte3", GOUTTE3),
    ("lanterne0", LANTERNE0),
    ("lanterne1", LANTERNE1),
    ("champignon0", CHAMPIGNON0),
    ("champignon1", CHAMPIGNON1),
    ("pointe", POINTE),
    ("friable0", FRIABLE0),
    ("friable1", FRIABLE1),
    ("friable_ardoise0", FRIABLE_ARDOISE0),
    ("friable_ardoise1", FRIABLE_ARDOISE1),
    ("mobile", MOBILE),
    ("panneau_deplacer", PANNEAU_DEPLACER),
    ("panneau_sauter", PANNEAU_SAUTER),
    ("panneau_planer", PANNEAU_PLANER),
    ("panneau_champignon", PANNEAU_CHAMPIGNON),
    ("panneau_vent", PANNEAU_VENT),
    ("drapeau0", DRAPEAU0),
    ("drapeau1", DRAPEAU1),
    ("grelon0", GRELON0),
    ("grelon1", GRELON1),
    ("grelon2", GRELON2),
    ("grelon3", GRELON3),
    ("grele_eclat", GRELE_ECLAT),
];

/// Goutte d'or : 4 images de brillance.
pub const GOUTTE: [&str; 4] = ["goutte0", "goutte1", "goutte2", "goutte3"];
/// Lanterne : éteinte puis allumée.
pub const LANTERNE: [&str; 2] = ["lanterne0", "lanterne1"];
/// Champignon-trampoline : détendu puis comprimé.
pub const CHAMPIGNON: [&str; 2] = ["champignon0", "champignon1"];
/// Plateforme friable (nuages, niveaux 4 et 5) : intacte puis fissurée.
pub const FRIABLE: [&str; 2] = ["friable0", "friable1"];
/// Plateforme friable (dalle d'ardoise, niveaux 1 à 3) : intacte puis fendue.
pub const FRIABLE_ARDOISE: [&str; 2] = ["friable_ardoise0", "friable_ardoise1"];
/// Drapeau d'arrivée (2 images).
pub const DRAPEAU: [&str; 2] = ["drapeau0", "drapeau1"];
/// Grêlon qui roule en tombant (4 images, le reflet fait le tour).
pub const GRELON: [&str; 4] = ["grelon0", "grelon1", "grelon2", "grelon3"];
