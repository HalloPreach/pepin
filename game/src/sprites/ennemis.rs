//! Sprites des ennemis : flaques qui patrouillent (16x10), corbeaux (18x14),
//! pigeons (18x14), chats endormis (16x14, avec leurs « z » et leur « ! »), et
//! l'escargot (16x12), qui n'est pas un ennemi mais patrouille comme eux ; le
//! Grand Nuage, le boss (48x32 : colère, essoufflé, touché), et ses points de vie
//! dans le HUD (10x6).

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

/// Corbeau en vol (18x14, de profil, tourné vers la droite) : aile HAUTE. Carte 6.1 :
/// l'ancien corbeau (16x12, deux ailes en V autour d'un corps rond) se lisait comme
/// une chauve-souris. Celui-ci ne se lit que d'un côté : gros bec gris clair (BRUME,
/// NUAGE) devant, œil orange juste derrière le bec, tête ronde au crâne éclairé,
/// queue en éventail derrière, et UNE aile qui bat depuis le dos, ses rémiges
/// écartées en « doigts » au bout. Plumage bleu nuit (`1`) ; reflets ARDOISE_CLAIRE
/// (`3`) sur le dos et ARDOISE_PALE (`4`) sur le bord d'attaque de l'aile et le
/// crâne : ce sont eux qui le détachent du ciel d'ardoise du grand orage (le bleu
/// nuit seul s'y perdrait). Contour sombre de 1 px tout autour.
pub const CORBEAU0: Image = &[
    "..0.0.0...........",
    ".0101040..........",
    ".01011340.........",
    "..011113400000....",
    "...011113343440...",
    "....011113411130..",
    ".00..0111341A1650.",
    "011003333111145550",
    "01111111111111140.",
    "0111111111111100..",
    "011001111111110...",
    ".00..011111110....",
    "......0000000.....",
    "..................",
];

/// Corbeau en vol : aile BASSE (elle descend sous le ventre, pointe vers l'arrière).
/// Le corps, la tête et le bec ne bougent pas : seule l'aile bat.
pub const CORBEAU1: Image = &[
    "..................",
    "..................",
    "..................",
    "...........000....",
    "..........03440...",
    ".........0111130..",
    ".00..0000011A1650.",
    "011003444444145550",
    "01111111333411140.",
    "0111111133411100..",
    "011001113411110...",
    ".0001113411110....",
    "...0113400000.....",
    "....0000..........",
];

/// Chauve-souris (carte 9.26b) : le corbeau du Monde 2 (grotte, égouts…), même
/// taille (18x14), même boîte, même battement en 2 images ; seul le dessin change.
/// De face : petites oreilles pointues, deux yeux rouges (`8`, CORAIL), corps
/// VIOLET_ORAGE (`F`) au ventre sombre (`1`), ailes membraneuses à pointes, leurs
/// doigts en bleu nuit (`1`) et un liseré ARDOISE_PALE (`4`) sur le bord d'attaque :
/// c'est lui qui la détache des fonds sombres de la grotte et des égouts. Contour
/// sombre de 1 px tout autour. Symétrique : le miroir du rendu ne change rien.
/// Image 0 : ailes HAUTES, pointes en l'air.
pub const CHAUVE_SOURIS0: Image = &[
    "00..............00",
    "040............040",
    "0F40..........04F0",
    "0FF40..0..0..04FF0",
    "01FF400F00F004FF10",
    "0F1FF40FFFF04FF1F0",
    "0F11F408FF804F11F0",
    "0FF1F40FFFF04F1FF0",
    "01F1FF0FFFF0FF1F10",
    "0F0F1F0FFFF0F1F0F0",
    ".0.0F00F11F00F0.0.",
    "....0.0F11F0.0....",
    ".......0FF0.......",
    "........00........",
];

