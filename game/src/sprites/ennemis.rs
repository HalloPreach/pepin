//! Sprites des ennemis : flaques qui patrouillent (16x10), corbeaux (16x12),
//! pigeons (16x12), chats endormis (16x14, avec leurs « z » et leur « ! »), et
//! l'escargot (16x12), qui n'est pas un ennemi mais patrouille comme eux.

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

/// Escargot (16x12, tourné vers la gauche) : « un champignon qui marche ». Sa
/// coquille reprend le chapeau du champignon (corail, reflet corail clair, pois
/// blancs) : on devine tout de suite qu'il fait rebondir. Corps bleu-gris pâle
/// (nuage, ventre brume), œil sombre et joue rose ; deux cornes sombres à
/// boule. 1re image de la marche : cornes dressées.
pub const ESCARGOT0: Image = &[
    ".00.00..........",
    ".00.00...00000..",
    "..0..0..0998880.",
    ".00000.098878880",
    "0666660888878880",
    "0606660888888780",
    "0669660878888880",
    "0666660888878880",
    "0666666000000000",
    "0666666666666660",
    "0555555555555550",
    ".00000000000000.",
];

/// Escargot, 2e image de la marche : les boules des cornes se penchent vers
/// l'arrière d'un pixel (seules les deux rangées du haut changent).
pub const ESCARGOT1: Image = &[
    "..00.00.........",
    "..00.00..00000..",
    "..0..0..0998880.",
    ".00000.098878880",
    "0666660888878880",
    "0606660888888780",
    "0669660878888880",
    "0666660888878880",
    "0666666000000000",
    "0666666666666660",
    "0555555555555550",
    ".00000000000000.",
];

/// Rebond : la coquille s'aplatit sous le poids de Pépin (comme le chapeau du
/// champignon), les cornes rentrées, l'œil fermé ; il reste mignon.
pub const ESCARGOT_REBOND: Image = &[
    "................",
    "................",
    "................",
    "................",
    ".........00000..",
    ".00000..0998880.",
    "0666660098878880",
    "0600960888888780",
    "0666666000000000",
    "0666666666666660",
    "0555555555555550",
    ".00000000000000.",
];

/// Chat endormi (16x14, tourné vers la gauche) : roulé en boule, tête posée sur
/// ses pattes blanches, yeux fermés, la queue rayée enroulée sous lui. Pelage
/// orange (A), rayures et queue corail (8), oreilles roses (9), museau et
/// pattes blancs (7). Les rangées du haut restent vides : l'image éveillée,
/// assise, est plus haute. 1re image : il inspire (le dos se soulève).
pub const CHAT_DORT0: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    ".0.....0........",
    "090...090.......",
    "0A90009A0.0000..",
    "0AAAAAAA00AAAA0.",
    "0A00A00A0AAAAAA0",
    "0AAA8AAA0AAAAAA0",
    "077777770AAAAAA0",
    ".0777770888888A0",
    "..00000000000000",
];

/// Chat endormi, 2e image : il expire (le dos s'abaisse d'un pixel).
pub const CHAT_DORT1: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    ".0.....0........",
    "090...090.......",
    "0A90009A0.......",
    "0AAAAAAA0.0000..",
    "0A00A00A00AAAA0.",
    "0AAA8AAA0AAAAAA0",
    "077777770AAAAAA0",
    ".0777770888888A0",
    "..00000000000000",
];

/// Réveil (l'annonce) : il lève la tête, oreilles dressées, un œil ouvert (or).
pub const CHAT_REVEIL: Image = &[
    "................",
    "................",
    "................",
    ".0.....0........",
    "090...090.......",
    "0A90009A0.......",
    "0AAAAAAA0.......",
    "0A0BA00A0.0000..",
    "0AAA8AAA00AAAA0.",
    "077777770AAAAAA0",
    ".0777770AAAAAAA0",
    ".0A777A0AAAAAAA0",
    ".0777770888888A0",
    "..00000000000000",
];

/// Éveillé : assis, dos rond, poil hérissé, queue dressée, grands yeux d'or :
/// le toucher tue (même par le dessus).
pub const CHAT_EVEILLE: Image = &[
    ".0.....0......0.",
    "090...090....0A0",
    "0A90009A0....0A0",
    "0AAAAAAA0.00.0A0",
    "0A0BA0BA00AA00A0",
    "0AAA8AAA0AAAA0A0",
    "077707770AAAAA80",
    ".0777770AAAAAAA0",
    "..07770AAAAAAAA0",
    "..07770AAAAAAA80",
    "..07A70AAAAAAA0.",
    "..07770AAAAAAA0.",
    "..0707088888880.",
    "...0.0000000000.",
];

/// Écrasé : aplati comme une crêpe, oreilles rentrées dans la tête. Il reste
/// mignon : yeux plissés en « ^ ^ », petit nez rose, museau blanc.
pub const CHAT_ECRASE: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    ".0.....0........",
    "090000090000000.",
    "0A0AAA0A0AAAAAA0",
    "00A080A00AAAAAA0",
    "0777777708888880",
    ".00000000000000.",
];

/// « z » du sommeil (6x6) : blanc cerné de sombre, lisible sur tous les ciels.
/// La scène en fait monter trois au-dessus du chat endormi, l'un après l'autre.
pub const CHAT_Z: Image = &[
    ".0000.",
    "077770",
    ".0070.",
    ".0700.",
    "077770",
    ".0000.",
];

/// « ! » de l'annonce du réveil (4x8) : or cerné de sombre ; la scène le fait
/// clignoter de plus en plus vite, comme l'annonce d'un éclair.
pub const CHAT_ALERTE: Image = &[
    "0000",
    "0BB0",
    "0BB0",
    "0BB0",
    "0000",
    "0BB0",
    "0000",
    "....",
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
    ("escargot0", ESCARGOT0),
    ("escargot1", ESCARGOT1),
    ("escargot_rebond", ESCARGOT_REBOND),
    ("chat_dort0", CHAT_DORT0),
    ("chat_dort1", CHAT_DORT1),
    ("chat_reveil", CHAT_REVEIL),
    ("chat_eveille", CHAT_EVEILLE),
    ("chat_ecrase", CHAT_ECRASE),
    ("chat_z", CHAT_Z),
    ("chat_alerte", CHAT_ALERTE),
];

/// Images de la patrouille d'une flaque (3 images).
pub const FLAQUE: [&str; 3] = ["flaque0", "flaque1", "flaque2"];
/// Images d'un corbeau (battement d'ailes).
pub const CORBEAU: [&str; 2] = ["corbeau0", "corbeau1"];
/// Pigeon perché : il hoche la tête (2 images).
pub const PIGEON: [&str; 2] = ["pigeon0", "pigeon1"];
/// Escargot qui marche (2 images).
pub const ESCARGOT: [&str; 2] = ["escargot0", "escargot1"];
/// Chat endormi : il respire (2 images).
pub const CHAT_DORT: [&str; 2] = ["chat_dort0", "chat_dort1"];
