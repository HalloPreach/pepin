//! Tuiles de décor 16x16 : toits d'ardoise, briques de cheminée, nuages.
//! Chaque matière a une version « haut » (arête éclairée) et une version « plein ».
//! La brique pleine a en plus trois variantes (fissure, brique manquante, coulure), et une bande
//! d'ombre se pose sous le rebord des toits.

use super::Image;

pub const TOIT_HAUT: Image = &[
    "0000000000000000",
    "4444444444444444",
    "3344333333443333",
    "1111111111111111",
    "3333133333331333",
    "2222132222221322",
    "2222132222221322",
    "1111111111111111",
    "1333333313333333",
    "1322222213222222",
    "1322222213222222",
    "1111111111111111",
    "3333133333331333",
    "2222132222221322",
    "2222132222221322",
    "1111111111111111",
];

pub const BRIQUE_HAUT: Image = &[
    "0000000000000000",
    "4444444444444444",
    "3333333333333333",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
];

pub const NUAGE_HAUT: Image = &[
    "..77..7...777...",
    ".7667767.76667..",
    "7666666676667677",
    "6666666666666666",
    "6766666666666666",
    "6666666766666666",
    "6666666666666766",
    "6666666666666666",
    "6676666666666666",
    "6666666676666666",
    "6666666666666676",
    "6666666666666666",
    "5567556655665566",
    "5665566557655665",
    "5555555555555555",
    "5555555555555555",
];

/// Bords de plateforme de nuage (arête éclairée) : coins arrondis, choisis quand la case voisine est vide.
pub const NUAGE_BORD_G: Image = &[
    "...7..7...777...",
    "..667767.76667..",
    ".666666676667677",
    "6666666666666666",
    "6766666666666666",
    "6666666766666666",
    "6666666666666766",
    "6666666666666666",
    "6676666666666666",
    "6666666676666666",
    "6666666666666676",
    "6666666666666666",
    ".567556655665566",
    ".665566557655665",
    "..55555555555555",
    "...5555555555555",
];

pub const NUAGE_BORD_D: Image = &[
    "..77..7...777...",
    ".7667767.76667..",
    "766666667666767.",
    "6666666666666666",
    "6766666666666666",
    "6666666766666666",
    "6666666666666766",
    "6666666666666666",
    "6676666666666666",
    "6666666676666666",
    "6666666666666676",
    "6666666666666666",
    "556755665566556.",
    "566556655765566.",
    "55555555555555..",
    "5555555555555...",
];

/// Nuage d'une seule case (voisins vides des deux côtés).
pub const NUAGE_ILOT: Image = &[
    "...7..7...777...",
    "..667767.76667..",
    ".66666667666767.",
    "6666666666666666",
    "6766666666666666",
    "6666666766666666",
    "6666666666666766",
    "6666666666666666",
    "6676666666666666",
    "6666666676666666",
    "6666666666666676",
    "6666666666666666",
    ".56755665566556.",
    ".66556655765566.",
    "..555555555555..",
    "...5555555555...",
];

pub const TOIT_PLEIN: Image = &[
    "1333333313333333",
    "1322222213222222",
    "1322222213222222",
    "1111111111111111",
    "3333133333331333",
    "2222132222221322",
    "2222132222221322",
    "1111111111111111",
    "1333333313333333",
    "1322222213222222",
    "1322222213222222",
    "1111111111111111",
    "3333133333331333",
    "2222132222221322",
    "2222132222221322",
    "1111111111111111",
];

pub const BRIQUE_PLEIN: Image = &[
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
];

pub const NUAGE_PLEIN: Image = &[
    "7666666666666666",
    "6666667666666666",
    "6666666666667666",
    "6666666666666666",
    "6766666666666666",
    "6666666766666666",
    "6666666666666766",
    "6666666666666666",
    "6676666666666666",
    "6666666676666666",
    "6666666666666676",
    "6666666666666666",
    "5567556655665566",
    "5665566557655665",
    "5555555555555555",
    "5555555555555555",
];

// --- Variantes de brique pleine ---
// Même taille (16 x 16) et mêmes couleurs que `BRIQUE_PLEIN` : seule une petite zone change, le
// reste de l'appareil (joints, assises) est recopié à l'identique pour que les raccords avec les
// tuiles voisines restent parfaits. La scène en pose environ une tuile sur six, à des places
// fixes (voir `variante_brique` dans `scene.rs`).

/// Fissure : un trait sombre (1 = ombre de nuit) qui descend en biais à travers deux assises.
pub const BRIQUE_FISSURE: Image = &[
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333233333132333",
    "FFFF2FFFFF1F2FFF",
    "FFFF2FFFF1FF2FFF",
    "2222222221222222",
    "2333333313333333",
    "2FFFFFF12FFFFFFF",
    "2FFFFF1F2FFFFFFF",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
];