/// Chauve-souris, image 1 : ailes BASSES, pointes vers le bas. La tête et le corps
/// (colonnes 6 à 11) ne bougent pas : seules les ailes battent.
pub const CHAUVE_SOURIS1: Image = &[
    "..................",
    "..................",
    "..................",
    ".......0..0.......",
    "......0F00F0......",
    "......0FFFF0......",
    "....0008FF8000....",
    "..00440FFFF04400..",
    ".044FF0FFFF0FF440.",
    "04F1F10FFFF01F1F40",
    "041FF10F11F01FF140",
    "040F1F0F11F0F1F040",
    "00.0F0.0FF0.0F0.00",
    "....0...00...0....",
];

/// Pigeon perché (18x14, tourné vers la droite). Carte 6.6 : l'ancien pigeon, une
/// masse d'ardoise sombre, ne se lisait pas (ni tête, ni queue) ; celui-ci est un
/// pigeon des villes. Corps rond et dodu gris-bleu MOYEN (`4`, ARDOISE_PALE), aile
/// repliée plus CLAIRE (`5`, BRUME) barrée de deux traits sombres (`1`), ventre et
/// dessous de la queue ombrés (`3`), queue courte au bout sombre ; petite tête
/// ronde ardoise (`3`) au front éclairé, œil orange (`A`), bec court sombre à la
/// cire claire ; cou irisé : sarcelle (`E`), cyan (`D`) et une touche violette
/// (`F`) ; pattes corail clair (`9`). Contour sombre de 1 px tout autour. Rien du
/// corbeau (bleu nuit presque noir, gros bec, queue en éventail) ni de l'oiseau
/// de décor (petit et blanc, `objets_decor::OISEAU0`), qui est inoffensif.
pub const PIGEON0: Image = &[
    "...........000....",
    "..........034300..",
    ".........0334A510.",
    ".........0333330..",
    ".......00EDE3300..",
    ".....00555DEF4440.",
    "....045555EF444440",
    "...045555554444440",
    ".00341111114444440",
    "01335555554444440.",
    "01331111114444440.",
    ".011003333333330..",
    "..00..009009000...",
    "........0..0......",
];

/// Pigeon perché, 2e image : il hoche la tête (tête plus basse et en avant) ; le
/// corps, l'aile et les pattes ne bougent pas.
pub const PIGEON1: Image = &[
    "..................",
    "............000...",
    "...........034300.",
    ".........00334A510",
    ".......00EDE33330.",
    ".....00555DEF4440.",
    "....045555EF444440",
    "...045555554444440",
    ".00341111114444440",
    "01335555554444440.",
    "01331111114444440.",
    ".011003333333330..",
    "..00..009009000...",
    "........0..0......",
];

/// Annonce (0,3 s avant la plongée) : il se ramasse, corps plus bas et tendu vers
/// l'avant, tête rentrée, ailes entrouvertes (les rémiges se dressent au-dessus du
/// dos), œil rouge corail (`8`) sous un sourcil sombre ; la scène le fait aussi
/// trembler d'un pixel.
pub const PIGEON_ANNONCE: Image = &[
    "..................",
    "..................",
    ".0.0.0......000...",
    "01010100...03330..",
    "01515155000033110.",
    ".01555555510333830",
    "..034555555EDE3350",
    "..0334555EDEF4440.",
    "..0334111111444440",
    "..0335555555444440",
    "...031111114444440",
    "....0333333333300.",
    ".....0009009000...",
    "........0..0......",
];

/// Plongée, 1re image : il s'élance, ailes grandes ouvertes en V, le corps déjà
/// couché sur la diagonale (tête en bas à droite, queue en haut à gauche).
pub const PIGEON_PLONGEE0: Image = &[
    "..................",
    ".0.0.0.........0..",
    "0101010......0010.",
    "0151510....001310.",
    ".0555500..013330..",
    "011055540043330...",
    "01331045544330....",
    ".01334445544330...",
    "..01344444EDE0....",
    "...0344444DE330...",
    "....03444EE33330..",
    ".....0344E3383300.",
    "......033333333550",
    ".......0000000000.",
];

