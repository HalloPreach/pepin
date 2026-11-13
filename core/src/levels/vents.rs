//! Niveau 4 : « Vents d'altitude ».
//!
//! Colonnes de vent à chevaucher et longs planés entre îlots de nuages ; au
//! milieu de l'escalier de nuages, un îlot friable ; deux corbeaux patrouillent
//! (sur l'îlot d'arrivée du 2e courant et avant le drapeau, `PORTEE_CORBEAU`).
//! Au-dessus du dernier toit, un pigeon guette sur un petit nuage (colonne 167,
//! 4 rangées plus haut) : tourné vers l'arrivée de Pépin, il plonge derrière
//! lui ; il ne touche que celui qui s'arrête ou revient sous lui.

use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 4,
    nom: "Vents d'altitude",
    theme: Theme::Altitude,
    intro: "Le ciel se déchire. Les courants montent : laisse-toi porter, puis plane.",
    mecanique: "Courants ascendants, longs planés, nuage friable, corbeaux, pigeon.",
    duree_cible_s: 55,
    ascii: ASCII,
};

/// Dessin du niveau (190 x 11 tuiles). Légende : voir `crate::level`.
const ASCII: &str = r"
..............................................................................................................................................................................................
............................o.....................vvv....................................................o.....................vvv......L.....................o...............................
................vvv...............................vvv.......c..................................................................vvv..NNNNNNNNNN................................................
................vvv....L..........................vvv...NNNNNNNN.................................vvv...NNNNNN..................vvv.....................................p......................
................vvv..NNNNNNNNNNN..................vvv.............NNNN...........................vvv...................L....Q..vvv.............NNNNNNNNN...............N......................
................vvv...............................vvv...................FFFF.....................vvv.............NNNNNNNNNNNNN.vvv............................M...............................
................vvv...............................vvv.........................NNNN...............vvv...........................vvv.......................NNNNNNNNN............................
..@........W.Q..vvv........................L..Q...vvv...................................L...Q....vvv...........................vvv........................................L.......c.....X.....
###############.vvv...............NNNNNNNNNNNNNN..vvv...............................NNNNNNNNNN...vvv...........................vvv.................................NNNNNNNNNNNNNNNNNNNNNNNNNNN
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
";