/// Brique manquante : le trou laisse voir l'ombre du mur (0 en haut : ombre portée par la brique
/// du dessus ; 1 au fond).
pub const BRIQUE_MANQUANTE: Image = &[
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333200000002333",
    "FFFF211111112FFF",
    "FFFF211111112FFF",
    "2222222222222222",
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
];

/// Coulure : l'eau de pluie suinte d'un joint et laisse une traînée humide plus sombre (2 =
/// ardoise, comme les joints) qui s'affine en descendant et finit par une goutte (1). Pas de cyan :
/// le cyan reste réservé à la pluie et aux flaques, qu'on ne doit pas confondre avec le mur.
pub const BRIQUE_COULURE: Image = &[
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222221122222222",
    "3333232233332333",
    "FFFF2F22FFFF2FFF",
    "FFFF2F2FFFFF2FFF",
    "2222221222222222",
    "2333332323333333",
    "2FFFFF2F2FFFFFFF",
    "2FFFFF1F2FFFFFFF",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
];

/// Indice d'un faux mur (carte 8.5) : la brique du milieu a glissé d'un pixel vers la droite ; le
/// jour qu'elle laisse à gauche montre l'ombre derrière le mur (0 en haut, 1 dessous). Le reste est
/// la brique pleine : seul un œil attentif voit le joint décalé.
pub const BRIQUE_DECALEE: Image = &[
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333203333333233",
    "FFFF21FFFFFFF2FF",
    "FFFF21FFFFFFF2FF",
    "2222222222222222",
    "2333333323333333",
    "2FFFFFFF2FFFFFFF",
    "2FFFFFFF2FFFFFFF",
    "2222222222222222",
    "3333233333332333",
    "FFFF2FFFFFFF2FFF",
    "FFFF2FFFFFFF2FFF",
    "2222222222222222",
];

