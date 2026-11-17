//! Sprites des ennemis d'eau du Monde 2 (carte 9.4) : grenouille (16x12),
//! poisson (12x16) et ses remous (16x6), crabe (18x12), rat (18x14) et
//! mouette (18x14, le pigeon du port : mêmes images que le pigeon, en blanc et
//! gris, bec jaune, pattes orange). Le crabe et la grenouille regardent à
//! gauche, comme le chat ; le rat et la mouette à droite, comme le pigeon.

use super::Image;

/// Grenouille (16x12, tournée vers la gauche) qui guette : corps cyan sombre,
/// reflets cyan, ventre or pâle, oeil blanc au-dessus de la tête. 1re image.
pub const GRENOUILLE0: Image = &[
    "................",
    "...000..........",
    "..07770.........",
    "..00770.0000....",
    ".0EEEE00EEEE00..",
    "0EEDEEEEEEEEEE0.",
    "0EDDEEEEEEEEDE0.",
    "00000EEEEEEDDE0.",
    ".0CCCCEEEEDDDDE0",
    "..0CCCEEEDEEEEE0",
    ".0EE0CEEDDDDDE0.",
    ".0000.000000000.",
];

/// Grenouille qui guette, 2e image : la gorge se gonfle (le ventre avance d'un pixel).
pub const GRENOUILLE1: Image = &[
    "................",
    "...000..........",
    "..07770.........",
    "..00770.0000....",
    ".0EEEE00EEEE00..",
    "0EEDEEEEEEEEEE0.",
    "0EDDEEEEEEEEDE0.",
    "0000CEEEEEEDDE0.",
    "0CCCCCEEEEDDDDE0",
    ".0CCCCEEEDEEEEE0",
    ".0EE0CEEDDDDDE0.",
    ".0000.000000000.",
];

/// Annonce de la grenouille : elle se ramasse (2 px plus bas, plus large) avant
/// de bondir.
pub const GRENOUILLE_ANNONCE: Image = &[
    "................",
    "................",
    "...000..........",
    "..07770.........",
    "..00770.0000....",
    ".0EEEE00EEEE00..",
    "0EEDEEEEEEEEEE0.",
    "0EDDEEEEEEEDDEE0",
    "00000EEEEEDDDDE0",
    ".0CCCCCEEDDDDDE0",
    "0EE0CCCEEEEEEEE0",
    "0000000000000000",
];

/// Bond de la grenouille : pattes avant tendues, pattes arrière allongées.
pub const GRENOUILLE_SAUT: Image = &[
    "................",
    "...000..........",
    "..07770.........",
    "..00770000......",
    ".0EEEEEEEE0.....",
    "0EEDEEEEEEE0....",
    "00000EEEEEEE0...",
    ".0CCCCCEEEEEE0..",
    "..0CCCCEEDDEEE0.",
    ".0EE00000DDEEEE0",
    "0EE0.....0DDDE0.",
    ".00.......0000..",
];

/// Grenouille écrasée : aplatie, les yeux encore visibles.
pub const GRENOUILLE_ECRASEE: Image = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "..00.00.........",
    ".0700700000000..",
    "0EEEEEEEEEDDDEE0",
    ".00000000000000.",
];

/// Poisson (12x16) qui monte hors de l'eau, la tête en haut : orange, reflets
/// or, ventre or pâle, nageoires corail. 1re image.
pub const POISSON0: Image = &[
    "....0000....",
    "...0AAAA0...",
    "..0AAAAAA0..",
    "..0A70AABA0.",
    "..0A00AABA0.",
    ".80AAAAABA08",
    "880CAAAABA88",
    ".80CCAAAAA08",
    "..0CCAAAAA0.",
    "..0CAAAABA0.",
    "...0CAAAA0..",
    "....0AAA0...",
    ".....0A0....",
    "....08980...",
    "...0889880..",
    "...0000000..",
];

