//! Niveau 1 : « Les toits ».
//!
//! Tutoriel en plein air : déplacer, sauter, puis planer pour franchir un grand vide.

use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 1,
    nom: "Les toits",
    theme: Theme::Toits,
    intro: "La pluie tambourine sur les toits. Pépin ouvre son parapluie et part vers le haut.",
    mecanique: "Course, saut, planer (maintenir SAUT en l'air).",
    duree_cible_s: 50,
    ascii: ASCII,
};

/// Dessin du niveau (176 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
................................................................................................................................................................................
........................................................................................................o.......................................................................
................................................................................................................................................................................
................................................................................................................................................................................
................................................................................................L...............................................................................
...............................................................o............................########################............................................................
.............o..........................................................BB......B.......####BBBBBBBBBBBBBBBBBBBBBBBB........................BB..................................
..@..D....S.....######.....S............L..............G................BB......B...####BBBBBBBBBBBBBBBBBBBBBBBBBBBB..............L.........BB..........................X.......
################BBBBBB########...##########################.......##################BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB#####..#####################################################
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBB.......BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBB.......BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
