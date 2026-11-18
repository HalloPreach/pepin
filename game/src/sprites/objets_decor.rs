//! Sprites des objets de décor (carte 4.2) : cheminée, antenne, paratonnerre,
//! manche à air, oiseau posé ; et ceux de la rue du Monde 2 (carte 9.6b) : bouche
//! d'égout, borne, poubelle. Aucun n'a de collision : ce
//! sont des dessins posés sur le toit ou la rue (leur dernière rangée touche la
//! tuile du dessous), d'une tuile de large au plus et de moins de deux tuiles de
//! haut. (La descente de gouttière de la rue n'est plus un sprite depuis la carte
//! 9.20h : `decor_rue` la dessine accrochée à sa maison.)
//!
//! Ils sont dessinés tels quels (sans la substitution de couleurs des tuiles) : chacun
//! ne sert que dans les niveaux de son thème (voir `level::GenreDecor`). Couleurs
//! réservées au jeu, donc absentes d'ici : l'or des gouttes (B, C), le cyan des
//! flaques et de la pluie (D, E), le corail du parapluie de Pépin (8, 9).

use super::Image;

/// Souche de cheminée : chapeau de pierre, deux mitrons, corps en brique violette
/// comme les murs, ombre sur le côté droit (12x20).
pub const CHEMINEE: Image = &[
    "..00....00..",
    ".0430..0430.",
    ".0320..0320.",
    "000000000000",
    "044444444440",
    "033333333330",
    "000000000000",
    ".0332333310.",
    ".0FF2FFFF10.",
    ".0222222210.",
    ".0333332310.",
    ".0FFFFF2F10.",
    ".0FFFFF2F10.",
    ".0222222210.",
    ".0332333310.",
    ".0FF2FFFF10.",
    ".0FF2FFFF10.",
    ".0222222210.",
    ".0333332310.",
    ".0FFFFF2F10.",
];

/// Antenne de télévision : mât et trois barres, de plus en plus larges vers le bas (15x24).
pub const ANTENNE: Image = &[
    ".......0.......",
    "......040......",
    "....0004000....",
    "...044444440...",
    "....0004000....",
    "......040......",
    "..00000400000..",
    ".0444444444440.",
    "..00000400000..",
    "......040......",
    "000000040000000",
    "044444444444440",
    "000000040000000",
    "......040......",
    "......040......",
    "......040......",
    "......040......",
    "......040......",
    "......040......",
    "......040......",
    "......040......",
    "......040......",
    ".....03430.....",
    "....0344430....",
];

/// Paratonnerre : pointe claire, boule, mât et pied (7x24).
pub const PARATONNERRE: Image = &[
    "...0...",
    "..070..",
    "..060..",
    "..050..",
    "..050..",
    ".00400.",
    "0466540",
    "0465540",
    "0455440",
    ".04440.",
    "..000..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    "..040..",
    ".03430.",
    "0344430",
];

/// Manche à air rayée orange et blanc, gonflée vers la droite (16x22). Dans le jeu,
/// elle s'étire vers le courant ascendant qu'elle signale (retournée s'il est à gauche).
pub const MANCHE_A_AIR: Image = &[
    ".0..............",
    "040.............",
    "040000000.......",
    "04AA77AA7000....",
    "04AA77AA77AA000.",
    "04AA77AA77AA77A0",
    "04AA55AA55AA55A0",
    "04AA55AA55AA000.",
    "04AA55AA5000....",
    "040000000.......",
    "040.............",
    "040.............",
    "040.............",
    "040.............",
    "040.............",
    "040.............",
    "040.............",
    "040.............",
    "040.............",
    "040.............",
    "030.............",
    "000.............",
];

/// Oiseau posé, petit et blanc, regard vers la droite (12x8).
pub const OISEAU0: Image = &[
    ".......00...",
    ".0....07700.",
    "0500000707A0",
    "05577777700.",
    ".055577770..",
    "..0666600...",
    "...0000.....",
    "....0.0.....",
];

/// Oiseau posé : il tourne la tête (bec vers la gauche), même silhouette.
pub const OISEAU1: Image = &[
    ".......00...",
    ".0....0770..",
    "050000A7070.",
    "0557777770..",
    ".055577770..",
    "..0666600...",
    "...0000.....",
    "....0.0.....",
];

/// Bouche d'égout du caniveau (carte 9.6b) : la pierre de la bordure, la bouche
/// sombre et sa grille de fonte (16x9).
pub const BOUCHE_EGOUT: Image = &[
    "0000000000000000",
    "0666666666666660",
    "0444444444444440",
    "0000000000000000",
    "0504040404040400",
    "0404040404040400",
    "0404040404040400",
    "0000000000000000",
    "0333333333333330",
];

/// Borne de trottoir en fonte : tête ronde, bague, fût, pied évasé (6x14). Carte
/// 9.20g : du décor sans collision, donc terne (ardoise sombre, aucune tête claire).
pub const BORNE: Image = &[
    "..00..",
    ".0220.",
    ".0210.",
    "000000",
    "021210",
    "000000",
    ".0210.",
    ".0110.",
    ".0110.",
    ".0110.",
    ".0110.",
    "002100",
    "021110",
    "011110",
];

/// Poubelle de rue : couvercle et sa poignée, cuve cannelée (12x14). Carte 9.20g :
/// du décor sans collision, donc terne (plus de couvercle clair : elle ne se
/// confond plus avec la poubelle SOLIDE du mobilier de rue, `mobilier_rue`).
pub const POUBELLE: Image = &[
    "....0000....",
    "...021120...",
    "..00000000..",
    ".0222222220.",
    "000000000000",
    ".0121212110.",
    ".0121212110.",
    ".0121212110.",
    ".0121212110.",
    ".0121212110.",
    ".0121212110.",
    ".0121212110.",
    ".0111111110.",
    ".0000000000.",
];

/// Tous les sprites de ce fichier : (nom, dessin).
pub const SPRITES: &[(&str, Image)] = &[
    ("cheminee", CHEMINEE),
    ("antenne", ANTENNE),
    ("paratonnerre", PARATONNERRE),
    ("manche_a_air", MANCHE_A_AIR),
    ("oiseau0", OISEAU0),
    ("oiseau1", OISEAU1),
    ("bouche_egout", BOUCHE_EGOUT),
    ("borne", BORNE),
    ("poubelle", POUBELLE),
];

/// Oiseau posé (2 images) : regard devant, puis tête tournée.
pub const OISEAU: [&str; 2] = ["oiseau0", "oiseau1"];
