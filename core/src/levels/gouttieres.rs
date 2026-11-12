//! Niveau 2 : « Gouttières et cheminées ».
//!
//! Flaques patrouilleuses (on les écrase en sautant dessus), trampolines et gouttières suspendues.

use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 2,
    nom: "Gouttières et cheminées",
    theme: Theme::Gouttieres,
    intro: "L'averse redouble. Les flaques rôdent entre les cheminées : saute-leur dessus !",
    mecanique: "Flaques à écraser, champignons-trampolines, gouttières suspendues.",
    duree_cible_s: 55,
    ascii: ASCII,
};

/// Dessin du niveau (190 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
..............................................................................................................................................................................................
............................................................................o...................................o.............................................................................
..................................................................L...........................................................................................................................
............................................................BB........f.......................................................................................................................
............................................................BB#####################.................................................F.........................................................
............................................................BBBBBBBBBBBBBBBBBBBBBBB.................BB........................................................................................
.........o..............BB..............BB..................BBBBBBBBBBBBBBBBBBBBBBB.........BB......BB.............................###.....................BB.........BB......................
..@.................f...BB........L.....BB....f.....f....M..BBBBBBBBBBBBBBBBBBBBBBB.......f.BB...f..BB..L...f...M.............###.......###...........f....BB...f.....BB....L...........X.....
############################...##############################.BBBBBBBBBBBBBBBBBBBBB...#################################..###.................###..############################################
BBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.BBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...........................BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.BBBBBBBBBBBBBBBBBBBBB...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...........................BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB
";
