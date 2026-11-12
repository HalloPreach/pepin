//! Niveau 3 : « Le grand orage ».
//!
//! Éclairs temporisés, plateformes mobiles (qu'on attend), nuages friables, ascenseur.

use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 3,
    nom: "Le grand orage",
    theme: Theme::Orage,
    intro: "L'orage éclate. Chaque éclair est annoncé : garde la tête froide !",
    mecanique: "Éclairs annoncés, plateformes mobiles et friables, ascenseur.",
    duree_cible_s: 60,
    ascii: ASCII,
};

/// Dessin du niveau (200 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
..............e........................e..........................e.......e..............................e..............e.............................e.........e...........e...........................
...............................................................................................................................B.....B...........................................#......................
...............................................................................................................................B.....B..................................................................
.......................................................o............................................................L.......^^.B..c..B..................................................................
...........................................................................................o...................3.#######################................................................................
.................................................................................................................BBBBBBBBBBBBBBBBBBBBBBB................................................................
.........o.......................................................................................................BBBBBBBBBBBBBBBBBBBBBBB................................................................
..@.................L............^^...........................L...............^^^................................BBBBBBBBBBBBBBBBBBBBBBB.........L........f..........^^.........M.............X.........
#######################...##################1.....1.2.....2.#########################.FF.FF.FF.FF.F..##########3.BBBBBBBBBBBBBBBBBBBBBBB...#############################################################
BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBB................BBBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBB................BBBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