/// Plongée, 2e image (le reste du piqué) : il fond sur Pépin le long de sa diagonale
/// à 45°, tête et bec devant (en bas à droite, œil rouge), ailes relevées et
/// repliées en flèche vers l'arrière, rémiges sombres en « doigts » au bout,
/// queue derrière (en haut à gauche).
pub const PIGEON_PLONGEE1: Image = &[
    "....0.0.0....0.0..",
    "...0101010..01010.",
    "...0151510.013130.",
    ".00.055550.03330..",
    "011004555504330...",
    "01331045554330....",
    ".01334445544330...",
    "..01344444EDE0....",
    "...0344444DE3300..",
    "....034444EE33330.",
    ".....03444E338330.",
    "......0333333330..",
    ".......0000033550.",
    "............0000..",
];

/// Remontée vers le perchoir, 1re image : ailes levées au-dessus du dos. La scène
/// le retourne pour qu'il regarde son perchoir (il revient en arrière).
pub const PIGEON_REMONTEE0: Image = &[
    "....0.0.0.........",
    "...0101010...000..",
    "...01515150.03430.",
    "....05555500334A50",
    "....04555510E3330.",
    ".....04555EDE300..",
    "....0044455DEF00..",
    "...03344444444440.",
    "..03344444444400..",
    ".013344444444440..",
    "011133334444440...",
    "01100333333330....",
    ".00..00000000.....",
    "..................",
];

/// Remontée, 2e image : ailes rabattues sous le ventre (battement d'ailes).
pub const PIGEON_REMONTEE1: Image = &[
    "..................",
    ".............000..",
    "............03430.",
    "...........0334A50",
    "..........00E3330.",
    ".......000EDE300..",
    "....0004444DEF00..",
    "...03344444444440.",
    "..03344444444400..",
    ".013344555544440..",
    "011130555555400...",
    "0110005555550.....",
    ".00...0151510.....",
    ".......00000......",
];