/// Poisson qui monte, 2e image : la queue bat d'un pixel.
pub const POISSON1: Image = &[
    "....0000....",
    "...0AAAA0...",
    "..0AAAAAA0..",
    "..0A70AABA0.",
    "..0A00AABA0.",
    ".80AAAAABA08",
    "880CAAAABA88",
    ".80CCAAAAA08",
    "..0CCAAAAA0.",
    "..0CAAAABA0.",
    "...0CAAAA0..",
    "....0AAA0...",
    ".....0A0....",
    ".....08980..",
    "....0889880.",
    "....0000000.",
];

/// Poisson qui retombe, la tête en bas (la 1re image retournée haut-bas).
pub const POISSON_CHUTE: Image = &[
    "...0000000..",
    "...0889880..",
    "....08980...",
    ".....0A0....",
    "....0AAA0...",
    "...0CAAAA0..",
    "..0CAAAABA0.",
    "..0CCAAAAA0.",
    ".80CCAAAAA08",
    "880CAAAABA88",
    ".80AAAAABA08",
    "..0A00AABA0.",
    "..0A70AABA0.",
    "..0AAAAAA0..",
    "...0AAAA0...",
    "....0000....",
];

/// Remous (16x3) à la surface, au-dessus du poisson caché : l'annonce de son saut.
/// 1re image, petits cercles.
pub const REMOUS0: Image = &[
    ".....7....7.....",
    "..7.D7DDDD7D.7..",
    ".7DD........DD7.",
];

/// Remous, 2e image (fin de l'annonce) : les cercles s'élargissent.
pub const REMOUS1: Image = &[
    "...7..7..7..7...",
    ".7D7DD7DD7DD7D7.",
    "7DD..........DD7",
];

/// Crabe (18x12, tourné vers la gauche) : carapace corail, une pince LEVÉE (la
/// carapace dure), yeux sur pédoncules. Marche, 1re image.
pub const CRABE0: Image = &[
    ".00.00............",
    "0990990...........",
    "0999990...........",
    ".09990..0..0......",
    "..080..070070.....",
    "..080...0..0......",
    "...0800000000000..",
    "....0889999999980.",
    "....08888888888090",
    "....00888888880990",
    ".....080808080000.",
    ".....00.00.00.00..",
];

/// Crabe, marche, 2e image : les pattes alternent.
pub const CRABE1: Image = &[
    ".00.00............",
    "0990990...........",
    "0999990...........",
    ".09990..0..0......",
    "..080..070070.....",
    "..080...0..0......",
    "...0800000000000..",
    "....0889999999980.",
    "....08888888888090",
    "....00888888880990",
    ".....0080808080000",
    "......00.00.00.00.",
];

/// Crabe en alerte (Pépin au-dessus de lui) : les deux pinces levées.
pub const CRABE_ALERTE: Image = &[
    ".00.00......00.00.",
    "0990990....0990990",
    "0999990....0999990",
    ".09990..0..009990.",
    "..080..070070080..",
    "..080...0..0.080..",
    "...0800000000080..",
    "....0889999999980.",
    "....088888888880..",
    "....008888888800..",
    ".....080808080000.",
    ".....00.00.00.00..",
];

/// Crabe qui claque des pinces, juste après un rebond de Pépin sur sa carapace.
pub const CRABE_CLAQUE: Image = &[
    "..000.......000...",
    ".09990.....09990..",
    ".09990.....09990..",
    "..080...0..0.080..",
    "..080..070070080..",
    "..080...0..0.080..",
    "...0800000000080..",
    "....0889999999980.",
    "....088888888880..",
    "....008888888800..",
    ".....080808080000.",
    ".....00.00.00.00..",
];

/// Rat (18x14, tourné vers la droite) qui patrouille : gris, ventre brume,
/// oreille, museau et queue corail clair. 1re image.
pub const RAT0: Image = &[
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "............00....",
    "...........0990...",
    "......000000990...",
    "....004444444440..",
    "...0444444440700..",
    "...045555555444490",
    "9990455555554440..",
    "9..00555555544400.",
    "....0440..0440....",
];

