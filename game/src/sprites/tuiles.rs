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

/// Ombre sous le rebord d'un toit : bande de 2 px posée PAR-DESSUS le haut de la brique qui est
/// juste sous une tuile de toit. 0 (ombre la plus profonde) contre le rebord, puis 1 (ombre de
/// nuit) : deux niveaux d'ombre, sans dégradé. Elle donne l'épaisseur de l'avancée du toit.
pub const OMBRE_TOIT: Image = &[
    "0000000000000000",
    "1111111111111111",
];

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
    ("ombre_toit", OMBRE_TOIT),
];