/// Pigeon écrasé : aplati, plumes ébouriffées qui dépassent du dos.
pub const PIGEON_ECRASE: Image = &[
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "...0..0.0...0.....",
    "..01005010.010....",
    ".0151051510151000.",
    "0134555555140333A0",
    "0133555555EEDE3350",
    "0133111111DEF44440",
    ".0333333333333330.",
    "..00000000000000..",
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

/// Prêt à bondir (pendant l'annonce, quand Pépin est à portée, carte 6.3) :
/// tête basse posée sur les pattes avant tendues, yeux d'or grand ouverts,
/// arrière-train levé, queue dressée au bout recourbé. Distinct de l'annonce
/// simple (`CHAT_REVEIL`, couché, un œil ouvert).
pub const CHAT_PRET: Image = &[
    ".............0..",
    "............080.",
    ".............080",
    "............0080",
    ".0.....0...0AAA0",
    "090...090.0AAAA0",
    "0A90009A00AA8AA0",
    "0AAAAAAA00AAAAA0",
    "0A0BA0BA0AAAA8A0",
    "0AAA8AAA0AAAAAA0",
    "077707770AA8AAA0",
    ".07777700AAAAAA0",
    "077AAAAA0AAAA770",
    ".0000000.000000.",
];

/// En plein bond (carte 6.3) : corps allongé, pattes avant tendues vers
/// l'avant, pattes arrière tendues vers l'arrière, queue en arrière.
pub const CHAT_BOND: Image = &[
    "................",
    ".0.....0......0.",
    "090...090....080",
    "0A90009A00000080",
    "0AAAAAAA0AAAA080",
    "0A0BA0BA08A8AA80",
    "0AAA8AAA0AAAA80.",
    "077707770AAAAA0.",
    ".0777770AAAAAA0.",
    "..000A000000A0..",
    "...0A0......0A0.",
    "..0A0........070",
    ".070..........0.",
    "..0.............",
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

/// Grand Nuage en colère (48x32) : sourcils froncés, yeux d'or, dents serrées.
/// Ventre violet d'orage (F), corps ardoise pâle (4), reflet nuage clair (6) sous le
/// contour du haut. Il gronde pendant le prélude et lance ses éclairs.
pub const NUAGE_COLERE0: Image = &[
    "...................0000000000...................",
    "................0006666666666000................",
    "...............066666666666446660...............",
    "..............06666444444444444460..............",
    ".............0664444444444444444460.............",
    ".............064444444444444444444600000........",
    "............0644444444444444444444406666000.....",
    "............06444444444444444444444444446660....",
    "......000000044444444444444444444444444444460...",
    "....00666666640044444444444444440044444444440...",
    "...0666666664400004444444444440000444444444460..",
    "..06664444444440000044444444000004444444444440..",
    "..04444444444444400004444440000444444444444440..",
    "..044444444444440000044444400000444444444444460.",
    ".0644444444444440BBB04444440BBB0444444444444440.",
    ".0444444444444440B000444444000B0444444444444440.",
    "04444444444444440B000444444000B04444444444444440",
    "044444444444444400000444444000004444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444000000000000444444444444444440",
    "044444444444444444077777777770444444444444444440",
    "044444444444444444070070070070444444444444444440",
    "044444444444444444000000000000444444444444444440",
    "044444444444444444444444444444444444444444444440",
    ".0444444444444444444444444444444444444444444440.",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFF0..",
    "...0FFFFFFFF0.0FFFFFFFF0.0FFFFFFFF0.0FFFFFFF0...",
    "....00FFFF00...00FFFF00...00FFFF00...00FFF00....",
    "......0000.......0000.......0000.......000......",
    "................................................",
];

/// En colère, 2e image : les sourcils descendent d'un pixel (il fronce).
pub const NUAGE_COLERE1: Image = &[
    "...................0000000000...................",
    "................0006666666666000................",
    "...............066666666666446660...............",
    "..............06666444444444444460..............",
    ".............0664444444444444444460.............",
    ".............064444444444444444444600000........",
    "............0644444444444444444444406666000.....",
    "............06444444444444444444444444446660....",
    "......000000044444444444444444444444444444460...",
    "....00666666644444444444444444444444444444440...",
    "...0666666664400444444444444444400444444444460..",
    "..06664444444400004444444444440000444444444440..",
    "..04444444444440000044444444000004444444444440..",
    "..044444444444440000044444400000444444444444460.",
    ".0644444444444440BBB04444440BBB0444444444444440.",
    ".0444444444444440B000444444000B0444444444444440.",
    "04444444444444440B000444444000B04444444444444440",
    "044444444444444400000444444000004444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444000000000000444444444444444440",
    "044444444444444444077777777770444444444444444440",
    "044444444444444444070070070070444444444444444440",
    "044444444444444444000000000000444444444444444440",
    "044444444444444444444444444444444444444444444440",
    ".0444444444444444444444444444444444444444444440.",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFF0..",
    "...0FFFFFFFF0.0FFFFFFFF0.0FFFFFFFF0.0FFFFFFF0...",
    "....00FFFF00...00FFFF00...00FFFF00...00FFF00....",
    "......0000.......0000.......0000.......000......",
    "................................................",
];

/// Essoufflé : paupières lourdes, sourcils tombants, bouche ouverte qui halète
/// (langue corail), goutte de sueur. C'est le moment de lui sauter sur la tête.
pub const NUAGE_ESSOUFFLE0: Image = &[
    "...................0000000000...................",
    "................0006666666666000................",
    "...............066666666666446660...............",
    "..............06666444444444444460..............",
    ".............0664444444444444444460.............",
    ".............064444444444444444444600000........",
    "............0644444444444444444444406666000.....",
    "............06444444444444444444444444446660....",
    "......000000044444444444444444444444444444460...",
    "....00666666644444444444444444444444444444440...",
    "...0666666664444444444444444444444444444444460..",
    "..06664444444444444444444444444444444444444440..",
    "..04444444444444400004444440000444444440444440..",
    "..0444444444444004444444444444400444440D0444460.",
    ".06444444444444444444444444444444444440D0444440.",
    ".0444444444444440000044444400000444444404444440.",
    "044444444444444407700444444007704444444444444440",
    "044444444444444440004444444400044444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444000000444444444444444444440",
    "044444444444444444440000000044444444444444444440",
    "044444444444444444440000000044444444444444444440",
    "044444444444444444440088880044444444444444444440",
    "044444444444444444444000000444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    ".0444444444444444444444444444444444444444444440.",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFF0..",
    "...0FFFFFFFF0.0FFFFFFFF0.0FFFFFFFF0.0FFFFFFF0...",
    "....00FFFF00...00FFFF00...00FFFF00...00FFF00....",
    "......0000.......0000.......0000.......000......",
    "................................................",
];

/// Essoufflé, 2e image : la bouche se referme un peu (il reprend son souffle).
pub const NUAGE_ESSOUFFLE1: Image = &[
    "...................0000000000...................",
    "................0006666666666000................",
    "...............066666666666446660...............",
    "..............06666444444444444460..............",
    ".............0664444444444444444460.............",
    ".............064444444444444444444600000........",
    "............0644444444444444444444406666000.....",
    "............06444444444444444444444444446660....",
    "......000000044444444444444444444444444444460...",
    "....00666666644444444444444444444444444444440...",
    "...0666666664444444444444444444444444444444460..",
    "..06664444444444444444444444444444444444444440..",
    "..04444444444444400004444440000444444440444440..",
    "..0444444444444004444444444444400444440D0444460.",
    ".06444444444444444444444444444444444440D0444440.",
    ".0444444444444440000044444400000444444404444440.",
    "044444444444444407700444444007704444444444444440",
    "044444444444444440004444444400044444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444000000444444444444444444440",
    "044444444444444444440000000044444444444444444440",
    "044444444444444444440088880044444444444444444440",
    "044444444444444444444000000444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    ".0444444444444444444444444444444444444444444440.",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFF0..",
    "...0FFFFFFFF0.0FFFFFFFF0.0FFFFFFFF0.0FFFFFFF0...",
    "....00FFFF00...00FFFF00...00FFFF00...00FFF00....",
    "......0000.......0000.......0000.......000......",
    "................................................",
];

/// Touché : yeux plissés « > < », bouche en vague. La scène le fait clignoter.
pub const NUAGE_TOUCHE: Image = &[
    "...................0000000000...................",
    "................0006666666666000................",
    "...............066666666666446660...............",
    "..............06666444444444444460..............",
    ".............0664444444444444444460.............",
    ".............064444444444444444444600000........",
    "............0644444444444444444444406666000.....",
    "............06444444444444444444444444446660....",
    "......000000044444444444444444444444444444460...",
    "....00666666644444444444444444444444444444440...",
    "...0666666664444444444444444444444444444444460..",
    "..06664444444444444444444444444444444444444440..",
    "..04444444444444444444444444444444444444444440..",
    "..044444444444440444444444444440444444444444460.",
    ".0644444444444444004444444444004444444444444440.",
    ".0444444444444444440044444400444444444444444440.",
    "044444444444444440044444444440044444444444444440",
    "044444444444444404444444444444404444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444400440044004444444444444444440",
    "044444444444444444044004400440444444444444444440",
    "044444444444444444444444444444444444444444444440",
    "044444444444444444444444444444444444444444444440",
    ".0444444444444444444444444444444444444444444440.",
    ".0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0.",
    "..0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFFF0FFFFFFFFF0..",
    "...0FFFFFFFF0.0FFFFFFFF0.0FFFFFFFF0.0FFFFFFF0...",
    "....00FFFF00...00FFFF00...00FFFF00...00FFF00....",
    "......0000.......0000.......0000.......000......",
    "................................................",
];

/// Point de vie du Grand Nuage dans le HUD (10x6) : un petit nuage plein.
pub const PV_NUAGE: Image = &[
    "...0000...",
    ".00666600.",
    "0664444460",
    "0444444440",
    "0FFFFFFFF0",
    ".00000000.",
];

/// Point de vie perdu (10x6) : le même petit nuage, éteint (ardoise sombre).
pub const PV_NUAGE_PERDU: Image = &[
    "...0000...",
    ".00222200.",
    "0222222220",
    "0222222220",
    "0111111110",
    ".00000000.",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("flaque0", FLAQUE0),
    ("flaque1", FLAQUE1),
    ("flaque2", FLAQUE2),
    ("flaque_ecrasee", FLAQUE_ECRASEE),
    ("corbeau0", CORBEAU0),
    ("corbeau1", CORBEAU1),
    ("chauve_souris0", CHAUVE_SOURIS0),
    ("chauve_souris1", CHAUVE_SOURIS1),
    ("pigeon0", PIGEON0),
    ("pigeon1", PIGEON1),
    ("pigeon_annonce", PIGEON_ANNONCE),
    ("pigeon_plongee0", PIGEON_PLONGEE0),
    ("pigeon_plongee1", PIGEON_PLONGEE1),
    ("pigeon_remontee0", PIGEON_REMONTEE0),
    ("pigeon_remontee1", PIGEON_REMONTEE1),
    ("pigeon_ecrase", PIGEON_ECRASE),
    ("escargot0", ESCARGOT0),
    ("escargot1", ESCARGOT1),
    ("escargot_rebond", ESCARGOT_REBOND),
    ("chat_dort0", CHAT_DORT0),
    ("chat_dort1", CHAT_DORT1),
    ("chat_reveil", CHAT_REVEIL),
    ("chat_eveille", CHAT_EVEILLE),
    ("chat_pret", CHAT_PRET),
    ("chat_bond", CHAT_BOND),
    ("chat_ecrase", CHAT_ECRASE),
    ("chat_z", CHAT_Z),
    ("chat_alerte", CHAT_ALERTE),
    ("nuage_colere0", NUAGE_COLERE0),
    ("nuage_colere1", NUAGE_COLERE1),
    ("nuage_essouffle0", NUAGE_ESSOUFFLE0),
    ("nuage_essouffle1", NUAGE_ESSOUFFLE1),
    ("nuage_touche", NUAGE_TOUCHE),
    ("pv_nuage", PV_NUAGE),
    ("pv_nuage_perdu", PV_NUAGE_PERDU),
];

/// Images de la patrouille d'une flaque (3 images).
pub const FLAQUE: [&str; 3] = ["flaque0", "flaque1", "flaque2"];
/// Images d'un corbeau (battement d'ailes).
pub const CORBEAU: [&str; 2] = ["corbeau0", "corbeau1"];
/// Le corbeau du Monde 2 dessiné en chauve-souris (battement d'ailes, carte 9.26b).
pub const CHAUVE_SOURIS: [&str; 2] = ["chauve_souris0", "chauve_souris1"];
/// Pigeon perché : il hoche la tête (2 images).
pub const PIGEON: [&str; 2] = ["pigeon0", "pigeon1"];
/// Pigeon en piqué : il s'élance ailes ouvertes, puis les relève en flèche (2 images).
pub const PIGEON_PLONGEE: [&str; 2] = ["pigeon_plongee0", "pigeon_plongee1"];
/// Pigeon qui remonte à son perchoir : il bat des ailes (2 images).
pub const PIGEON_REMONTEE: [&str; 2] = ["pigeon_remontee0", "pigeon_remontee1"];
/// Escargot qui marche (2 images).
pub const ESCARGOT: [&str; 2] = ["escargot0", "escargot1"];
/// Chat endormi : il respire (2 images).
pub const CHAT_DORT: [&str; 2] = ["chat_dort0", "chat_dort1"];
/// Grand Nuage en colère : il fronce les sourcils (2 images).
pub const NUAGE_COLERE: [&str; 2] = ["nuage_colere0", "nuage_colere1"];
/// Grand Nuage essoufflé : il halète (2 images).
pub const NUAGE_ESSOUFFLE: [&str; 2] = ["nuage_essouffle0", "nuage_essouffle1"];