/// Indice d'un faux mur de brique, en plus de `BRIQUE_DECALEE` : le bout d'une brique qui dépasse
/// de 3 px de la façade, posé sur la case libre VOISINE (à gauche de la façade : `_G` ; à droite :
/// `_D`). Les variantes de brique tirées au hasard restent dans leur case : seul l'indice sort du
/// mur.
pub const BRIQUE_SAILLIE_G: Image = &[
    "................",
    "................",
    "................",
    "................",
    ".............333",
    ".............FFF",
    ".............FFF",
    ".............222",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

/// Même chose, la façade à droite de la case libre : le bout de brique sort vers la gauche.
pub const BRIQUE_SAILLIE_D: Image = &[
    "................",
    "................",
    "................",
    "................",
    "333.............",
    "FFF.............",
    "FFF.............",
    "222.............",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

/// Indice d'un faux mur de nuage (carte 8.5) : un petit creux d'ombre (5) dans le nuage plein,
/// comme si le nuage était moins épais à cet endroit.
pub const NUAGE_CREUX: Image = &[
    "7666666666666666",
    "6666667666666666",
    "6666666666667666",
    "6666666666666666",
    "6766666666666666",
    "6666655566666666",
    "6666555556666766",
    "6666555556666666",
    "6676655566666666",
    "6666666676666666",
    "6666666666666676",
    "6666666666666666",
    "5567556655665566",
    "5665566557655665",
    "5555555555555555",
    "5555555555555555",
];

/// Ombre sous le rebord d'un toit : bande de 2 px posée PAR-DESSUS le haut de la brique qui est
/// juste sous une tuile de toit. 0 (ombre la plus profonde) contre le rebord, puis 1 (ombre de
/// nuit) : deux niveaux d'ombre, sans dégradé. Elle donne l'épaisseur de l'avancée du toit.
pub const OMBRE_TOIT: Image = &[
    "0000000000000000",
    "1111111111111111",
];

// --- Eau (carte 9.2, redessinée à la carte 9.6b) ---
// Trois bandes nettes, sans dégradé : la LIGNE de surface claire (D, 1 px), la BANDE
// sous la surface (E, 6 px, quelques reflets D) et le FOND plus sombre (2, l'ardoise,
// quelques traits E) : l'eau tranche sur le sol, même quand il est sombre. Chaque
// matière d'eau a trois dessins : « haut » (le dessus de l'étendue), « vague » (le
// même, 1 px plus bas : `nom_tuile` alterne les deux pour que la surface ondule d'un
// pixel) et « plein » (l'intérieur). Le SENS d'un courant ne se dessine plus dans la
// tuile (les anciens chevrons « > » se lisaient comme du texte) : de courtes stries
// défilent par-dessus, dans le sens du courant (`crate::dessin_courant`).

/// Eau, dessus de l'étendue : la ligne claire, la bande, le fond.
pub const EAU_CALME_HAUT: Image = &[
    "DDDDDDDDDDDDDDDD",
    "EEEEEEEEEEEEEEEE",
    "EEEDDDEEEEEEEEEE",
    "EEEEEEEEEEEDDDEE",
    "EEEEEEEEEEEEEEEE",
    "EEEEEEEDDEEEEEEE",
    "EEEEEEEEEEEEEEEE",
    "2222222222222222",
    "2222222222222222",
    "22222222EE222222",
    "2222222222222222",
    "2EE2222222222222",
    "2222222222222222",
    "2222222222EE2222",
    "2222222222222222",
    "2222222222222222",
];

/// Eau, dessus, l'autre temps de la vague : la même surface 1 px plus bas.
pub const EAU_CALME_VAGUE: Image = &[
    "................",
    "DDDDDDDDDDDDDDDD",
    "EEEEEEEEEEEEEEEE",
    "EEEDDDEEEEEEEEEE",
    "EEEEEEEEEEEDDDEE",
    "EEEEEEEEEEEEEEEE",
    "EEEEEEEDDEEEEEEE",
    "EEEEEEEEEEEEEEEE",
    "2222222222222222",
    "2222222222222222",
    "22222222EE222222",
    "2222222222222222",
    "2EE2222222222222",
    "2222222222222222",
    "2222222222EE2222",
    "2222222222222222",
];

/// Eau, intérieur de l'étendue : le fond sombre, quelques traits de reflet.
pub const EAU_CALME_PLEIN: Image = &[
    "2222222222222222",
    "222222EE22222222",
    "2222222222222222",
    "2222222222222222",
    "2222222222222EE2",
    "2222222222222222",
    "2222222222222222",
    "2EE2222222222222",
    "2222222222222222",
    "2222222222222222",
    "222222222EE22222",
    "2222222222222222",
    "2222222222222222",
    "22222EE222222222",
    "2222222222222222",
    "2222222222222222",
];

// Les courants ont les mêmes dessins que l'eau calme (les stries animées disent le
// sens) ; leurs noms restent, pour que le choix de `nom_tuile` ne change pas.
/// Courant vers la gauche (`{`), dessus.
pub const EAU_COURANT_G_HAUT: Image = EAU_CALME_HAUT;
/// Courant vers la gauche, dessus, l'autre temps de la vague.
pub const EAU_COURANT_G_VAGUE: Image = EAU_CALME_VAGUE;
/// Courant vers la gauche, intérieur.
pub const EAU_COURANT_G_PLEIN: Image = EAU_CALME_PLEIN;
/// Courant vers la droite (`}`), dessus.
pub const EAU_COURANT_D_HAUT: Image = EAU_CALME_HAUT;
/// Courant vers la droite, dessus, l'autre temps de la vague.
pub const EAU_COURANT_D_VAGUE: Image = EAU_CALME_VAGUE;
/// Courant vers la droite, intérieur.
pub const EAU_COURANT_D_PLEIN: Image = EAU_CALME_PLEIN;

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("toit_haut", TOIT_HAUT),
    ("brique_haut", BRIQUE_HAUT),
    ("nuage_haut", NUAGE_HAUT),
    ("nuage_bord_g", NUAGE_BORD_G),
    ("nuage_bord_d", NUAGE_BORD_D),
    ("nuage_ilot", NUAGE_ILOT),
    ("toit_plein", TOIT_PLEIN),
    ("brique_plein", BRIQUE_PLEIN),
    ("nuage_plein", NUAGE_PLEIN),
    ("brique_fissure", BRIQUE_FISSURE),
    ("brique_manquante", BRIQUE_MANQUANTE),
    ("brique_coulure", BRIQUE_COULURE),
    ("brique_decalee", BRIQUE_DECALEE),
    ("nuage_creux", NUAGE_CREUX),
    ("brique_saillie_g", BRIQUE_SAILLIE_G),
    ("brique_saillie_d", BRIQUE_SAILLIE_D),
    ("ombre_toit", OMBRE_TOIT),
    ("eau_calme_haut", EAU_CALME_HAUT),
    ("eau_calme_vague", EAU_CALME_VAGUE),
    ("eau_calme_plein", EAU_CALME_PLEIN),
    ("eau_courant_g_haut", EAU_COURANT_G_HAUT),
    ("eau_courant_g_vague", EAU_COURANT_G_VAGUE),
    ("eau_courant_g_plein", EAU_COURANT_G_PLEIN),
    ("eau_courant_d_haut", EAU_COURANT_D_HAUT),
    ("eau_courant_d_vague", EAU_COURANT_D_VAGUE),
    ("eau_courant_d_plein", EAU_COURANT_D_PLEIN),
];