/// Rat qui patrouille, 2e image : les pattes et la queue bougent.
pub const RAT1: Image = &[
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "............00....",
    "...........0990...",
    "......000000990...",
    "....004444444440..",
    "...0444444440700..",
    "...045555555444490",
    "9990455555554440..",
    ".9.00555555544400.",
    ".....0440..0440...",
];

/// Annonce du rat : il se dresse sur ses pattes arrière avant de foncer.
pub const RAT_ANNONCE: Image = &[
    "..................",
    "..........00......",
    ".........0990.....",
    ".........0990.....",
    "........0444400...",
    "........04440700..",
    "........044444490.",
    ".......0044444400.",
    "......0904555540..",
    ".......004555540..",
    ".......045555540..",
    "99.....045555540..",
    "9.9.0004555554400.",
    "...0..00440.0440..",
];

/// Rat qui fonce : allongé, oreille couchée, traits de vitesse. 1re image.
pub const RAT_FONCE0: Image = &[
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..............00..",
    "............00990.",
    ".....00000000990..",
    "..5.0444444444070.",
    "5..045555555444490",
    "..0455555555444400",
    "990045555555440...",
    "...00440...004400.",
    "....00......00....",
];

/// Rat qui fonce, 2e image.
pub const RAT_FONCE1: Image = &[
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..............00..",
    "............00990.",
    ".....00000000990..",
    "5..0444444444070..",
    "..5045555555444490",
    "5.0455555555444400",
    "990045555555440...",
    "....0044..0440....",
    ".....00....00.....",
];

/// Rat étourdi contre le mur : étoiles d'or, yeux en croix.
pub const RAT_ETOURDI: Image = &[
    "..................",
    "..................",
    "..................",
    ".........B...B....",
    "........BCB.BCB...",
    ".........B.0.B....",
    "...........0990...",
    "......000000990...",
    "....004444444440..",
    "...0444444440F0F..",
    "...0455555544F4490",
    "9990455555554440..",
    "9..00555555544400.",
    "....0440..0440....",
];

/// Rat écrasé : aplati.
pub const RAT_ECRASE: Image = &[
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "............0.0...",
    ".....0000000099000",
    "999044555555544490",
    "9..00000000000000.",
];

/// Mouette : image0 du pigeon, recolorée.
pub const MOUETTE0: Image = &[
    "...........000....",
    "..........067600..",
    ".........06670BA0.",
    ".........0666660..",
    ".......007776600..",
    ".....005557767770.",
    "....07555576777770",
    "...075555557777770",
    ".00673333337777770",
    "03665555557777770.",
    "03663333337777770.",
    ".033006666666660..",
    "..00..00A00A000...",
    "........0..0......",
];

/// Mouette : image1 du pigeon, recolorée.
pub const MOUETTE1: Image = &[
    "..................",
    "............000...",
    "...........067600.",
    ".........006670BA0",
    ".......0077766660.",
    ".....005557767770.",
    "....07555576777770",
    "...075555557777770",
    ".00673333337777770",
    "03665555557777770.",
    "03663333337777770.",
    ".033006666666660..",
    "..00..00A00A000...",
    "........0..0......",
];

/// Mouette : image_annonce du pigeon, recolorée.
pub const MOUETTE_ANNONCE: Image = &[
    "..................",
    "..................",
    ".0.0.0......000...",
    "03030300...06660..",
    "03535355000066330.",
    ".03555555530666A60",
    "..0675555557776650",
    "..066755577767770.",
    "..0667333333777770",
    "..0665555555777770",
    "...063333337777770",
    "....0666666666600.",
    ".....000A00A000...",
    "........0..0......",
];

