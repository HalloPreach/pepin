//! Niveau 4 : « Le grand orage ».
//!
//! Éclairs temporisés, plateformes mobiles (qu'on attend), nuages friables, ascenseur.
//! Entre les deux plateformes mobiles, une cheminée d'une tuile (colonne 51)
//! sert de point de repos : on y attend la seconde plateforme.

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 4,
    nom: "Le grand orage",
    theme: Theme::Orage,
    genre: GenreNiveau::Normal,
    intro: "L'orage éclate. Chaque éclair est annoncé : garde la tête froide !",
    mecanique: "Éclairs annoncés, plateformes mobiles et friables, ascenseur.",
    duree_cible_s: 60,
    ascii: ASCII,
    aiguilles: &[],
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
..@..R..............L........R...^^...........................L.......R.......^^^................................BBBBBBBBBBBBBBBBBBBBBBB.........L........f..........^^.........M.............X.........
#######################...##################1....1.#2.....2.#########################.FF.FF.FF.FF.F..##########3.BBBBBBBBBBBBBBBBBBBBBBB...#############################################################
BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBB.......B........BBBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBB.......B........BBBBBBBBBBBBBBBBBBBBBBBBB................BBBBBBBBBB..BBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
