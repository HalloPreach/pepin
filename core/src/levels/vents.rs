//! Niveau 7 : « Vents d'altitude ».
//!
//! Colonnes de vent à chevaucher et longs planés entre îlots de nuages ; au
//! milieu de l'escalier de nuages, un îlot friable ; deux corbeaux patrouillent
//! (sur l'îlot d'arrivée du 2e courant et avant le drapeau, `PORTEE_CORBEAU`).
//! Au-dessus du dernier toit, un pigeon guette sur un petit nuage (colonne 167,
//! 4 rangées plus haut) : tourné vers l'arrivée de Pépin, il plonge derrière
//! lui ; il ne touche que celui qui s'arrête ou revient sous lui.
//!
//! Carte 6.4b (« plus d'ennemis ») : deux corbeaux et un pigeon de plus.
//! - Un corbeau au-dessus des nuages friables (colonne 76, rangée 3 : il
//!   patrouille des colonnes 71 à 81), juste au-dessus de la tête de Pépin posé
//!   sur les friables (colonnes 72 à 75) : on passe dessous en marchant, mais il
//!   faut sauter pour en repartir, avant qu'ils ne cèdent, et un saut au mauvais
//!   moment s'y cogne.
//! - Un corbeau à hauteur de Pépin sur l'îlot des colonnes 143 à 151 (colonne 147,
//!   rangée 3 : il patrouille des colonnes 142 à 152, au-delà des deux bords) : on
//!   se laisse tomber dessus depuis l'îlot de la lanterne 136 (il s'écrase), ou on
//!   le saute ; la lanterne 136 est hors de sa patrouille.
//! - Un second pigeon, sur un petit nuage posé pour lui (colonne 38, rangée 4),
//!   4 rangées au-dessus de l'îlot de la lanterne 43, comme celui du dernier toit :
//!   tourné vers l'arrivée de Pépin, qui s'y pose en descendant de l'îlot du
//!   premier courant, il plonge derrière lui (sa plongée finit au ras de l'îlot,
//!   colonne 34, loin de la lanterne).

use crate::levels::GenreNiveau;
use crate::zones::{InfosNiveau, Theme};

pub const INFOS: InfosNiveau = InfosNiveau {
    numero: 7,
    nom: "Vents d'altitude",
    theme: Theme::Altitude,
    genre: GenreNiveau::Normal,
    intro: "Le ciel se déchire. Les courants montent : laisse-toi porter, puis plane.",
    mecanique: "Courants ascendants, longs planés, nuage friable, corbeaux, pigeons.",
    duree_cible_s: 55,
    ascii: ASCII,
    aiguilles: &[],
};

/// Dessin du niveau (190 x 11 tuiles). Légende : voir `crate::level`.
/// - Corbeaux (carte 6.4b) : (76, 3) et (147, 3) ; pigeon (38, 3) sur le nuage (38, 4).
const ASCII: &str = r"
..............................................................................................................................................................................................
............................o.....................vvv....................................................o.....................vvv......L.....................o...............................
................vvv...............................vvv.......c..................................................................vvv..NNNNNNNNNN................................................
................vvv....L..............p...........vvv...NNNNNNNN............c....................vvv...NNNNNN..................vvv.................c...................p......................
................vvv..NNNNNNNNNNN......N...........vvv.............NNNN...........................vvv...................L....Q..vvv.............NNNNNNNNN...............N......................
................vvv...............................vvv...................FFFF.....................vvv.............NNNNNNNNNNNNN.vvv............................M...............................
................vvv...............................vvv.........................NNNN...............vvv...........................vvv.......................NNNNNNNNN............................
..@........W.Q..vvv........................L..Q...vvv...................................L...Q....vvv...........................vvv........................................L.......c.....X.....
###############.vvv...............NNNNNNNNNNNNNN..vvv...............................NNNNNNNNNN...vvv...........................vvv.................................NNNNNNNNNNNNNNNNNNNNNNNNNNN
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
BBBBBBBBBBBBBBB.vvv...............................vvv............................................vvv...........................vvv............................................................
";