/// Mouette : image_plongee0 du pigeon, recolorée.
pub const MOUETTE_PLONGEE0: Image = &[
    "..................",
    ".0.0.0.........0..",
    "0303030......0030.",
    "0353530....003630.",
    ".0555500..036660..",
    "033055570076660...",
    "03663075577660....",
    ".03667775577660...",
    "..036777777770....",
    "...067777777660...",
    "....067777766660..",
    ".....0677766A6600.",
    "......066666666550",
    ".......0000000000.",
];

/// Mouette : image_plongee1 du pigeon, recolorée.
pub const MOUETTE_PLONGEE1: Image = &[
    "....0.0.0....0.0..",
    "...0303030..03030.",
    "...0353530.036360.",
    ".00.055550.06660..",
    "033007555507660...",
    "03663075557660....",
    ".03667775577660...",
    "..036777777770....",
    "...0677777776600..",
    "....0677777766660.",
    ".....06777766A660.",
    "......0666666660..",
    ".......0000066550.",
    "............0000..",
];

/// Mouette : image_remontee0 du pigeon, recolorée.
pub const MOUETTE_REMONTEE0: Image = &[
    "....0.0.0.........",
    "...0303030...000..",
    "...03535350.06760.",
    "....055555006670B0",
    "....0755553076660.",
    ".....07555777600..",
    "....007775577600..",
    "...06677777777770.",
    "..06677777777700..",
    ".036677777777770..",
    "033366667777770...",
    "03300666666660....",
    ".00..00000000.....",
    "..................",
];

/// Mouette : image_remontee1 du pigeon, recolorée.
pub const MOUETTE_REMONTEE1: Image = &[
    "..................",
    ".............000..",
    "............06760.",
    "...........06670B0",
    "..........0076660.",
    ".......000777600..",
    "....000777777600..",
    "...06677777777770.",
    "..06677777777700..",
    ".036677555577770..",
    "033360555555700...",
    "0330005555550.....",
    ".00...0353530.....",
    ".......00000......",
];

/// Mouette : image_ecrasee du pigeon, recolorée.
pub const MOUETTE_ECRASEE: Image = &[
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "..................",
    "...0..0.0...0.....",
    "..03005030.030....",
    ".0353053530353000.",
    "036755555537066600",
    "036655555577776650",
    "036633333377677770",
    ".0666666666666660.",
    "..00000000000000..",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("grenouille0", GRENOUILLE0),
    ("grenouille1", GRENOUILLE1),
    ("grenouille_annonce", GRENOUILLE_ANNONCE),
    ("grenouille_saut", GRENOUILLE_SAUT),
    ("grenouille_ecrasee", GRENOUILLE_ECRASEE),
    ("poisson0", POISSON0),
    ("poisson1", POISSON1),
    ("poisson_chute", POISSON_CHUTE),
    ("remous0", REMOUS0),
    ("remous1", REMOUS1),
    ("crabe0", CRABE0),
    ("crabe1", CRABE1),
    ("crabe_alerte", CRABE_ALERTE),
    ("crabe_claque", CRABE_CLAQUE),
    ("rat0", RAT0),
    ("rat1", RAT1),
    ("rat_annonce", RAT_ANNONCE),
    ("rat_fonce0", RAT_FONCE0),
    ("rat_fonce1", RAT_FONCE1),
    ("rat_etourdi", RAT_ETOURDI),
    ("rat_ecrase", RAT_ECRASE),
    ("mouette0", MOUETTE0),
    ("mouette1", MOUETTE1),
    ("mouette_annonce", MOUETTE_ANNONCE),
    ("mouette_plongee0", MOUETTE_PLONGEE0),
    ("mouette_plongee1", MOUETTE_PLONGEE1),
    ("mouette_remontee0", MOUETTE_REMONTEE0),
    ("mouette_remontee1", MOUETTE_REMONTEE1),
    ("mouette_ecrasee", MOUETTE_ECRASEE),
];
